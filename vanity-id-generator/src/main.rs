use std::{time::Instant, thread::JoinHandle};

use crossbeam_channel::{unbounded, Sender};
use qubic_types::QubicWallet;
use rand::Rng;
use clap::Parser;

#[macro_use]
extern crate log;

#[derive(Debug, Parser)]
struct Args {
    
    #[arg(short, long)]
    threads: usize,

    #[arg(short, long)]
    prefix: String
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::new().filter_level(log::LevelFilter::Info).init();
    let opt = Args::parse();
    let (tx, rx) = unbounded::<Update>();

    info!("Looking for an ID matching {}", opt.prefix);
    let expected = (26f64).powi(opt.prefix.len() as i32);
    info!("Expected iterations required: {}", expected);
    
    let mut now = Instant::now();
    info!("Starting {} worker threads", opt.threads);
    let _ = start_threads(opt.threads, opt.prefix, tx);

    let mut total = 0;
    let mut im = 0;

    'outer: loop {
        std::thread::sleep(std::time::Duration::from_secs(3));
        if !rx.is_empty() {
            while !rx.is_empty() {
                let recv = rx.recv().unwrap();

                match recv {
                    Update::Match(a) => {
                        info!("Found matching seed: {a}");

                        break 'outer;
                    },
                    Update::Checks(sims) => {
                        total += sims;
                        im += sims;
                    } 
                }
            }
        }

        let checks_per_second = (im * 1000) as f64 / now.elapsed().as_millis() as f64;

        info!("Expected Progress: {:.2}% | Time left (est): {:.0}s | Checkrate: {:.2} ch/s", (total as f64/expected)*100f64, (expected - total as f64)/checks_per_second, checks_per_second);

        now = Instant::now();
        im = 0;
    }

    Ok(())
}

enum Update {
    Checks(u64),
    Match(String)
}

fn start_threads(threads: usize, match_value: String, tx: Sender<Update>) -> Vec<JoinHandle<()>> {
    let mut handles = Vec::new();
    for _ in 0..threads {
        let tx = tx.clone();
        let mv = match_value.clone();
        handles.push(std::thread::spawn(move || {
            let tx = tx;
            let mut i = 0;
            let mut now = Instant::now();
            loop {
                let seed = get_random_seed();
                let id = QubicWallet::from_seed(&seed).unwrap();

                if id.get_identity().starts_with(&mv) {
                    println!("Match Found for seed {seed}");

                    tx.send(Update::Match(seed)).unwrap();
                    break;
                }

                i += 1;

                if now.elapsed().as_secs() > 2 {
                    tx.send(Update::Checks(i)).unwrap();

                    i = 0;
                    now = Instant::now();
                }
            }
        }));
    }

    handles
}

pub fn get_random_seed() -> String {
    let mut seed: [u8; 55] = [0; 55];

    let mut rng = rand::thread_rng();

    for s in seed.iter_mut() {
        *s = b'a' + rng.gen::<u8>() % 26;
    }

    String::from_utf8(seed.to_vec()).unwrap()
}