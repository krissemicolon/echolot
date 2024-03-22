mod audio_devices;
mod codec;
mod fft;
mod frequency;
mod modulation;
mod packets;
mod playback;

use clap::{Parser, Subcommand};
use indicatif::ProgressBar;
use packets::{ControlPacket, FileInfo, FileTransmission, Packet};
use playback::playback;
use std::{fs, path::Path, thread, time::Duration};

use crate::{
    codec::Codec,
    modulation::modulate,
    packets::{get_binary_data, Response},
};

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
        .map(|m| m.len())
        .unwrap_or_else(|| panic!("Could not retrieve Metadata from \"{filename}\""));
    let file = match fs::read(path) {
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
        file_name: filename.clone(),
        file_size: filesize,
        checksum: crc32fast::hash(&file),
    };
    let file_info_packet_encoded = file_info_packet.encode();

    let file_transmission_packet = FileTransmission { file };
    let file_transmission_packet_encoded = file_transmission_packet.encode();

    let response_packet = Response {
        file_info_size: get_binary_data(&file_info_packet_encoded).unwrap().len(),
    };
    let response_packet_encoded = response_packet.encode();

    packets_prep_spinner.finish_with_message("Packets are Ready");

    let audio_output_setup_spinner = ProgressBar::new_spinner();
    audio_output_setup_spinner.set_message("Setting up Audio Output..");
    audio_output_setup_spinner.enable_steady_tick(Duration::from_millis(60));

    let audio_output = match audio_devices::AudioOutputDevice::default() {
        Ok(audio_output) => {
            audio_output_setup_spinner
                .finish_with_message(format!("Using Audio Output Device: {}", audio_output.name));
            audio_output
        }
        Err(err) => {
            audio_output_setup_spinner.abandon_with_message(err);
            return;
        }
    };

    let audio_input_setup_spinner = ProgressBar::new_spinner();
    audio_input_setup_spinner.set_message("Setting up Input Output..");
    audio_input_setup_spinner.enable_steady_tick(Duration::from_millis(60));

    let audio_input = match audio_devices::AudioInputDevice::default() {
        Ok(audio_input) => {
            audio_input_setup_spinner
                .finish_with_message(format!("Using Audio Input Device: {}", audio_input.name));
            audio_input
        }
        Err(err) => {
            audio_input_setup_spinner.abandon_with_message(err);
            return;
        }
    };

    let handshake_spinner = ProgressBar::new_spinner();

    match audio_input.start() {
        Ok(_) => handshake_spinner.set_message("Listening for Receiver's Handshake Initiation"),
        Err(e) => panic!("Could not start microphone input: {}", e),
    }

    handshake_spinner.set_message("Listening for Receiver's Handshake Initiation");
    handshake_spinner.enable_steady_tick(Duration::from_millis(60));

    thread::sleep(Duration::from_millis(5000)); // replace with initiation demodulation

    handshake_spinner.set_message("Received Initiation. Transmitting Response");

    playback(
        modulate(&response_packet_encoded)
            .into_iter()
            .map(|f| f.sine_wave)
            .collect(),
        0.25,
        Duration::from_millis(BYTE_DURATION_MS),
        &audio_output,
    );
    audio_output.sink.sleep_until_end();
    handshake_spinner.set_message("Listening for Receiver's Handshake Agreement");

    thread::sleep(Duration::from_millis(500)); // replace with agreement demodulation
                                               /*
                                                   audio_input
                                                       .stream
                                                       .play()
                                                       .expect("Could Not Start Listening from Audio Input");
                                                   // Let recording go for roughly three seconds.
                                                   std::thread::sleep(std::time::Duration::from_secs(3));
                                                   drop(audio_input.stream);
                                               */
    handshake_spinner.finish_with_message("Established Handshake");

    playback(
        modulate(&file_info_packet_encoded)
            .into_iter()
            .map(|f| f.sine_wave)
            .collect(),
        0.25,
        Duration::from_millis(BYTE_DURATION_MS),
        &audio_output,
    );
    audio_output.sink.sleep_until_end();

    thread::sleep(Duration::from_millis(500)); // replace with confirmation demodulation

    let transmission_size = get_binary_data(&file_transmission_packet_encoded)
        .unwrap()
        .len();
    let transmission_progress = ProgressBar::new(filesize);
    transmission_progress.set_message(format!(
        "Transmitting {}.. {}B/{}B",
        &filename, 0, transmission_size
    ));
    transmission_progress.enable_steady_tick(Duration::from_millis(60));

    playback(
        modulate(&file_transmission_packet_encoded)
            .into_iter()
            .map(|f| f.sine_wave)
            .collect(),
        0.25,
        Duration::from_millis(BYTE_DURATION_MS),
        &audio_output,
    );
    audio_output.sink.sleep_until_end();

    transmission_progress.finish_with_message(format!("{} has been transmitted", &filename));
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

    let initation_packet = Packet::Control(ControlPacket::Initiation);

    playback(
        modulate(&initation_packet)
            .into_iter()
            .map(|f| f.sine_wave)
            .collect(),
        0.25,
        Duration::from_millis(BYTE_DURATION_MS),
        &audio_output,
    );

    handshake_spinner.finish_with_message("Established Handshake");
}
