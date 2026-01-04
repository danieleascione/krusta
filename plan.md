# Krusta Implementation Plan

## Progress
- ✅ Step 1: Error Handling (PR: claude/error-handling-LbutD)
- ✅ Step 2: Segment Structure (Branch: claude/implement-plan-item-w9iIE)
- ✅ Step 3: Storage Abstraction (Branch: claude/implement-plan-item-w9iIE)
- ✅ Step 4: Memory Backend (Branch: claude/implement-plan-item-w9iIE)
- ✅ Step 5: Segment Index (Branch: claude/implement-plan-item-w9iIE)
- 🔄 **Step 6: Update Log** ← YOU ARE HERE
- ⏳ Step 7: Batching
- ⏳ Step 8: S3 Backend
- ⏳ Step 9: Configuration
- ⏳ Step 10: Integration Tests

## Current State
- `src/log.rs` - HashMap storage, sync API
- `src/error.rs` - KrustaError enum with 4 variants ✅
- `src/segment.rs` - Segment structure with serialization ✅
- `src/storage/mod.rs` - StorageBackend trait ✅
- `src/storage/memory.rs` - MemoryBackend implementation ✅
- `src/segment_index.rs` - SegmentIndex with overlap detection ✅
- Tests: 22/22 passing
- Dependencies: tokio, aws-sdk-s3, bytes, thiserror, async-trait

## Goal
Implement S3 storage backend with segment management.

## Architecture
```
Messages → Batches → Segments → S3
```

## Steps

### Step 1: Error Handling ✅ COMPLETE

**Status:** Merged
**Branch:** `claude/error-handling-LbutD`
**Files:** `src/error.rs`, `src/main.rs`

Implemented:
- KrustaError enum with 4 variants (Storage, SegmentNotFound, InvalidOffset, Serialization)
- Result<T> type alias
- Full test coverage (4 tests)

---

### Step 2: Segment Structure ← NEXT

**File:** `src/segment.rs`

```rust
pub struct Segment {
    start_offset: u64,
    messages: Vec<Vec<u8>>,
}
```

**Binary format:**
```
[message_count: u32][msg1_len: u32][msg1_data][msg2_len: u32][msg2_data]...
```

**Tests:**
1. Create empty segment
2. Add message
3. Serialize to bytes
4. Deserialize from bytes
5. Round-trip matches

---

### Step 3: Storage Abstraction

**File:** `src/storage/mod.rs`

```rust
#[async_trait]
pub trait StorageBackend: Send + Sync {
    async fn write(&self, key: &str, data: &[u8]) -> Result<()>;
    async fn read(&self, key: &str) -> Result<Vec<u8>>;
    async fn list(&self, prefix: &str) -> Result<Vec<String>>;
}
```

No tests (trait only).

---

### Step 4: Memory Backend

**File:** `src/storage/memory.rs`

```rust
pub struct MemoryBackend {
    data: Arc<RwLock<HashMap<String, Vec<u8>>>>,
}
```

**Tests:**
1. Write/read key
2. Write two keys, list both
3. Read missing key → error
4. List empty prefix → empty vec
5. List with prefix → filtered

---

### Step 5: Segment Index

**File:** `src/segment_index.rs`

```rust
pub struct SegmentIndex {
    segments: Vec<SegmentMetadata>,
}

struct SegmentMetadata {
    start_offset: u64,
    end_offset: u64,
    key: String,
}
```

**Tests:**
1. Empty index, find offset → None
2. Add [0..100], find 50 → Some(key)
3. Add [0..100], find 150 → None
4. Add [0..100] + [100..200], find 150 → correct key
5. Overlapping ranges → error

---

### Step 6: Update Log

**File:** `src/log.rs` (modify)

```rust
pub struct Log {
    storage: Arc<dyn StorageBackend>,
    index: SegmentIndex,
    current_batch: Vec<Vec<u8>>,
    next_offset: u64,
    segment_size_bytes: usize,
}
```

**Tests:**
1. Existing tests pass with MemoryBackend
2. Append until full → auto-flush → in storage
3. Read after flush → from storage
4. Read across segments

---

### Step 7: Batching

**File:** `src/batch.rs` or inline in `log.rs`

Flush triggers:
- Size threshold (1MB default)
- Time threshold (5s default)
- Manual flush()

**Tests:**
1. Small append → not flushed
2. Append to threshold → auto-flush
3. Mock time → time-based flush
4. Manual flush → immediate write

Start with size-based only.

---

### Step 8: S3 Backend

**File:** `src/storage/s3.rs`

```rust
pub struct S3Backend {
    client: aws_sdk_s3::Client,
    bucket: String,
}

async fn write(&self, key: &str, data: &[u8]) -> Result<()> {
    self.client
        .put_object()
        .bucket(&self.bucket)
        .key(key)
        .body(data.to_vec().into())
        .send()
        .await
        .map_err(|e| KrustaError::Storage(e.to_string()))?;
    Ok(())
}
```

**Tests:** Same as MemoryBackend (optional: localstack).

---

### Step 9: Configuration

**File:** `src/config.rs`

```rust
pub struct StorageConfig {
    pub backend: BackendType,
    pub s3_bucket: Option<String>,
    pub s3_region: Option<String>,
    pub segment_size_bytes: usize,
    pub flush_interval_secs: u64,
}

pub enum BackendType { Memory, S3 }
```

**Tests:** Defaults, overrides, validation.

---

### Step 10: Integration Tests

**File:** `tests/integration_test.rs`

1. Happy path: append 1000, read all
2. Multiple segments: 1KB size, 3 segments, read across boundary
3. Flush: manual flush, verify storage

---

## TDD Cycle

1. **Red:** Write failing test
2. **Green:** Simplest code to pass
3. **Refactor:** Clean up
4. **Commit:** When green

---

## Next Features

After this: metadata layer, HTTP API, consumer groups, Kafka protocol.
