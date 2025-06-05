use hyper::{Client, Request, Response, Body, Method};
use hyper_tls::HttpsConnector;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct HttpRequestConfig {
    pub method: String,
    pub url: String,
    pub headers: Option<Vec<(String, String)>>,
    pub body: Option<String>,
}

pub struct HttpEngine {
    client: Arc<Client<HttpsConnector<hyper::client::HttpConnector>>>,
}

impl HttpEngine {
    pub fn new() -> Self {
        let https = HttpsConnector::new();
        let client = Client::builder().build::<_, hyper::Body>(https);
        Self {
            client: Arc::new(client),
        }
    }

    pub async fn execute(
        &self,
        config: HttpRequestConfig,
    ) -> Result<Response<Body>, hyper::Error> {
        let method = parse_method(&config.method);

        let mut req_builder = Request::builder()
            .method(method)
            .uri(&config.url);

        if let Some(headers) = &config.headers {
            for (key, value) in headers {
                req_builder = req_builder.header(key, value);
            }
        }

        let req = req_builder
            .body(Body::from(config.body.clone().unwrap_or_default()))
            .expect("Failed to build request");

        self.client.request(req).await
    }
}

fn parse_method(method: &str) -> Method {
    match method.to_uppercase().as_str() {
        "GET" => Method::GET,
        "POST" => Method::POST,
        "PUT" => Method::PUT,
        "DELETE" => Method::DELETE,
        "PATCH" => Method::PATCH,
        "HEAD" => Method::HEAD,
        "OPTIONS" => Method::OPTIONS,
        _ => Method::GET, 
    }
}
