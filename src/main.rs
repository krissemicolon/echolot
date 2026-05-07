mod audio;
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

const BYTE_DURATION_MS: u64 = 200;
const STD_TOLERANCE: f32 = 10.0;

// Modulation
const MOD_OFFSET: f32 = 300.0;
const MOD_STEP_SIZE: f32 = 150.0;

// Control Frequencies
const EOT_FREQ: f32 = 3150.0;
const HANDSHAKE_RECEIVER_FREQ: f32 = 3000.0;
const HANDSHAKE_TRANSMITTER_FREQ: f32 = 3050.0;
const CONFIRMATION_FREQ: f32 = 3100.0;
const PREAMBLE_FIRST_FREQ: f32 = 3400.0;
const PREAMBLE_SECOND_FREQ: f32 = 2400.0;
const PREAMBLE_THIRD_FREQ: f32 = 2800.0;

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
