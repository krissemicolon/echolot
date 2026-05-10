use crate::{
    CONFIRMATION_ACCEPT_FREQ, CONFIRMATION_DENY_FREQ, EOT_FREQ, SOT_FREQ, STD_TOLERANCE,
    SYMBOL_DURATION_MS, audio,
    frequency::Frequency,
    modulation::demodulate,
    packets::{FileInfo, FileTransmission, Packet},
    pitch_detection,
    quantise::{is_within_tolerance_to, quantise_to_codec},
};
use indicatif::{HumanBytes, ProgressBar};
use inquire::{InquireError, Select};
use std::fs;
use std::{
    panic::{AssertUnwindSafe, catch_unwind},
    process::exit,
    sync::atomic::Ordering,
    thread::sleep,
    time::{Duration, Instant, SystemTime},
};

pub fn receive() {
    // Setting up Audio Output
    let audio_setup_spinner = ProgressBar::new_spinner();
    audio_setup_spinner.set_message("Setting up Audio Output..");
    audio_setup_spinner.enable_steady_tick(Duration::from_millis(60));
    let mut audio_output = match audio::AudioOutputDevice::default() {
        Ok(audio_output) => {
            audio_setup_spinner
                .finish_with_message(format!(
                    "Using Audio Output Device: {} ({}Hz, {}ch)",
                    audio_output.name, audio_output.sample_rate.0, audio_output.channels
                ));
            audio_output
        }
        Err(err) => {
            audio_setup_spinner.abandon_with_message(err);
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

    if let Err(e) = audio_input.start() {
        eprintln!("Could not start microphone: {}", e);
        return;
    }

    let fileinfo_spinner = ProgressBar::new_spinner();
    fileinfo_spinner.enable_steady_tick(Duration::from_millis(120));
    fileinfo_spinner.set_message("Listening for FileInfo Transmission");

    let half_symbol_samples = audio::half_symbol_samples(audio_input.sample_rate);
    let mut sample_window: Vec<f32> = Vec::with_capacity(half_symbol_samples);
    let mut detector =
        pitch_detection::DominantFrequencyDetector::new(half_symbol_samples, audio_input.sample_rate);
    let mut in_packet = false;
    let mut packet_complete = false;
    let mut fileinfo_freqs: Vec<Frequency> = vec![];
    let wait_timeout = Duration::from_secs(60);
    let packet_idle_timeout = Duration::from_secs(8);
    let started_waiting = Instant::now();
    let mut last_symbol_at = Instant::now();
    let mut last_drop_report = SystemTime::UNIX_EPOCH;

    loop {
        if let Ok(chunk) = audio_input.consumer.read_chunk(512) {
            for sample in chunk {
                sample_window.push(sample);
                if sample_window.len() == half_symbol_samples {
                    last_symbol_at = Instant::now();
                    let freq = detector.detect(&sample_window).unwrap_or(0.0);
                    sample_window.clear();

                    // EOT Detection
                    if in_packet && is_within_tolerance_to(freq, EOT_FREQ, STD_TOLERANCE) {
                        in_packet = false;
                        packet_complete = true;
                        break;
                    }

                    if in_packet {
                        fileinfo_freqs.push(Frequency::new(quantise_to_codec(freq)));
                    }

                    // SOT Detection
                    if !in_packet && is_within_tolerance_to(freq, SOT_FREQ, STD_TOLERANCE) {
                        fileinfo_spinner.set_message("Receiving FileInfo Transmission");
                        in_packet = true;
                    }
                }
            }
        } else {
            sleep(Duration::from_millis(1));
        }

        if !in_packet && started_waiting.elapsed() > wait_timeout {
            fileinfo_spinner.abandon_with_message("Timed out waiting for FileInfo SOT");
            return;
        }
        if in_packet && last_symbol_at.elapsed() > packet_idle_timeout {
            fileinfo_spinner.abandon_with_message("FileInfo reception timed out before EOT");
            return;
        }

        if let Ok(now) = SystemTime::now().duration_since(last_drop_report) {
            if now >= Duration::from_secs(2) {
                let dropped = audio_input.dropped_samples.load(Ordering::Relaxed);
                if dropped > 0 {
                    eprintln!("Warning: input dropped {} samples during FileInfo", dropped);
                }
                last_drop_report = SystemTime::now();
            }
        }

        if packet_complete {
            break;
        }
    }

    let processed_fileinfo_freqs: Vec<Frequency> = fileinfo_freqs
        .chunks_exact(2)
        .map(|pair| pair[1].clone())
        .collect();

    let Some(fileinfo_bytes) = demodulate(processed_fileinfo_freqs) else {
        fileinfo_spinner.abandon_with_message("Could not demodulate FileInfo packet");
        return;
    };
    let received_fileinfo = match catch_unwind(AssertUnwindSafe(|| FileInfo::decode(fileinfo_bytes)))
    {
        Ok(packet) => packet,
        Err(_) => {
            fileinfo_spinner.abandon_with_message("Failed to decode FileInfo packet");
            return;
        }
    };

    fileinfo_spinner.finish_with_message("Received File Info Packet");

    sleep(Duration::from_millis(SYMBOL_DURATION_MS));

    // Present Receiver with File Information
    // & Confirm/Deny Transmission
    let options: Vec<&str> = vec!["Accept", "Deny"];

    let ans: Result<&str, InquireError> = Select::new(
        &format!(
            "Do you accept '{}' with size {}",
            received_fileinfo.file_name,
            HumanBytes(received_fileinfo.file_size)
        ),
        options,
    )
    .prompt();

    match ans {
        Ok(choice) => match choice {
            "Accept" => {
                audio_output.playback(vec![Frequency::new(CONFIRMATION_ACCEPT_FREQ)]);
                audio_output.sink.sleep_until_end();
            }
            "Deny" => {
                audio_output.playback(vec![Frequency::new(CONFIRMATION_DENY_FREQ)]);
                audio_output.sink.sleep_until_end();
                exit(0)
            }
            _ => (),
        },
        Err(_) => println!("There was an error, please try again"),
    }

    sleep(Duration::from_millis(SYMBOL_DURATION_MS));

    // Receiving File
    let filetransmission_spinner = ProgressBar::new_spinner();
    filetransmission_spinner.enable_steady_tick(Duration::from_millis(120));
    filetransmission_spinner.set_message("Listening for File Transmission");

    let half_symbol_samples = audio::half_symbol_samples(audio_input.sample_rate);
    let mut sample_window: Vec<f32> = Vec::with_capacity(half_symbol_samples);
    let mut detector =
        pitch_detection::DominantFrequencyDetector::new(half_symbol_samples, audio_input.sample_rate);
    let mut in_packet = false;
    let mut packet_complete = false;
    let mut file_freqs: Vec<Frequency> = vec![];
    let started_waiting = Instant::now();
    let mut last_symbol_at = Instant::now();

    loop {
        if let Ok(chunk) = audio_input.consumer.read_chunk(512) {
            for sample in chunk {
                sample_window.push(sample);
                if sample_window.len() == half_symbol_samples {
                    last_symbol_at = Instant::now();
                    let freq = detector.detect(&sample_window).unwrap_or(0.0);
                    sample_window.clear();

                    // EOT Detection
                    if in_packet && is_within_tolerance_to(freq, EOT_FREQ, STD_TOLERANCE) {
                        in_packet = false;
                        packet_complete = true;
                        break;
                    }

                    if in_packet {
                        file_freqs.push(Frequency::new(quantise_to_codec(freq)));
                    }

                    // SOT Detection
                    if !in_packet && is_within_tolerance_to(freq, SOT_FREQ, STD_TOLERANCE) {
                        filetransmission_spinner.set_message("Receiving File Transmission");
                        in_packet = true;
                    }
                }
            }
        } else {
            sleep(Duration::from_millis(1));
        }

        if !in_packet && started_waiting.elapsed() > wait_timeout {
            filetransmission_spinner.abandon_with_message("Timed out waiting for File SOT");
            return;
        }
        if in_packet && last_symbol_at.elapsed() > packet_idle_timeout {
            filetransmission_spinner.abandon_with_message("File reception timed out before EOT");
            return;
        }

        if let Ok(now) = SystemTime::now().duration_since(last_drop_report) {
            if now >= Duration::from_secs(2) {
                let dropped = audio_input.dropped_samples.load(Ordering::Relaxed);
                if dropped > 0 {
                    eprintln!("Warning: input dropped {} samples during file payload", dropped);
                }
                last_drop_report = SystemTime::now();
            }
        }

        if packet_complete {
            break;
        }
    }
    filetransmission_spinner
        .set_message(format!("Received '{}' File", received_fileinfo.file_name));

    let processed_file_freqs: Vec<Frequency> = file_freqs
        .chunks_exact(2)
        .map(|pair| pair[1].clone())
        .collect();

    let Some(file_bytes) = demodulate(processed_file_freqs) else {
        filetransmission_spinner.abandon_with_message("Could not demodulate File packet");
        return;
    };
    let received_file = match catch_unwind(AssertUnwindSafe(|| FileTransmission::decode(file_bytes)))
    {
        Ok(packet) => packet,
        Err(_) => {
            filetransmission_spinner.abandon_with_message("Failed to decode file packet");
            return;
        }
    };
    let computed_checksum = crc32fast::hash(&received_file.file);

    if received_file.checksum != crc32fast::hash(&received_file.file) {
        filetransmission_spinner.finish_with_message(format!(
            "Received File is Corrupt! crc32({} != {})",
            received_file.checksum, computed_checksum
        ));
        exit(1);
    }

    filetransmission_spinner.set_message(format!(
        "Writing '{}' to filesystem",
        received_fileinfo.file_name
    ));
    fs::write(&received_fileinfo.file_name, &received_file.file)
        .expect("Failed writing to filesystem");
    filetransmission_spinner.finish_with_message(format!(
        "Wrote '{}' to filesystem",
        received_fileinfo.file_name
    ));
}
