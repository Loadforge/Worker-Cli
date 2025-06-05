use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::{sync::Notify, task};
use crate::dsl::DslConfig;
use crate::http_engine::{HttpEngine, HttpRequestConfig};

pub struct Executor {
    config: DslConfig,
    engine: Arc<HttpEngine>,
    notify: Arc<Notify>,
}

impl Executor {
    pub fn new(config: DslConfig) -> Self {
        Self {
            config,
            engine: Arc::new(HttpEngine::new()),
            notify: Arc::new(Notify::new()),
        }
    }

    pub async fn run(&self) {
        let deadline = Instant::now() + Duration::from_secs(self.config.duration);

        for _ in 0..self.config.concurrency {
            let engine = Arc::clone(&self.engine);
            let notify = Arc::clone(&self.notify);
            let target = self.config.target.clone();
            let method = self.config.method.clone();

            task::spawn(async move {
                while Instant::now() < deadline {
                    let start = Instant::now();

                    let req = HttpRequestConfig {
                        method: method.clone(),
                        url: target.clone(),
                        headers: None,
                        body: None,
                    };

                    match engine.execute(req).await {
                        Ok(res) => {
                            println!(
                                "✅ {} - Status: {} - {} ms",
                                target,
                                res.status(),
                                start.elapsed().as_millis()
                            );
                        }
                        Err(err) => {
                            println!("❌ {} - Error: {:?}", target, err);
                        }
                    }
                }
                notify.notify_one();
            });
        }

        for _ in 0..self.config.concurrency {
            self.notify.notified().await;
        }

        println!("🏁 Teste finalizado.");
    }
}
