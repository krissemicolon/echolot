use crate::audio::Frequency;
use crate::packets::Packet;

/// samples / 2 > max_freq = 8192 / 2 > 2860
pub const FFT_WINDOW: usize = 8192;

/// 256-MFSK Modulation for packets
/// with reserved frequencies for control packets
pub fn modulate(data: Vec<u8>) -> Vec<Frequency> {
    data.iter()
        .map(|byte| Frequency::new(*byte as f32 * 10.0 + 300.0))
        .collect()
}

/// 266-MFSK Demodulation for data packets
/// with reserved frequencies for control packets
pub fn demodulate(freqs: Vec<Frequency>) -> Option<Vec<u8>> {
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
            file_name: "foo.txt".to_string(),
            file_size: 1711,
            checksum: 0, // TODO
        };
        let freqs: Vec<Frequency> = modulate(fileinfo_packet.encode());

        let expected_freqs: Vec<Frequency> = vec![
            2830.0, 850.0, 1520.0, 1180.0, 1200.0, 300.0, 300.0, 340.0, 2600.0, 2440.0, 2100.0,
            1000.0, 320.0, 300.0, 630.0, 310.0, 580.0, 300.0, 300.0, 300.0, 460.0, 2370.0, 1180.0,
            2340.0, 2540.0, 300.0, 560.0, 300.0, 480.0, 1230.0, 300.0, 330.0, 1580.0, 850.0,
            2690.0, 370.0, 1400.0, 590.0, 900.0, 2500.0, 2390.0, 1990.0, 1920.0, 1650.0, 2370.0,
            1920.0, 1100.0, 1360.0, 1260.0, 300.0, 300.0, 300.0, 640.0, 680.0, 1990.0, 1510.0,
            780.0, 1530.0, 2070.0, 1450.0, 300.0, 310.0, 760.0, 570.0, 2650.0, 1470.0, 2580.0,
            1910.0, 610.0, 2120.0, 2730.0, 1550.0, 310.0, 300.0, 300.0, 300.0, 300.0, 340.0,
            1190.0, 1200.0,
        ]
        .into_iter()
        .map(Frequency::new)
        .collect();

        //        assert_eq!(true, true);
        assert_eq!(expected_freqs, freqs);
    }

    #[test]
    fn test_demodulate_data() {
        let freqs: Vec<Frequency> = vec![
            2830.0, 850.0, 1520.0, 1180.0, 1200.0, 300.0, 300.0, 340.0, 2600.0, 2440.0, 2100.0,
            1000.0, 320.0, 300.0, 630.0, 310.0, 580.0, 300.0, 300.0, 300.0, 460.0, 2370.0, 1180.0,
            2340.0, 2540.0, 300.0, 560.0, 300.0, 480.0, 1230.0, 300.0, 330.0, 1580.0, 850.0,
            2690.0, 370.0, 1400.0, 590.0, 900.0, 2500.0, 2390.0, 1990.0, 1920.0, 1650.0, 2370.0,
            1920.0, 1100.0, 1360.0, 1260.0, 300.0, 300.0, 300.0, 640.0, 680.0, 1990.0, 1510.0,
            780.0, 1530.0, 2070.0, 1450.0, 300.0, 310.0, 760.0, 570.0, 2650.0, 1470.0, 2580.0,
            1910.0, 610.0, 2120.0, 2730.0, 1550.0, 310.0, 300.0, 300.0, 300.0, 300.0, 340.0,
            1190.0, 1200.0,
        ]
        .into_iter()
        .map(|f| Frequency::new(f))
        .collect();

        let packet_encoded = demodulate(freqs).unwrap();
        let packet = FileInfo::decode(packet_encoded);

        let expected_packet = FileInfo {
            file_name: "foo.txt".to_string(),
            file_size: 1711,
            checksum: 0, // TODO
        };
        assert_eq!(expected_packet, packet);
    }
}
