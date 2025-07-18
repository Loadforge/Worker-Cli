use clap::Parser;
mod models;
mod client;
mod executor;
use models::dsl_model::DslConfig;
use executor::run_load_test;
pub mod utils;
use utils::hardware::get_hardware_info;

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

    let (cpu_cores, total_mem_kb, free_mem_kb) = get_hardware_info();

    let min_ram_kb = 500 * 1024; 
    let ram_per_thread_kb = 50 * 1024; 

    if free_mem_kb < min_ram_kb {
        eprintln!(
            "\x1b[1;31m[ERRO]\x1b[0m Memória RAM livre insuficiente para rodar o teste.\n\
             Memória livre detectada: {:.2} MB\n\
             Memória mínima necessária: 500 MB",
            free_mem_kb as f64 / 1024.0
        );
        std::process::exit(1);
    }

    if config.concurrency > cpu_cores * 3 {
        eprintln!(
            "\x1b[1;31m[ERRO]\x1b[0m Concurrency ({}) is too high for your CPU cores ({}). Max allowed is {}.",
            config.concurrency, cpu_cores, cpu_cores * 3
        );
        std::process::exit(1);
    }

    if (config.concurrency as u64) * ram_per_thread_kb > free_mem_kb {
        eprintln!(
            "\x1b[1;31m[ERRO]\x1b[0m Concorrência ({}) muito alta para memória RAM livre.\n\
             Memória RAM necessária: {:.2} MB\n\
             Memória RAM livre: {:.2} MB",
            config.concurrency,
            (config.concurrency as u64 * ram_per_thread_kb) as f64 / 1024.0,
            free_mem_kb as f64 / 1024.0
        );
        std::process::exit(1);
    }

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

    println!(
        "\x1b[1;94mℹ️  Hardware Info:\x1b[0m CPU cores: {}, Total RAM: {:.2} MB, Free RAM: {:.2} MB",
        cpu_cores,
        total_mem_kb as f64 / 1024.0,
        free_mem_kb as f64 / 1024.0
    );

    println!();
    run_load_test(config).await;
}
