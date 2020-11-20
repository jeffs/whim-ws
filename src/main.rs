use serde::Deserialize;
use std::io;
use std::str;
use tokio::fs;
use tokio::process::Command;
use warp::Filter;

#[derive(Debug, Deserialize)]
struct TLS {
    crt: String,
    key: String,
}

#[derive(Debug, Deserialize)]
struct Config {
    tls: TLS,
}

async fn openssl(args: &[&str]) -> io::Result<Vec<u8>> {
    Ok(Command::new("openssl").args(args).output().await?.stdout)
}

async fn read_key(path: &str) -> io::Result<Vec<u8>> {
    let key = fs::read(&path).await?;
    if key.starts_with("-----BEGIN ENCRYPTED".as_bytes()) {
        openssl(&["pkcs8", "-in", path]).await
    } else {
        Ok(key)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = fs::read("whim.toml").await?;
    let config: Config = toml::from_slice(&config)?;
    let key = read_key(&config.tls.key).await?;
    let routes = warp::any().map(|| "Hello, world.\n");
    println!("https://localhost:3000/");
    warp::serve(routes)
        .tls()
        .cert_path(config.tls.crt)
        .key(key)
        .run(([0, 0, 0, 0], 3000))
        .await;
    Ok(())
}
