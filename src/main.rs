use std::net::SocketAddr;
use std::sync::Arc;
use anyhow::*;
use hyper::{Body, Client, Request, Server};
use hyper::service::{make_service_fn, service_fn};

pub fn proxy_crate(req: &mut Request<Body>) -> Result<()> {
    for key in &["content-length", "accept-encoding", "content-encoding", "transfer-encoding"] {
        req.headers_mut().remove(*key);
    }
    let uri = req.uri();
    let uri_string = match uri.query() {
        Some(s) => format!("https://crates.io{}?{}", uri.path(), s),
        None => format!("https://crates.io{}", uri.path())
    };
    *req.uri_mut() = uri_string.parse().context("Parse URI Error")?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let https = hyper_rustls::HttpsConnector::with_native_roots();
    let client: Client<_, Body> = Client::builder().build(https);
    let client: Arc<Client<_, Body>> = Arc::new(client);
    let addr = SocketAddr::from(([0, 0, 0, 0], 7000));
    let make_svc = make_service_fn(move |_| {
        let client = Arc::clone(&client);
        async move {
            Ok(service_fn(move |mut req| {
                let client = Arc::clone(&client);
                async move {
                    println!("proxy {}", req.uri().path());
                    proxy_crate(&mut req)?;
                    client.request(req).await.context("proxy request")
                }
            }))
        }
    });

    Server::bind(&addr).serve(make_svc).await.context("Run server")?;

    Ok(())
}
