use std::env;
use std::process::exit;

mod encode;
mod decode;
mod cli;

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.get(1).is_none() {
        cli::help();
        return Err(anyhow::format_err!("Missing argument."));
    }
    
    match args.get(1).unwrap().as_str() {
        "transmit" => println!("transmitting"),
        "receive"  => println!("receiving"),
        _          => cli::help()
    }

    Ok(())
}
