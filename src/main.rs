use clap::Parser;
mod models;
mod client;
mod executor;
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
        .expect("Error reading configuration file");

    let config: DslConfig = serde_json::from_str(&content)
        .expect("Error parsing configuration JSON");

    println!("\n\x1b[1;97;44m🚀 Starting load test: {}\x1b[0m", config.name);
    println!("\x1b[1;94m🌐 Target       :\x1b[0m {}", config.target);
    println!("\x1b[1;93m🔧 Method       :\x1b[0m {:?}", config.method);
    println!("\x1b[1;92m👥 Concurrency  :\x1b[0m {}", config.concurrency);
    println!("\x1b[1;96m⏱️  Duration    :\x1b[0m {} seconds", config.duration);

    if let Some(ref auth) = config.auth {
        println!("\x1b[1;95m🔐 Authentication :\x1b[0m {:?}", auth);
    }

    if let Some(ref body) = config.body {
        println!("\x1b[1;91m📦 Request Body    :\x1b[0m {:?}", body);
    }

    if let Some(ref params) = config.query_params {
        println!("\x1b[1;90m🔎 Query Params    :\x1b[0m {:?}", params);
    }

    println!();

    run_load_test(config).await;
}
