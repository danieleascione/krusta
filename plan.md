# Krusta Implementation Plan

*"Make it work, make it right, make it fast" - in that order.*

## Where We Are

We have a working append-only log. It lives in memory. The tests pass. This is good.

Current implementation:
- `src/log.rs` - HashMap-based storage, synchronous API
- Tests prove: create, append, read by offset
- Dependencies ready: tokio, aws-sdk-s3, bytes, thiserror

## Where We're Going

Next: **S3 Storage Backend with Segment Management**

Why this next? Because:
1. It's the foundation for everything else
2. We can test it without networking (memory backend)
3. It makes the system cloud-native
4. We learn about our domain by implementing it

## The Big Idea

Messages don't live in memory. They live in S3, grouped into immutable segments.

```
Messages → Batches → Segments → S3
```

Each segment is a file in S3. Each file contains many messages. We write once, read many times.

## Implementation Strategy

We'll work in small, tested increments. Each step will be the simplest thing that could possibly work.

### Step 1: Error Handling (Start Here)

**File:** `src/error.rs`

We need errors before we need the code that produces them.

```rust
#[derive(Error, Debug)]
pub enum KrustaError {
    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Segment not found: {0}")]
    SegmentNotFound(String),

    #[error("Invalid offset: {0}")]
    InvalidOffset(u64),

    #[error("Serialization error: {0}")]
    Serialization(String),
}

pub type Result<T> = std::result::Result<T, KrustaError>;
```

**Tests:**
- Create each error type
- Convert to strings
- That's enough for now

---

### Step 2: Segment Structure

**File:** `src/segment.rs`

A segment is:
- A range of offsets (start, end)
- A collection of messages
- Serializable to bytes

**The simplest design:**
```rust
pub struct Segment {
    start_offset: u64,
    messages: Vec<Vec<u8>>,
}
```

**Format (on disk):**
```
[message_count: u32]
[msg1_len: u32][msg1_data]
[msg2_len: u32][msg2_data]
...
```

**Test-drive these behaviors:**
1. Create empty segment
2. Add message to segment
3. Serialize segment to bytes
4. Deserialize bytes to segment
5. Round-trip: segment → bytes → segment (messages match)

**Listen to the code.** If serialization feels hard, we chose the wrong format. Keep it simple.

---

### Step 3: Storage Abstraction

**File:** `src/storage/mod.rs`

We want to write code that works with storage, without caring if it's memory or S3.

```rust
#[async_trait]
pub trait StorageBackend: Send + Sync {
    async fn write(&self, key: &str, data: &[u8]) -> Result<()>;
    async fn read(&self, key: &str) -> Result<Vec<u8>>;
    async fn list(&self, prefix: &str) -> Result<Vec<String>>;
}
```

**No tests yet.** Traits aren't testable. Implementations are.

---

### Step 4: Memory Backend

**File:** `src/storage/memory.rs`

Before we touch S3, we prove the abstraction works.

```rust
pub struct MemoryBackend {
    data: Arc<RwLock<HashMap<String, Vec<u8>>>>,
}
```

**Test-drive these behaviors:**
1. Write a key/value, read it back
2. Write two keys, list them
3. Read missing key → error
4. List empty prefix → empty vec
5. List with prefix → filtered results

**This is our safety net.** Every test we write here will eventually run against S3.

---

### Step 5: Segment Index

**File:** `src/segment_index.rs`

We need to answer: "Which segment contains offset 1234?"

```rust
pub struct SegmentIndex {
    // Maps offset ranges to S3 keys
    segments: Vec<SegmentMetadata>,
}

struct SegmentMetadata {
    start_offset: u64,
    end_offset: u64,
    key: String,
}
```

