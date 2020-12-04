use serde::Deserialize;
use std::env::{self, VarError};
use std::error::Error;
use std::path::PathBuf;
use tokio::fs;

pub const HOST_MASK: [u8; 4] = [0, 0, 0, 0];
pub const HTTP_PORT: u16 = 8000;
pub const HTTPS_PORT: u16 = 3000;

fn expand_home(p: PathBuf) -> Result<PathBuf, VarError> {
    Ok(match p.strip_prefix("~") {
        Ok(suffix) => PathBuf::from(env::var("HOME")?).join(suffix),
        Err(_) => p,
    })
}

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
        let mut config: Config = toml::from_slice(&bytes)?;
        config.tls.crt = expand_home(config.tls.crt)?;
        config.tls.key = expand_home(config.tls.key)?;
        Ok(config)
    }
}
