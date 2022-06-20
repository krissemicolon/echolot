use std::{fs::File, sync::mpsc};
use std::io::prelude::*;
use base64::decode as decode_base64;
use cpal::traits::StreamTrait;
use cpal::{traits::{HostTrait, DeviceTrait}, StreamConfig};

use crate::coding::quantize;
use crate::pitch_detection;

mod util;

pub fn receive() {
    let host = cpal::default_host();
    let device = host.default_input_device().expect("failed to find a default input device");
    let config = device.default_input_config().unwrap();
    let sample_rate = config.sample_rate().0;
    let mut transmission_samples: Vec<f32> = Vec::new();

    // microphone capture stage
    let (sender, receiver) = mpsc::channel();

    let stream = device.build_input_stream(
        &StreamConfig::from(config.to_owned()),
        move |data: &[f32], &_| {
            sender.send(data.to_vec()).unwrap();
        },
        util::stream_error,
    ).expect("Couln't Open Stream");

    stream.play().unwrap();

    let mut transmission_started = false;
    let mut transmission_ended = false;

    while !transmission_ended {
        let mut samples: Vec<f32> = receiver.recv().unwrap();

        // TODO: Remove Logging:
        // println!("samples: {:?}", &samples);
        // println!("pitch: {}", pitch_detection::zero_crossing(&samples, sample_rate));

        if quantize(pitch_detection::zero_crossing(&samples, sample_rate)) == 440.0 {
            transmission_started = true;
        } else if quantize(pitch_detection::zero_crossing(&samples, sample_rate)) == 460.0 {
            transmission_ended = true;
        }

        if transmission_started && !transmission_ended {
            // save transmission sample for decoding stage
            transmission_samples.append(&mut samples);
        }
    }
    
    // decoding stage
    // decode(transmission);

    // writing file stage
    // let received_content = String::from_utf8(decode().unwrap()).unwrap();
    // let mut file = File::create("demonstration.dat.rec").unwrap();
    // file.write_all(&received_content.as_bytes()).expect("Couldn't Save Received File");
}
