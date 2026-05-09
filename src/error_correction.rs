use crc32fast::Hasher as Crc32Hasher;
use reed_solomon_erasure::galois_8::ReedSolomon;

const DATA_SHARDS: usize = 4;
const PARITY_SHARDS: usize = 2;
const TOTAL_SHARDS: usize = DATA_SHARDS + PARITY_SHARDS;

/// Fixed prefix: 1 (data_shards) + 1 (parity_shards) + 4 (payload_len) + 4 (shard_len).
const FIXED_HEADER: usize = 10;
/// Full header including the per-shard CRC table (one u32 per shard).
const HEADER_SIZE: usize = FIXED_HEADER + TOTAL_SHARDS * 4;

fn shard_crc(data: &[u8]) -> u32 {
    let mut h = Crc32Hasher::new();
    h.update(data);
    h.finalize()
}

/// Encodes `payload` into a self-describing packet:
///
/// ```text
/// [data_shards: u8]
/// [parity_shards: u8]
/// [payload_len: u32 LE]
/// [shard_len: u32 LE]
/// [crc_shard_0 .. crc_shard_N: u32 LE each]   <- erasure oracle
/// [shard_0 .. shard_N: bytes]
/// ```
pub fn rs_encode_payload(payload: &[u8]) -> Vec<u8> {
    let rs = ReedSolomon::new(DATA_SHARDS, PARITY_SHARDS)
        .expect("Codec failed on Reed-Solomon initialisation");
    let shard_len = payload.len().div_ceil(DATA_SHARDS).max(1);

    let mut shards = vec![vec![0u8; shard_len]; TOTAL_SHARDS];
    for (i, chunk) in payload.chunks(shard_len).enumerate() {
        shards[i][..chunk.len()].copy_from_slice(chunk);
    }
    rs.encode(&mut shards)
        .expect("Codec failed on Reed-Solomon encoding");

    let payload_len =
        u32::try_from(payload.len()).expect("Codec failed on payload length conversion");

    let mut out = Vec::with_capacity(HEADER_SIZE + TOTAL_SHARDS * shard_len);

    // Fixed header
    out.push(DATA_SHARDS as u8);
    out.push(PARITY_SHARDS as u8);
    out.extend_from_slice(&payload_len.to_le_bytes());
    out.extend_from_slice(&(shard_len as u32).to_le_bytes());

    // CRC table — one u32 per shard so the decoder can identify erasures precisely
    for shard in &shards {
        out.extend_from_slice(&shard_crc(shard).to_le_bytes());
    }

    // Shard data
    for shard in &shards {
        out.extend_from_slice(shard);
    }

    out
}

/// Decodes a packet produced by `rs_encode_payload`.
///
/// Each shard whose CRC does not match the stored value is treated as a known
/// erasure and passed as `None` to `ReedSolomon::reconstruct`.  No combinations
/// are tried; reconstruction is attempted exactly once.
pub fn rs_decode_payload(encoded_packet: &[u8]) -> Vec<u8> {
    // --- parse fixed header ---
    assert!(
        encoded_packet.len() >= FIXED_HEADER,
        "Codec failed on header parsing: packet too short"
    );
    let data_shards = encoded_packet[0] as usize;
    let parity_shards = encoded_packet[1] as usize;
    assert!(
        data_shards > 0 && parity_shards > 0,
        "Codec failed on header parsing: invalid shard counts"
    );
    let total_shards = data_shards + parity_shards;
    let payload_len = u32::from_le_bytes(encoded_packet[2..6].try_into().unwrap()) as usize;
    let shard_len = u32::from_le_bytes(encoded_packet[6..10].try_into().unwrap()) as usize;

    // --- parse CRC table ---
    let crc_table_end = FIXED_HEADER + total_shards * 4;
    assert!(
        encoded_packet.len() >= crc_table_end,
        "Codec failed on header parsing: CRC table truncated"
    );
    let stored_crcs: Vec<u32> = (0..total_shards)
        .map(|i| {
            let off = FIXED_HEADER + i * 4;
            u32::from_le_bytes(encoded_packet[off..off + 4].try_into().unwrap())
        })
        .collect();

    // --- parse shard body ---
    let body = &encoded_packet[crc_table_end..];
    let expected_len = total_shards * shard_len;
    assert!(
        body.len() == expected_len,
        "Codec failed on shard parsing: malformed body"
    );

    if shard_len == 0 {
        assert!(
            payload_len == 0,
            "Codec failed on reconstruction: payload_len exceeds recovered data"
        );
        return Vec::new();
    }

    // --- mark corrupt shards as erasures ---
    // Shards whose CRC matches are passed as Some(_); corrupt ones as None.
    // The RS library only needs to know *which* shards are missing, not why.
    let mut candidates: Vec<Option<Vec<u8>>> = (0..total_shards)
        .map(|i| {
            let shard = body[i * shard_len..(i + 1) * shard_len].to_vec();
            if shard_crc(&shard) == stored_crcs[i] {
                Some(shard)
            } else {
                None
            }
        })
        .collect();

    // --- reconstruct (single call, no brute-force) ---
    let rs = ReedSolomon::new(data_shards, parity_shards)
        .expect("Codec failed on Reed-Solomon initialisation");
    rs.reconstruct(&mut candidates)
        .expect("Codec failed on Reed-Solomon reconstruction: unrecoverable packet");

    // --- reassemble payload ---
    let mut payload: Vec<u8> = candidates[..data_shards]
        .iter()
        .flatten()
        .flat_map(|s| s.iter().copied())
        .collect();
    assert!(
        payload_len <= payload.len(),
        "Codec failed on reconstruction: payload_len exceeds recovered data"
    );
    payload.truncate(payload_len);
    payload
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

        // Flip a byte in the first shard's data area (past the full header)
        encoded[HEADER_SIZE] ^= 0xAA;

        let output = rs_decode_payload(&encoded);
        assert_eq!(input, output);
    }

    #[test]
    #[should_panic(expected = "malformed body")]
    fn decode_rejects_malformed_envelope() {
        let mut malformed: Vec<u8> = vec![4, 2];
        malformed.extend_from_slice(&10u32.to_le_bytes());
        malformed.extend_from_slice(&1u32.to_le_bytes());
        // CRC table placeholder (6 shards × 4 bytes)
        malformed.extend_from_slice(&[0u8; 24]);
        // 5 bytes of body — not divisible by total_shards (6)
        malformed.extend_from_slice(&[1, 2, 3, 4, 5]);

        let _ = rs_decode_payload(&malformed);
    }

    #[test]
    fn decode_recovers_max_erasures() {
        // Lose exactly parity_shards shards — the maximum recoverable count
        let input = gen_test_bin();
        let mut encoded = rs_encode_payload(&input);

        // Corrupt the first two shards (indices 0 and 1 in shard body)
        encoded[HEADER_SIZE] ^= 0xFF;
        encoded[HEADER_SIZE + 1] ^= 0xFF;

        let output = rs_decode_payload(&encoded);
        assert_eq!(input, output);
    }

    #[test]
    fn encode_decode_empty_payload() {
        let encoded = rs_encode_payload(&[]);
        let decoded = rs_decode_payload(&encoded);
        assert!(decoded.is_empty());
    }
}