**Test-drive these behaviors:**
1. Empty index → find offset → None
2. Add segment [0..100] → find 50 → Some(key)
3. Add segment [0..100] → find 150 → None
4. Add segments [0..100], [100..200] → find 150 → correct segment
5. Overlapping offsets → error (shouldn't happen, but be safe)

**Keep it simple.** Linear search is fine. Optimize later if needed.

---

### Step 6: Update Log to Use Storage

**File:** `src/log.rs` (modify)

Now we refactor. Change Log to use StorageBackend instead of HashMap.

**The new Log:**
```rust
pub struct Log {
    storage: Arc<dyn StorageBackend>,
    index: SegmentIndex,
    current_batch: Vec<Vec<u8>>,
    next_offset: u64,
    segment_size_bytes: usize,
}
```

**Test-drive the changes:**
1. All existing tests pass (with MemoryBackend)
2. New test: append until segment full → auto-flush → storage has data
3. New test: read after flush → fetches from storage
4. New test: read old + new messages → combines segments

**Refactor constantly.** The tests tell us when we break something.

---

### Step 7: Batching Logic

**File:** `src/batch.rs` (or fold into `log.rs` if simple)

Messages accumulate. We flush when:
- Batch reaches size threshold (1MB default)
- Time threshold passes (5 seconds default)
- Manual flush() called

**Test-drive these behaviors:**
1. Append small message → not flushed yet
2. Append until size threshold → auto-flush
3. Mock time → verify time-based flush
4. Manual flush → writes immediately

**Start without time-based flushing.** Add it when size-based works.

---

### Step 8: S3 Backend

**File:** `src/storage/s3.rs`

Now we're ready for the real thing.

```rust
pub struct S3Backend {
    client: aws_sdk_s3::Client,
    bucket: String,
}
```

**Test strategy:**
- All MemoryBackend tests should pass with S3Backend
- Optional: Use localstack/minio for local testing
- Manual testing against real S3

**Implementation:**
```rust
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

**Start simple.** No retries. No fancy error handling. Make it work first.

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

pub enum BackendType {
    Memory,
    S3,
}
```

**Test-drive:**
- Create config with defaults
- Override each field
- Validate: S3 backend requires bucket + region

---

### Step 10: Integration Tests

**File:** `tests/integration_test.rs`

End-to-end scenarios:

1. **Happy path:**
   - Create log with MemoryBackend
   - Append 1000 messages
   - Read all back in order
   - All match

2. **Multiple segments:**
   - Small segment size (1KB)
   - Append until 3 segments created
   - Read from each segment
   - Read across segment boundary

3. **Flush behavior:**
   - Append messages
   - Manual flush
   - Verify in storage
   - Append more
   - Verify both segments exist

**These tests protect us.** They catch regressions.

---

## Working Rhythm

For each step:

1. **Red:** Write a failing test
2. **Green:** Write the simplest code that passes
3. **Refactor:** Clean up, remove duplication

Commit after each green. Small commits, always working.

## Questions We'll Answer By Implementing

1. Is 1MB the right segment size? (We'll feel it)
2. How do we handle partial reads? (The code will tell us)
3. What about concurrent writes? (Defer until it's a problem)
4. Do we need compression? (Not yet)

## Success Looks Like

- [ ] All tests pass
- [ ] Can write messages to S3
- [ ] Can read messages back from S3
- [ ] MemoryBackend works (for tests)
- [ ] S3Backend works (for production)
- [ ] Code is simple and clear
- [ ] We learned something about our domain

## After This

The foundation enables:
1. Metadata layer (offset tracking)
2. HTTP API (producers/consumers)
3. Consumer groups
4. Kafka protocol compatibility

But that's later. First, make this work.

---

## Notes on Style

**Prefer:**
- Small functions
- Clear names
- Few dependencies
- Simple data structures

**Avoid:**
- Premature optimization
- Fancy abstractions
- Speculative generality
- Big bang integration

**Remember:**
- Tests are specifications
- Code is communication
- Simple is not easy
- Listen to the code

---

*"I'm not a great programmer; I'm just a good programmer with great habits."*
