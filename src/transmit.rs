use crate::frequency::Frequency;
use crate::packets::{FileInfo, FileTransmission, Packet};
use crate::quantise::is_within_tolerance_to;
use crate::{
    CONFIRMATION_ACCEPT_FREQ, CONFIRMATION_DENY_FREQ, STD_TOLERANCE, SYMBOL_DURATION_MS, audio,
};
use crate::{EOT_FREQ, pitch_detection};
use crate::{SOT_FREQ, modulation};
use indicatif::ProgressBar;
use std::path::Path;
use std::process::exit;
use std::thread::sleep;
use std::{fs, time::Duration};
use std::{
    sync::atomic::Ordering,
    time::{Instant, SystemTime},
};

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
                .finish_with_message(format!(
                    "Using Audio Output Device: {} ({}Hz, {}ch)",
                    audio_output.name, audio_output.sample_rate.0, audio_output.channels
                ));
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
                .finish_with_message(format!(
                    "Using Audio Input Device: {} ({}Hz, {}ch)",
                    audio_input.name, audio_input.sample_rate.0, audio_input.channels
                ));
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

    if let Err(e) = audio_input.start() {
        fileinfo_spinner.abandon_with_message(format!("Could not start microphone: {}", e));
        return;
    }

    let half_symbol_samples = audio::half_symbol_samples(audio_input.sample_rate);
    let mut sample_window: Vec<f32> = Vec::with_capacity(half_symbol_samples);
    let mut detector =
        pitch_detection::DominantFrequencyDetector::new(half_symbol_samples, audio_input.sample_rate);
    let started_waiting = Instant::now();
    let confirmation_timeout = Duration::from_secs(30);
    let mut last_drop_report = SystemTime::UNIX_EPOCH;
    let mut confirmation_received = false;

    loop {
        if let Ok(chunk) = audio_input.consumer.read_chunk(512) {
            for sample in chunk {
                sample_window.push(sample);

                if sample_window.len() == half_symbol_samples {
                    let freq = detector.detect(&sample_window).unwrap_or(0.0);
                    sample_window.clear();

                    if is_within_tolerance_to(freq, CONFIRMATION_ACCEPT_FREQ, STD_TOLERANCE) {
                        fileinfo_spinner.finish_with_message("Received Confirmation");
                        confirmation_received = true;
                        break;
                    } else if is_within_tolerance_to(freq, CONFIRMATION_DENY_FREQ, STD_TOLERANCE) {
                        fileinfo_spinner.finish_with_message("Receiver denied file");
                        exit(0);
                    }
                }
            }
        } else {
            sleep(Duration::from_millis(1));
        }

        if started_waiting.elapsed() > confirmation_timeout {
            fileinfo_spinner.abandon_with_message("Timed out waiting for receiver confirmation");
            return;
        }
        if confirmation_received {
            break;
        }

        if let Ok(now) = SystemTime::now().duration_since(last_drop_report) {
            if now >= Duration::from_secs(2) {
                let dropped = audio_input.dropped_samples.load(Ordering::Relaxed);
                if dropped > 0 {
                    eprintln!("Warning: input dropped {} samples while listening", dropped);
                }
                last_drop_report = SystemTime::now();
            }
        }
    }
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
