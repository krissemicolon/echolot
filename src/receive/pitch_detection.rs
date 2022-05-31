pub fn gen_demo_samples() -> Vec<f32> {
    let mut samples: Vec<f32> = Vec::new();

    let mut sample_clock = 0f32;
    for i in 0..100 {
        sample_clock = (sample_clock + 1.0) % 44100.0;
        samples.push((sample_clock * 440.0 * 2.0 * std::f32::consts::PI / 44100.0).sin());
    }

    samples
}

pub fn detect_pitch(samples: Vec<f32>) -> f32 {
    let mut zs = 1;

    for i in 0..(samples.len() - 1) {
        if samples[i] < 0.0 && samples[i + 1] > 0.0
        || samples[i] > 0.0 && samples[i + 1] < 0.0 {
            zs += 1;
        }
    }

    ((1.0 / (samples.len() as f32 / 44100.0)) * zs as f32) / 2.0
}