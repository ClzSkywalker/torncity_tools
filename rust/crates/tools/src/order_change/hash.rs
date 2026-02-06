use std::hash::Hash;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct ContentHash(pub u64);

pub trait ContentHashable {
    fn content_hash(&self) -> ContentHash;
}

#[derive(Debug, Default)]
pub struct StableHasher {
    state: u64,
}

impl StableHasher {
    pub fn new() -> Self {
        Self {
            state: 0xcbf29ce484222325,
        }
    }

    pub fn write_u64(&mut self, value: u64) {
        self.state = self.state.wrapping_mul(0x100000001b3);
        self.state ^= value;
    }

    pub fn write_i64(&mut self, value: i64) {
        self.write_u64(value as u64);
    }

    pub fn write_u32(&mut self, value: u32) {
        self.write_u64(value as u64);
    }

    pub fn write_i32(&mut self, value: i32) {
        self.write_u32(value as u32);
    }

    pub fn write_f32(&mut self, value: f32) {
        self.write_u32(value.to_bits());
    }

    pub fn write_f64(&mut self, value: f64) {
        self.write_u64(value.to_bits());
    }

    pub fn write_bytes(&mut self, bytes: &[u8]) {
        for chunk in bytes.chunks(8) {
            let mut val = [0u8; 8];
            val[..chunk.len()].copy_from_slice(chunk);
            self.write_u64(u64::from_le_bytes(val));
        }
    }

    pub fn write_str(&mut self, s: &str) {
        self.write_bytes(s.as_bytes());
    }

    pub fn finish(&self) -> ContentHash {
        ContentHash(self.state)
    }
}

impl ContentHashable for String {
    fn content_hash(&self) -> ContentHash {
        let mut hasher = StableHasher::new();
        hasher.write_str(self);
        hasher.finish()
    }
}

impl ContentHashable for &str {
    fn content_hash(&self) -> ContentHash {
        let mut hasher = StableHasher::new();
        hasher.write_str(self);
        hasher.finish()
    }
}

impl ContentHashable for i32 {
    fn content_hash(&self) -> ContentHash {
        let mut hasher = StableHasher::new();
        hasher.write_i32(*self);
        hasher.finish()
    }
}

impl ContentHashable for i64 {
    fn content_hash(&self) -> ContentHash {
        let mut hasher = StableHasher::new();
        hasher.write_i64(*self);
        hasher.finish()
    }
}

impl ContentHashable for u32 {
    fn content_hash(&self) -> ContentHash {
        let mut hasher = StableHasher::new();
        hasher.write_u32(*self);
        hasher.finish()
    }
}

impl ContentHashable for u64 {
    fn content_hash(&self) -> ContentHash {
        let mut hasher = StableHasher::new();
        hasher.write_u64(*self);
        hasher.finish()
    }
}

impl ContentHashable for f32 {
    fn content_hash(&self) -> ContentHash {
        let mut hasher = StableHasher::new();
        hasher.write_f32(*self);
        hasher.finish()
    }
}

impl ContentHashable for f64 {
    fn content_hash(&self) -> ContentHash {
        let mut hasher = StableHasher::new();
        hasher.write_f64(*self);
        hasher.finish()
    }
}

impl<T: ContentHashable> ContentHashable for Vec<T> {
    fn content_hash(&self) -> ContentHash {
        let mut hasher = StableHasher::new();
        for item in self {
            let hash = item.content_hash().0;
            hasher.write_u64(hash);
        }
        hasher.finish()
    }
}

impl<T: ContentHashable> ContentHashable for Option<T> {
    fn content_hash(&self) -> ContentHash {
        let mut hasher = StableHasher::new();
        match self {
            Some(val) => {
                hasher.write_u64(1);
                let hash = val.content_hash().0;
                hasher.write_u64(hash);
            }
            None => {
                hasher.write_u64(0);
            }
        }
        hasher.finish()
    }
}
