use std::fs;
use base64::encode as encode_base64;

use crate::coding::encode;

mod speaker;

pub fn transmit() -> anyhow::Result<()> {
    let freq_seq = encode("demonstration.dat")?;
    println!("{:?}", freq_seq);

    let mut speaker = speaker::Speaker::new()?;

    for freq in freq_seq {
        speaker.play_freq(freq, 50)?;
    }

    Ok(())
}
