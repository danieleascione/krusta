/// Append-only log backed by S3 storage
/// Messages are batched into segments and written as immutable objects

use std::collections::HashMap;

pub struct Log {
    // In-memory storage for messages (offset -> data)
    // Later this will be replaced with S3-backed storage
    messages: HashMap<u64, Vec<u8>>,
    next_offset: u64,
}

impl Log {
    pub fn new() -> Self {
        Log {
            messages: HashMap::new(),
            next_offset: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.messages.len()
    }

    pub fn append(&mut self, data: &[u8]) {
        self.messages.insert(self.next_offset, data.to_vec());
        self.next_offset += 1;
    }

    pub fn read(&self, offset: u64) -> Option<&[u8]> {
        self.messages.get(&offset).map(|v| v.as_slice())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_empty_log() {
        let log = Log::new();
        assert_eq!(log.len(), 0);
    }

    #[test]
    fn test_append_single_message() {
        let mut log = Log::new();
        log.append("hello world".as_bytes());
        assert_eq!(log.len(), 1);
    }

    #[test]
    fn test_read_by_offset() {
        let mut log = Log::new();
        log.append("hello".as_bytes());
        log.append("world".as_bytes());

        let msg = log.read(0).unwrap();
        assert_eq!(msg, "hello".as_bytes());

        let msg = log.read(1).unwrap();
        assert_eq!(msg, "world".as_bytes());
    }
}
