use serde::Deserialize;
use std::io;
use std::process;
use std::str;
use tokio::fs;
use tokio::process::Command;
use tokio_compat_02::FutureExt;
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

async fn async_main() -> Result<(), Box<dyn std::error::Error>> {
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
        .compat()
        .await;
    Ok(())
}

fn main() {
    if let Err(err) = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async_main())
    {
        eprintln!("error: {}", err);
        process::exit(1);
    }
}
