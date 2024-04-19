# Trusted Signing CLI

A simple CLI tool to sign files with Trusted Signing

## Prerequisites

-   [Trusted Signing Account](https://learn.microsoft.com/en-us/azure/security/trusted-signing/trusted-signing-overview) and permissions configured
-   [Azure CLI](https://learn.microsoft.com/en-us/cli/azure/install-azure-cli-windows?tabs=azure-cli#install-or-update)
-   [Signtool](https://learn.microsoft.com/en-us/dotnet/framework/tools/signtool-exe) (Windows 11 SDK 10.0.22000.0 or later recommended)
-   [Rust](https://www.rust-lang.org/) (Optional if you want to build from source)

## Installation

`cargo install trusted-signing-cli`

## Usage

The CLI expects the following environment variables to be set or you can pass them as arguments (you can use [this](https://dev.to/pwd9000/bk-1iij) article to get the credentials):

-   `AZURE_CLIENT_ID`
-   `AZURE_CLIENT_SECRET`
-   `AZURE_TENANT_ID`

Signing a single file:
    `trusted-signing-cli -e <url> -a <account name> -c <certificate profile name> file1.exe`

Signing multiple files:
    `trusted-signing-cli -e <url> -a <account name> -c <certificate profile name> file1.exe file2.exe file3.exe`

For more information run `trusted-signing-cli --help`
