use serde::{Deserialize, Serialize};

pub trait Packet {
    fn encode(&self) -> Vec<u8> {
	bincode::serialize(self).expect("Codec failed on Serialisation");

	lzma::compress(self, 9).expect("Codec failed on Compression")
    }
    
    fn decode(encoded_packet: Vec<u8>) -> Self {
	let decompressed = lzma::decompress(&mut encoded_packet).expect("Codec failed on Serialisation");
	
	bincode::deserialize(&decompressed).ok().expect("Codec failed on Serialisation")
    }
    
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct FileInfo {
    pub file_name: String,
    pub file_size: u64,
    pub checksum: u32,
}

impl Packet for FileInfo {}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Confirmation {
    pub state: bool,
}

impl Packet for Confirmation {}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct FileTransmission {
    pub file: Vec<u8>,
    pub checksum: u32,
}

impl Packet for FileTransmission {}
