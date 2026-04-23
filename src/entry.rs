use std::time::{SystemTime, UNIX_EPOCH};

pub struct Entry {
    timestamp: u64,
    payload: Vec<u8>,
}

impl Entry {
    pub fn new(payload: Vec<u8>) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Operating system clock needs settings, it is running at a time before UNIX_EPOCH !")
            .as_millis() as u64;
        Entry { timestamp, payload }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        // Prefix with a fixed-size 8-byte header (Timestamp)
        // This allows the decoder to easily slice the first 8 bytes before reading the payload.
        bytes.extend_from_slice(&self.timestamp.to_le_bytes());
        bytes.extend_from_slice(&self.payload);
        bytes
    }

    pub fn from_bytes(data: &[u8]) -> Result<Self, String> {
        if data.len() < 8 {
            // 8 (timestamp)
            return Err(String::from("Invalid data length"));
        }
        let timestamp_bytes_vec: [u8; 8] = [
            data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7],
        ];
        let timestamp = u64::from_le_bytes(timestamp_bytes_vec);
        let payload: Vec<u8> = data[8..].to_vec();
        Ok(Entry { timestamp, payload })
    }
}
