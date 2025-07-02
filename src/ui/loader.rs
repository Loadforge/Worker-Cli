use std::sync::atomic::{AtomicBool, Ordering};
use std::io::{self, Write};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

pub fn show_loader(running: Arc<AtomicBool>, duration_secs: u64, start_time: Instant) {
    thread::spawn(move || {
        while running.load(Ordering::SeqCst) {
            let elapsed = start_time.elapsed().as_secs();
            let remaining = duration_secs.saturating_sub(elapsed);

            print!("\r⏱️  Tempo restante: {:02}s ", remaining);
            io::stdout().flush().unwrap();

            if remaining == 0 {
                break;
            }

            thread::sleep(Duration::from_millis(500));
        }

        println!("\n\x1b[1;92m✔️ Teste finalizado.\x1b[0m");
    });
}
