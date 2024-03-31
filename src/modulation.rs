use crate::{
    frequency::Frequency,
    packets::{
        ControlPacket::{Agreement, Initiation},
        Packet::{self, Control, Data},
    },
};

/// 256-MFSK Modulation for data packets
/// with reserved frequencies for control packets
pub fn modulate(packet: &Packet) -> Vec<Frequency> {
    match packet {
        Control(control_packet) => match control_packet {
            Initiation => vec![Frequency::new(500.0)],
            Agreement => vec![Frequency::new(600.0)],
        },
        Data(data) => data
            .iter()
            .map(|byte| Frequency::new(*byte as f32 * 10.0 + 300.0))
            .collect(),
    }
}

/// 266-MFSK Demodulation for data packets
/// with reserved frequencies for control packets
pub fn demodulate(freqs: Vec<Frequency>, expected_packet: Packet) -> Option<Packet> {
    if freqs.is_empty() {
        return None;
    }
    match &expected_packet {
        Control(_) => {
            let control_packet_freq = modulate(&expected_packet).pop().map(|f| f.freq).unwrap();
            let freq_mean = freqs.iter().map(|f| f.freq).sum::<f32>() / freqs.len() as f32;
            // ±5Hz
            if ((freq_mean - 5.0)..=(freq_mean + 5.0)).contains(&control_packet_freq) {
                Some(expected_packet)
            } else {
                None
            }
        }
        Data(_) => {
            let data: Vec<u8> = freqs
                .into_iter()
                .map(|f| ((f.freq - 300.0) / 10.0) as u8)
                .collect();
            Some(Packet::Data(data))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{codec::Codec, packets::Response};

    use super::*;

    #[test]
    fn test_modulate_data() {
        let response_packet = Response {
            file_info_size: 1711,
        };
        let freqs: Vec<Frequency> = modulate(&response_packet.encode());

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
