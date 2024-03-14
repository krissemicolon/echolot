use cpal::{traits::HostTrait, FromSample, Stream};
use dasp_sample::Sample;
use rodio::{DeviceTrait, OutputStream, OutputStreamHandle, Sink};
use rtrb::{Consumer, Producer, RingBuffer};

pub struct AudioOutputDevice {
    pub name: String,
    pub sink: Sink,
    _stream: (OutputStream, OutputStreamHandle),
}

impl AudioOutputDevice {
    pub fn default() -> Result<Self, String> {
        let device = match cpal::default_host().default_output_device() {
            Some(device) => device,
            None => return Err("No Audio Output Device Available".to_string()),
        };
        let (_stream, stream_handle) = match OutputStream::try_from_device(&device) {
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
        let name = match device.name() {
            Ok(name) => name,
            Err(err) => {
                return Err(format!(
                    "Unable to Retrieve Audio Output Device Name: {}",
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

pub struct AudioInputDevice {
    pub name: String,
    pub consumer: Consumer<f32>,
    pub stream: Stream,
}
/*
impl AudioInputDevice {
    pub fn default() -> Result<Self, String> {
        let (mut producer, mut consumer) = RingBuffer::new(1024);

        let write_data_fn = move |data: f32, _: &_| producer.push(data).unwrap();
        let err_fn = move |err| {
            eprintln!("An Error Occurred On Audio Input: {}", err);
        };

        let device = match cpal::default_host().default_input_device() {
            Some(device) => device,
            None => return Err("No Audio Input Device Available".to_string()),
        };
        let config = match device.default_input_config() {
            Ok(config) => config,
            Err(_) => return Err("Failed to get default input config".to_string()),
        };

        let name = match device.name() {
            Ok(name) => name,
            Err(err) => {
                return Err(format!(
                    "Unable to Retrieve Audio Input Device Name: {}",
                    err
                ))
            }
        };

        let stream = match config.sample_format() {
            cpal::SampleFormat::F32 => device
                .build_input_stream(
                    &config.into(),
                    move |data, _: &_| {
                        write_input_data(
                            data.iter().map(|s| SupportedSample::F32(s)).collect(),
                            &producer,
                        )
                    },
                    err_fn,
                    None,
                )
                .unwrap(),
            sample_format => return Err(format!("Unsupported sample format '{sample_format}'")),
        };

        Ok(Self {
            name,
            consumer,
            stream,
        })
    }
}

enum SupportedSample {
    F32(f32),
    I16(i16),
    U16(u16),
}

impl From<SupportedSample> for f32 {
    fn from(sample: SupportedSample) -> Self {
        match sample {
            SupportedSample::F32(val) => val,
            SupportedSample::I16(val) => f32::from_sample(val),
            SupportedSample::U16(val) => f32::from_sample(val),
        }
    }
}

fn write_input_data(input: &[SupportedSample], producer: &Producer<f32>) {
    input.iter().for_each(|&sample| {
        let converted_sample = f32::from(sample);
        producer.push(converted_sample).unwrap_or_else(|_| {
            eprintln!("Failed to push sample");
        });
    });
}
 */
