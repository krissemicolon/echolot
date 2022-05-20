use std::env;
use std::process::exit;

mod encode;
mod decode;
mod cli;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.get(1).is_none() {
        cli::help();
        exit(1);
    }

    match args.get(1).unwrap().as_str() {
        "transmit" => println!("transmitting"),
        "receive"  => println!("receiving"),
        _          => cli::help()
    }
}