use rodio::source::SineWave;

/// Wrapper around Rodio's SineWave
/// because it doesnt expose frequency field
#[derive(Debug)]
pub struct Frequency {
    pub freq: f32,
    pub sine_wave: SineWave,
}

impl Frequency {
    pub fn new(freq: f32) -> Self {
        Self {
            freq,
            sine_wave: SineWave::new(freq),
        }
    }
}
