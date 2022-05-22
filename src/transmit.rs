use std::fs;
use base64::encode as encode_base64;

mod sound;

pub fn encode() -> anyhow::Result<String> {
    let content = fs::read_to_string("demonstration.dat")?;
    Ok(encode_base64(content))
}

pub fn transmit() -> anyhow::Result<()> {
    let content_encoded = encode()?;

    // To Play Sound:
    sound::Sound::new()?.play_freq(440.0, 6000)?;

    Ok(())
}
