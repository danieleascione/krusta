use crate::error::{KrustaError, Result};

pub struct Segment {
    start_offset: u64,
    messages: Vec<Vec<u8>>,
}

impl Segment {
    pub fn new(start_offset: u64) -> Self {
        Segment {
            start_offset,
            messages: Vec::new(),
        }
    }

    pub fn add_message(&mut self, message: Vec<u8>) {
        self.messages.push(message);
    }

    pub fn len(&self) -> usize {
        self.messages.len()
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        // Write message count
        let message_count = self.messages.len() as u32;
        bytes.extend_from_slice(&message_count.to_le_bytes());

        // Write each message: [length: u32][data]
        for message in &self.messages {
            let msg_len = message.len() as u32;
            bytes.extend_from_slice(&msg_len.to_le_bytes());
            bytes.extend_from_slice(message);
        }

        bytes
    }

    pub fn deserialize(data: &[u8]) -> Result<Self> {
        if data.len() < 4 {
            return Err(KrustaError::Serialization(
                "Not enough data for message count".to_string(),
            ));
        }

        let mut offset = 0;

        // Read message count
        let message_count = u32::from_le_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]);
        offset += 4;

        // Read each message
        let mut messages = Vec::new();
        for _ in 0..message_count {
            if offset + 4 > data.len() {
                return Err(KrustaError::Serialization(
                    "Not enough data for message length".to_string(),
                ));
            }

            let msg_len = u32::from_le_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
            ]) as usize;
            offset += 4;

            if offset + msg_len > data.len() {
                return Err(KrustaError::Serialization(
                    "Not enough data for message content".to_string(),
                ));
            }

            let message = data[offset..offset + msg_len].to_vec();
            messages.push(message);
            offset += msg_len;
        }

        Ok(Segment {
            start_offset: 0, // start_offset is not serialized
            messages,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_empty_segment() {
        let segment = Segment::new(0);
        assert_eq!(segment.len(), 0);
    }

    #[test]
    fn test_add_message() {
        let mut segment = Segment::new(0);
        segment.add_message(b"hello".to_vec());
        assert_eq!(segment.len(), 1);
    }

    #[test]
    fn test_serialize_to_bytes() {
        let mut segment = Segment::new(0);
        segment.add_message(b"hello".to_vec());

        let bytes = segment.serialize();

        // Binary format: [message_count: u32][msg1_len: u32][msg1_data]
        // Expected: [1, 0, 0, 0][5, 0, 0, 0][h, e, l, l, o]
        assert_eq!(bytes.len(), 4 + 4 + 5); // message_count + length + data
    }

    #[test]
    fn test_deserialize_from_bytes() {
        // Binary format: [1, 0, 0, 0][5, 0, 0, 0][h, e, l, l, o]
        let mut bytes = vec![];
        bytes.extend_from_slice(&1u32.to_le_bytes()); // message_count = 1
        bytes.extend_from_slice(&5u32.to_le_bytes()); // msg1_len = 5
        bytes.extend_from_slice(b"hello");            // msg1_data

        let segment = Segment::deserialize(&bytes).unwrap();
        assert_eq!(segment.len(), 1);
    }

    #[test]
    fn test_round_trip_matches() {
        let mut segment = Segment::new(42);
        segment.add_message(b"hello".to_vec());
        segment.add_message(b"world".to_vec());

        let bytes = segment.serialize();
        let deserialized = Segment::deserialize(&bytes).unwrap();

        assert_eq!(deserialized.len(), 2);
        assert_eq!(deserialized.serialize(), bytes);
    }
}
