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
