use std::sync::Arc;
use std::time::{Instant};
use tokio::task;
use hyper::Client;
use crate::models::dsl_model::DslConfig;
use crate::client::send_request;

pub async fn run_load_test(config: DslConfig) {
    let client = Arc::new(Client::new());
    let config = Arc::new(config);

    let mut handles = Vec::new();

    for _ in 0..config.concurrency {
        let client = Arc::clone(&client);
        let config = Arc::clone(&config);

        let handle = task::spawn(async move {
            let start_time = Instant::now();

            while start_time.elapsed().as_secs() < config.duration {
                if let Err(err) = send_request(&client, &config).await {
                    eprintln!("[Erro] Falha ao enviar requisição: {:?}", err);
                }
            }
        });

        handles.push(handle);
    }

    for handle in handles {
        let _ = handle.await;
    }
}
