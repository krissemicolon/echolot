use std::time::Duration;
use std::{
    f32::consts::PI,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

use cpal::traits::HostTrait;
use cpal::{SampleRate, Stream};
use cpal::{
    StreamConfig,
    traits::{DeviceTrait, StreamTrait},
};
use rodio::{OutputStream, OutputStreamHandle, Sink};
use rtrb::{Consumer, RingBuffer};

use crate::frequency::Frequency;
use crate::SYMBOL_DURATION_MS;

pub struct AudioOutputDevice {
    pub name: String,
    pub sink: Sink,
    pub sample_rate: SampleRate,
    pub channels: u16,
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
        let channels = config.channels();

        Ok(Self {
            name,
            sink,
            sample_rate,
            channels,
            _stream: (_stream, stream_handle),
        })
    }

    pub fn playback(&mut self, freqs: Vec<Frequency>) {
        for freq in freqs {
            let mono = build_symbol_samples(freq.freq, freq.len, self.sample_rate.0);
            let samples = interleave_channels(&mono, self.channels);
            self.sink.append(rodio::buffer::SamplesBuffer::new(
                self.channels,
                self.sample_rate.0,
                samples,
            ));
        }
    }

    pub fn playback_len(&self) -> usize {
        self.sink.len()
    }
}

pub struct AudioInputDevice {
    pub name: String,
    pub stream: Stream,
    pub sample_rate: SampleRate,
    pub channels: u16,
    pub consumer: Consumer<f32>,
    pub dropped_samples: Arc<AtomicUsize>,
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

        const INPUT_RING_CAPACITY: usize = 16_384;
        let (mut producer, consumer) = RingBuffer::<f32>::new(INPUT_RING_CAPACITY);

        let sample_format = config.sample_format();
        let stream_config: StreamConfig = config.into();
        let sample_rate = stream_config.sample_rate;
        let channels = stream_config.channels;
        let dropped_samples = Arc::new(AtomicUsize::new(0));
        let dropped_samples_callback = Arc::clone(&dropped_samples);

        let stream = match sample_format {
            cpal::SampleFormat::F32 => device
                .build_input_stream(
                    &stream_config,
                    move |data: &[f32], _: &_| {
                        if producer.push_entire_slice(data).is_err() {
                            let dropped = data.len();
                            dropped_samples_callback.fetch_add(dropped, Ordering::Relaxed);
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
            channels,
            consumer,
            dropped_samples,
        })
    }

    /// Starts the audio input stream.
    pub fn start(&self) -> Result<(), cpal::PlayStreamError> {
        self.stream.play()
    }
}

pub fn half_symbol_samples(sample_rate: SampleRate) -> usize {
    (((SYMBOL_DURATION_MS as f64 / 1000.0) / 2.0) * f64::from(sample_rate.0))
        .round()
        .max(1.0) as usize
}

fn build_symbol_samples(freq: f32, len_ms: u64, sample_rate: u32) -> Vec<f32> {
    let symbol_len =
        ((len_ms as f64 / 1000.0) * f64::from(sample_rate)).round().max(1.0) as usize;
    let fade_samples = ((symbol_len as f32 * 0.05).round() as usize).clamp(1, symbol_len / 2);
    let mut out = Vec::with_capacity(symbol_len);
    let sample_rate_f = sample_rate as f32;

    for i in 0..symbol_len {
        let t = i as f32 / sample_rate_f;
        let mut sample = (2.0 * PI * freq * t).sin();

        // Apply a short fade in/out envelope to reduce spectral splatter.
        let gain = if i < fade_samples {
            i as f32 / fade_samples as f32
        } else if i >= symbol_len - fade_samples {
            (symbol_len - i) as f32 / fade_samples as f32
        } else {
            1.0
        };
        sample *= gain;
        out.push(sample);
    }

    out
}

fn interleave_channels(mono: &[f32], channels: u16) -> Vec<f32> {
    if channels <= 1 {
        return mono.to_vec();
    }

    let channels_usize = channels as usize;
    let mut interleaved = Vec::with_capacity(mono.len() * channels_usize);
    for sample in mono {
        for _ in 0..channels_usize {
            interleaved.push(*sample);
        }
    }
    interleaved
}
