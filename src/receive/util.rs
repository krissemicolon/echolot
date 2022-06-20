#[allow(unused_must_use)]

use std::sync::mpsc;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{StreamError, Stream, Device};

use super::{MicrophoneEvent};

pub fn handle_microphone_events(receiver: mpsc::Receiver<MicrophoneEvent>) {
    let mut data: Vec<f32> = Vec::new();

    // TODO: conditional based on beginning detection

    loop {
        println!("MICROPHONING");
        if let Ok(event) = receiver.recv() {
            println!("loggin: {:?}", data);
            match event {
                MicrophoneEvent::SendData(mut d) => {
                    data.append(&mut d);
                }
                MicrophoneEvent::ReceiveData(sender) => {
                    //sender.send(data.clone());
                    if !data.is_empty() {
                        sender.send(Some(data.clone()));
                    } else {
                        sender.send(None);
                    }
                    data.drain(..);
                }
            }
        }
    }
}

pub fn microphone_stream(
    host: &cpal::platform::Host,
    sender: mpsc::Sender<MicrophoneEvent>,
    device: &Device,
    // returns channel-count, stream and sampling-rate
) -> Result<(u16, cpal::Stream, u32), String> {
    let config: cpal::SupportedStreamConfig = match device.default_input_config() {
        Ok(c) => c,
        Err(_) => return Err("Device Not Available".to_string()),
    };

    let channel_count = config.channels();
    let sampling_rate = config.sample_rate();

    #[allow(unused_must_use)]
    let stream = match config.sample_format() {
        cpal::SampleFormat::F32 => device.build_input_stream(
            &config.into(),
            move |data: &[f32], _: &_| {
                sender.send(MicrophoneEvent::SendData(data.to_vec()));
            },
            |e| eprintln!("error occurred on capture-stream: {}", e),
        ),
        cpal::SampleFormat::I16 => device.build_input_stream(
            &config.into(),
            move |data: &[i16], _: &_| {
                let data = i16_to_f32(data);
                sender.send(MicrophoneEvent::SendData(data.to_vec()));
            },
            |e| eprintln!("error occurred on capture-stream: {}", e),
        ),
        cpal::SampleFormat::U16 => device.build_input_stream(
            &config.into(),
            move |data: &[u16], _: &_| {
                let data = u16_to_f32(data);
                sender.send(MicrophoneEvent::SendData(data.to_vec()));
            },
            |e| eprintln!("error occurred on capture-stream: {}", e),
        ),
    };

    let stream = match stream {
        Ok(s) => s,
        Err(e) => match e {
            cpal::BuildStreamError::DeviceNotAvailable => return Err("Device Not Available".to_string()),
            cpal::BuildStreamError::StreamConfigNotSupported => {
                return Err("Unsupported Config".to_string())
            }
            cpal::BuildStreamError::BackendSpecific { err } => {
                return Err(err.to_string())
            }
            err => return Err(err.to_string()),
        },
    };

    stream.play().unwrap();

    Ok((channel_count, stream, sampling_rate.0))
}

pub fn base64_to_value(c: char) -> u8 {
    match c {
        'A' => 01, 'B' => 02, 'C' => 03, 'D' => 04, 'E' => 05,
        'F' => 06, 'G' => 07, 'H' => 08, 'I' => 09, 'J' => 10,
        'K' => 11, 'L' => 12, 'M' => 13, 'N' => 14, 'O' => 15,
        'P' => 16, 'Q' => 17, 'R' => 18, 'S' => 19, 'T' => 20,
        'U' => 21, 'V' => 22, 'W' => 23, 'X' => 24, 'Y' => 25,
        'Z' => 26, 'a' => 27, 'b' => 28, 'c' => 29, 'd' => 30,
        'e' => 31, 'f' => 32, 'g' => 33, 'h' => 34, 'i' => 35,
        'j' => 36, 'k' => 37, 'l' => 38, 'm' => 39, 'n' => 40,
        'o' => 41, 'p' => 42, 'q' => 43, 'r' => 44, 's' => 45,
        't' => 46, 'u' => 47, 'v' => 48, 'w' => 49, 'x' => 50,
        'y' => 51, 'z' => 52, '0' => 53, '1' => 54, '2' => 55,
        '3' => 56, '4' => 57, '5' => 58, '6' => 59, '7' => 60,
        '8' => 61, '9' => 62, '+' => 63, '/' => 64, '=' => 65,
        _   => unreachable!(),
    }
}

pub fn base64_value_to_frequency(n: u8) -> f32 {
    440.0 + ((n - 1) as f32 * 20.0)
}

pub fn quantize() -> u16 {
    todo!();
}

pub fn stream_error(err: StreamError) {
    eprintln!("an error occurred on stream: {}", err);
}

pub fn i16_to_f32(sample: &[i16]) -> Vec<f32> {
    let f32_sample: Vec<f32> = sample
        .into_iter()
        .map(|x| *x as f32 / i16::MAX as f32)
        .collect();

    f32_sample
}

pub fn u16_to_f32(sample: &[u16]) -> Vec<f32> {
    let f32_sample: Vec<f32> = sample
        .into_iter()
        .map(|x| *x as f32 / u16::MAX as f32 - 0.5)
        .collect();

    f32_sample
}