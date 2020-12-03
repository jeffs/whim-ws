use serde::Deserialize;
use std::error::Error;
use std::path::PathBuf;
use tokio::fs;

pub const HOST_MASK: [u8; 4] = [0, 0, 0, 0];
pub const HTTP_PORT: u16 = 8000;
pub const HTTPS_PORT: u16 = 3000;

#[derive(Debug, Deserialize)]
pub struct TLS {
    pub crt: PathBuf,
    pub key: PathBuf,
    pub pass: String,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    // TODO:
    // host_mask: [u8; 4],
    // http_port: u16,
    // https_port: u16,
    pub tls: TLS,
}

impl Config {
    pub async fn from_file(path: &str) -> Result<Config, Box<dyn Error>> {
        let bytes: Vec<u8> = fs::read(path).await?;
        Ok(toml::from_slice(&bytes)?)
    }
}
