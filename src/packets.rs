use std::time::Duration;

use rodio::{
    source::{SineWave, TakeDuration},
    Source,
};

pub struct Initiation;

#[derive(Debug)]
pub struct Response {
    pub file_info_size: u32,
}

pub struct Agreement;

#[derive(Debug)]
pub struct FileInfo {
    pub file_name: String,
    pub file_size: u64,
    pub base64_content_size: usize,
    pub checksum: String,
}

#[derive(Debug)]
pub struct Confirmation {
    pub state: bool,
}

#[derive(Debug)]
pub struct FileTransmission {
    pub base64_content: String,
}

pub enum PacketType {
    Control,
    Data,
}

pub trait Packet {
    fn packet_type(&self) -> PacketType;
}

impl Packet for Initiation {
    fn packet_type(&self) -> PacketType {
        PacketType::Control
    }
}

impl Packet for Response {
    fn packet_type(&self) -> PacketType {
        PacketType::Data
    }
}

type Codec = String;

pub trait Coding: Packet {
    fn encode(&self) -> Codec;
    fn decode(codec: Codec) -> Self;
}

impl Coding for Initiation {
    fn encode(&self) -> Codec {
        todo!()
    }

    fn decode(codec: Codec) -> Self {
        todo!()
    }
}

pub trait Modulation: Packet + Coding {
    fn modulate(&self) -> Vec<TakeDuration<SineWave>>;
    fn demodulate() -> Self;
}

impl Modulation for Initiation {
    fn modulate(&self) -> Vec<TakeDuration<SineWave>> {
        vec![SineWave::new(440.0).take_duration(Duration::from_secs_f32(0.25))]
    }

    fn demodulate() -> Self {
        todo!()
    }
}
