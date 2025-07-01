use hyper::{Client, Request, Body as HyperBody, Method, Uri};
use hyper::client::HttpConnector;
use hyper::header::{AUTHORIZATION, CONTENT_TYPE};
use crate::models::dsl_model::{DslConfig, Body, Auth, HttpMethod};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;

pub async fn send_request(
    client: &Client<HttpConnector>,
    config: &DslConfig
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut uri = config.target.clone();

    if let Some(params) = &config.query_params {
        let query: String = params.iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("&");

        if uri.contains('?') {
            uri.push('&');
        } else {
            uri.push('?');
        }
        uri.push_str(&query);
    }

    let uri: Uri = uri.parse()?;

    let method = match config.method {
        HttpMethod::GET => Method::GET,
        HttpMethod::POST => Method::POST,
        HttpMethod::PUT => Method::PUT,
        HttpMethod::DELETE => Method::DELETE,
        HttpMethod::PATCH => Method::PATCH,
        HttpMethod::HEAD => Method::HEAD,
        HttpMethod::OPTIONS => Method::OPTIONS,
    };

    let body = match &config.body {
        Some(Body::Json(value)) => {
            HyperBody::from(serde_json::to_string(value)?)
        },
        Some(Body::Xml(s)) => HyperBody::from(s.clone()),
        None => HyperBody::empty(),
    };

    let mut req_builder = Request::builder()
        .method(method)
        .uri(uri);

    if let Some(Body::Json(_)) = &config.body {
        req_builder = req_builder.header(CONTENT_TYPE, "application/json");
    } else if let Some(Body::Xml(_)) = &config.body {
        req_builder = req_builder.header(CONTENT_TYPE, "application/xml");
    }

    if let Some(auth) = &config.auth {
        match auth {
            Auth::Basic { username, password } => {
                let encoded = BASE64.encode(format!("{}:{}", username, password));
                req_builder = req_builder.header(AUTHORIZATION, format!("Basic {}", encoded));
            }
            Auth::Bearer { token } => {
                req_builder = req_builder.header(AUTHORIZATION, format!("Bearer {}", token));
            }
            Auth::ApiKey { key_name, key_value, in_header } => {
                if *in_header {
                    req_builder = req_builder.header(key_name, key_value);
                }
            }
            Auth::None => {}
        }
    }

    let request = req_builder.body(body)?;
    let _response = client.request(request).await?;

    Ok(())
}
