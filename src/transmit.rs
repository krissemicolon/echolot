use std::fs;
use base64::encode as encode_base64;

mod sound;

pub fn encode() -> anyhow::Result<Vec<f32>> {
    let base64_file = encode_base64(fs::read_to_string("demonstration.dat")?);
    let mut frequency_sequence: Vec<f32> = Vec::new();

    for xs in base64_file.chars() {
        frequency_sequence.push(sound::semitone_freq(xs as u32));
    }

    println!("Max: {}", frequency_sequence.clone().into_iter().fold(f32::NEG_INFINITY, f32::max));
    println!("Min: {}", frequency_sequence.clone().into_iter().fold(f32::INFINITY, f32::min)); 

    Ok(frequency_sequence)
}

pub fn transmit() -> anyhow::Result<()> {
    let freq_seq = encode()?;

    let mut sound = sound::Sound::new()?;

    for freq in freq_seq {
        sound.play_freq(freq, 50)?;
    }

    Ok(())
}
