use crate::{
    BYTE_DURATION_MS, EOT_FREQ, SOT_FREQ, STD_TOLERANCE, audio,
    frequency::Frequency,
    modulation::demodulate,
    packets::{FileInfo, Packet},
    pitch_detection,
    quantise::{is_within_tolerance_to, quantise_to_codec},
};
use circular_buffer::CircularBuffer;
use indicatif::ProgressBar;
use ringbuf::{
    HeapRb,
    traits::{Consumer, Observer, RingBuffer},
};
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

    if let Err(e) = audio_input.start() {
        eprintln!("Could not start microphone: {}", e);
        return;
    }

    // Transmit Handshake Tone
    let fileinfo_spinner = ProgressBar::new_spinner();
    fileinfo_spinner.enable_steady_tick(Duration::from_millis(60));
    fileinfo_spinner.set_message("Listening for FileInfo Transmission");

    let num_samples =
        (((BYTE_DURATION_MS as f32 / 1000.0) / 2.0) * audio_input.sample_rate.0 as f32) as usize;
    let mut interval_samples = HeapRb::<f32>::new(num_samples);
    let mut in_packet = false;
    let mut fileinfo_freqs: Vec<Frequency> = vec![];
    let interval = Duration::from_millis(BYTE_DURATION_MS as u64 / 2);
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

    println!(
        "Freqs Received: {:?}",
        fileinfo_freqs.iter().map(|f| f.freq).collect::<Vec<f32>>()
    );

    let processed_fileinfo_freqs: Vec<Frequency> = fileinfo_freqs
        .chunks_exact(2)
        .map(|pair| pair[1].clone())
        .collect();

    println!(
        "Freqs Received: {:?}",
        processed_fileinfo_freqs
            .iter()
            .map(|f| f.freq)
            .collect::<Vec<f32>>()
    );

    let received_fileinfo = FileInfo::decode(demodulate(processed_fileinfo_freqs).unwrap());

    println!("Demodulated: {:?}", received_fileinfo);
}
