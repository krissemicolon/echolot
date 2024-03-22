use std::sync::{Arc, Mutex};
use std::time::Duration;

use cpal::traits::HostTrait;
use cpal::{
    traits::{DeviceTrait, StreamTrait},
    StreamConfig,
};
use cpal::{PlayStreamError, SampleRate, Stream};
use rodio::{OutputStream, OutputStreamHandle, Sink};
use std::sync::mpsc::{self, Receiver, Sender};

use crate::fft::freq_fft;

pub struct AudioOutputDevice {
    pub name: String,
    pub sink: Sink,
    _stream: (OutputStream, OutputStreamHandle),
}

impl AudioOutputDevice {
    pub fn default() -> Result<Self, String> {
        let device = cpal::default_host()
            .default_output_device()
            .ok_or_else(|| "No Audio Output Device Available".to_string())?;

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

        Ok(Self {
            name,
            sink,
            _stream: (_stream, stream_handle),
        })
    }
}

pub struct AudioInputDevice {
    pub name: String,
    pub stream: Stream,
    pub sample_rate: SampleRate,
    pub buffer: Arc<Mutex<Vec<f32>>>,
    pub rx: Receiver<Vec<f32>>,
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

        let (tx, rx) = mpsc::channel();
        let buffer = Arc::new(Mutex::new(Vec::new()));
        let buffer_clone = Arc::clone(&buffer);

        let err_fn = move |err| eprintln!("An Error Occurred On Audio Input: {}", err);

        let sample_format = config.sample_format();
        let stream_config: StreamConfig = config.into();
        let sample_rate = stream_config.sample_rate;
        let stream = match sample_format {
            cpal::SampleFormat::F32 => device
                .build_input_stream(
                    &stream_config,
                    move |data: &[f32], _: &_| Self::write_input_data(data, &tx),
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
            buffer: buffer_clone,
            rx,
        })
    }

    fn write_input_data(data: &[f32], tx: &Sender<Vec<f32>>) {
        println!("{:?}", freq_fft(data, 48000));
        if let Err(e) = tx.send(data.to_vec()) {
            eprintln!("Failed to send data to buffer: {}", e);
        }
    }

    /// Starts the audio input stream.
    pub fn start(&self) -> Result<(), PlayStreamError> {
        self.stream.play()
    }

    /// Reads the buffered audio data.
    pub fn read_buffer(&self) -> Vec<f32> {
        self.buffer.lock().unwrap().clone()
    }

    /// Updates the internal buffer with data from the receiver.
    pub fn update_buffer(&self) {
        while let Ok(data) = self.rx.try_recv() {
            let mut buffer = self.buffer.lock().unwrap();
            buffer.extend(data);
        }
    }
}
