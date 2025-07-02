use std::sync::atomic::{AtomicBool, Ordering};
use std::io::{self, Write};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

pub fn show_loader(running: Arc<AtomicBool>, duration_secs: u64, start_time: Instant) {
    thread::spawn(move || {
        let bar_length = 30;
        let mut pos = 0;
        let mut direction = 1;

        while running.load(Ordering::SeqCst) {
            let elapsed = start_time.elapsed().as_secs();
            let remaining = if elapsed >= duration_secs {
                0
            } else {
                duration_secs - elapsed
            };

            let mut bar = String::new();
            for i in 0..bar_length {
                if i == pos {
                    bar.push_str("\x1b[44m \x1b[0m");
                } else {
                    bar.push(' ');
                }
            }

            print!(
                "\rExecutando teste... [{}] Tempo restante: {:02}s ",
                bar, remaining
            );
            io::stdout().flush().unwrap();

            if pos == bar_length - 1 {
                direction = -1;
            } else if pos == 0 {
                direction = 1;
            }
            pos = (pos as i32 + direction) as usize;

            thread::sleep(Duration::from_millis(100));
        }

        println!("\rTeste finalizado.                            ");
    });
}
