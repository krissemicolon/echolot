use crate::frequency::Frequency;
use crate::packets::{FileInfo, FileTransmission, Packet};
use crate::quantise::{is_within_tolerance_to, quantise_to_codec};
use crate::{
    CONFIRMATION_ACCEPT_FREQ, CONFIRMATION_DENY_FREQ, STD_TOLERANCE, SYMBOL_DURATION_MS, audio,
};
use crate::{EOT_FREQ, pitch_detection};
use crate::{SOT_FREQ, modulation};
use indicatif::ProgressBar;
use ringbuf::HeapRb;
use ringbuf::traits::{Consumer, Observer, RingBuffer};
use std::path::Path;
use std::process::exit;
use std::thread::sleep;
use std::time::Instant;
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

    // FileInfo Transmission
    let fileinfo_spinner = ProgressBar::new_spinner();
    fileinfo_spinner.set_message("Transmitting FileInfo");

    let fileinfo_freqs = modulation::modulate(file_info_packet_encoded);

    audio_output.playback(vec![Frequency::new(SOT_FREQ)]);
    audio_output.playback(fileinfo_freqs);
    audio_output.playback(vec![Frequency::new_with_len(EOT_FREQ, 500)]);

    audio_output.sink.sleep_until_end();

    // Awaiting Confirmation/Denial
    fileinfo_spinner.set_message("Listening for Confirmation");

    let num_samples =
        (((SYMBOL_DURATION_MS as f32 / 1000.0) / 2.0) * audio_input.sample_rate.0 as f32) as usize;
    let mut interval_samples = HeapRb::<f32>::new(num_samples);
    let interval = Duration::from_millis(SYMBOL_DURATION_MS as u64 / 2);
    let mut next_tick = Instant::now();

    loop {
        if let Ok(chunk) = audio_input.consumer.read_chunk(512) {
            for sample in chunk {
                interval_samples.push_overwrite(sample);
            }
        }

        if Instant::now() >= next_tick {
            next_tick += interval;
            let samples: Vec<f32> = interval_samples.iter().copied().collect::<Vec<f32>>();
            let freq = pitch_detection::dominant_frequency(&samples, audio_input.sample_rate);

            if is_within_tolerance_to(freq, CONFIRMATION_ACCEPT_FREQ, STD_TOLERANCE) {
                break;
            } else if is_within_tolerance_to(freq, CONFIRMATION_DENY_FREQ, STD_TOLERANCE) {
                fileinfo_spinner.finish_with_message("Receiver denied file");
                exit(0);
            }
        }
    }
    fileinfo_spinner.finish_with_message("Received Confirmation");
    sleep(Duration::from_millis(SYMBOL_DURATION_MS));

    let transmission_size = &file_transmission_packet_encoded.len();
    let transmission_progress = ProgressBar::new(filesize);
    transmission_progress.set_message(format!(
        "Transmitting {}.. {}B/{}B",
        &filename, 0, transmission_size
    ));
    transmission_progress.enable_steady_tick(Duration::from_millis(60));

    audio_output.playback(vec![Frequency::new(SOT_FREQ)]);
    audio_output.playback(modulation::modulate(file_transmission_packet_encoded));
    audio_output.playback(vec![Frequency::new_with_len(EOT_FREQ, 500)]);
    audio_output.sink.sleep_until_end();

    transmission_progress.set_position((transmission_size - audio_output.playback_len()) as u64);

    transmission_progress.finish_with_message(format!("{} has been transmitted", &filename));
}
