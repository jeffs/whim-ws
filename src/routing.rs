use crate::client::ClientPointer;
use crate::configuration::{HTTPS_PORT, HTTP_PORT};
use crate::shell::{Shell, ShellID};
use serde::Serialize;
use std::convert::Infallible;
use warp::http::uri::{Authority, PathAndQuery, Scheme};
use warp::http::Uri;
use warp::path::FullPath;
use warp::ws::WebSocket;
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

fn get_health() -> impl Reply {
    warp::reply::json(&Health::fine())
}

fn get_shell(id: u32) -> impl Reply {
    warp::reply::json(&Shell {
        id: ShellID(id),
        name: String::from("Default"),
        history: Vec::new(),
        columns: Vec::new(),
    })
}

// Maps all rejections to 404 Not Found.  The result of this function is always
// a rejection.  Note that this function should **not** be used where a non-404
// rejection might be a desirable response.
//
// When all branches of an `.or()` route fail, Warp has no way of knowing which
// error to return to the client.  It defaults to returning the most "specific"
// error code from any branch of the route, but that means that when a page
// isn't found at all (which ought to be `404 Not Found`), Warp might actually
// return some irrelevant code that made an alternate `.or()` branch fail:  A
// WebSocket route might return `400 Invalid Request Header "connection"`, a
// compression route might return `405 Missing Header "accept-encoding"`, etc.
// This function exists to work around that problem by quashing such errors.
//
// This function's `Ok` result type is arbitrary, except that it must be
// something `warp::Reply` is implemented for, or else the calling route's
// filter chain won't type-check:  Warp accumulates all of a route's `or`s and
// `recover`s into a typelist of nested Eithers, so for the Either to be a
// valid Reply, every one of its component types must also be a valid Reply,
// even if that Reply will never actually be returned.
//
// There may be a better way to do this, but I couldn't find one.  Maybe Warp
// ought to return the first error, like Bash `pipefail`, rather than the most
// specific one.  See also <https://github.com/seanmonstar/warp/issues/77>, but
// note that `into_response()` is now private.
async fn to_not_found(_: Rejection) -> Result<String, Rejection> {
    Err(warp::reject::not_found())
}

fn with_client(
    client: ClientPointer,
) -> impl Filter<Extract = (ClientPointer,), Error = Infallible> + Clone {
    warp::any().map(move || client.clone())
}

async fn connect_client(ws: WebSocket, client: ClientPointer) {
    client.connect(ws).await
}

async fn ws_handler(ws: warp::ws::Ws, client: ClientPointer) -> Result<impl Reply, Rejection> {
    Ok(ws.on_upgrade(move |s| connect_client(s, client)))
}

// Defines all routes under /v0.
pub fn api_routes(
    client: ClientPointer,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    let health = warp::path("health").and(warp::get()).map(get_health);
    let shell = warp::path!("shell" / u32).and(warp::get()).map(get_shell);
    let socket = warp::ws()
        .and(with_client(client))
        .and_then(ws_handler)
        .recover(to_not_found);
    warp::path("v0").and(health.or(shell).or(socket))
}

// Permanently redirects all traffic to the HTTPS port.
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

// Merges the API and static content routes, adding compression (when
// applicable) and logging.
pub fn https_routes(
    client: ClientPointer,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    let raw = api_routes(client).or(warp::fs::dir("web"));
    let zip = warp::header::exact_ignore_case("accept-encoding", "gzip")
        .and(raw.clone())
        .recover(to_not_found)
        .with(warp::compression::gzip());
    zip.or(raw).with(warp::log("HTTPS"))
}

fn path_and_query() -> impl Filter<Extract = (PathAndQuery,), Error = Infallible> + Copy {
    // TODO: Take the existing PathAndQuery from the FullPath.  It's private in
    // Warp, so we have to do this silly as_str/parse dance.
    warp::path::full().map(|path: FullPath| path.as_str().parse::<PathAndQuery>().unwrap())
}
