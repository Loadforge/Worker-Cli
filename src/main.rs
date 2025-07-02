use clap::Parser;
mod models;
mod client;
mod executor;
mod ui;

use models::dsl_model::DslConfig;
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

    println!("Iniciando teste de carga: {}", config.name);

    run_load_test(config).await;

    println!("Teste de carga finalizado.");
}
