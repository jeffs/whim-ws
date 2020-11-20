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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = fs::read("whim.toml").await?;
    let config: Config = toml::from_slice(&config)?;
    let routes = warp::any().map(|| "Hello, world.\n");

    let mut key = fs::read(&config.tls.key).await?;
    if key.starts_with("-----BEGIN ENCRYPTED".as_bytes()) {
        key = openssl(&["pkcs8", "-in", &config.tls.key]).await?
    }

    println!("https://localhost:3000/");
    warp::serve(routes)
        .tls()
        .cert_path(config.tls.crt)
        .key(key)
        .run(([0, 0, 0, 0], 3000))
        .await;

    Ok(())
}
