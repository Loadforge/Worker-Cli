use hyper::{Client, Request, Response, Body};
use hyper::client::HttpConnector;
use hyper_tls::HttpsConnector;
use std::sync::Arc;
use crate::dsl::{DslConfig, Auth, Body as DslBody};
use base64::{engine::general_purpose, Engine as _};

pub struct HttpEngine {
    client: Arc<Client<HttpsConnector<HttpConnector>>>,
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
        config: DslConfig,
    ) -> Result<Response<Body>, hyper::Error> {
        let mut req_builder = Request::builder()
            .method(config.method.as_str())
            .uri(&config.target);

        if let Some(headers) = Self::generate_headers(&config) {
            for (key, value) in headers {
                req_builder = req_builder.header(key, value);
            }
        }

        let body = config
            .body
            .as_ref()
            .map(Self::serialize_body)
            .unwrap_or_default();

        let req = req_builder
            .body(Body::from(body))
            .expect("Failed to build request");

        self.client.request(req).await
    }

    fn generate_headers(config: &DslConfig) -> Option<Vec<(String, String)>> {
        let mut headers = Vec::new();

        if let Some(auth) = &config.auth {
            match auth {
                Auth::Basic { username, password } => {
                    let credentials = format!("{}:{}", username, password);
                    let encoded = general_purpose::STANDARD.encode(credentials);
                    headers.push(("Authorization".into(), format!("Basic {}", encoded)));
                }
                Auth::Bearer { token } => {
                    headers.push(("Authorization".into(), format!("Bearer {}", token)));
                }
                Auth::ApiKey { key_name, key_value, in_header } => {
                    if *in_header {
                        headers.push((key_name.clone(), key_value.clone()));
                    }
                }
                Auth::None => {}
            }
        }

        if let Some(body) = &config.body {
            match body {
                DslBody::Json(_) => {
                    headers.push(("Content-Type".into(), "application/json".into()));
                }
                DslBody::Xml(_) => {
                    headers.push(("Content-Type".into(), "application/xml".into()));
                }
            }
        }

        Some(headers).filter(|h| !h.is_empty())
    }

    fn serialize_body(body: &DslBody) -> String {
        match body {
            DslBody::Json(json) => json.to_string(),
            DslBody::Xml(xml) => xml.clone(),
        }
    }
}
