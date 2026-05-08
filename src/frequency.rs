use rodio::source::SineWave;

use crate::BYTE_DURATION_MS;

/// Wrapper around Rodio's SineWave
/// because it doesnt expose frequency field
/// this is a memory overhead that could be improved
#[derive(Debug, Clone)]
pub struct Frequency {
    pub freq: f32,
    pub len: u64,
    pub sine_wave: SineWave,
}

impl Frequency {
    pub fn new(freq: f32) -> Self {
        Self {
            freq,
            len: BYTE_DURATION_MS,
            sine_wave: SineWave::new(freq),
        }
    }
    pub fn new_with_len(freq: f32, len: u64) -> Self {
        Self {
            freq,
            len,
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
