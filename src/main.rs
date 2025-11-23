use clap::Parser;
use directories::BaseDirs;
use duct::cmd;
use serde::{Deserialize, Serialize};
use std::{ffi::OsString, fs, iter, path::Path, vec};
use trauma::{download::Download, downloader::DownloaderBuilder};
use zip_extensions::zip_extract;

/// Metadata object
#[derive(Serialize, Deserialize, Debug)]
pub struct Metadata {
    #[serde(rename = "Endpoint")]
    pub endpoint: String,

    #[serde(rename = "CodeSigningAccountName")]
    pub code_signing_account_name: String,

    #[serde(rename = "CertificateProfileName")]
    pub certificate_profile: String,
}

/// Simple CLI tool to sign files with Trusted Signing
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// File(s) to sign
    #[arg(required = true, value_name = "FILE(S)", num_args = 1..=99)]
    file: Vec<String>,

    /// Azure client secret
    #[arg(long, env = "AZURE_CLIENT_SECRET")]
    azure_client_secret: String,

    /// Azure client secret
    #[arg(long, env = "AZURE_CLIENT_ID")]
    azure_client_id: String,

    /// Azure tenant id
    #[arg(long, env = "AZURE_TENANT_ID")]
    azure_tenant_id: String,

    /// Azure CLI path
    #[arg(
        long,
        env = "AZURE_CLI_PATH",
        default_value = r"C:\Program Files\Microsoft SDKs\Azure\CLI2\wbin\az.cmd"
    )]
    azure_cli_path: String,

    /// Signtool path
    #[arg(
        long,
        env = "SIGNTOOL_PATH",
        default_value = r"C:\Program Files (x86)\Windows Kits\10\bin\10.0.26100.0\x64\signtool.exe"
    )]
    sign_tool_path: String,

    /// Signing Endpoint
    /// Example: https://eus.codesigning.azure.net
    #[arg(long, short = 'e', verbatim_doc_comment)]
    endpoint: String,

    /// Trusted Signing Account name
    #[arg(
        long,
        env = "AZURE_TRUSTED_SIGNING_ACCOUNT_NAME",
        short = 'a'
    )]
    account: String,

    /// Certificate Profile name
    #[arg(
        long,
        env = "AZURE_CERTIFICATE_PROFILE_NAME",
        short = 'c'
    )]
    certificate: String,

    /// File digest algorithm
    #[arg(long, default_value = "SHA256")]
    fd: String,

    /// RFC 3161 timestamp server URL
    #[arg(long, default_value = "http://timestamp.acs.microsoft.com")]
    tr: String,

    /// Timestamp server digest algorithm
    #[arg(long, default_value = "SHA256")]
    td: String,

    /// Description of the signed content.
    /// When signing a .msi installer, this description will appear as the installer's name in the
    /// UAC prompt or will be a random string of characters if unset.
    #[arg(long, short = 'd')]
    description: Option<String>,

    /// Ignore unsupported files
    /// If set to true, the tool will ignore files that are not supported by the signing process.
    /// This is useful when signing a large number of files and you want to ignore files that are
    /// not supported.
    #[arg(long, short = 'i', default_value = "false")]
    ignore_unsupported: bool,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    match run(args).await {
        Ok(_) => (),
        Err(err) => {
            eprintln!("The application signing was not successful.\n\r{}", err);
            std::process::exit(1);
        }
    }
}

