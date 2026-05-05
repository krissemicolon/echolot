use std::time::Duration;

use cpal::traits::HostTrait;
use cpal::{
    traits::{DeviceTrait, StreamTrait},
    StreamConfig,
};
use cpal::{SampleRate, Stream};
use rodio::{OutputStream, OutputStreamHandle, Sink, Source};
use rtrb::{Consumer, RingBuffer};

use crate::frequency::Frequency;

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
        freqs.into_iter().for_each(|freq| {
            self.sink.append(
                freq.sine_wave
                    .take_duration(Duration::from_millis(100))
                    .amplify(1.0),
            );
        });
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

        let err_fn = |err| eprintln!("An Error Occurred On Audio Input: {}", err);

        let (mut producer, mut consumer) = RingBuffer::<f32>::new(16384);

        let sample_format = config.sample_format();
        let stream_config: StreamConfig = config.into();
        let sample_rate = stream_config.sample_rate;

        let stream = match sample_format {
            cpal::SampleFormat::F32 => device
                .build_input_stream(
                    &stream_config,
                    move |data: &[f32], _: &_| {
                        let _ = producer.push_entire_slice(data);
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
