use std::{fs::File, sync::mpsc};
use std::thread;
use std::sync::mpsc::{channel, Sender};
use std::io::prelude::*;
use base64::decode as decode_base64;
use cpal::{traits::{HostTrait, DeviceTrait}, Host, SupportedStreamConfig, StreamConfig};

use crate::receive::pitch_detection::gen_demo_samples;

use self::microphone::Microphone;

mod util;
mod microphone;
mod pitch_detection;

#[derive(Clone, Debug)]
pub enum MicrophoneEvent {
    SendData(Vec<f32>),
    ReceiveData(Sender<Option<Vec<f32>>>),
}

pub fn decode(samples: Vec<f32>) -> anyhow::Result<Vec<u8>> {
    // base64 conversion stage
    // let input = "TG9yZW0gaXBzdW0gZG9sb3Igc2l0IGFtZXQsIGNvbnNldGV0dXIgc2FkaXBzY2luZyBlbGl0ciwKc2VkIGRpYW0gbm9udW15IGVpcm1vZCB0ZW1wb3IgaW52aWR1bnQgdXQgbGFib3JlIGV0IGRvbG9yZSBtYWduYSBhbGlxdXlhbSBlcmF0LApzZWQgZGlhbSB2b2x1cHR1YS4=";
    // return Ok(decode_base64(input)?);
    Ok(vec![1])
}

pub fn receive() {
    let host = cpal::default_host();
    let device = host.default_input_device().expect("failed to find a default input device");
    let config = device.default_input_config().unwrap();
    let sample_rate = config.sample_rate();

    // microphone buffer analyze stage
    let recording_buffer: Vec<f32> = Vec::new();
    let (sender, receiver) = mpsc::channel::<MicrophoneEvent>();

    let (channel_count, stream, sampling_rate) = match util::microphone_stream(&host, sender.clone(), &device) {
        Ok(s) => s,
        Err(e) => panic!("Error with audio device: {}", e)
    };

    thread::spawn(move || {
        util::handle_microphone_events(receiver);
    });

    // microphone buffer write stage

    // ::: TESTING:

    // println!("Detected Pitch = {}", pitch_detection::detect_pitch(pitch_detection::gen_demo_samples()));
    
    // microphone rec
    // let mut mic = microphone::Microphone::new(&mut sample_buffer).unwrap();
    // mic.start_listen().unwrap();
    // std::thread::sleep(std::time::Duration::from_millis(10000));
    // mic.stop_listen().unwrap();

    // ::: END TESTING

    // base64 conversion stage
    // let received_content = String::from_utf8(decode().unwrap()).unwrap();
    // let mut file = File::create("demonstration.dat.rec").unwrap();
    // file.write_all(&received_content.as_bytes()).expect("Couldn't Save Received File");
}
