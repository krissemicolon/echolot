use crate::audio;
use crate::modulation;
use crate::modulation::quantise;
use crate::packets::{FileInfo, FileTransmission, Packet};
use circular_buffer::CircularBuffer;
use indicatif::ProgressBar;
use rtrb::{Consumer, RingBuffer};
use rustfft::{num_complex::Complex, FftPlanner};
use std::path::Path;
use std::slice;
use std::{fs, thread, time::Duration};

fn dominant_frequency(samples: &[f32], sample_rate: f32) -> f32 {
    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(samples.len());

    let mut buffer: Vec<Complex<f32>> = samples.iter().map(|&v| Complex::new(v, 0.0)).collect();

    fft.process(&mut buffer);

    let mut max_mag = 0.0;
    let mut max_index = 0;

    for (i, c) in buffer.iter().enumerate().take(samples.len() / 2) {
        let mag = c.re * c.re + c.im * c.im;

        if mag > max_mag {
            max_mag = mag;
            max_index = i;
        }
    }

    max_index as f32 * sample_rate / samples.len() as f32
}

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

    // Handshake: Agreement Part
    handshake_spinner.set_message("Listening for Receiver's Handshake Agreement");

    // gives ringbuffer time to populate with audio samples
    // takes approx 371.51ms with 44.1kHz
    thread::sleep(Duration::from_millis(500));

    let mut agreement_detected = false;

    let mut sliding_window = CircularBuffer::<16384, f32>::new();

    while !agreement_detected {
        if let Ok(chunk) = audio_input.consumer.read_chunk(512) {
            for sample in chunk {
                sliding_window.push_back(sample);
            }

            if sliding_window.is_full() {
                let freq = dominant_frequency(sliding_window.as_slices().0, 44100.0);
                if (freq - 3000.0).abs() <= 10.0 {
                    agreement_detected = true;
                }
            }
        } else {
            std::thread::yield_now();
        }
    }

    // Handshake Established
    handshake_spinner.finish_with_message("Established Handshake");

    // File Info
    let fileinfo_spinner = ProgressBar::new_spinner();
    fileinfo_spinner.set_message("Transmitting FileInfo");

    // Transmitting FileInfo
    audio_output.playback(modulation::modulate(file_info_packet_encoded));
    audio_output.sink.sleep_until_end();

    fileinfo_spinner.set_message("Listening for Confirmation");
    thread::sleep(Duration::from_millis(500)); // replace with confirmation demodulation

    fileinfo_spinner.finish_with_message("Received Confirmation");

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
