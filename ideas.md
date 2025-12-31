# ideas.md: Core Log Structure & High-Throughput on S3

This document explores the first item on the Krust roadmap: the implementation of the core log structure. It specifically addresses the challenge of achieving high throughput for a message bus built on a high-latency object store like Amazon S3.

## 1. The Core Log Structure

The fundamental data structure in Krust is a **distributed, append-only log**. This log is composed of a sequence of immutable files (log segments) stored in an object store. Each message is assigned a unique, monotonically increasing offset.

### Logical View

Logically, a topic partition is a continuous sequence of messages, ordered by their offsets.

```
| Msg 0 | Msg 1 | Msg 2 | ... | Msg N |
+-------+-------+-------+-----+-------+
Offset: 0       1       2           N
```

### Physical Implementation on S3

Physically, this continuous logical log is broken down into multiple **log segment files** stored in S3. A separate **metadata layer** is responsible for mapping a given offset to the specific S3 object (and the position within that object) where the message data resides.

-   **Log Segments**: These are immutable files containing a batch of messages. Once a segment is written to S3, it is never modified.
-   **Manifest/Index**: A file or set of files that contains the mapping from offsets to log segments. This is a critical component for efficient reads.

## 2. The High-Throughput Challenge with S3

Amazon S3 is designed for high throughput and durability, but not for low latency. A single S3 PUT or GET request can take hundreds of milliseconds. For a message bus, this latency is unacceptable on a per-message basis. The key to high throughput is to **amortize this latency** across many messages.

Here are several proposed approaches to achieve this:

### Approach A: Aggressive Batching and Buffering

This is the most fundamental strategy, inspired by systems like WarpStream.

-   **How it works**: The Krust Agent buffers incoming messages in memory for a configurable period (e.g., 50-250ms) or until a certain size is reached. It then writes this entire batch as a single log segment file to S3. A single S3 PUT operation thus commits hundreds or thousands of messages at once.
-   **Pros**: Simple to implement, dramatically increases write throughput.
-   **Cons**: Introduces a small amount of latency (the batching window). There is a trade-off between latency and cost (smaller batches mean more S3 PUT requests).

### Approach B: Tiered Storage with S3 Express One Zone

This approach leverages AWS's low-latency storage tier.

-   **How it works**: 
    1.  **Ingestion Tier (Hot)**: All new data is written to **S3 Express One Zone**, which offers single-digit millisecond latency. This allows for very small batching windows and low produce latency.
    2.  **Compaction Tier (Cold)**: A background process in the Agent pool continuously compacts smaller log segments from S3 Express into larger segments and moves them to standard S3 for long-term, cost-effective storage.
-   **Pros**: Achieves low end-to-end latency for producers. Optimizes storage costs by only keeping recent data in the more expensive tier.
-   **Cons**: Increased complexity. S3 Express is single-AZ, so for durability, data must be replicated across multiple S3 Express buckets in different AZs, which adds to the cost and complexity.

### Approach C: Lock-Free Coordination with Conditional Writes

This is a more advanced approach, inspired by Chroma's `wal3`, that uses a new S3 feature to build concurrent data structures directly on object storage.

-   **How it works**: It uses S3's `If-None-Match` and `If-Match` conditional write features to perform atomic operations. This allows multiple agents to write to the log concurrently without a central coordinator, using a lock-free linked-list-style algorithm on S3 objects.
    -   A **manifest** file acts as the head of the log.
    -   New writes create a new **log fragment** file (using `If-None-Match` to ensure it's created only once).
    -   The writer then atomically updates the manifest to point to this new fragment (using `If-Match` to prevent lost updates).
-   **Pros**: Potentially very high concurrency and throughput, as it avoids centralized bottlenecks.
-   **Cons**: Highly complex to implement correctly. Relies on a relatively new S3 feature.

### Approach D: Parallel Writes with Quorum

This approach focuses on improving both latency and durability.

-   **How it works**: When an Agent flushes a batch, it writes the same log segment file to multiple S3 buckets (ideally in different regions or AZs) in parallel. The write is acknowledged back to the producer as soon as a quorum of writes (e.g., 2 out of 3) has succeeded.
-   **Pros**: The perceived latency is that of the fastest `k` of `n` writes, which can be lower than waiting for a single write. It also provides built-in durability against single-bucket/region failures.
-   **Cons**: Higher cost due to data duplication. Requires careful management of the quorum.

## Summary of Approaches

| Approach | Key Idea | Latency | Throughput | Complexity | Cost |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **A: Batching** | Amortize latency over many messages | Medium | High | Low | Low |
| **B: Tiered Storage** | Use S3 Express for ingestion | Low | Very High | Medium | Medium |
| **C: Lock-Free** | Atomic S3 operations for coordination | Low | Very High | High | Low |
| **D: Parallel/Quorum** | Parallel writes to multiple buckets | Low | High | Medium | High |

For the initial implementation of Krust, **Approach A (Aggressive Batching)** is the most practical starting point. It provides a solid foundation for high throughput and can be extended later with the other, more advanced approaches as the project matures.
