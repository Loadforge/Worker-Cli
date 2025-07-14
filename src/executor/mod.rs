use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;
use tokio::task;
use hyper_tls::HttpsConnector;
use hyper::Client;
use chrono::Utc;

use crate::client::{send_request, HttpsClient};
use crate::models::dsl_model::DslConfig;
use crate::models::metrics::Metrics;

pub async fn run_load_test(config: DslConfig) {
    let https = HttpsConnector::new();
    let client: HttpsClient = Client::builder().build::<_, hyper::Body>(https);
    let client = Arc::new(client);
    let config = Arc::new(config);

    let metrics = Arc::new(Mutex::new(Metrics {
        fastest_response: f64::MAX,
        slowest_response: f64::MIN,
        ..Default::default()
    }));

    let response_times = Arc::new(Mutex::new(Vec::new()));

    let start_time = Instant::now();
    let running = Arc::new(AtomicBool::new(true));
    let mut handles = Vec::new();

    for _ in 0..config.concurrency {
        let client = Arc::clone(&client);
        let config = Arc::clone(&config);
        let metrics = Arc::clone(&metrics);
        let response_times = Arc::clone(&response_times);
        let global_start = start_time.clone();

        let handle = task::spawn(async move {
            while global_start.elapsed().as_secs() < config.duration {
                let request_start = Instant::now();
                let success = send_request(&client, &config).await.is_ok();
                let elapsed = request_start.elapsed().as_secs_f64() * 1000.0; // em ms

                {
                    let mut rt = response_times.lock().unwrap();
                    rt.push(elapsed);
                }

                let mut m = metrics.lock().unwrap();
                m.total_requests += 1;
                m.total_duration += elapsed;

                if success {
                    m.successful_requests += 1;
                } else {
                    m.failed_requests += 1;
                }

                if elapsed < m.fastest_response {
                    m.fastest_response = elapsed;
                }

                if elapsed > m.slowest_response {
                    m.slowest_response = elapsed;
                }
            }
        });

        handles.push(handle);
    }

    for handle in handles {
        let _ = handle.await;
    }

    running.store(false, Ordering::SeqCst);

    let mut final_metrics = metrics.lock().unwrap();
    let response_times = response_times.lock().unwrap();

    let median = calculate_median(&response_times);
    let total_time_secs = config.duration as f64;
    let throughput = final_metrics.total_requests as f64 / total_time_secs;

    final_metrics.target_url = config.target.clone();
    final_metrics.http_method = format!("{:?}", config.method);
    final_metrics.duration_secs = config.duration;
    final_metrics.concurrency = config.concurrency;
    final_metrics.throughput = throughput;
    final_metrics.median_response_time = median;
    final_metrics.timestamp = Utc::now().to_rfc3339();

    println!();
    println!("\x1b[1;97;44m🔥 ======== RESULTADOS DO TESTE ======== 🔥\x1b[0m");
    println!("\x1b[1;92m✅ Total de requisições         : \x1b[0m\x1b[1;97m{}\x1b[0m", final_metrics.total_requests);
    println!("\x1b[1;92m✅ Requisições bem-sucedidas    : \x1b[0m\x1b[1;97m{}\x1b[0m", final_metrics.successful_requests);
    println!("\x1b[1;91m❌ Requisições com erro         : \x1b[0m\x1b[1;97m{}\x1b[0m", final_metrics.failed_requests);
    println!("\x1b[1;96m⚡ Resposta mais rápida (ms)    : \x1b[0m\x1b[1;97m{:.2}\x1b[0m", final_metrics.fastest_response);
    println!("\x1b[1;93m🐢 Resposta mais lenta (ms)     : \x1b[0m\x1b[1;97m{:.2}\x1b[0m", final_metrics.slowest_response);
    println!("\x1b[1;95m📊 Mediana das respostas (ms)   : \x1b[0m\x1b[1;97m{:.2}\x1b[0m", final_metrics.median_response_time);
    println!("\x1b[1;94m📈 Requisições por segundo (RPS): \x1b[0m\x1b[1;97m{:.2}\x1b[0m", final_metrics.throughput);
}

fn calculate_median(data: &[f64]) -> f64 {
    let mut sorted = data.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let len = sorted.len();
    if len == 0 {
        return 0.0;
    }
    if len % 2 == 0 {
        (sorted[len / 2 - 1] + sorted[len / 2]) / 2.0
    } else {
        sorted[len / 2]
    }
}
