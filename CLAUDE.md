# Instructions for Claude

## Your Job

You're implementing Krusta using Test-Driven Development. Not "tests after." Not "tests eventually." Tests first, always.

## The Rhythm: Red, Green, Refactor

This is not a suggestion. This is the process.

### Red
Write a test that fails. Run it. Watch it fail.

If it doesn't fail, you didn't write a test. You wrote a tautology.

### Green
Write the simplest code that makes the test pass. Not the best code. Not the clever code. The simplest code.

Hard-code return values if that passes the test. Write duplicate code if that passes the test. The refactoring step is coming.

### Refactor
Now make it right. Remove duplication. Improve names. Extract functions. Clean up.

Run the tests after each tiny change. If they fail, undo. If they pass, commit.

## The Rules

1. **Never write production code without a failing test**
   - No "I'll test it later"
   - No "This is too simple to test"
   - No "I'm just debugging"

2. **Write the minimal test to fail**
   - One assertion, if possible
   - Test one behavior
   - Clear name describing what should happen

3. **Write the minimal code to pass**
   - Don't think ahead
   - Don't solve tomorrow's problems
   - If the code feels wrong, that's what refactoring is for

4. **Commit when green**
   - Small commits
   - Always working code
   - Easy to undo mistakes

5. **Refactor only on green**
   - Never refactor with failing tests
   - Change structure, not behavior
   - Tests prove you didn't break anything

## Your Reference

Read `plan.md` to see what to implement next.

The plan is organized in steps. Each step builds on the last. Each step is small enough to test-drive.

Don't skip steps. Don't combine steps. Do one thing at a time.

## Example Session

Let's say you're implementing `src/segment.rs`. Here's how it goes:

### Cycle 1: Create Empty Segment

**Red:**
```rust
#[test]
fn test_create_empty_segment() {
    let segment = Segment::new(0);
    assert_eq!(segment.len(), 0);
}
```

Run: `cargo test` → Fails (Segment doesn't exist)

**Green:**
```rust
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

    pub fn len(&self) -> usize {
        self.messages.len()
    }
}
```

Run: `cargo test` → Passes

**Refactor:**
Nothing to refactor yet. It's simple.

Commit: "feat: add empty Segment creation"

### Cycle 2: Add Message to Segment

**Red:**
```rust
#[test]
fn test_add_message_to_segment() {
    let mut segment = Segment::new(0);
    segment.add(b"hello");
    assert_eq!(segment.len(), 1);
}
```

Run: `cargo test` → Fails (add method doesn't exist)

**Green:**
```rust
impl Segment {
    pub fn add(&mut self, data: &[u8]) {
        self.messages.push(data.to_vec());
    }
}
```

Run: `cargo test` → Passes

**Refactor:**
Looks good. Simple and clear.

Commit: "feat: add message to segment"

### Cycle 3: Serialize Segment

**Red:**
```rust
#[test]
fn test_serialize_empty_segment() {
    let segment = Segment::new(0);
    let bytes = segment.to_bytes().unwrap();
    assert_eq!(bytes.len(), 4); // Just the message count (0)
}
```

Run: `cargo test` → Fails (to_bytes doesn't exist)

**Green:**
```rust
impl Segment {
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        let mut buffer = Vec::new();
        let count = self.messages.len() as u32;
        buffer.extend_from_slice(&count.to_be_bytes());
        Ok(buffer)
    }
}
```

Run: `cargo test` → Passes

**Refactor:**
Good enough for now. We'll add message serialization next.

Commit: "feat: serialize empty segment"

### And so on...

See the pattern?
- Tiny steps
- Always a test first
- Simplest code that works
- Refactor when green
- Commit frequently

## When You Get Stuck

### "I don't know how to test this"

Make it testable. Extract a function. Inject a dependency. Use a trait.

If code is hard to test, it's telling you something about the design.

### "The test is too hard to write"

The test is too big. Break it into smaller tests.

What's the simplest assertion you could make? Start there.

### "I need to refactor before I can add this feature"

Fine. But refactor with tests, not without them. Existing tests should stay green during refactoring.

### "This is taking forever"

Good. Slow is smooth. Smooth is fast. You're building a foundation.

Fast, untested code is slow when you have to debug it.

## Anti-Patterns to Avoid

**Don't do this:**
```rust
// Write a bunch of code
pub struct MyComplexThing {
    // ... 50 lines of code
}

// Then write tests
#[cfg(test)]
mod tests {
    // ... try to figure out what to test
}
```

**Do this:**
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_one_simple_thing() {
        // What should happen?
    }
}

// Now write the code to pass it
```

## Dealing with Async Code

Async doesn't change the process. It just changes the syntax.

```rust
#[tokio::test]
async fn test_write_to_storage() {
    let storage = MemoryBackend::new();
    storage.write("key", b"data").await.unwrap();
    let result = storage.read("key").await.unwrap();
    assert_eq!(result, b"data");
}
```

Still red, green, refactor. Still simple code first.

## Dealing with S3

Mock it with MemoryBackend. Get everything working with MemoryBackend first.

Then, when you implement S3Backend, the same tests should pass. That's the power of the trait.

## Progress Tracking

Use the TodoWrite tool to track your progress through plan.md.

Mark items as:
- `in_progress` when you start a step
- `completed` when all tests pass for that step

Don't skip ahead. Don't work on multiple steps at once.

## Your Mantras

1. **"Red, Green, Refactor"** - Say it out loud. Mean it.

2. **"Make it work, make it right, make it fast"** - In that order.

3. **"The simplest thing that could possibly work"** - Not the best thing. The simplest.

4. **"Listen to the code"** - If it's hard, you're forcing it. Try something simpler.

5. **"Tests are specifications"** - They describe what the code should do. Write them like documentation.

## Signs You're Doing It Right

- You're committing every 5-10 minutes
- Your tests are small and focused
- Your functions are small and clear
- You're not writing speculative code
- You're refactoring frequently
- The tests always pass before you commit
- You're surprised by how simple the solution is

## Signs You're Doing It Wrong

- You have failing tests for more than a few minutes
- You're writing code without tests
- You're writing tests without watching them fail first
- Your functions are big and complicated
- You're thinking about performance before correctness
- You're adding features not in plan.md
- You're trying to be clever

## Remember

You're not building a monument. You're growing software.

Plant a test. Grow some code. Prune and shape. Repeat.

Small steps. Constant feedback. Always working.

That's TDD.

---

Now go read `plan.md`. Start with Step 1. Write a test. Watch it fail.

Then make it pass.
