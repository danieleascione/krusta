use crate::error::Result;
use crate::storage::StorageBackend;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

pub struct MemoryBackend {
    data: Arc<RwLock<HashMap<String, Vec<u8>>>>,
}

impl MemoryBackend {
    pub fn new() -> Self {
        MemoryBackend {
            data: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl StorageBackend for MemoryBackend {
    async fn write(&self, key: &str, data: &[u8]) -> Result<()> {
        let mut map = self.data.write().unwrap();
        map.insert(key.to_string(), data.to_vec());
        Ok(())
    }

    async fn read(&self, key: &str) -> Result<Vec<u8>> {
        let map = self.data.read().unwrap();
        map.get(key).cloned().ok_or_else(|| {
            crate::error::KrustaError::Storage(format!("Key not found: {}", key))
        })
    }

    async fn list(&self, prefix: &str) -> Result<Vec<String>> {
        let map = self.data.read().unwrap();
        let keys: Vec<String> = map
            .keys()
            .filter(|k| k.starts_with(prefix))
            .cloned()
            .collect();
        Ok(keys)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_write_read_key() {
        let backend = MemoryBackend::new();
        backend.write("key1", b"value1").await.unwrap();
        let result = backend.read("key1").await.unwrap();
        assert_eq!(result, b"value1");
    }

    #[tokio::test]
    async fn test_read_missing_key_error() {
        let backend = MemoryBackend::new();
        let result = backend.read("missing").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_list_empty_returns_empty() {
        let backend = MemoryBackend::new();
        let result = backend.list("").await.unwrap();
        assert_eq!(result, Vec::<String>::new());
    }

    #[tokio::test]
    async fn test_list_returns_all_keys() {
        let backend = MemoryBackend::new();
        backend.write("key1", b"value1").await.unwrap();
        backend.write("key2", b"value2").await.unwrap();
        let mut result = backend.list("").await.unwrap();
        result.sort();
        assert_eq!(result, vec!["key1", "key2"]);
    }

    #[tokio::test]
    async fn test_list_with_prefix_filters() {
        let backend = MemoryBackend::new();
        backend.write("segments/001", b"data1").await.unwrap();
        backend.write("segments/002", b"data2").await.unwrap();
        backend.write("other/003", b"data3").await.unwrap();
        let mut result = backend.list("segments/").await.unwrap();
        result.sort();
        assert_eq!(result, vec!["segments/001", "segments/002"]);
    }
}
