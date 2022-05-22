#![allow(clippy::precedence)]

use cpal::{traits::{DeviceTrait, HostTrait, StreamTrait}, Host, Device, SupportedStreamConfig, StreamConfig, PlayStreamError};
use fundsp::hacker::*;

pub fn semitone_freq(n: u32) -> f32 {
    440.0 * f32::powf(f32::powf(2.0, 1.0 / 12.0), n as f32)
}

fn write_data<T>(output: &mut [T], channels: usize, next_sample: &mut dyn FnMut() -> f32)
where
    T: cpal::Sample,
{
    for frame in output.chunks_mut(channels) {
        let value: T = cpal::Sample::from::<f32>(&next_sample());
        for sample in frame.iter_mut() {
            *sample = value;
        }
    }
}

pub struct Sound {
    device: Device,
    config: SupportedStreamConfig,
}

impl Sound {
    pub fn new() -> Result<Self, std::io::Error> {
        let host = cpal::default_host();
        let device = host.default_output_device().expect("failed to find a default output device");
        let config = device.default_output_config().unwrap();

        Ok(Self {
            device,
            config,
        })
    }

    pub fn play_freq(&mut self, frequency: f32, length: u64) -> Result<(), anyhow::Error> {
        let sample_rate = self.config.sample_rate().0 as f32;
        let channels = self.config.channels() as usize;

        let mut sample_clock = 0f32;
        let mut next_value = move || {
            sample_clock = (sample_clock + 1.0) % sample_rate;
            (sample_clock * frequency * 2.0 * std::f32::consts::PI / sample_rate).sin()
        };
    
        let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

        let stream = self.device.build_output_stream(
            &StreamConfig::from(self.config.to_owned()),
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                write_data(data, channels, &mut next_value)
            },
            err_fn,
        )?;
        stream.play()?;
    
        std::thread::sleep(std::time::Duration::from_millis(length));
    
        Ok(())
    }
}