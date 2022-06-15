use std::fs;
use base64::encode as encode_base64;

mod speaker;

pub fn encode() -> anyhow::Result<Vec<f32>> {
    let base64_file = encode_base64(fs::read_to_string("demonstration.dat")?);
    let mut frequency_sequence: Vec<f32> = Vec::new();

    println!("{}", base64_file);

    for c in base64_file.chars() {
        frequency_sequence.push((c as u32 * 10) as f32);
    }

    println!("Max: {}", frequency_sequence.clone().into_iter().fold(f32::NEG_INFINITY, f32::max));
    println!("Min: {}", frequency_sequence.clone().into_iter().fold(f32::INFINITY, f32::min)); 
    println!("Len: {}", frequency_sequence.len());

    Ok(frequency_sequence)
}

pub fn transmit() -> anyhow::Result<()> {
    let freq_seq = encode()?;
    println!("{:?}", freq_seq);

    let mut speaker = speaker::Speaker::new()?;

    for freq in freq_seq {
        speaker.play_freq(freq, 50)?;
    }

    Ok(())
}
