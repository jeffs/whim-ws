use crate::configuration::{HTTPS_PORT, HTTP_PORT};
use crate::shell::{Shell, ShellID};
use serde::Serialize;
use std::convert::Infallible;
use warp::http::uri::{Authority, PathAndQuery, Scheme};
use warp::http::Uri;
use warp::path::FullPath;
use warp::Rejection;
use warp::{Filter, Reply};

#[derive(Debug, Serialize)]
struct Health {
    status: String,
}

impl Health {
    fn fine() -> Health {
        Health {
            status: String::from("Fine, thanks for asking."),
        }
    }
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

fn get_shell(id: u32) -> impl Reply {
    warp::reply::json(&Shell {
        id: ShellID(id),
        name: String::from("Default"),
        history: Vec::new(),
        columns: Vec::new(),
    })
}

fn get_health() -> impl Reply {
    warp::reply::json(&Health::fine())
}

pub fn https_routes() -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    // GET /               => web directory
    // GET /v0/health/     => json
    // GET /v0/shell/0     => default shell
    let health = warp::path("health").map(get_health);
    let shell = warp::path!("shell" / u32).map(get_shell);
    let api = warp::get().and(warp::path("v0")).and(health.or(shell));
    let uncompressed = api.or(warp::fs::dir("web")).with(warp::log("HTTPS"));
    let compressed = warp::header::exact_ignore_case("accept-encoding", "gzip")
        .and(uncompressed.clone())
        .with(warp::compression::gzip());
    compressed.or(uncompressed)
}

fn path_and_query() -> impl Filter<Extract = (PathAndQuery,), Error = Infallible> + Copy {
    // TODO: Take the existing PathAndQuery from the FullPath.  It's private in
    // Warp, so we have to do this silly as_str/parse dance.
    warp::path::full().map(|path: FullPath| path.as_str().parse::<PathAndQuery>().unwrap())
}
