use clap::Parser;
mod models;
mod client;
mod executor;
mod ui;

use models::dsl_model::{DslConfig, Auth, Body};
use executor::run_load_test;

#[derive(Parser)]
struct Cli {
    #[arg(short, long)]
    config: String,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let content = std::fs::read_to_string(&cli.config)
        .expect("Erro ao ler o arquivo de configuração");

    let config: DslConfig = serde_json::from_str(&content)
        .expect("Erro ao parsear o JSON de configuração");

    println!("\n\x1b[1;97;44m🚀 Iniciando teste de carga: {}\x1b[0m", config.name);
    println!("\x1b[1;94m🌐 Target       :\x1b[0m {}", config.target);
    println!("\x1b[1;93m🔧 Método       :\x1b[0m {:?}", config.method);
    println!("\x1b[1;92m👥 Concorrência :\x1b[0m {}", config.concurrency);
    println!("\x1b[1;96m⏱️  Duração     :\x1b[0m {} segundos", config.duration);

    if let Some(ref auth) = config.auth {
        println!("\x1b[1;95m🔐 Autenticação :\x1b[0m {:?}", auth);
    }

    if let Some(ref body) = config.body {
        println!("\x1b[1;91m📦 Corpo da req :\x1b[0m {:?}", body);
    }

    if let Some(ref params) = config.query_params {
        println!("\x1b[1;90m🔎 Query Params :\x1b[0m {:?}", params);
    }

    println!(); 

    run_load_test(config).await;
}
