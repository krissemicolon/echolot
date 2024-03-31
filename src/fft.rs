use rustfft::{num_complex::Complex, FftPlanner};
use std::f32::consts::PI;

use crate::frequency::Frequency;

pub const FFT_WINDOW: usize = 2048;

#[inline]
fn apply_hann_window(samples: &mut [f32]) {
    let n = samples.len();
    for (i, sample) in samples.iter_mut().enumerate() {
        let window_value = 0.5 * (1.0 - (2.0 * PI * i as f32 / (n as f32 - 1.0)).cos());
        *sample *= window_value;
    }
}

#[inline]
fn quantize_frequency(freq: f32) -> f32 {
    // optimise this computation
    let modulation_freqs: Vec<f32> = (0..256)
        .into_iter()
        .map(|byte| byte as f32 * 10.0 + 300.0)
        .collect();
    modulation_freqs
        .iter()
        .fold((f32::MAX, 0.0), |(min_diff, closest), &f| {
            let diff = (freq - f).abs();
            if diff < min_diff {
                (diff, f)
            } else {
                (min_diff, closest)
            }
        })
        .1
}

#[inline]
pub fn freq_fft(mut samples: [f32; FFT_WINDOW], sample_rate: u32) -> Frequency {
    println!("{:?}", samples); // TODO

    apply_hann_window(&mut samples);
    let mut buffer: Vec<Complex<f32>> = samples.iter().map(|&s| Complex::new(s, 0.0)).collect();

    let mut planner = FftPlanner::<f32>::new();
    let fft = planner.plan_fft_forward(FFT_WINDOW);
    fft.process(&mut buffer);
    let n = samples.len();

    let half_n = n / 2;
    let frequency_resolution = sample_rate as f32 / n as f32;

    println!("{:?}", buffer); // TODO

    // Process FFT results to find the closest frequency among the possible frequencies
    let main_frequency = buffer
        .iter()
        .take(half_n)
        .enumerate()
        .map(|(i, &c)| (i as f32 * frequency_resolution, c.norm_sqr()))
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
        .map(|(freq, _)| Frequency::new(quantize_frequency(freq)))
        .expect("FFT Failed");

    main_frequency
}

#[cfg(test)]
mod tests {
    use crate::freq_fft;

    use super::*;

    #[test]
    fn test_fft_freq() {}
}
