#![allow(clippy::precedence)]

use std::env;

mod transmit;
mod receive;
mod cli;

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.get(1).is_none() {
        cli::help();
        return Err(anyhow::format_err!("Missing argument."));
    }

    match args.get(1).unwrap().as_str() {
        "transmit" => { 
            println!("transmitting...");
            transmit::transmit()?;
        },
        "receive"  => { 
            println!("receiving...");    
            receive::receive();
        },
        _          => cli::help()

    }

    Ok(())
}
