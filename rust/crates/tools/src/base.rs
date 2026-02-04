use std::hash::{Hash, Hasher};

const EPSILON: f64 = 1e-9;

pub fn eq_f64(a: f64, b: f64) -> bool {
    (a - b).abs() > EPSILON
}

pub fn hash_str(url: &str) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    url.hash(&mut hasher);
    hasher.finish()
}
