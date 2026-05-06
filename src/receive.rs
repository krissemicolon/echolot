use crate::{
    audio,
    frequency::Frequency,
    modulation::demodulate,
    packets::{FileInfo, Packet},
    pitch_detection,
    quantise::{is_within_tolerance_to, quantise_to_codec},
    BYTE_DURATION_MS, EOT_FREQ, HANDSHAKE_RECEIVER_FREQ, HANDSHAKE_TRANSMITTER_FREQ,
    PREAMBLE_FIRST_FREQ, PREAMBLE_SECOND_FREQ, PREAMBLE_THIRD_FREQ, STD_TOLERANCE,
};
use circular_buffer::CircularBuffer;
use indicatif::ProgressBar;
use std::{
    process::exit,
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

    // Handshake: Initiation Part
    let handshake_spinner = ProgressBar::new_spinner();
    handshake_spinner.set_message("Establishing Handshake");

    // 1. Play 500ms of handshake control freq
    audio_output.playback(vec![Frequency::new_with_len(HANDSHAKE_RECEIVER_FREQ, 500)]);
    audio_output.sink.sleep_until_end();

    let mut sliding_window = CircularBuffer::<16384, f32>::new();

    // 2. Listen for Response
    let mut handshake_detected = false;
    let timer = Instant::now();
    while !handshake_detected {
        if let Ok(chunk) = audio_input.consumer.read_chunk(512) {
            for sample in chunk {
                sliding_window.push_back(sample);
            }

            if sliding_window.is_full() {
                let freq =
                    pitch_detection::dominant_frequency(sliding_window.as_slices().0, 44100.0);
                if is_within_tolerance_to(freq, HANDSHAKE_TRANSMITTER_FREQ, STD_TOLERANCE) {
                    handshake_detected = true;
                }
            }
        } else {
            std::thread::yield_now();
        }
        if Instant::now() > timer + Duration::from_secs(2) {
            handshake_spinner.finish_with_message("Handshake failed");
            exit(1);
        }
    }

    handshake_spinner.finish_with_message("Established Handshake");

    let mut sliding_window = CircularBuffer::<16384, f32>::new();

    let mut fileinfo_freqs: Vec<Frequency> = vec![];
    let mut preamble_first = false;
    let mut preamble_second = false;
    let mut preamble_detected = false;
    let mut eot_detected = false;
    let mut timer: Instant = Instant::now();
    let mut first_time: Option<Instant> = None;
    let mut second_time: Option<Instant> = None;
    let mut interval_time: Option<Duration> = None;

    while !eot_detected {
        if let Ok(chunk) = audio_input.consumer.read_chunk(512) {
            for sample in chunk {
                sliding_window.push_back(sample);
            }

            if sliding_window.is_full() {
                let raw_freq =
                    pitch_detection::dominant_frequency(sliding_window.as_slices().0, 44100.0);
                let quantised_freq = Frequency::new(quantise_to_codec(raw_freq));
                // println!("{raw_freq}\t\t{:?}", quantised_freq);

                if preamble_detected {
                    // EOT
                    if is_within_tolerance_to(raw_freq, EOT_FREQ, STD_TOLERANCE) {
                        eot_detected = true;
                        break;
                    }

                    if let Some(interval) = interval_time {
                        if Instant::now() >= timer + interval {
                            println!("ADDED FREQ {:?}", &quantised_freq);
                            fileinfo_freqs.push(quantised_freq);
                            timer = Instant::now();
                        }
                    }
                } else {
                    if !preamble_first
                        && is_within_tolerance_to(raw_freq, PREAMBLE_FIRST_FREQ, STD_TOLERANCE)
                    {
                        preamble_first = true;
                        first_time = Some(Instant::now());
                    }
                    if !preamble_second
                        && is_within_tolerance_to(raw_freq, PREAMBLE_SECOND_FREQ, STD_TOLERANCE)
                    {
                        preamble_second = true;
                        second_time = Some(Instant::now());
                    }

                    if preamble_first && preamble_second && !preamble_detected {
                        interval_time = Some(
                            second_time.expect("Time Calculation Error")
                                - first_time.expect("Time Calculation Error"),
                        );

                        if let Some(interval) = interval_time {
                            if Instant::now()
                                >= second_time.expect("Time Calculation Error")
                                    + interval
                                    + (interval / 2)
                            {
                                assert!(is_within_tolerance_to(
                                    raw_freq,
                                    PREAMBLE_THIRD_FREQ,
                                    STD_TOLERANCE
                                ));
                                preamble_detected = true;
                                timer = Instant::now();
                            }
                        }
                    }
                }
            }
        } else {
            std::thread::yield_now();
        }
    }

    println!(
        "Freqs Received: {:?}",
        fileinfo_freqs.iter().map(|f| f.freq).collect::<Vec<f32>>()
    );

    let received_fileinfo = FileInfo::decode(demodulate(fileinfo_freqs).unwrap());

    println!("Demodulated: {:?}", received_fileinfo);
}
