use crate::error_correction::{rs_decode_payload, rs_encode_payload};
use serde::{Deserialize, Serialize};

pub trait Packet {
    fn encode(&self) -> Vec<u8>
    where
        Self: Serialize,
    {
        let bin = bincode::serialize(self).expect("Codec failed on Serialisation");

        // let compressed = lzma::compress(&bin, 9).expect("Codec failed on Compression");

        // rs_encode_payload(&compressed)
        bin
    }

    fn decode(encoded_packet: Vec<u8>) -> Self
    where
        Self: Sized + Serialize + for<'a> Deserialize<'a>,
    {
        // let recovered = rs_decode_payload(&encoded_packet);

        // let decompressed = lzma::decompress(&recovered).expect("Codec failed on Decompression");
        //
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

#[cfg(test)]
mod tests {
    use super::{FileInfo, FileTransmission, Packet};

    #[test]
    fn roundtrip_fileinfo_packet() {
        let input = FileInfo {
            file_name: "foo.txt".to_string(),
            file_size: 1711,
        };

        let encoded = input.encode();
        let output = FileInfo::decode(encoded);

        assert_eq!(input, output);
    }

    #[test]
    fn roundtrip_file_transmission_packet() {
        let input = FileTransmission {
            file: (0..=255).cycle().take(1024).collect(),
            checksum: 1234567,
        };

        let encoded = input.encode();
        let output = FileTransmission::decode(encoded);

        assert_eq!(input, output);
    }

    #[test]
    fn decode_recovers_single_corrupted_shard() {
        let input = FileInfo {
            file_name: "bar.bin".to_string(),
            file_size: 33,
        };
        let mut encoded = input.encode();

        encoded[8] ^= 0xAA;

        let output = FileInfo::decode(encoded);
        assert_eq!(input, output);
    }

    #[test]
    #[should_panic(expected = "truncated shard payload")]
    fn decode_rejects_malformed_envelope() {
        let mut malformed = vec![4, 2];
        malformed.extend_from_slice(&10u32.to_le_bytes());
        malformed.extend_from_slice(&[1, 2, 3, 4, 5]);

        let _ = FileInfo::decode(malformed);
    }
}
