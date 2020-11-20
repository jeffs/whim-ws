use warp::Filter;

use std::env;

#[tokio::main]
async fn main() {
    // TODO: Add ArgParse.
    // TODO: Automate self-signed key gen.

    let routes = warp::any().map(|| "Hello, world.");
    let args: Vec<_> = env::args().collect();
    warp::serve(routes)
        .tls()
        .cert_path(&args[1])
        .key_path(&args[2])
        .run(([0, 0, 0, 0], 3000))
        .await;
}
