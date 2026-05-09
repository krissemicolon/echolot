use crate::{MOD_OFFSET, MOD_STEP_SIZE};

pub fn quantise_to_codec(freq: f32) -> f32 {
    ((freq - MOD_OFFSET) / MOD_STEP_SIZE)
        .round()
        .clamp(0.0, 15.0)
        * MOD_STEP_SIZE
        + MOD_OFFSET
}

pub fn is_within_tolerance_to(n: f32, goal: f32, tolerance: f32) -> bool {
    (n - goal).abs() <= tolerance
}

#[cfg(test)]
mod tests {
    use crate::modulation::modulate;
    use crate::packets::{FileInfo, Packet};
    use rand::{RngExt, SeedableRng};

    use super::*;

    #[test]
    fn test_quantisation() {
        let mut rng = rand::rngs::StdRng::seed_from_u64(42);

        let expected_freqs: Vec<f32> = modulate(
            (FileInfo {
                file_name: "foo.txt".to_string(),
                file_size: 1711,
            })
            .encode(),
        )
        .iter()
        .map(|f| f.freq)
        .collect();

        let max_distortion = MOD_STEP_SIZE / 3.0;

        let distorted_freqs: Vec<f32> = expected_freqs
            .iter()
            .map(|&f| f + rng.random_range(-max_distortion..max_distortion))
            .collect::<Vec<f32>>();

        let quantised_freqs = distorted_freqs
            .iter()
            .map(|&f| quantise_to_codec(f))
            .collect::<Vec<f32>>();

        assert_eq!(quantised_freqs, expected_freqs);
    }
}
