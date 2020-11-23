use std::process;
use tokio::fs;
use tokio_compat_02::FutureExt;

use whim::{self, Config, HOST_MASK, HTTPS_PORT, HTTP_PORT};

async fn async_main() -> Result<(), Box<dyn std::error::Error>> {
    let config: Config = toml::from_slice(&fs::read("whim.toml").await?)?;
    let http = warp::serve(whim::http_routes());
    let https = warp::serve(whim::https_routes())
        .tls()
        .cert_path(&config.tls.crt)
        .key(whim::read_key(&config.tls).await?);
    println!("https://localhost:{}/", HTTPS_PORT);
    futures::join!(
        http.run((HOST_MASK, HTTP_PORT)).compat(),
        https.run((HOST_MASK, HTTPS_PORT)).compat()
    );
    Ok(())
}

fn main() {
    env_logger::init();
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
