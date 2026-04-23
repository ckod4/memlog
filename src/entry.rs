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
        let mut bytes = Vec::with_capacity(12 + self.payload.len());
        // Prefix with a fixed-size 8-byte header (timestamp)
        // This allows the decoder to easily slice the first 8 bytes before reading the payload.
        bytes.extend_from_slice(&self.timestamp.to_le_bytes());
        // Append the payload length
        // By storing the length, the decoder to knows exactly how many bytes should be there / should read next.
        // u32 -> 32/8 -> 4 bytes -> 8..12 slice tell the length to check for the payload  
        bytes.extend_from_slice(&(self.payload.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&self.payload);
        bytes
    }

    pub fn from_bytes(data: &[u8]) -> Result<Self, String> {
        if data.len() < 12 {
            // 8 (timestamp) + 4 (payload length)
            return Err(String::from("Incorrect length of data to deserialize, the minimum length requirement is not met !"));
        }
        let timestamp_bytes_vec: [u8; 8] = [
            data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7],
        ];
        let timestamp = u64::from_le_bytes(timestamp_bytes_vec);
        let payload_length_bytes_vec: [u8; 4] = [
            data[8], data[9], data[10], data[11],
        ];
        let payload_length = u32::from_le_bytes(payload_length_bytes_vec) as usize;
        let expected_length = 8 + 4 + payload_length;
        let actual_length = data.len();


        if actual_length < expected_length {
            // 8 (timestamp) + 4 (payload length) + payload
            return Err(String::from("Incorrect length of data to deserialize, expected {expected_length} but got {actual_length}"));
        }
        let payload: Vec<u8> = data[12..expected_length].to_vec();
        Ok(Entry { timestamp, payload })
    }

    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }

    pub fn payload(&self) -> &[u8] {
        &self.payload
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
        let initial_entry: Entry = Entry::new("File extraction encountered an error".as_bytes().to_vec());
        let serialized_entry: Vec<u8> = initial_entry.to_bytes();
        let Ok(Entry{timestamp: _, payload: _ }) = Entry::from_bytes(&serialized_entry[0..3]) else {
            panic!("Invalid entry");
        };
        let Ok(Entry{timestamp: _, payload: _ }) = Entry::from_bytes(&serialized_entry[0..10]) else {
            panic!("Invalid entry");
        };
        let Ok(Entry{timestamp: _, payload: _ }) = Entry::from_bytes(&serialized_entry[0..13]) else {
            panic!("Invalid entry");
        };
    }
    #[test]
    fn deserialize_valid_empty_entry() -> Result<(), String> {
        let initial_entry: Entry = Entry::new("".as_bytes().to_vec());
        let serialized_entry: Vec<u8> = initial_entry.to_bytes();
        let Ok(Entry{timestamp: _, payload: _ }) = Entry::from_bytes(&serialized_entry[0..12]) else {
            panic!("Invalid entry");
        };
        Ok(())
    }
}
