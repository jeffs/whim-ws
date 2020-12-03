use crate::configuration::TLS;
use std::io;
use tokio::fs;
use tokio::process::Command;

async fn decrypt(tls: &TLS) -> io::Result<Vec<u8>> {
    Ok(Command::new("openssl")
        .arg("pkcs8")
        .arg("-in")
        .arg(&tls.key)
        .arg("-passin")
        .arg(&tls.pass)
        .output()
        .await?
        .stdout)
}

pub async fn read_key(tls: &TLS) -> io::Result<Vec<u8>> {
    let key = fs::read(&tls.key).await?;
    if key.starts_with("-----BEGIN ENCRYPTED".as_bytes()) {
        Ok(decrypt(tls).await?)
    } else {
        Ok(key)
    }
}