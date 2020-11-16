use warp::Filter;

use std::env;

#[tokio::main]
async fn main() {
    // TODO: Add ArgParse.

    // TODO: Automate self-signed key gen.
    //
    // Init CA:
    // * genpkey -> ca.key (rsa/des3 as pem)
    // * req -> ca.pem (x509 as pem)
    // * install...
    //
    // Generate site key and cert:
    // * genrsa
    // * generate_csr(
    // * Cert::sign(Key) -> Cert


    let routes = warp::any().map(|| "Hello, world.");
    let args: Vec<_> = env::args().collect();
    warp::serve(routes)
        .tls()
        .cert_path(&args[1])
        .key_path(&args[2])
        .run(([0, 0, 0, 0], 3000))
        .await;
}
