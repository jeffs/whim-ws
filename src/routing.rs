use crate::configuration::{HTTPS_PORT, HTTP_PORT};
use percent_encoding::percent_decode_str;
use std::convert::Infallible;
use warp::http::uri::{Authority, PathAndQuery, Scheme};
use warp::http::Uri;
use warp::path::FullPath;
use warp::Rejection;
use warp::{Filter, Reply};

fn greet_param(param: String) -> String {
    let name = percent_decode_str(&param).decode_utf8_lossy();
    format!("Hello, {}.\n", name)
}

pub fn http_routes() -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
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
        .with(warp::log("HTTP"))
}

fn get_shell(id: u32) -> String {
    format!("shell #{}", id)
}

pub fn https_routes() -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    // GET /               => web directory
    // GET /v0/hello/      => Hello, world.
    // GET /v0/hello/:name => Hello, {name}.
    // GET /v0/shell/0     => default shell
    let world = warp::path::end().map(|| "Hello, world.\n");
    let param = warp::path::param().map(greet_param);
    let hello = warp::path("hello").and(world.or(param));
    let shell = warp::path!("shell" / u32).map(get_shell);
    let api = warp::get().and(warp::path("v0")).and(hello.or(shell));
    api.or(warp::fs::dir("web"))
        .with(warp::compression::gzip())
        .with(warp::log("HTTPS"))
}

fn path_and_query() -> impl Filter<Extract = (PathAndQuery,), Error = Infallible> + Copy {
    // TODO: Take the existing PathAndQuery from the FullPath.  It's private in
    // Warp, so we have to do this silly as_str/parse dance.
    warp::path::full().map(|path: FullPath| path.as_str().parse::<PathAndQuery>().unwrap())
}
