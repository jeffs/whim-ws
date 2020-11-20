use serde::Deserialize;
use tokio::fs;
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = fs::read("whim.toml").await?;
    let config: Config = toml::from_slice(&config)?;
    let routes = warp::any().map(|| "Hello, world.\n");

    warp::serve(routes)
        .tls()
        .cert_path(config.tls.crt)
        .key_path(config.tls.key)
        .run(([0, 0, 0, 0], 3000))
        .await;

    Ok(())
}
