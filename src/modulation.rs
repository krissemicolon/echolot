use crate::{MOD_OFFSET, MOD_STEP_SIZE, frequency::Frequency};

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
            2550.0, 2250.0, 750.0, 1350.0, 1350.0, 1800.0, 1050.0, 1500.0, 1050.0, 1800.0, 300.0,
            300.0, 300.0, 300.0, 300.0, 900.0, 2400.0, 1200.0, 2250.0, 1200.0, 1950.0, 900.0,
            900.0, 1200.0, 300.0, 600.0, 300.0, 300.0, 600.0, 450.0, 300.0, 450.0, 450.0, 2100.0,
            300.0, 300.0, 300.0, 300.0, 300.0, 300.0, 450.0, 300.0, 2100.0, 2550.0, 1050.0, 1500.0,
            2100.0, 2100.0, 2400.0, 300.0, 300.0, 300.0, 450.0, 1800.0, 300.0, 300.0, 450.0, 600.0,
            1050.0, 2250.0, 300.0, 300.0, 300.0, 750.0, 1500.0, 300.0, 750.0, 1350.0, 2400.0,
            2550.0, 300.0, 1350.0, 1200.0, 2400.0, 450.0, 2250.0, 750.0, 2100.0, 2250.0, 2100.0,
            2250.0, 450.0, 1800.0, 1650.0, 1800.0, 600.0, 1500.0, 1350.0, 2100.0, 2550.0, 1800.0,
            600.0, 1050.0, 300.0, 1200.0, 1800.0, 1200.0, 300.0, 300.0, 300.0, 300.0, 300.0, 300.0,
            300.0, 600.0, 600.0, 600.0, 1200.0, 1800.0, 1650.0, 1350.0, 1650.0, 750.0, 300.0,
            1350.0, 1950.0, 1950.0, 450.0, 1350.0, 750.0, 300.0, 300.0, 300.0, 450.0, 600.0,
            2400.0, 450.0, 1950.0, 2400.0, 1950.0, 1350.0, 1050.0, 2400.0, 900.0, 1800.0, 450.0,
            450.0, 2550.0, 1950.0, 1200.0, 2550.0, 750.0, 1350.0, 2250.0, 300.0, 450.0, 300.0,
            300.0, 300.0, 300.0, 300.0, 300.0, 300.0, 300.0, 300.0, 900.0, 1050.0, 1650.0, 1050.0,
            1800.0,
        ]
        .into_iter()
        .map(Frequency::new)
        .collect();

        assert_eq!(expected_freqs, freqs);
    }

    #[test]
    fn test_demodulate_data() {
        let freqs: Vec<Frequency> = vec![
            2550.0, 2250.0, 750.0, 1350.0, 1350.0, 1800.0, 1050.0, 1500.0, 1050.0, 1800.0, 300.0,
            300.0, 300.0, 300.0, 300.0, 900.0, 2400.0, 1200.0, 2250.0, 1200.0, 1950.0, 900.0,
            900.0, 1200.0, 300.0, 600.0, 300.0, 300.0, 600.0, 450.0, 300.0, 450.0, 450.0, 2100.0,
            300.0, 300.0, 300.0, 300.0, 300.0, 300.0, 450.0, 300.0, 2100.0, 2550.0, 1050.0, 1500.0,
            2100.0, 2100.0, 2400.0, 300.0, 300.0, 300.0, 450.0, 1800.0, 300.0, 300.0, 450.0, 600.0,
            1050.0, 2250.0, 300.0, 300.0, 300.0, 750.0, 1500.0, 300.0, 750.0, 1350.0, 2400.0,
            2550.0, 300.0, 1350.0, 1200.0, 2400.0, 450.0, 2250.0, 750.0, 2100.0, 2250.0, 2100.0,
            2250.0, 450.0, 1800.0, 1650.0, 1800.0, 600.0, 1500.0, 1350.0, 2100.0, 2550.0, 1800.0,
            600.0, 1050.0, 300.0, 1200.0, 1800.0, 1200.0, 300.0, 300.0, 300.0, 300.0, 300.0, 300.0,
            300.0, 600.0, 600.0, 600.0, 1200.0, 1800.0, 1650.0, 1350.0, 1650.0, 750.0, 300.0,
            1350.0, 1950.0, 1950.0, 450.0, 1350.0, 750.0, 300.0, 300.0, 300.0, 450.0, 600.0,
            2400.0, 450.0, 1950.0, 2400.0, 1950.0, 1350.0, 1050.0, 2400.0, 900.0, 1800.0, 450.0,
            450.0, 2550.0, 1950.0, 1200.0, 2550.0, 750.0, 1350.0, 2250.0, 300.0, 450.0, 300.0,
            300.0, 300.0, 300.0, 300.0, 300.0, 300.0, 300.0, 300.0, 900.0, 1050.0, 1650.0, 1050.0,
            1800.0,
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

    #[test]
    fn test_interplay() {
        let input_packet = FileInfo {
            file_name: "foo.txt".to_string(),
            file_size: 1711,
            checksum: 0,
        };
        let freqs: Vec<Frequency> = modulate(input_packet.encode());
        let packet_encoded = demodulate(freqs).unwrap();
        let output_packet = FileInfo::decode(packet_encoded);

        assert_eq!(output_packet, input_packet);
    }

    #[test]
    fn test_binops() {
        let expected_values: Vec<u8> = (0..=255).collect();
        let rebuilt_values: Vec<u8> = expected_values
            .iter()
            .copied()
            .map(|v| split_byte(v))
            .map(|partials| create_byte(partials[0], partials[1]))
            .collect();

        assert_eq!(rebuilt_values, expected_values);
    }
}
