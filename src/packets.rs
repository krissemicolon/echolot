use serde::{Deserialize, Serialize};

pub trait Packet {
    fn encode(&self) -> Vec<u8>
    where
        Self: Serialize,
    {
        let bin = bincode::serialize(self).expect("Codec failed on Serialisation");

        // lzma::compress(&bin, 9).expect("Codec failed on Compression")
        bin
    }

    fn decode(encoded_packet: Vec<u8>) -> Self
    where
        Self: Sized + Serialize + for<'a> Deserialize<'a>,
    {
        // let decompressed =
        //     lzma::decompress(&encoded_packet).expect("Codec failed on Decompression");
        let decompressed = encoded_packet;

        bincode::deserialize(&decompressed)
            .ok()
            .expect("Codec failed on Deserialisation")
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
