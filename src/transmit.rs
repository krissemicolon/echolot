use crate::audio;
use crate::modulation;
use crate::packets::{FileInfo, FileTransmission, Packet};
use indicatif::ProgressBar;
use modulation::FFT_WINDOW;
use std::path::Path;
use std::{fs, thread, time::Duration};

pub fn transmit(path: &Path) {
    // Readying Packets
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
    let checksum = crc32fast::hash(&file);

    let file_info_packet = FileInfo {
        file_name: filename.clone(),
        file_size: filesize,
        checksum: crc32fast::hash(&file),
    };
    let file_info_packet_encoded = file_info_packet.encode();

    let file_transmission_packet = FileTransmission { file, checksum };
    let file_transmission_packet_encoded = file_transmission_packet.encode();

    packets_prep_spinner.finish_with_message("Packets are Ready");

    // Setting up Audio Output
    let audio_output_setup_spinner = ProgressBar::new_spinner();
    audio_output_setup_spinner.set_message("Setting up Audio Output..");
    audio_output_setup_spinner.enable_steady_tick(Duration::from_millis(60));

    let mut audio_output = match audio::AudioOutputDevice::default() {
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

    // Setting up Audio Input
    let audio_input_setup_spinner = ProgressBar::new_spinner();
    audio_input_setup_spinner.set_message("Setting up Input Output..");
    audio_input_setup_spinner.enable_steady_tick(Duration::from_millis(60));

    let mut audio_input = match audio::AudioInputDevice::default() {
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

    // Handshake: Initiation Part
    let handshake_spinner = ProgressBar::new_spinner();
    handshake_spinner.enable_steady_tick(Duration::from_millis(60));

    match audio_input.start() {
        Ok(_) => handshake_spinner.set_message("Listening for Receiver's Handshake Initiation"),
        Err(e) => panic!("Could Not Start Listening To Microphone: {}", e),
    }

    loop {
        thread::sleep(Duration::from_millis(1000));
        println!(
            "{:?}",
            audio_input
                .consumer
                .read_chunk(FFT_WINDOW)
                .unwrap()
                .as_slices()
        );
    }

    /*
    processor::process_audio_samples(audio_input.consumer);

    loop {
        println!(
            "{:?}",
            audio_input
                .consumer
                .read_chunk(FFT_WINDOW)
                .unwrap()
                .as_slices()
        );
    }
    let calc_freq = || {
        vec![freq_fft(
            audio_input.consumer.buffer(),
            audio_input.sample_rate.0,
        )]
    };
    audio_input.update_buffer();
    while demodulate(calc_freq(), Packet::Control(ControlPacket::Initiation)).is_none() {
        audio_input.update_buffer();
    }*/

    // Handshake: Agreement Part
    handshake_spinner.set_message("Listening for Receiver's Handshake Agreement");

    thread::sleep(Duration::from_millis(500)); // replace with agreement demodulation

    // Handshake Established
    handshake_spinner.finish_with_message("Established Handshake");

    // Transmitting FileInfo
    audio_output.playback(modulation::modulate(file_info_packet_encoded));
    audio_output.sink.sleep_until_end();

    thread::sleep(Duration::from_millis(500)); // replace with confirmation demodulation

    let transmission_size = &file_transmission_packet_encoded.len();
    let transmission_progress = ProgressBar::new(filesize);
    transmission_progress.set_message(format!(
        "Transmitting {}.. {}B/{}B",
        &filename, 0, transmission_size
    ));
    transmission_progress.enable_steady_tick(Duration::from_millis(60));

    audio_output.playback(modulation::modulate(file_transmission_packet_encoded));
    audio_output.sink.sleep_until_end();

    transmission_progress.finish_with_message(format!("{} has been transmitted", &filename));
}
