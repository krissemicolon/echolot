use crate::audio::Frequency;
use crate::packets::{Confirmation, FileInfo, FileTransmission, Packet};

/// samples / 2 > max_freq = 8192 / 2 > 2860
pub const FFT_WINDOW: usize = 8192;

/// 256-MFSK Modulation for data packets
/// with reserved frequencies for control packets
pub fn modulate(data: Vec<u8>) -> Vec<Frequency> {
    data.iter()
        .map(|byte| Frequency::new(*byte as f32 * 10.0 + 300.0))
        .collect()
}

/// 266-MFSK Demodulation for data packets
/// with reserved frequencies for control packets
pub fn demodulate(freqs: Vec<Frequency>) -> Option<Packet> {
    if freqs.is_empty() {
        return None;
    }
    let data: Vec<u8> = freqs
        .into_iter()
        .map(|f| ((f.freq - 300.0) / 10.0) as u8)
        .collect();
    Some(data)
}

#[cfg(test)]
mod tests {
    use crate::packets::FileInfo;

    use super::*;

    #[test]
    fn test_modulate_data() {
	let fileinfo_packet = FileInfo {
	    filename: "foo.txt".to_string(),
	    file_size: 1711,
	    checksum: 0,
	};
        let freqs: Vec<Frequency> = modulate(&fileinfo_packet.encode());

        let expected_freqs: Vec<Frequency> =
            vec![2050.0, 360.0, 300.0, 300.0, 300.0, 300.0, 300.0, 300.0]
                .into_iter()
                .map(Frequency::new)
                .collect();

        assert_eq!(expected_freqs, freqs);
    }

    #[test]
    fn test_demodulate_data() {
        let freqs: Vec<Frequency> = vec![2050.0, 360.0, 300.0, 300.0, 300.0, 300.0, 300.0, 300.0]
            .into_iter()
            .map(|f| Frequency::new(f))
            .collect();
        let response_packet_encoded = demodulate(freqs, Packet::Data(Vec::new())).unwrap();
        let response_packet = Response::decode(response_packet_encoded).unwrap();

        let expected_response_packet = Response {
            file_info_size: 1711,
        };

        assert_eq!(expected_response_packet, response_packet);
    }
}
