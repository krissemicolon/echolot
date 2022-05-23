use std::fs::File;
use std::io::prelude::*;
use base64::decode as decode_base64;

pub fn decode() -> anyhow::Result<Vec<u8>> {
    let input = "TG9yZW0gaXBzdW0gZG9sb3Igc2l0IGFtZXQsIGNvbnNldGV0dXIgc2FkaXBzY2luZyBlbGl0ciwKc2VkIGRpYW0gbm9udW15IGVpcm1vZCB0ZW1wb3IgaW52aWR1bnQgdXQgbGFib3JlIGV0IGRvbG9yZSBtYWduYSBhbGlxdXlhbSBlcmF0LApzZWQgZGlhbSB2b2x1cHR1YS4=";
    return Ok(decode_base64(input)?);
}

pub fn receive() {
    let received_content = String::from_utf8(decode().unwrap()).unwrap();

    let mut file = File::create("demonstration.dat.rec").unwrap();
    file.write_all(&received_content.as_bytes()).expect("Couldn't Save Received File");
}
