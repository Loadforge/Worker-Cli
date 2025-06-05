mod dsl;
mod executor;
mod http_engine;

use crate::dsl::DslConfig;
use crate::executor::Executor;
use std::env;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("⚠️  Uso: cargo run -- <arquivo.dsl.yaml>");
        return;
    }

    let dsl_file = &args[1];

    println!("🚀 Carregando DSL de {}", dsl_file);

    let config = match DslConfig::from_file(dsl_file) {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("❌ Erro ao ler DSL: {:?}", e);
            return;
        }
    };

    println!("🧠 Executando plano '{}'", config.name);

    let executor = Executor::new(config);

    executor.run().await;
}
