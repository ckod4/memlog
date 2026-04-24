use crc32fast::Hasher;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq)]
pub enum DeserializationError {
    InvalidBytesLength(String),
    ChecksumNotConform(String),
}

pub struct Entry {
    timestamp: u64,
    payload: Vec<u8>,
    checksum: u32,
}

impl Entry {
    pub fn new(payload: Vec<u8>) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Operating system clock needs settings, it is running at a time before UNIX_EPOCH !")
            .as_millis() as u64;

        let checksum = Self::compute_checksum(timestamp, &payload);
        Entry {
            timestamp,
            payload,
            checksum,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(8 + 4 + self.payload.len() + 4);
        // Prefix with a fixed-size 8-byte header (timestamp)
        // This allows the decoder to easily slice the first 8 bytes before reading the payload.
        bytes.extend_from_slice(&self.timestamp.to_le_bytes());
        // Append the payload length
        // By storing the length, the decoder to knows exactly how many bytes should be there / should read next.
        // u32 -> 32/8 -> 4 bytes -> 8..12 slice tell the length to check for the payload
        bytes.extend_from_slice(&(self.payload.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&self.payload);
        // Append checksum, u32 -> 4 last bytes of the vector
        bytes.extend_from_slice(&self.checksum.to_le_bytes());
        bytes
    }

    pub fn from_bytes(data: &[u8]) -> Result<Entry, DeserializationError> {
        let actual_length = data.len();
        if data.len() < 16 {
            // 8 (timestamp) + 4 (payload-length) + 4 (checksum)
            return Err(
                DeserializationError::InvalidBytesLength(
                    format!("Expected data length >= 16, received {actual_length}")
                )
            );
        }
        let timestamp_bytes_vec: [u8; 8] = [
            data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7],
        ];
        let timestamp = u64::from_le_bytes(timestamp_bytes_vec);
        let payload_length_bytes_vec: [u8; 4] = [data[8], data[9], data[10], data[11]];
        let payload_length = u32::from_le_bytes(payload_length_bytes_vec) as usize;
        let expected_length = 8 + 4 + payload_length + 4;
        
        if actual_length < expected_length {
            // 8 (timestamp) + 4 (payload-length) + payload + checksum
            return Err(
                DeserializationError::InvalidBytesLength(
                    format!("Computed data length expected {expected_length}, actual data received {actual_length}").to_string()
                )
            );
        }
        let payload_start_index = 12;
        let payload_end_index = payload_start_index + payload_length;
        let payload: Vec<u8> = data[payload_start_index..payload_end_index].to_vec();
        let checksum_vec: [u8; 4] = [
            data[payload_end_index],
            data[payload_end_index + 1],
            data[payload_end_index + 2],
            data[payload_end_index + 3],
        ];
        let checksum_extracted: u32 = u32::from_le_bytes(checksum_vec);

        // Let's verify the checksum
        let computed_checksum = Self::compute_checksum(timestamp, &payload);

        if checksum_extracted != computed_checksum {
            return Err(
                DeserializationError::ChecksumNotConform(
                    format!("Computed checksum is different from provided data. Computed checksum: {computed_checksum}, provided data checksum: {checksum_extracted}")
                )
            );
        }
        Ok(Entry {
            timestamp,
            payload,
            checksum: checksum_extracted,
        })
    }

    fn compute_checksum(timestamp: u64, payload: &[u8]) -> u32 {
        let mut hasher = Hasher::new();
        hasher.update(&timestamp.to_le_bytes());
        hasher.update(&(payload.len() as u32).to_le_bytes());
        hasher.update(payload);
        hasher.finalize()
    }

    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }

    pub fn payload(&self) -> &[u8] {
        &self.payload
    }

    pub fn checksum(&self) -> u32 {
        self.checksum
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize_entry() {
        let new_entry = Entry::new("Log this event".as_bytes().to_vec());
        let serialized_entry: Vec<u8> = new_entry.to_bytes();
        assert!(!serialized_entry.is_empty());
    }

    #[test]
    fn deserialize_entry() {
        let initial_entry: Entry = Entry::new("Request done !".as_bytes().to_vec());
        let serialized_entry: Vec<u8> = initial_entry.to_bytes();
        let final_entry: Entry = Entry::from_bytes(&serialized_entry).unwrap();
        assert_eq!(initial_entry.payload, final_entry.payload);
        let initial_payload: Vec<u8> = initial_entry.payload.to_vec();
        let initial_payload_value: Result<String, _> = String::from_utf8(initial_payload.to_vec());
        assert!(initial_payload_value.is_ok());
        let x = initial_payload_value.unwrap();
        assert_eq!(&x, "Request done !");
        assert_eq!(initial_entry.timestamp, final_entry.timestamp);
    }

    #[test]
    #[should_panic]
    fn deserialize_invalid_entry() {
        let initial_entry: Entry =
            Entry::new("File extraction encountered an error".as_bytes().to_vec());
        let serialized_entry: Vec<u8> = initial_entry.to_bytes();
        let Ok(_) = Entry::from_bytes(&serialized_entry[0..3]) else {
            panic!("Invalid entry");
        };
        let Ok(_) = Entry::from_bytes(&serialized_entry[0..10]) else {
            panic!("Invalid entry");
        };
        let Ok(_) = Entry::from_bytes(&serialized_entry[0..17]) else {
            panic!("Invalid entry");
        };
    }
    #[test]
    fn deserialize_valid_empty_entry() -> Result<(), String> {
        let initial_entry: Entry = Entry::new("".as_bytes().to_vec());
        let serialized_entry: Vec<u8> = initial_entry.to_bytes();
        let Ok(_) = Entry::from_bytes(&serialized_entry[0..16]) else {
            panic!("Invalid entry");
        };
        Ok(())
    }
}
