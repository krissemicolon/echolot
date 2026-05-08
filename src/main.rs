mod audio;
mod error_correction;
mod frequency;
mod modulation;
mod packets;
mod pitch_detection;
mod quantise;
mod receive;
mod transmit;

use crate::receive::receive;
use crate::transmit::transmit;
use clap::{Parser, Subcommand};
use std::path::Path;

const SYMBOL_DURATION_MS: u64 = 120;
const STD_TOLERANCE: f32 = 20.0;

// Modulation
const MOD_OFFSET: f32 = 1024.0;
const MOD_STEP_SIZE: f32 = 256.0;

// Control Frequencies
const SOT_FREQ: f32 = 8192.0;
const EOT_FREQ: f32 = 8704.0;
const CONFIRMATION_ACCEPT_FREQ: f32 = 3100.0;
const CONFIRMATION_DENY_FREQ: f32 = 4100.0;

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
