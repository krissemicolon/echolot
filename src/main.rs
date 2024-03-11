mod audio_devices;
mod codec;
mod frequency;
mod modulation;
mod packets;

use clap::{Parser, Subcommand};
use indicatif::ProgressBar;
use packets::{FileInfo, FileTransmission, Initiation, Packet};
use rodio::{source::SineWave, Source};
use std::{fs, path::Path, thread, time::Duration};

use crate::packets::Response;

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
    Transmit {
        #[arg(required = true)]
        file: String,
    },
    /// Receive a File
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

fn transmit(path: &Path) {
    let packets_prep_spinner = ProgressBar::new_spinner();
    packets_prep_spinner.set_message(format!("Readying Packets for {}..", path.display()));
    packets_prep_spinner.enable_steady_tick(Duration::from_millis(60));

    let filename = path
        .file_name()
        .and_then(|f| f.to_str())
        .unwrap_or_else(|| panic!("Unsupported Filename: {:?}", path))
        .to_owned();
    let filesize = path
        .metadata()
        .ok()
        .and_then(|m| Some(m.len()))
        .unwrap_or_else(|| panic!("Could not retrieve Metadata from \"{filename}\""));
    println!("{}", filename);
    println!("{}", filesize);
    let file = match fs::read(&path) {
        Ok(contents) => contents,
        Err(err) => {
            packets_prep_spinner.abandon_with_message(format!(
                "Unable to open the file '{}': {}",
                path.display(),
                err
            ));
            return;
        }
    };

    let file_info_packet = FileInfo {
        file_name: filename,
        file_size: filesize,
        checksum: "TODO".to_string(),
    };
    let file_transmission_packet = FileTransmission { file };
    let response_packet = Response {
        file_info_size: todo!(),
    };
    packets_prep_spinner.finish_with_message("Packets are Ready");

    let handshake_spinner = ProgressBar::new_spinner();
    handshake_spinner.set_message("Listening for Receiver's Handshake Initiation");
    handshake_spinner.enable_steady_tick(Duration::from_millis(60));

    thread::sleep(Duration::from_millis(500));

    handshake_spinner.finish_with_message("Established Handshake");

    let audio_setup_spinner = ProgressBar::new_spinner();
    audio_setup_spinner.set_message("Setting up Audio Output..");
    audio_setup_spinner.enable_steady_tick(Duration::from_millis(60));

    let audio_output = match audio_devices::AudioOutputDevice::default() {
        Ok(audio_output) => {
            audio_setup_spinner
                .finish_with_message(format!("Using Audio Output Device: {}", audio_output.name));
            audio_output
        }
        Err(err) => {
            audio_setup_spinner.abandon_with_message(err);
            return;
        }
    };

    let transmission_progress = ProgressBar::new_spinner();
    transmission_progress.set_message(format!("Transmitting File.. len {}/{}", 0, filesize));
    transmission_progress.enable_steady_tick(Duration::from_millis(60));
    //let transmission_progress = ProgressBar::new(encoded_content.len());
    //transmission_progress.set_message("Transmitting File..");

    let source = SineWave::new(440.0)
        .take_duration(Duration::from_secs_f32(0.25))
        .amplify(0.20);
    audio_output.sink.append(source);
    audio_output.sink.sleep_until_end();
    transmission_progress.finish_with_message("File has been transmitted");
}

fn receive() {
    let audio_setup_spinner = ProgressBar::new_spinner();
    audio_setup_spinner.set_message("Setting up Audio Output..");
    audio_setup_spinner.enable_steady_tick(Duration::from_millis(60));
    let audio_output = match audio_devices::AudioOutputDevice::default() {
        Ok(audio_output) => {
            audio_setup_spinner
                .finish_with_message(format!("Using Audio Output Device: {}", audio_output.name));
            audio_output
        }
        Err(err) => {
            audio_setup_spinner.abandon_with_message(err);
            return;
        }
    };

    let handshake_spinner = ProgressBar::new_spinner();
    handshake_spinner.set_message("Establishing Handshake");
    handshake_spinner.enable_steady_tick(Duration::from_millis(60));
    /*
    let initiation = Initiation;
    initiation
        .modulate()
        .into_iter()
        .for_each(|f| audio_output.sink.append(f));
    audio_output.sink.sleep_until_end();
    */
    handshake_spinner.finish_with_message("Established Handshake")
}
