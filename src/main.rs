use std::error::Error;
use std::sync::Arc;
use std::process;
use tokio_compat_02::FutureExt;
use whim::{self, ClientPointer, Config, HOST_MASK, HTTPS_PORT, HTTP_PORT};

async fn async_main(rt: Arc<tokio::runtime::Runtime>) -> Result<(), Box<dyn Error>> {
    let config = Config::from_file("whim.toml").await?;
    let client = ClientPointer::new(rt);
    let http = warp::serve(whim::http_routes());
    let https = warp::serve(whim::https_routes(client))
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
    let rt = Arc::new(tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap());
    if let Err(err) = rt.block_on(async_main(rt.clone())) {
        eprintln!("error: {}", err);
        process::exit(1);
    }
}
