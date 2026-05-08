use crate::{frequency::Frequency, MOD_OFFSET, MOD_STEP_SIZE};

/// 16-MFSK Modulation for packets
/// with reserved frequencies for control packets
pub fn modulate(data: Vec<u8>) -> Vec<Frequency> {
    let m = |n: u8| -> f32 { n as f32 * MOD_STEP_SIZE + MOD_OFFSET };

    data.iter()
        .flat_map(|&byte| split_byte(byte))
        .map(|n| Frequency::new(m(n)))
        .collect()
}

/// 16-MFSK Demodulation for data packets
/// with reserved frequencies for control packets
pub fn demodulate(freqs: Vec<Frequency>) -> Option<Vec<u8>> {
    let d = |f: f32| -> u8 { ((f - MOD_OFFSET) / MOD_STEP_SIZE) as u8 };

    if freqs.len() % 2 != 0 {
        return None;
    }

    Some(
        freqs
            .chunks_exact(2)
            .map(|pair| create_byte(d(pair[0].freq), d(pair[1].freq)))
            .collect(),
    )
}

fn split_byte(value: u8) -> [u8; 2] {
    let high = (value >> 4) & 0x0F;
    let low = value & 0x0F;
    [high, low]
}

fn create_byte(high: u8, low: u8) -> u8 {
    ((high & 0x0F) << 4) | (low & 0x0F)
}

#[cfg(test)]
mod tests {
    use crate::packets::{FileInfo, Packet};

    use super::*;

    #[test]
    fn test_modulate_data() {
        let fileinfo_packet = FileInfo {
            file_name: "foo.txt".to_string(),
            file_size: 1711,
            checksum: 0,
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
            checksum: 0,
        };

        assert_eq!(expected_packet, packet);
    }
}
