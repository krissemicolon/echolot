use serde::{Deserialize, Serialize};

pub enum ControlPacket {
    Initiation,
    Agreement,
}

pub enum Packet {
    Control(ControlPacket),
    Data(Vec<u8>),
}

pub struct Initiation;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Response {
    pub file_info_size: u32,
}

pub struct Agreement;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct FileInfo {
    pub file_name: String,
    pub file_size: u64,
    pub checksum: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Confirmation {
    pub state: bool,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct FileTransmission {
    pub file: Vec<u8>,
}