async fn run(args: Args) -> Result<(), String> {
    dbg!(&args);
    if fs::metadata(&args.azure_cli_path).is_err() {
        Err(format!(
            "azure cli {} does not exists, please specify PATH with env AZURE_CLI_PATH",
            &args.azure_cli_path
        ))?;
    }

    if fs::metadata(&args.sign_tool_path).is_err() {
        Err(format!(
            "signtool {} does not exists, please specify PATH with env SIGNTOOL_PATH",
            &args.sign_tool_path
        ))?;
    }

    // Get home directory
    let base = BaseDirs::new().expect("could not find home directory");
    let home = base.home_dir();

    // Create config directory
    let config_dir = home.join(".trusted-signing-cli");
    if !config_dir.exists() {
        fs::create_dir_all(&config_dir).map_err(|err| {
            format!(
                "config dir '{:?}' could not be created: {:?}",
                &config_dir, err
            )
        })?;
    }

    // Check if lib is downloaded
    let lib_path = config_dir
        .join("lib")
        .join("bin")
        .join("x64")
        .join("Azure.CodeSigning.Dlib.dll");

    // Download and extract lib
    if !lib_path.exists() {
        let link = "https://www.nuget.org/api/v2/package/Microsoft.Trusted.Signing.Client/1.0.95";
        let downloads = vec![Download::try_from(link).map_err(|err| {
            format!("could not download signing client from {}: {:?}", link, err)
        })?];
        let downloader = DownloaderBuilder::new()
            .directory(config_dir.clone())
            .build();
        downloader.download(&downloads).await;
        let archive = config_dir.join("1.0.95");
        let target_dir = config_dir.join("lib");

        zip_extract(&archive, &target_dir)
            .map_err(|err| format!("signing client can't be unzipped: {:?}", err))?;
    }

    // Check if metadata exists
    let metadata_path = config_dir.join("metadata.json");

    let data = Metadata {
        certificate_profile: args.certificate,
        code_signing_account_name: args.account,
        endpoint: args.endpoint,
    };

    fs::write(
        config_dir.join("metadata.json"),
        serde_json::to_string(&data)
            .map_err(|err| format!("metadata.json could not be parsed: {:?}", err))?,
    )
    .map_err(|err| format!("metadata.json could not be written: {:?}", err))?;

    // Login to azure cli
    cmd!(
        &args.azure_cli_path,
        "login",
        "--service-principal",
        "-t",
        args.azure_tenant_id,
        "-u",
        args.azure_client_id,
        "-p",
        args.azure_client_secret
    )
    .run()
    .map_err(|err| {
        format!(
            "login via azure cli '{}' failed: {:?}",
            &args.azure_cli_path, err
        )
    })?;

    // iterate over files
    let mut cmd_args: Vec<OsString> = vec![
        "sign".into(),
        "/v".into(),
        "/fd".into(),
        args.fd.into(),
        "/tr".into(),
        args.tr.into(),
        "/td".into(),
        args.td.into(),
        "/dlib".into(),
        lib_path.into(),
        "/dmdf".into(),
        metadata_path.into(),
    ];

    if let Some(description) = args.description {
        cmd_args.push("/d".into());
        cmd_args.push(description.into());
    }

    for file in args.file {
        if args.ignore_unsupported {
            if !is_supported(&file) {
                continue;
            }
        }

        cmd(
            &args.sign_tool_path,
            cmd_args.iter().chain(iter::once(&file.clone().into())),
        )
        .run()
        .map_err(|err| {
            format!(
                "signtool '{}' could not sign the file '{:?}', error: {:?}",
                &args.sign_tool_path, &file, &err
            )
        })?;
    }

    Ok(())
}

fn is_supported(file: &str) -> bool {
    let supported_extensions = vec![
        "appx",
        "msix",
        "appxbundle",
        "msixbundle", // Packaged Windows Apps
        "cab",        // Self-contained files used for application installation and setup
        "cat",        // Files that contain digital thumbprints
        "dll",        // Files that contain executable functions
        "exe",        // Files that contain executable programs
        "js",
        "vbs",
        "wsf", // Windows shell files
        "msi",
        "msp",
        "mst", // Windows installer files
        "ocx", // Files that contain Microsoft ActiveX controls
        "ps1", // Files that contain PowerShell scripts
        "stl", // Files that contain a certificate trust list
        "sys", // System files
    ];
    let extension = Path::new(file).extension().unwrap_or_default();
    supported_extensions.contains(&extension.to_str().unwrap_or_default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build() {
        // build the app
        cmd!("cargo", "build",).run().unwrap();
        cmd!("cargo", "build", "--release").run().unwrap();

        // attempt to sign a file
        cmd!(
            "target/debug/trusted-signing-cli.exe",
            "target/release/trusted-signing-cli.exe",
            "-e",
            "https://wus2.codesigning.azure.net",
            "-a",
            "mnr",
            "-c",
            "Profile3",
        )
        .run()
        .unwrap();
    }
}
