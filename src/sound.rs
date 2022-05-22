#![allow(clippy::precedence)]

use cpal::{traits::{DeviceTrait, HostTrait, StreamTrait}, Host, Device, SupportedStreamConfig, StreamConfig, PlayStreamError};
use fundsp::hacker::*;

pub fn write_data<T>(output: &mut [T], channels: usize, next_sample: &mut dyn FnMut() -> (f64, f64))
where
    T: cpal::Sample,
{
    for frame in output.chunks_mut(channels) {
        let sample = next_sample();
        let left: T = cpal::Sample::from::<f32>(&(sample.0 as f32));
        let right: T = cpal::Sample::from::<f32>(&(sample.1 as f32));

        for (channel, sample) in frame.iter_mut().enumerate() {
            if channel & 1 == 0 {
                *sample = left;
            } else {
                *sample = right;
            }
        }
    }
}

pub struct Sound {
    host: Host,
    device: Device,
    config: SupportedStreamConfig,
    sample_rate: f64,
    channels: usize
}

impl Sound {
    pub fn new() -> Result<Self, std::io::Error> {
        let host = cpal::default_host();
        let device = host.default_output_device().expect("failed to find a default output device");
        let config = device.default_output_config().unwrap();
        let sample_rate = config.sample_rate().0 as f64;
        let channels = config.channels() as usize;

        Ok(Self {
            host,
            device,
            config,
            sample_rate,
            channels
        })
    }

    pub fn semitone_freq(n: u32) -> f32 {
        440.0 * f32::powf(f32::powf(2.0, 1.0 / 12.0), n as f32)
    }

    pub fn play_freq(&mut self, frequency: f32) -> Result<(), PlayStreamError> {
        // Pulse wave.
        let c = lfo(|t| {
            let pitch = 110.0;
            let duty = lerp11(0.01, 0.99, sin_hz(0.05, t));
            (pitch, duty)
        }) >> pulse();

        let c = (c | lfo(|t| (xerp11(110.0, 11000.0, sin_hz(0.15, t)), 0.6))) >> moog();
        let c = c >> split::<U2>();
    
        let mut c = c
            >> (declick() | declick()) >> (dcblock() | dcblock())
            //>> reverb_stereo(0.2, 5.0)
            >> limiter_stereo((1.0, 5.0));
        //let mut c = c * 0.1;
        c.reset(Some(self.sample_rate));
    
        let mut next_value = move || c.get_stereo();
    
        let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

        let data_callback = match self.config.sample_format() {
            cpal::SampleFormat::F32 => {
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                        write_data(data, self.channels, &mut next_value)
                }
            },
            cpal::SampleFormat::I16 => {
                move |data: &mut [i16], _: &cpal::OutputCallbackInfo| {
                        write_data(data, self.channels, &mut next_value)
                }
            },
            cpal::SampleFormat::U16 => {
                move |data: &mut [u16], _: &cpal::OutputCallbackInfo| {
                        write_data(data, self.channels, &mut next_value)
                }
            }
        };
    
        let stream = self.device.build_output_stream(
            &StreamConfig::from(self.config),
            data_callback,
            err_fn,
        )?;
        stream.play()?;
    
        std::thread::sleep(std::time::Duration::from_millis(50000));
    
        Ok(())
    }


}