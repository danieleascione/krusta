use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::error::Result;
use crate::storage::StorageBackend;

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
        let mut map = self.data.write().await;
        map.insert(key.to_string(), data.to_vec());
        Ok(())
    }

    async fn read(&self, key: &str) -> Result<Vec<u8>> {
        let map = self.data.read().await;
        map.get(key)
            .cloned()
            .ok_or_else(|| crate::error::KrustaError::Storage(format!("Key not found: {}", key)))
    }

    async fn list(&self, prefix: &str) -> Result<Vec<String>> {
        let map = self.data.read().await;
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
        backend.write("test-key", b"test-data").await.unwrap();
        let data = backend.read("test-key").await.unwrap();
        assert_eq!(data, b"test-data");
    }

    #[tokio::test]
    async fn test_write_two_keys_list_both() {
        let backend = MemoryBackend::new();
        backend.write("key1", b"data1").await.unwrap();
        backend.write("key2", b"data2").await.unwrap();

        let mut keys = backend.list("").await.unwrap();
        keys.sort();
        assert_eq!(keys, vec!["key1", "key2"]);
    }

    #[tokio::test]
    async fn test_read_missing_key_error() {
        let backend = MemoryBackend::new();
        let result = backend.read("missing-key").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_list_empty_prefix() {
        let backend = MemoryBackend::new();
        let keys = backend.list("").await.unwrap();
        assert_eq!(keys.len(), 0);
    }

    #[tokio::test]
    async fn test_list_with_prefix() {
        let backend = MemoryBackend::new();
        backend.write("prefix1/key1", b"data1").await.unwrap();
        backend.write("prefix1/key2", b"data2").await.unwrap();
        backend.write("prefix2/key3", b"data3").await.unwrap();

        let mut keys = backend.list("prefix1/").await.unwrap();
        keys.sort();
        assert_eq!(keys, vec!["prefix1/key1", "prefix1/key2"]);
    }
}
