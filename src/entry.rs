use std::time::{SystemTime, UNIX_EPOCH};

struct Entry {
    timestamp: u64,
    payload: String,
}

impl Entry {
    pub fn new(payload: String) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Operating system clock needs settings, it is running at a time before UNIX_EPOCH !")
            .as_millis() as u64;
        Entry { timestamp, payload }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        let payload = self.payload.as_bytes();
        for &item in payload {
            bytes.push(item);
        }
        bytes
    }
}

