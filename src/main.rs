use std::env;
use std::process::exit;

mod transmit;
mod receive;
mod cli;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.get(1).is_none() {
        cli::help();
        exit(1);
    }

    match args.get(1).unwrap().as_str() {
        "transmit" => { println!("transmitting..."); transmit::transmit(); },
        "receive"  => { println!("receiving...");    receive::receive(); },
        _          => cli::help()
    }
}