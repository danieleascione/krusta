# Instructions for Claude

## Your Job

Implement Krusta using TDD. Tests first, always.

## TDD Cycle: Red, Green, Refactor

### Red
Write a failing test. Run `cargo test`. Watch it fail.

### Green
Write simplest code to pass. Hard-code if needed. Duplication is OK.

### Refactor
Clean up. Remove duplication. Improve names. Run tests after each change.

## Rules

1. **Never write code without a failing test first**
2. **One test at a time** - single assertion, single behavior
3. **Simplest code to pass** - don't solve future problems
4. **Commit when green** - small commits, always working
5. **Refactor only on green** - tests must pass before refactoring

## Implementation Reference

Read `plan.md` for steps. Do them in order. One step at a time.

## Example TDD Cycle

**Red:**
```rust
#[test]
fn test_create_empty_segment() {
    let segment = Segment::new(0);
    assert_eq!(segment.len(), 0);
}
```
Run: `cargo test` → Fails

**Green:**
```rust
pub struct Segment {
    start_offset: u64,
    messages: Vec<Vec<u8>>,
}

impl Segment {
    pub fn new(start_offset: u64) -> Self {
        Segment { start_offset, messages: Vec::new() }
    }
    pub fn len(&self) -> usize { self.messages.len() }
}
```
Run: `cargo test` → Passes

**Refactor:** Nothing needed.

**Commit:** "feat: add empty Segment creation"

## Async Tests

```rust
#[tokio::test]
async fn test_write_to_storage() {
    let storage = MemoryBackend::new();
    storage.write("key", b"data").await.unwrap();
    assert_eq!(storage.read("key").await.unwrap(), b"data");
}
```

## S3 Testing

Test with MemoryBackend first. Then S3Backend should pass same tests.

## Progress Tracking

Use TodoWrite tool:
- Mark `in_progress` when starting a step
- Mark `completed` when tests pass

## Key Principles

- Write test first (always)
- Simplest code to pass
- Commit on green
- Small steps (5-10 min per cycle)
- Tests are specifications
