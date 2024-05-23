use reqwest::{Client, Url};
use std::env;
use std::process::exit;

#[tokio::main]
async fn main() {
    let port = env::var("PORT").unwrap_or("8080".into());
    let path = env::var("HEALTHCHECK_PATH").unwrap_or("/".into());
    let url_str = format!("http://localhost:{}{}", port, path);
    let url = Url::parse(&url_str).expect("URL parse");
    let client = Client::new();
    let res = client.get(url.clone()).send().await;
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
