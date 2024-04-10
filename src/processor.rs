use std::thread;
use rtrb::Consumer;
use crate::fft::FFT_WINDOW;

pub fn process_audio_samples(mut consumer: Consumer<f32>) {
    thread::spawn(move || {
        let mut buffer = Vec::with_capacity(FFT_WINDOW);
        loop {
            // Fill the buffer with 2048 samples before processing
            while buffer.len() < FFT_WINDOW {
                if let Ok(sample) = consumer.pop() {
                    buffer.push(sample);
                }
            }

            // Process the buffer here
            println!("Processing {} samples", buffer.len());
            // Example processing: calculate the mean
            let mean: f32 = buffer.iter().map(|s| s).sum::<f32>() / buffer.len() as f32;
            println!("Mean sample value: {}", mean);

            // Clear the buffer for the next batch of samples
            buffer.clear();
        }
    });
}
