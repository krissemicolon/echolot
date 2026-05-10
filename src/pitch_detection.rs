use cpal::SampleRate;
use rustfft::{Fft, FftPlanner, num_complex::Complex};
use std::sync::Arc;

pub struct DominantFrequencyDetector {
    fft: Arc<dyn Fft<f32>>,
    buffer: Vec<Complex<f32>>,
    window: Vec<f32>,
    sample_rate: SampleRate,
}

impl DominantFrequencyDetector {
    pub fn new(sample_len: usize, sample_rate: SampleRate) -> Self {
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(sample_len.max(1));
        let window = hann_window(sample_len.max(1));

        Self {
            fft,
            buffer: vec![Complex::new(0.0, 0.0); sample_len.max(1)],
            window,
            sample_rate,
        }
    }

    pub fn detect(&mut self, samples: &[f32]) -> Option<f32> {
        if samples.is_empty() {
            return None;
        }
        if samples.len() != self.buffer.len() {
            *self = Self::new(samples.len(), self.sample_rate);
        }

        for (idx, sample) in samples.iter().enumerate() {
            self.buffer[idx] = Complex::new(sample * self.window[idx], 0.0);
        }

        self.fft.process(&mut self.buffer);

        let mut max_mag = 0.0;
        let mut max_index = 0;

        for (i, c) in self.buffer.iter().enumerate().take(samples.len() / 2) {
            let mag = c.re * c.re + c.im * c.im;

            if mag > max_mag {
                max_mag = mag;
                max_index = i;
            }
        }

        Some(max_index as f32 * self.sample_rate.0 as f32 / samples.len() as f32)
    }
}

fn hann_window(len: usize) -> Vec<f32> {
    if len <= 1 {
        return vec![1.0; len.max(1)];
    }

    let denom = (len - 1) as f32;
    (0..len)
        .map(|n| {
            let phase = (2.0 * std::f32::consts::PI * n as f32) / denom;
            0.5 * (1.0 - phase.cos())
        })
        .collect()
}
