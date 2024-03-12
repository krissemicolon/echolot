use serde::{Deserialize, Serialize};

pub enum ControlPacket {
    Initiation,
    Agreement,
}

pub enum Packet {
    Control(ControlPacket),
    Data(Vec<u8>),
}

pub fn get_binary_data(packet: &Packet) -> Option<&Vec<u8>> {
    match packet {
        Packet::Control(_) => None,
        Packet::Data(data) => Some(data),
    }
}

pub struct Initiation;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Response {
    pub file_info_size: usize,
}

pub struct Agreement;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct FileInfo {
    pub file_name: String,
    pub file_size: u64,
    pub checksum: u32,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Confirmation {
    pub state: bool,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct FileTransmission {
    pub file: Vec<u8>,
}
