use num_complex::Complex;
use rustfft::FftPlanner;

pub fn freq_fft(samples: &[f32], sample_rate: usize) -> f32 {
    let n = samples.len();

    // Convert the audio samples into complex numbers (real part is the sample, imaginary part is 0).
    // This buffer will be used in-place for both input and output of the FFT.
    let mut buffer: Vec<Complex<f32>> = samples.iter().map(|&s| Complex::new(s, 0.0)).collect();

    // Create an FFT planner and perform the FFT in-place.
    let mut planner = FftPlanner::<f32>::new();
    let fft = planner.plan_fft_forward(n);
    fft.process(&mut buffer); // Perform the FFT in-place

    // Find the index of the bin with the highest magnitude.
    // Skip the first bin (DC offset) when searching for the main frequency.
    let (max_index, _max_magnitude) = buffer
        .iter()
        .enumerate()
        .skip(1)
        .map(|(i, &c)| (i, c.norm()))
        .fold((0, 0.0), |(max_i, max_mag), (i, mag)| {
            if mag > max_mag {
                (i, mag)
            } else {
                (max_i, max_mag)
            }
        });

    // Calculate the frequency of the bin with the highest magnitude.
    // Frequency resolution is sample_rate / N.
    let frequency_resolution = sample_rate as f32 / n as f32;
    max_index as f32 * frequency_resolution
}
