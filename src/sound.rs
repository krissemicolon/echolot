pub fn semitone_freq(n: u32) -> f32 {
    440.0 * f32::powf(f32::powf(2.0, (1.0 / 12.0)), n as f32)
}