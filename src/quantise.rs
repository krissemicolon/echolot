pub fn quantise_to_codec(freq: f32) -> f32 {
    ((freq - 300.0) / 10.0).round().clamp(0.0, 255.0) * 10.0 + 300.0
}

pub fn is_within_tolerance_to(n: f32, goal: f32, tolerance: f32) -> bool {
    (n - goal).abs() <= tolerance
}
