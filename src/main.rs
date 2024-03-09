mod packets;

use base64::{engine::general_purpose, Engine};
use clap::{Parser, Subcommand};
use indicatif::ProgressBar;
use packets::{FileInfo, FileTransmission, Initiation, Modulation};
use rodio::{
    cpal::{self, traits::HostTrait},
    source::SineWave,
    DeviceTrait, OutputStream, Sink, Source,
};
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
    let metadata = path
        .metadata()
        .unwrap_or_else(|e| panic!("Could not retrieve Metadata from file: {}", e));
    println!("{}", filename);
    println!("{:?}", metadata.len());
    let content = match fs::read(&path) {
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
    let base64_content = general_purpose::STANDARD.encode(content);

    let file_info_packet = FileInfo {
        file_name: filename,
        file_size: metadata.len(),
        base64_content_size: base64_content.len(),
        checksum: "TODO".to_string(),
    };
    let file_transmission_packet = FileTransmission { base64_content };
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
    audio_setup_spinner.set_message("Setting up Audio..");
    audio_setup_spinner.enable_steady_tick(Duration::from_millis(60));

    let default_device = match cpal::default_host().default_output_device() {
        Some(device) => device,
        None => {
            audio_setup_spinner.abandon_with_message("Unable to Access Current Audio Device");
            return;
        }
    };
    let (_stream, stream_handle) = match OutputStream::try_from_device(&default_device) {
        Ok(output_stream) => output_stream,
        Err(err) => {
            audio_setup_spinner
                .abandon_with_message(format!("Unable to Access Current Audio Device: {}", err));
            return;
        }
    };
    let sink = match Sink::try_new(&stream_handle) {
        Ok(sink) => sink,
        Err(err) => {
            audio_setup_spinner.abandon_with_message(format!(
                "Something went wrong while setting up Audio: {}",
                err
            ));
            return;
        }
    };

    // Print finish message with consideration for name retrieval returning Error
    if let Ok(name) = &default_device.name() {
        audio_setup_spinner.finish_with_message(format!("Using Audio Device: {}", name))
    } else {
        audio_setup_spinner.finish_with_message("Using Default Audio Device")
    }

    let transmission_progress = ProgressBar::new_spinner();
    transmission_progress.set_message(format!(
        "Transmitting File.. len {}/{}",
        0, file_info_packet.base64_content_size
    ));
    transmission_progress.enable_steady_tick(Duration::from_millis(60));
    //let transmission_progress = ProgressBar::new(encoded_content.len());
    //transmission_progress.set_message("Transmitting File..");

    let source = SineWave::new(440.0)
        .take_duration(Duration::from_secs_f32(0.25))
        .amplify(0.20);
    sink.append(source);
    sink.sleep_until_end();
    transmission_progress.finish_with_message("File has been transmitted");
}

fn receive() {
    let audio_setup_spinner = ProgressBar::new_spinner();
    audio_setup_spinner.set_message("Setting up Audio..");
    audio_setup_spinner.enable_steady_tick(Duration::from_millis(60));

    let default_device = match cpal::default_host().default_output_device() {
        Some(device) => device,
        None => {
            audio_setup_spinner.abandon_with_message("Unable to Access Current Audio Device");
            return;
        }
    };
    let (_stream, stream_handle) = match OutputStream::try_from_device(&default_device) {
        Ok(output_stream) => output_stream,
        Err(err) => {
            audio_setup_spinner
                .abandon_with_message(format!("Unable to Access Current Audio Device: {}", err));
            return;
        }
    };
    let sink = match Sink::try_new(&stream_handle) {
        Ok(sink) => sink,
        Err(err) => {
            audio_setup_spinner.abandon_with_message(format!(
                "Something went wrong while setting up Audio: {}",
                err
            ));
            return;
        }
    };

    // Print finish message with consideration for name retrieval returning Error
    if let Ok(name) = &default_device.name() {
        audio_setup_spinner.finish_with_message(format!("Using Audio Device: {}", name))
    } else {
        audio_setup_spinner.finish_with_message("Using Default Audio Device")
    }

    let handshake_spinner = ProgressBar::new_spinner();
    handshake_spinner.set_message("Establishing Handshake");
    handshake_spinner.enable_steady_tick(Duration::from_millis(60));
    /* */
    let initiation = Initiation;
    initiation
        .modulate()
        .into_iter()
        .for_each(|f| sink.append(f));
    sink.sleep_until_end();
    /* */
    handshake_spinner.finish_with_message("Established Handshake")
}
