use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use tokio::task;
use tokio::time::sleep;
use hyper_tls::HttpsConnector;
use hyper::Client;
use chrono::Local;
use colored::*;

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
        status_counts: HashMap::new(),
        ..Default::default()
    }));

    let response_times = Arc::new(Mutex::new(Vec::new()));
    let running = Arc::new(AtomicBool::new(true));
    let mut handles = Vec::new();

    let duration_secs = config.duration;
    let end_time = Instant::now() + Duration::from_secs(duration_secs);

    for _ in 0..config.concurrency {
        let client = Arc::clone(&client);
        let config = Arc::clone(&config);
        let metrics = Arc::clone(&metrics);
        let response_times = Arc::clone(&response_times);
        let running = Arc::clone(&running);

        let handle = task::spawn(async move {
            while running.load(Ordering::Relaxed) && Instant::now() < end_time {
                let result = send_request(&client, &config).await;

                match result {
                    Ok((status, duration)) => {
                        let elapsed = duration as f64;

                        {
                            let mut rt = response_times.lock().unwrap();
                            rt.push(elapsed);
                        }

                        let mut m = metrics.lock().unwrap();
                        m.total_requests += 1;
                        m.successful_requests += 1;
                        m.total_duration += elapsed;

                        println!(
                            "{} {} {} {}",
                            "status :".green().bold(),
                            status.as_u16().to_string().bold(),
                            "| duration :".blue().bold(),
                            format!("{:.0}ms", elapsed).bold()
                        );

                        let key = status.as_u16().to_string();
                        *m.status_counts.entry(key).or_insert(0) += 1;

                        if elapsed < m.fastest_response {
                            m.fastest_response = elapsed;
                        }
                        if elapsed > m.slowest_response {
                            m.slowest_response = elapsed;
                        }
                    }
                    Err((err_msg, duration)) => {
                        let elapsed = duration as f64;

                        {
                            let mut rt = response_times.lock().unwrap();
                            rt.push(elapsed);
                        }

                        let mut m = metrics.lock().unwrap();
                        m.total_requests += 1;
                        m.failed_requests += 1;
                        m.total_duration += elapsed;

                        eprintln!(
                            "{} {} {} {}",
                            "status :".red().bold(),
                            err_msg.red().bold(),
                            "| duration :".blue().bold(),
                            format!("{:.0}ms", elapsed).bold()
                        );

                        *m.status_counts.entry("REQUEST_ERROR".to_string()).or_insert(0) += 1;

                        if elapsed < m.fastest_response {
                            m.fastest_response = elapsed;
                        }
                        if elapsed > m.slowest_response {
                            m.slowest_response = elapsed;
                        }
                    }
                }
            }
        });

        handles.push(handle);
    }

    sleep(Duration::from_secs(duration_secs)).await;
    for handle in handles.iter() {
        handle.abort();
    }
    for handle in handles {
        let _ = handle.await;
    }

    let mut final_metrics = metrics.lock().unwrap();
    let response_times = response_times.lock().unwrap();

    let median = calculate_median(&response_times);
    let total_time_secs = duration_secs as f64;
    let throughput = final_metrics.total_requests as f64 / total_time_secs;

    final_metrics.target_url = config.target.clone();
    final_metrics.http_method = format!("{:?}", config.method);
    final_metrics.duration_secs = config.duration;
    final_metrics.concurrency = config.concurrency;
    final_metrics.throughput = throughput;
    final_metrics.median_response_time = median;
    final_metrics.timestamp = Local::now().format("%Y/%m/%d %H:%M:%S").to_string();

    println!();
    println!("\x1b[1;97;44m🔥 ======== TEST RESULTS ======== 🔥\x1b[0m");
    println!("\x1b[1;94m🌐 Target URL               : \x1b[0m\x1b[1;97m{}\x1b[0m", final_metrics.target_url);
    println!("\x1b[1;94m🔁 HTTP Method              : \x1b[0m\x1b[1;97m{}\x1b[0m", final_metrics.http_method);
    println!("\x1b[1;94m📅 Duration (s)             : \x1b[0m\x1b[1;97m{}\x1b[0m", final_metrics.duration_secs);
    println!("\x1b[1;94m👥 Concurrency              : \x1b[0m\x1b[1;97m{}\x1b[0m", final_metrics.concurrency);
    println!("\x1b[1;94m⏰ Timestamp                : \x1b[0m\x1b[1;97m{}\x1b[0m", final_metrics.timestamp);
    println!("\x1b[1;92m✅ Total requests           : \x1b[0m\x1b[1;97m{}\x1b[0m", final_metrics.total_requests);
    println!("\x1b[1;92m✅ Successful requests      : \x1b[0m\x1b[1;97m{}\x1b[0m", final_metrics.successful_requests);
    println!("\x1b[1;91m❌ Failed requests          : \x1b[0m\x1b[1;97m{}\x1b[0m", final_metrics.failed_requests);
    println!("\x1b[1;96m⚡ Fastest response (ms)    : \x1b[0m\x1b[1;97m{:.2}\x1b[0m", final_metrics.fastest_response);
    println!("\x1b[1;93m🐢 Slowest response (ms)    : \x1b[0m\x1b[1;97m{:.2}\x1b[0m", final_metrics.slowest_response);
    println!("\x1b[1;95m📊 Median response time (ms): \x1b[0m\x1b[1;97m{:.2}\x1b[0m", final_metrics.median_response_time);
    println!("\x1b[1;94m📈 Requests per second (RPS): \x1b[0m\x1b[1;97m{:.2}\x1b[0m", final_metrics.throughput);


    println!();
    println!("\x1b[1;97;44m📦 ======== STATUS BREAKDOWN ========\x1b[0m");
    for (status, count) in &final_metrics.status_counts {
        println!("\x1b[1;97mCode {}: {}\x1b[0m", status, count);
    }
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
