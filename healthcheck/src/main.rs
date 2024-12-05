use reqwest::{Client, Url};
use std::env;
use std::process::exit;

#[tokio::main]
async fn main() {
    // discard the host portion of the address, we always use localhost and keep
    // only the port, which can be changed via configuration
    let site_addr = env::var("LEPTOS_SITE_ADDR").unwrap_or("127.0.0.1:3000".into());
    let port = site_addr.split(":").nth(1).unwrap();
    let path = env::var("HEALTHCHECK_PATH").unwrap_or("/".into());
    let url_str = format!("http://localhost:{}{}", port, path);
    let url = Url::parse(&url_str).expect("URL parse");
    let client = Client::new();
    let res = client.head(url.clone()).send().await;
    res.map(|res| {
        let status_code = res.status();
        if status_code.is_client_error() || status_code.is_server_error() {
            exit(1)
        }
        exit(0)
    })
    .map_err(|_| exit(1))
    .ok();
}
