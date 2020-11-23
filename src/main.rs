use percent_encoding::percent_decode_str;
use serde::Deserialize;
use std::convert::Infallible;
use std::{io, process};
use tokio::fs;
use tokio::process::Command;
use tokio_compat_02::FutureExt;
use warp::http::uri::{Authority, PathAndQuery, Scheme};
use warp::http::Uri;
use warp::path::FullPath;
use warp::Rejection;
use warp::{Filter, Reply};

const HOST_MASK: [u8; 4] = [0, 0, 0, 0];
const HTTP_PORT: u16 = 8000;
const HTTPS_PORT: u16 = 3000;

#[derive(Debug, Deserialize)]
struct TLS {
    crt: String,
    key: String,
    pass: String,
}

#[derive(Debug, Deserialize)]
struct Config {
    // TODO:
    // host_mask: [u8; 4],
    // http_port: u16,
    // https_port: u16,
    tls: TLS,
}

async fn openssl(args: &[&str]) -> io::Result<Vec<u8>> {
    Ok(Command::new("openssl").args(args).output().await?.stdout)
}

async fn read_key(tls: &TLS) -> io::Result<Vec<u8>> {
    let key = fs::read(&tls.key).await?;
    if key.starts_with("-----BEGIN ENCRYPTED".as_bytes()) {
        openssl(&["pkcs8", "-in", &tls.key, "-passin", &tls.pass]).await
    } else {
        Ok(key)
    }
}

fn path_and_query() -> impl Filter<Extract = (PathAndQuery,), Error = Infallible> + Copy {
    // TODO: Take the existing PathAndQuery from the FullPath.  It's private in
    // Warp, so we have to do this silly as_str/parse dance.
    warp::path::full().map(|path: FullPath| path.as_str().parse::<PathAndQuery>().unwrap())
}

fn http_routes() -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    // TODO: Support remote clients (requiring secure login).
    // TODO: Forward user:password@.
    let from = format!("localhost:{}", HTTP_PORT);
    let to: Authority = format!("localhost:{}", HTTPS_PORT).parse().unwrap();
    warp::any()
        .and(warp::host::exact(&from))
        .and(path_and_query())
        .map(move |path_and_query: PathAndQuery| {
            let target = Uri::builder()
                .scheme(Scheme::HTTPS)
                .authority(to.clone())
                .path_and_query(path_and_query)
                .build()
                .unwrap();
            warp::redirect(target)
        })
}

fn greet_param(param: String) -> String {
    let name = percent_decode_str(&param).decode_utf8_lossy();
    format!("Hello, {}.\n", name)
}

fn https_routes() -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    // GET /                   => web directory
    // GET /api/v0/hello/      => Hello, world.
    // GET /api/v0/hello/:name => Hello, {name}.
    let world = warp::path::end().map(|| "Hello, world.\n");
    let param = warp::path::param().map(greet_param);
    let api = warp::get()
        .and(warp::path("api"))
        .and(warp::path("v0"))
        .and(warp::path("hello").and(world.or(param)));
    api.or(warp::fs::dir("web")).with(warp::log("HTTPS"))
}

async fn async_main() -> Result<(), Box<dyn std::error::Error>> {
    let config: Config = toml::from_slice(&fs::read("whim.toml").await?)?;
    let http = warp::serve(http_routes());
    let https = warp::serve(https_routes())
        .tls()
        .cert_path(&config.tls.crt)
        .key(read_key(&config.tls).await?);
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
