use clap::Parser;
use directories::BaseDirs;
use duct::cmd;
use serde::{Deserialize, Serialize};
use std::fs;
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
        default_value = r"C:\Program Files (x86)\Windows Kits\10\bin\10.0.22000.0\x64\signtool.exe"
    )]
    sing_tool_path: String,

    /// Signing Endpoint
    /// Example: https://eus.codesigning.azure.net
    #[arg(long, short = 'e', verbatim_doc_comment)]
    endpoint: String,

    /// Code Signing Account name
    #[arg(long, short = 'a')]
    account: String,

    /// Certificate Profile name
    #[arg(long, short = 'c')]
    certificate: String,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    // Get home directory
    let base = BaseDirs::new().unwrap();
    let home = base.home_dir();

    // Create config directory
    let config_dir = home.join(".trusted-signing-cli");
    if !config_dir.exists() {
        std::fs::create_dir_all(&config_dir).unwrap();
    }

    // Check if lib is downloaded
    let lib_path = config_dir
        .join("lib")
        .join("bin")
        .join("x64")
        .join("Azure.CodeSigning.Dlib.dll");

    // Download and extract lib
    if !lib_path.exists() {
        let link = "https://www.nuget.org/api/v2/package/Microsoft.Trusted.Signing.Client/1.0.60";
        let downloads = vec![Download::try_from(link).unwrap()];
        let downloader = DownloaderBuilder::new()
            .directory(config_dir.clone())
            .build();
        downloader.download(&downloads).await;
        let archive = config_dir.join("1.0.60");
        let target_dir = config_dir.join("lib");

        zip_extract(&archive, &target_dir).unwrap();
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
        serde_json::to_string(&data).unwrap(),
    )
    .unwrap();

    // Login to azure cli
    cmd!(
        args.azure_cli_path,
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
    .unwrap();

    // iterate over files
    for file in args.file {
        cmd!(
            &args.sing_tool_path,
            "sign",
            "/v",
            "/fd",
            "SHA256",
            "/tr",
            "http://timestamp.acs.microsoft.com",
            "/td",
            "SHA256",
            "/dlib",
            &lib_path,
            "/dmdf",
            &metadata_path,
            file
        )
        .run()
        .unwrap();
    }
}
