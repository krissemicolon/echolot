use base64;

pub fn decode() -> anyhow::Result<Vec<u8>> {
    let input = "TG9yZW0gaXBzdW0gZG9sb3Igc2l0IGFtZXQsIGNvbnNldGV0dXIgc2FkaXBzY2luZyBlbGl0ciwKc2VkIGRpYW0gbm9udW15IGVpcm1vZCB0ZW1wb3IgaW52aWR1bnQgdXQgbGFib3JlIGV0IGRvbG9yZSBtYWduYSBhbGlxdXlhbSBlcmF0LApzZWQgZGlhbSB2b2x1cHR1YS4=";

    return Ok(base64::decode(input)?);
}

pub fn receive() {
    println!("{}", String::from_utf8_lossy(&decode().unwrap()));
}