mod audio;
mod frequency;
mod modulation;
mod packets;
mod pitch_detection;
mod receive;
mod transmit;

use crate::receive::receive;
use crate::transmit::transmit;
use clap::{Parser, Subcommand};
use std::path::Path;

const BYTE_DURATION_MS: u64 = 100;

#[derive(Parser)]
#[clap(version, about)]
pub struct Args {
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Transmit a File
    #[command(arg_required_else_help = true)]
    #[clap(aliases = &["t"])]
    Transmit {
        #[arg(required = true)]
        file: String,
    },
    /// Receive a File
    #[clap(aliases = &["r"])]
    Receive,
}

fn main() {
    let args = Args::parse();

    match args.command {
        Commands::Transmit { file } => {
            transmit(Path::new(&file));
        }
        Commands::Receive => {
            receive();
        }
    }
}
