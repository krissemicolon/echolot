#![allow(clippy::precedence)]
#![allow(dead_code)]
#![allow(unused_variables)]

use cpal::{traits::{DeviceTrait, HostTrait, StreamTrait}, Device, SupportedStreamConfig, StreamConfig, Stream};

pub struct Microphone {
    device: Device,
    config: SupportedStreamConfig,
    data: Vec<f32>,
    stream: Stream
}

impl Microphone {
    pub fn new() -> Result<Self, std::io::Error> {
        let host = cpal::default_host();
        let device = host.default_input_device().expect("failed to find a default input device");
        let config = device.default_input_config().unwrap();
        let data: Vec<f32> = Vec::new();

        let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

        let stream = device.build_input_stream(
            &StreamConfig::from(config.to_owned()),
            |data: &[f32], &_| {
                println!("{:?}", data[data.len() - 1]);
                println!("{:?}", data.len());
            },
            err_fn,
        ).expect("Couln't Open Stream");

        Ok(Self {
            device,
            config,
            data,
            stream
        })
    }

    pub fn start_listen(&mut self) -> Result<(), anyhow::Error> {
        self.stream.play()?;

        Ok(())
    }

    pub fn stop_listen(&mut self) -> Result<(), anyhow::Error> {
        drop(&self.stream);

        Ok(())
    }

    pub fn write_input_data(&mut self, input: &[f32]) {
        unimplemented!();
    }
}
