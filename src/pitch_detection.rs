use rustfft::{num_complex::Complex, FftPlanner};

pub fn dominant_frequency(samples: &[f32], sample_rate: f32) -> f32 {
    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(samples.len());

    let mut buffer: Vec<Complex<f32>> = samples.iter().map(|&v| Complex::new(v, 0.0)).collect();

    fft.process(&mut buffer);

    let mut max_mag = 0.0;
    let mut max_index = 0;

    for (i, c) in buffer.iter().enumerate().take(samples.len() / 2) {
        let mag = c.re * c.re + c.im * c.im;

        if mag > max_mag {
            max_mag = mag;
            max_index = i;
        }
    }

    max_index as f32 * sample_rate / samples.len() as f32
}
