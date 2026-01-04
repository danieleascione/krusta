use thiserror::Error;

#[derive(Error, Debug)]
pub enum KrustaError {
    #[error("Storage error: {0}")]
    Storage(String),
    #[error("Segment not found: {0}")]
    SegmentNotFound(String),
}

pub type Result<T> = std::result::Result<T, KrustaError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_error() {
        let err = KrustaError::Storage("connection failed".to_string());
        assert_eq!(err.to_string(), "Storage error: connection failed");
    }

    #[test]
    fn test_segment_not_found_error() {
        let err = KrustaError::SegmentNotFound("segment-123".to_string());
        assert_eq!(err.to_string(), "Segment not found: segment-123");
    }
}
