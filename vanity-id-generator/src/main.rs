use std::{time::Instant, thread::JoinHandle};

use crossbeam_channel::{unbounded, Sender};
use qubic_types::{QubicWallet, QubicId};
use rand::Rng;
use structopt::StructOpt;

#[macro_use]
extern crate log;

#[derive(Debug, StructOpt)]
struct Opt {
    
    #[structopt(short="t")]
    threads: usize,

    #[structopt(short="m", long="match")]
    match_value: String
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::new().filter_level(log::LevelFilter::Info).init();
    let opt = Opt::from_args();
    let (tx, rx) = unbounded::<Update>();

    info!("Looking for an ID matching {}", opt.match_value);
    let expected = (26f64).powi(opt.match_value.len() as i32);
    info!("Expected iterations required: {}", expected);
    
    let mut now = Instant::now();
    info!("Starting {} worker threads", opt.threads);
    let _ = start_threads(opt.threads, opt.match_value, tx);

    let mut total = 0;
    let mut im = 0;

    'outer: loop {
        if !rx.is_empty() {
            while !rx.is_empty() {
                let recv = rx.recv().unwrap();

                match recv {
                    Update::Match(a) => {
                        info!("Found matching seed: {a}");

                        break 'outer;
                    },
                    Update::Sims(sims) => {
                        total += sims;
                        im += sims;
                    } 
                }
            }
        }

        info!("Expected Progress: {:.2}% | Checkrate: {:.2} ch/s", (total as f64/expected), (im * 1000) as f64 / now.elapsed().as_millis() as f64);

        now = Instant::now();
        im = 0;

        std::thread::sleep(std::time::Duration::from_secs(3));
    }

    Ok(())
}

enum Update {
    Sims(u64),
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
                    tx.send(Update::Sims(i)).unwrap();

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

    for i in 0..55 {
        seed[i] = b'a' + rng.gen::<u8>() % 26;
    }

    String::from_utf8(seed.to_vec()).unwrap()
}

#[inline(always)]
pub fn get_identity(seed: &[u8]) -> [u8; 60] {
    let ss = QubicWallet::get_subseed_unchecked(&seed);
    let pk = QubicWallet::get_private_key(&ss);
    let id = QubicId(QubicWallet::get_public_key(&pk));

    id.get_identity_bytes()
}