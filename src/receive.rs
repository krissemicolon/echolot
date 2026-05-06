use crate::{
    audio,
    frequency::Frequency,
    modulation::{demodulate, quantise},
    packets::{FileInfo, Packet},
    pitch_detection,
};
use circular_buffer::CircularBuffer;
use indicatif::ProgressBar;
use std::time::Duration;

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

    // Handshake: Initiation Part
    let handshake_spinner = ProgressBar::new_spinner();
    handshake_spinner.set_message("Establishing Handshake");

    // 1. Play 500ms of handshake control freq
    audio_output.playback(vec![Frequency::new_with_len(3000.0, 500)]);
    audio_output.sink.sleep_until_end();

    let mut sliding_window = CircularBuffer::<16384, f32>::new();

    // 2. Listen for Response
    let mut handshake_detected = false;
    while !handshake_detected {
        if let Ok(chunk) = audio_input.consumer.read_chunk(512) {
            for sample in chunk {
                sliding_window.push_back(sample);
            }

            if sliding_window.is_full() {
                let freq =
                    pitch_detection::dominant_frequency(sliding_window.as_slices().0, 44100.0);
                if (freq - 3050.0).abs() <= 10.0 {
                    handshake_detected = true;
                }
            }
        } else {
            std::thread::yield_now();
        }
    }

    handshake_spinner.finish_with_message("Established Handshake");

    let mut freqs: Vec<Frequency> = vec![];
    let mut eot_detected = false;

    while !eot_detected {
        if let Ok(chunk) = audio_input.consumer.read_chunk(512) {
            for sample in chunk {
                sliding_window.push_back(sample);
            }

            if sliding_window.is_full() {
                let raw_freq =
                    pitch_detection::dominant_frequency(sliding_window.as_slices().0, 44100.0);
                let quantised_freq = Frequency::new(quantise(raw_freq));

                // EOT
                if (raw_freq - 3150.0).abs() <= 10.0 {
                    eot_detected = true;
                    break;
                }

                if freqs.last() != Some(&quantised_freq) {
                    freqs.push(quantised_freq);
                }
            }
        } else {
            std::thread::yield_now();
        }
    }

    let received_fileinfo = FileInfo::decode(demodulate(freqs).unwrap());

    println!("Demodulated: {:?}", received_fileinfo);
}
