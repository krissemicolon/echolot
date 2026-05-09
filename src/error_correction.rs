use reed_solomon_erasure::galois_8::ReedSolomon;

const DATA_SHARDS: usize = 4;
const PARITY_SHARDS: usize = 2;
const HEADER_SIZE: usize = 6;

pub fn rs_encode_payload(payload: &[u8]) -> Vec<u8> {
    let rs = ReedSolomon::new(DATA_SHARDS, PARITY_SHARDS)
        .expect("Codec failed on Reed-Solomon initialisation");
    let shard_len = payload.len().div_ceil(DATA_SHARDS);
    let total_shards = DATA_SHARDS + PARITY_SHARDS;

    let mut shards = vec![vec![0u8; shard_len]; total_shards];
    for (i, chunk) in payload.chunks(shard_len).enumerate() {
        shards[i][..chunk.len()].copy_from_slice(chunk);
    }

    rs.encode(&mut shards)
        .expect("Codec failed on Reed-Solomon encoding");

    let payload_len =
        u32::try_from(payload.len()).expect("Codec failed on Packet length conversion");
    let mut out = Vec::with_capacity(HEADER_SIZE + total_shards * shard_len);
    out.push(u8::try_from(DATA_SHARDS).expect("Codec failed on Data shard count conversion to u8"));
    out.push(
        u8::try_from(PARITY_SHARDS).expect("Codec failed on Parity shard count conversion to u8"),
    );
    out.extend_from_slice(&payload_len.to_le_bytes());
    for shard in shards {
        out.extend_from_slice(&shard);
    }

    out
}

pub fn rs_decode_payload(encoded_packet: &[u8]) -> Vec<u8> {
    assert!(
        encoded_packet.len() >= HEADER_SIZE,
        "Codec failed on Packet header parsing: packet too short"
    );

    let data_shards = encoded_packet[0] as usize;
    let parity_shards = encoded_packet[1] as usize;
    assert!(
        data_shards > 0 && parity_shards > 0,
        "Codec failed on Packet header parsing: invalid shard counts"
    );
    let payload_len = u32::from_le_bytes([
        encoded_packet[2],
        encoded_packet[3],
        encoded_packet[4],
        encoded_packet[5],
    ]) as usize;

    let rs = ReedSolomon::new(data_shards, parity_shards)
        .expect("Codec failed on Reed-Solomon initialisation");
    let total_shards = data_shards + parity_shards;
    let body = &encoded_packet[HEADER_SIZE..];
    assert!(
        !body.is_empty(),
        "Codec failed on Packet parsing: missing shard payload"
    );
    assert!(
        body.len().is_multiple_of(total_shards),
        "Codec failed on Packet parsing: truncated shard payload"
    );

    let shard_len = body.len() / total_shards;
    assert!(
        shard_len > 0,
        "Codec failed on Packet parsing: zero-length shards"
    );

    let shards: Vec<Vec<u8>> = body.chunks_exact(shard_len).map(|s| s.to_vec()).collect();

    recover_payload(&rs, &shards, data_shards, parity_shards, payload_len).unwrap_or_else(|| {
        panic!("Codec failed on Reed-Solomon reconstruction: unrecoverable packet")
    })
}

fn recover_payload(
    rs: &ReedSolomon,
    shards: &[Vec<u8>],
    data_shards: usize,
    parity_shards: usize,
    payload_len: usize,
) -> Option<Vec<u8>> {
    for missing_count in 0..=parity_shards {
        for missing_indices in combinations(shards.len(), missing_count) {
            let mut candidates: Vec<Option<Vec<u8>>> = shards
                .iter()
                .enumerate()
                .map(|(idx, shard)| {
                    if missing_indices.contains(&idx) {
                        None
                    } else {
                        Some(shard.clone())
                    }
                })
                .collect();

            if rs.reconstruct(&mut candidates).is_err() {
                continue;
            }

            let mut rebuilt_payload = candidates[..data_shards]
                .iter()
                .flatten()
                .flat_map(|shard| shard.iter().copied())
                .collect::<Vec<u8>>();
            if payload_len > rebuilt_payload.len() {
                continue;
            }
            rebuilt_payload.truncate(payload_len);

            return Some(rebuilt_payload);
        }
    }

    None
}

fn combinations(total: usize, choose: usize) -> Vec<Vec<usize>> {
    fn walk(
        start: usize,
        total: usize,
        choose: usize,
        current: &mut Vec<usize>,
        out: &mut Vec<Vec<usize>>,
    ) {
        if current.len() == choose {
            out.push(current.clone());
            return;
        }

        for idx in start..total {
            current.push(idx);
            walk(idx + 1, total, choose, current, out);
            current.pop();
        }
    }

    let mut out = Vec::new();
    let mut current = Vec::new();
    walk(0, total, choose, &mut current, &mut out);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{RngExt, SeedableRng};

    fn gen_test_bin() -> Vec<u8> {
        let mut rng = rand::rngs::StdRng::seed_from_u64(42);
        (0..1000).map(|_| rng.random::<u8>()).collect()
    }

    #[test]
    fn decode_recovers_single_corrupted_shard() {
        let input = gen_test_bin();
        let mut encoded = rs_encode_payload(&input);

        encoded[8] ^= 0xAA;

        let output = rs_decode_payload(&encoded);
        assert_eq!(input, output);
    }

    #[test]
    #[should_panic(expected = "truncated shard payload")]
    fn decode_rejects_malformed_envelope() {
        let mut malformed: Vec<u8> = vec![4, 2];
        malformed.extend_from_slice(&10u32.to_le_bytes());
        malformed.extend_from_slice(&[1, 2, 3, 4, 5]);

        let _ = rs_decode_payload(&malformed);
    }
}
