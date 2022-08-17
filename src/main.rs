use azure_identity::{
    CertificateCredentialOptions, ClientCertificateCredential, DefaultAzureCredential,
};
use clap::Parser;
use std::{process::Stdio, sync::Arc};

use acrlogin::{acr_client::AcrClient, Result};
use azure_security_keyvault::KeyvaultClient;
use log::*;
use simplelog::{Config, SimpleLogger};
use tokio::{io::AsyncWriteExt, process::Command};

async fn get_certficate(vault_name: &str, certificate_name: &str) -> Result<Vec<u8>> {
    let creds = Arc::new(DefaultAzureCredential::default());
    let client = KeyvaultClient::new(
        format!("https://{}.vault.azure.net", vault_name).as_str(),
        creds,
    )?
    .secret_client();

    debug!(
        "Fetching {} certificate from keyvault {}",
        certificate_name, vault_name
    );
    let secret = client.get(certificate_name).into_future().await?;
    trace!("Certificate fetched successfully {:?}", secret);
    let cert = base64::decode(secret.value)?;
    Ok(cert)
}

/// Jit tool to easily jit into any preprod bastion resource instance
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Name of the vault that has the certificate
    #[clap(long)]
    vault_name: String,

    /// Name of the certificate in the vault
    #[clap(long)]
    certificate_name: String,

    /// The client id that has permission on the ACR
    #[clap(short, long)]
    client_id: String,

    /// The tenant id of the ACR
    #[clap(short, long)]
    tenant_id: String,

    /// Use subject name validation
    #[clap(short, long)]
    use_cert_sn_issuer: bool,

    /// ACR to login to
    #[clap(short, long)]
    acr_name: String,

    // The number of occurrences of the `v/verbose` flag
    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[clap(short, long, parse(from_occurrences))]
    verbose: u8,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let log_level = match args.verbose {
        0 => LevelFilter::Info,
        1 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    };
    SimpleLogger::init(log_level, Config::default())?;

    let cert = get_certficate(&args.vault_name, &args.certificate_name).await?;
    let mut options = CertificateCredentialOptions::default();
    // set as true to to send certificate chain
    options.set_send_certificate_chain(args.use_cert_sn_issuer);

    // pass is empty by default when certificate is fetched from keyvault
    let creds = Arc::new(ClientCertificateCredential::new(
        args.tenant_id.to_string(),
        args.client_id.to_string(),
        base64::encode(cert),
        "".to_string(),
        options,
    ));

    let acr_client = AcrClient::new(args.acr_name.as_str(), &args.tenant_id, creds)?;
    let access_token = acr_client.get_access_token().await?;

    info!("Login to ACR {}", args.acr_name);
    let mut child = Command::new("docker")
        .arg("login")
        .arg("-u 00000000-0000-0000-0000-000000000000")
        .arg("--password-stdin")
        .arg(args.acr_name)
        .stdin(Stdio::piped())
        .spawn()?;
    child
        .stdin
        .take()
        .unwrap()
        .write_all(access_token.refresh_token.as_bytes())
        .await?;
    let _output = child.wait().await?;

    Ok(())
}
