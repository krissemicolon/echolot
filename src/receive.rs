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
use ringbuf::{
    HeapRb,
    traits::{Consumer, Observer, RingBuffer},
};
use std::fs;
use std::{
    process::exit,
    thread::sleep,
    time::{Duration, Instant},
};

pub fn receive() {
    // Setting up Audio Output
    let audio_setup_spinner = ProgressBar::new_spinner();
    audio_setup_spinner.set_message("Setting up Audio Output..");
    audio_setup_spinner.enable_steady_tick(Duration::from_millis(60));
    let mut audio_output = match audio::AudioOutputDevice::default() {
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

    if let Err(e) = audio_input.start() {
        eprintln!("Could not start microphone: {}", e);
        return;
    }

    let fileinfo_spinner = ProgressBar::new_spinner();
    fileinfo_spinner.enable_steady_tick(Duration::from_millis(120));
    fileinfo_spinner.set_message("Listening for FileInfo Transmission");

    let num_samples =
        (((SYMBOL_DURATION_MS as f32 / 1000.0) / 2.0) * audio_input.sample_rate.0 as f32) as usize;
    let mut interval_samples = HeapRb::<f32>::new(num_samples);
    let mut in_packet = false;
    let mut fileinfo_freqs: Vec<Frequency> = vec![];
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

            // EOT Detection
            if in_packet && is_within_tolerance_to(freq, EOT_FREQ, STD_TOLERANCE) {
                in_packet = false;
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

    let processed_fileinfo_freqs: Vec<Frequency> = fileinfo_freqs
        .chunks_exact(2)
        .map(|pair| pair[1].clone())
        .collect();

    let received_fileinfo = FileInfo::decode(demodulate(processed_fileinfo_freqs).unwrap());

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

    let num_samples =
        (((SYMBOL_DURATION_MS as f32 / 1000.0) / 2.0) * audio_input.sample_rate.0 as f32) as usize;
    let mut interval_samples = HeapRb::<f32>::new(num_samples);
    let mut in_packet = false;
    let mut file_freqs: Vec<Frequency> = vec![];
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

            // EOT Detection
            if in_packet && is_within_tolerance_to(freq, EOT_FREQ, STD_TOLERANCE) {
                in_packet = false;
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
    filetransmission_spinner
        .set_message(format!("Received '{}' File", received_fileinfo.file_name));

    let processed_file_freqs: Vec<Frequency> = file_freqs
        .chunks_exact(2)
        .map(|pair| pair[1].clone())
        .collect();

    let received_file = FileTransmission::decode(demodulate(processed_file_freqs).unwrap());
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
