use std::time::Duration;

use cpal::traits::HostTrait;
use cpal::{
    traits::{DeviceTrait, StreamTrait},
    StreamConfig,
};
use cpal::{SampleRate, Stream};
use rodio::source::SineWave;
use rodio::{OutputStream, OutputStreamHandle, Sink};
use rtrb::{Consumer, RingBuffer};

use crate::modulation;

/// Wrapper around Rodio's SineWave
/// because it doesnt expose frequency field
/// this is a memory overhead that could be improved
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

pub struct AudioOutputDevice {
    pub name: String,
    pub sink: Sink,
    pub sample_rate: SampleRate,
    _stream: (OutputStream, OutputStreamHandle),
}

impl AudioOutputDevice {
    pub fn default() -> Result<Self, String> {
        let device = cpal::default_host()
            .default_output_device()
            .ok_or_else(|| "No Audio Output Device Available".to_string())?;
        let config = device
            .default_output_config()
            .map_err(|err| format!("Failed to get default audio output config: {}", err))?;
        let (_stream, stream_handle) = OutputStream::try_from_device(&device)
            .map_err(|err| format!("Unable to Access Current Audio Output Device: {}", err))?;
        let sink = Sink::try_new(&stream_handle).map_err(|err| {
            format!(
                "Something went wrong while setting up Audio Output Device: {}",
                err
            )
        })?;
        let name = device
            .name()
            .map_err(|err| format!("Unable to Retrieve Audio Output Device Name: {}", err))?;
        let sample_rate = config.sample_rate();

        Ok(Self {
            name,
            sink,
            sample_rate,
            _stream: (_stream, stream_handle),
        })
    }

    pub fn playback(&mut self, freqs: Vec<Frequency>) {
        self.sink.append(freqs[0].sine_wave.clone());
    }
}

fn find_sync(data: &[u8], pattern: &[u8]) -> Option<usize> {
    data.windows(pattern.len())
        .position(|window| window == pattern)
}

pub struct AudioInputDevice {
    pub name: String,
    pub stream: Stream,
    pub sample_rate: SampleRate,
    pub consumer: Consumer<f32>,
}

impl AudioInputDevice {
    pub fn default() -> Result<Self, String> {
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or("No Audio Input Device Available".to_string())?;
        let config = device
            .default_input_config()
            .map_err(|_| "Failed to get default input config".to_string())?;
        let name = device
            .name()
            .map_err(|e| format!("Unable to Retrieve Audio Input Device Name: {}", e))?;
        let (producer, consumer) = RingBuffer::<f32>::new(2 * 1024); // Example buffer size

        let err_fn = |err| eprintln!("An Error Occurred On Audio Input: {}", err);

        let sample_format = config.sample_format();
        let stream_config: StreamConfig = config.into();
        let sample_rate = stream_config.sample_rate;

        let stream = match sample_format {
            cpal::SampleFormat::F32 => device
                .build_input_stream(
                    &stream_config,
                    move |data: &[f32], _: &_| {
                        let binary_data = modulation::demodulate(
                            data.into_iter()
                                .map(|f| Frequency::new(f.clone()))
                                .collect(),
                        );
                        let sync_pattern = vec![1, 0, 1, 0, 1, 0];

                        if let Some(index) = find_sync(&binary_data.unwrap(), &sync_pattern) {
                            // Synchronization found, process data starting from `index`
                            println!("Synchronized at index: {}", index);
                        }
                    },
                    err_fn,
                    Some(Duration::from_millis(5)),
                )
                .map_err(|e| e.to_string()),
            _ => Err("Unsupported sample format".to_string()),
        }?;

        Ok(Self {
            name,
            stream,
            sample_rate,
            consumer,
        })
    }

    /// Starts the audio input stream.
    pub fn start(&self) -> Result<(), cpal::PlayStreamError> {
        self.stream.play()
    }
}
