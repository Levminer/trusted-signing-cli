# Trusted Signing CLI

A simple CLI tool to sign files with Trusted Signing

## Prerequisites

-   [Trusted Signing Account](https://learn.microsoft.com/en-us/azure/trusted-signing/quickstart?tabs=registerrp-portal,account-portal,certificateprofile-portal,deleteresources-portal) and permissions configured
-   [.NET](https://dotnet.microsoft.com/en-us/download/dotnet/8.0) (.NET 6 or later recommended)
-   [Azure CLI](https://learn.microsoft.com/en-us/cli/azure/install-azure-cli-windows?tabs=azure-cli#install-or-update)
-   [Signtool](https://learn.microsoft.com/en-us/dotnet/framework/tools/signtool-exe) (Windows 11 SDK 10.0.26100.0 or later recommended)
-   [Rust](https://www.rust-lang.org/) (Optional if you want to build from source)

## Installation

`cargo install trusted-signing-cli` or download the binary from the [latest releases](https://github.com/levminer/trusted-signing-cli/releases)

## Usage

The CLI expects the following environment variables to be set or you can pass them as arguments. You need to create an Azure App Registration (you can use [this](https://learn.microsoft.com/en-us/power-apps/developer/data-platform/walkthrough-register-app-azure-active-directory) article to get the credentials):

-   `AZURE_CLIENT_ID` (or use `--azure-client-id`)
-   `AZURE_CLIENT_SECRET` (or use `--azure-tenant-id`)
-   `AZURE_TENANT_ID` (or use `--azure-tenant-id`)
-   `AZURE_TRUSTED_SIGNING_ACCOUNT_NAME` (or use `--account/-a`)
-   `AZURE_CERTIFICATE_PROFILE_NAME` (or use `--certificate/-c`)

Signing a single file:
`trusted-signing-cli -e <url> -a <account name> -c <certificate profile name> file1.exe`

Signing multiple files:
`trusted-signing-cli -e <url> -a <account name> -c <certificate profile name> file1.exe file2.exe file3.exe`

For more information run `trusted-signing-cli --help`
