use serde::Deserialize;
use std::{io, process};
use tokio::fs;
use tokio::process::Command;
use tokio_compat_02::FutureExt;
use warp::http::uri::{Authority, PathAndQuery, Scheme};
use warp::http::Uri;
use warp::path::FullPath;
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
    let port: u16 = 3000;
    let config = fs::read("whim.toml").await?;
    let config: Config = toml::from_slice(&config)?;
    let key = read_key(&config.tls.key).await?;
    let routes = warp::any().map(|| "Hello, world.\n");
    println!("https://localhost:{}/", port);
    let https = warp::serve(routes)
        .tls()
        .cert_path(config.tls.crt)
        .key(key)
        .run(([0, 0, 0, 0], port))
        .compat();
    let http_routes = warp::any()
        .and(warp::header::<String>("host"))
        .and(warp::path::full())
        .map(move |host: String, path: FullPath| {
            // TODO: Retain "user:password@".
            // TODO: Reject request unless "host" header is a valid authority.
            let authority: Authority = host.parse().unwrap();
            let authority: Authority = format!("{}:{}", authority.host(), port).parse().unwrap();
            let path_and_query: PathAndQuery = path.as_str().parse().unwrap();
            let target = Uri::builder()
                .scheme(Scheme::HTTPS)
                .authority(authority)
                .path_and_query(path_and_query)
                .build()
                .unwrap();
            warp::redirect(target)
        });
    // TODO: response.redirect('https://' + request.headers.host + request.url);
    let http = warp::serve(http_routes).run(([0, 0, 0, 0], 8000)).compat();
    futures::join!(https, http);
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
