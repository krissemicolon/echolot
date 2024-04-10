use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use cpal::traits::HostTrait;
use cpal::{
    traits::{DeviceTrait, StreamTrait},
    StreamConfig,
};
use cpal::{PlayStreamError, SampleRate, Stream};
use rodio::{OutputStream, OutputStreamHandle, Sink};
use rtrb::{Consumer, PopError, Producer, PushError, RingBuffer};
use std::sync::mpsc::{self, Receiver, Sender};

use crate::fft::{freq_fft, freq_fft_legacy, FFT_WINDOW};

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
        let (mut producer, mut consumer) = RingBuffer::<f32>::new(2 * FFT_WINDOW);

        let err_fn = move |err| eprintln!("An Error Occurred On Audio Input: {}", err);

        let sample_format = config.sample_format();
        let stream_config: StreamConfig = config.into();
        let sample_rate = stream_config.sample_rate;

        let stream = match sample_format {
            cpal::SampleFormat::F32 => device
                .build_input_stream(
                    &stream_config,
                    move |data: &[f32], _: &_| {
			/*
                        data.into_iter().for_each(|x| {
                            //producer.push(x.to_owned()).expect("Internal Audio Error!")
			    print!("{}", x);
                    })
			 */
			print!("{:?}\n\n", data) ;
			println!("{}", freq_fft_legacy(&data, 48000));
			
			thread::sleep(Duration::from_secs(1));
			panic!();
                    },
                    err_fn,
                    Some(Duration::from_millis(10)),
                )
                .map_err(|e| e.to_string())?,
            _ => return Err("Unsupported sample format".to_string()),
        };

        Ok(Self {
            name,
            stream,
            sample_rate,
            consumer,
        })
    }

    /// Starts the audio input stream.
    pub fn start(&self) -> Result<(), PlayStreamError> {
        self.stream.play()
    }
}
