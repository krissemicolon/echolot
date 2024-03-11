use crate::{
    frequency::Frequency,
    packets::{
        ControlPacket::{Agreement, Initiation},
        Packet::{self, Control, Data},
    },
};

/// 16-MFSK Modulation for data packets
/// with reserved frequencies for control packets
pub fn modulate(packet: &Packet) -> Vec<Frequency> {
    match packet {
        Control(control_packet) => match control_packet {
            Initiation => vec![Frequency::new(500.0)],
            Agreement => vec![Frequency::new(600.0)],
        },
        Data(data) => data
            .into_iter()
            .map(|byte| Frequency::new(*byte as f32 * 10.0 + 100.0))
            .collect(),
    }
}

/// 16-MFSK Demodulation for data packets
/// with reserved frequencies for control packets
fn demodulate(expected_packet: Packet, freqs: Vec<f32>) -> Option<Packet> {
    match &expected_packet {
        Control(_) => {
            let control_packet_freq = modulate(&expected_packet)
                .pop()
                .and_then(|f| Some(f.freq))
                .unwrap();
            let freq_mean = &freqs.iter().sum::<f32>() / freqs.len() as f32;
            // ± 5Hz
            if ((freq_mean - 5.0)..=(freq_mean + 5.0)).contains(&control_packet_freq) {
                Some(expected_packet)
            } else {
                None
            }
        }
        Data(data) => todo!(),
    }
}
