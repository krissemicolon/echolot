use std::fs;
use base64::encode as encode_base64;
use base64::decode as decode_base64;

use crate::pitch_detection;

/*
    frequency field is compact and can easily be shifted to uhf in future
    stats under base64 input
    min:         500  Hz
    max:         1140 Hz
    start tone:  440  Hz
    end tone:    460  Hz
 */

pub fn encode(file: &str) -> anyhow::Result<Vec<f32>> {
    // file stringify stage
    let base64_file = encode_base64(fs::read_to_string(file)?);

    // frequency sequence calculation stage
    let frequency_sequence: Vec<f32> = Vec::from_iter(base64_file.chars().map(|c| base64_to_freq(c)));

    // TODO: Remove logging
    println!("Max: {}", frequency_sequence.clone().into_iter().fold(f32::NEG_INFINITY, f32::max));
    println!("Min: {}", frequency_sequence.clone().into_iter().fold(f32::INFINITY, f32::min)); 
    println!("Len: {}", frequency_sequence.len());

    Ok(frequency_sequence)
}

pub fn decode(transmission: Vec<f32>, sample_rate: u32) -> anyhow::Result<Vec<u8>> {
    // splitting & multithreaded frequency detection stage
    let mut transmission_sliced: Vec<Vec<f32>> = Vec::new();

    // 0.050 as in 50ms of played frequency
    let slice_size = sample_rate * 0.050 as u32;

    for n in 0..(slice_size / transmission.len() as u32) {
        transmission_sliced.push(transmission[((n * slice_size) as usize)..((n * slice_size + slice_size) as usize)].iter().cloned().collect());
    }

    // slices to frequencies stage
    let transmission_frequencies: Vec<f32> = Vec::from_iter(transmission_sliced.into_iter()
        .map(|samples| quantize(pitch_detection::zero_crossing(&samples, sample_rate))));

    
    // frequencies to base64
    let base64_content = String::from_iter(transmission_frequencies.into_iter().map(|freq| freq_to_base64(freq)));

    // base64 conversion stage
    let content = decode_base64(base64_content)?;

    Ok(content)
}

pub fn quantize(freq: f32) -> f32 {
    freq
}

fn base64_to_freq(c: char) -> f32 {
    440.0 + ((((base64_char_to_base64_value(c) + 5) * 10) as f32)) 
}

fn freq_to_base64(freq: f32) -> char {
    base64_value_to_base64_char((((freq as u32 - 440) / 10) - 5) as u8)
}

fn base64_char_to_base64_value(c: char) -> u8 {
    match c {
        'A' => 01, 'B' => 02, 'C' => 03, 'D' => 04, 'E' => 05,
        'F' => 06, 'G' => 07, 'H' => 08, 'I' => 09, 'J' => 10,
        'K' => 11, 'L' => 12, 'M' => 13, 'N' => 14, 'O' => 15,
        'P' => 16, 'Q' => 17, 'R' => 18, 'S' => 19, 'T' => 20,
        'U' => 21, 'V' => 22, 'W' => 23, 'X' => 24, 'Y' => 25,
        'Z' => 26, 'a' => 27, 'b' => 28, 'c' => 29, 'd' => 30,
        'e' => 31, 'f' => 32, 'g' => 33, 'h' => 34, 'i' => 35,
        'j' => 36, 'k' => 37, 'l' => 38, 'm' => 39, 'n' => 40,
        'o' => 41, 'p' => 42, 'q' => 43, 'r' => 44, 's' => 45,
        't' => 46, 'u' => 47, 'v' => 48, 'w' => 49, 'x' => 50,
        'y' => 51, 'z' => 52, '0' => 53, '1' => 54, '2' => 55,
        '3' => 56, '4' => 57, '5' => 58, '6' => 59, '7' => 60,
        '8' => 61, '9' => 62, '+' => 63, '/' => 64, '=' => 65,
        _   => unreachable!(),
    }
}

fn base64_value_to_base64_char(n: u8) -> char {
    match n {
        01 => 'A', 02 => 'B', 03 => 'C', 04 => 'D', 05 => 'E',
        06 => 'F', 07 => 'G', 08 => 'H', 09 => 'I', 10 => 'J',
        11 => 'K', 12 => 'L', 13 => 'M', 14 => 'N', 15 => 'O',
        16 => 'P', 17 => 'Q', 18 => 'R', 19 => 'S', 20 => 'T',
        21 => 'U', 22 => 'V', 23 => 'W', 24 => 'X', 25 => 'Y',
        26 => 'Z', 27 => 'a', 28 => 'b', 29 => 'c', 30 => 'd',
        31 => 'e', 32 => 'f', 33 => 'g', 34 => 'h', 35 => 'i',
        36 => 'j', 37 => 'k', 38 => 'l', 39 => 'm', 40 => 'n',
        41 => 'o', 42 => 'p', 43 => 'q', 44 => 'r', 45 => 's',
        46 => 't', 47 => 'u', 48 => 'v', 49 => 'w', 50 => 'x',
        51 => 'y', 52 => 'z', 53 => '0', 54 => '1', 55 => '2',
        56 => '3', 57 => '4', 58 => '5', 59 => '6', 60 => '7',
        61 => '8', 62 => '9', 63 => '+', 64 => '/', 65 => '=',
        _   => unreachable!(),
    }
}