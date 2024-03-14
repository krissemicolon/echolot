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

impl From<Frequency> for f32 {
    fn from(item: Frequency) -> Self {
        item.freq
    }
}

impl PartialEq for Frequency {
    fn eq(&self, other: &Self) -> bool {
        self.freq == other.freq
    }
}
