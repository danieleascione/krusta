use crate::error::Result;

pub struct SegmentIndex {
    segments: Vec<SegmentMetadata>,
}

struct SegmentMetadata {
    start_offset: u64,
    end_offset: u64,
    key: String,
}

impl SegmentIndex {
    pub fn new() -> Self {
        SegmentIndex {
            segments: Vec::new(),
        }
    }

    pub fn add_segment(&mut self, start_offset: u64, end_offset: u64, key: String) -> Result<()> {
        // Check for overlapping ranges
        for segment in &self.segments {
            // Two ranges [a1, a2) and [b1, b2) overlap if (a1 < b2) && (b1 < a2)
            if start_offset < segment.end_offset && segment.start_offset < end_offset {
                return Err(crate::error::KrustaError::Storage(format!(
                    "Segment range [{}, {}) overlaps with existing segment [{}, {})",
                    start_offset, end_offset, segment.start_offset, segment.end_offset
                )));
            }
        }

        self.segments.push(SegmentMetadata {
            start_offset,
            end_offset,
            key,
        });
        Ok(())
    }

    pub fn find_segment(&self, offset: u64) -> Option<String> {
        for segment in &self.segments {
            if offset >= segment.start_offset && offset < segment.end_offset {
                return Some(segment.key.clone());
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_index_find_offset() {
        let index = SegmentIndex::new();
        assert_eq!(index.find_segment(50), None);
    }

    #[test]
    fn test_add_segment_find_inside() {
        let mut index = SegmentIndex::new();
        index.add_segment(0, 100, "segment-0".to_string()).unwrap();
        assert_eq!(index.find_segment(50), Some("segment-0".to_string()));
    }

    #[test]
    fn test_add_segment_find_outside() {
        let mut index = SegmentIndex::new();
        index.add_segment(0, 100, "segment-0".to_string()).unwrap();
        assert_eq!(index.find_segment(150), None);
    }

    #[test]
    fn test_multiple_segments_find_correct() {
        let mut index = SegmentIndex::new();
        index.add_segment(0, 100, "segment-0".to_string()).unwrap();
        index.add_segment(100, 200, "segment-100".to_string()).unwrap();
        assert_eq!(index.find_segment(150), Some("segment-100".to_string()));
    }

    #[test]
    fn test_overlapping_ranges_error() {
        let mut index = SegmentIndex::new();
        index.add_segment(0, 100, "segment-0".to_string()).unwrap();
        let result = index.add_segment(50, 150, "segment-50".to_string());
        assert!(result.is_err());
    }
}
