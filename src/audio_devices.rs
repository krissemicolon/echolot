use cpal::traits::HostTrait;
use rodio::{DeviceTrait, OutputStream, OutputStreamHandle, Sink};

pub struct AudioOutputDevice {
    pub name: String,
    pub sink: Sink,
    _stream: (OutputStream, OutputStreamHandle),
}

impl AudioOutputDevice {
    pub fn default() -> Result<Self, String> {
        let default_device = match cpal::default_host().default_output_device() {
            Some(device) => device,
            None => return Err("No Audio Output Device Available".to_string()),
        };
        let (_stream, stream_handle) = match OutputStream::try_from_device(&default_device) {
            Ok(output_stream) => output_stream,
            Err(err) => {
                return Err(format!(
                    "Unable to Access Current Audio Output Device: {}",
                    err
                ))
            }
        };
        let sink = match Sink::try_new(&stream_handle) {
            Ok(sink) => sink,
            Err(err) => {
                return Err(format!(
                    "Something went wrong while setting up Audio Output Device: {}",
                    err
                ))
            }
        };
        let name = match default_device.name() {
            Ok(name) => name,
            Err(err) => {
                return Err(format!(
                    "Unable to Retrieve Audio Output Source Name: {}",
                    err
                ))
            }
        };

        Ok(Self {
            name,
            sink,
            _stream: (_stream, stream_handle),
        })
    }
}

struct AudioInputDevice {
    name: String,
}

impl AudioInputDevice {
    pub fn default() -> Result<Self, String> {
        let default_device = match cpal::default_host().default_input_device() {
            Some(device) => device,
            None => return Err("No Audio Input Device Available".to_string()),
        };

        todo!();
    }
}
