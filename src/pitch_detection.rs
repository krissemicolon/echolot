pub fn zero_crossing(samples: &Vec<f32>, sample_rate: u32) -> f32 {
    let mut zs = 1;
    for i in 0..(samples.len() - 1) {
        if samples[i] < 0.0 && samples[i + 1] > 0.0
        || samples[i] > 0.0 && samples[i + 1] < 0.0 {
            zs += 1;
        }
    }
    ((1.0 / (samples.len() as f32 / (sample_rate as f32))) * zs as f32) / 2.0
}

pub fn autocorrelation(samples: &Vec<f32>, sample_rate: u32) -> f32 {
    todo!();
}
