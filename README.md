# Krust: A Cloud-Native Message Bus for the Modern Era

Krust is a next-generation, distributed message bus built in Rust. It is designed from the ground up to be fully cloud-compatible, leveraging external blob storage (like Amazon S3) to provide a durable, scalable, and cost-effective messaging solution. Inspired by the architectural principles of systems like Apache Kafka and WarpStream, Krust aims to provide the power of a log-based message bus with the operational simplicity of a stateless, cloud-native application.

## Getting Started

### Prerequisites

You'll need Rust installed on your system. If you don't have it yet:

**Install Rust** (all platforms):
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

After installation, restart your terminal or run:
```bash
source $HOME/.cargo/env
```

Verify installation:
```bash
rustc --version
cargo --version
```

### Running the Project

**1. Clone the repository:**
```bash
git clone https://github.com/danieleascione/krusta.git
cd krusta
```

**2. Run tests:**
```bash
cargo test
```
This will:
- Download and compile all dependencies (first run takes a few minutes)
- Run all unit tests
- Show you test results

**3. Build the project:**
```bash
cargo build
```
For an optimized release build:
```bash
cargo build --release
```

**4. Run the application:**
```bash
cargo run
```

**5. Check your code (linting):**
```bash
cargo clippy
```

**6. Format your code:**
```bash
cargo fmt
```

### Common Cargo Commands

- `cargo test` - Run all tests
- `cargo test test_name` - Run a specific test
- `cargo run` - Build and run the application
- `cargo build` - Compile the project
- `cargo build --release` - Compile with optimizations
- `cargo check` - Quickly check if code compiles (no binary)
- `cargo clean` - Remove build artifacts
- `cargo doc --open` - Generate and open documentation

### Project Status

Currently implemented (using TDD):
- ✅ Basic append-only log structure
- ✅ In-memory message storage
- ✅ Append and read operations

Next steps:
- ⏳ S3 storage backend integration
- ⏳ Batching and segment management
- ⏳ Stateless agent with HTTP endpoints

## Core Concepts

At its heart, Krust is a **distributed, append-only log**. This simple yet powerful data structure provides an immutable, ordered sequence of records. Messages are written to the end of the log and are identified by an offset. Consumers can read from any point in the log, allowing for flexible and replayable data consumption patterns. This log-centric design is the foundation for Krust's durability, scalability, and performance.

## Proposed Architecture

Krust's architecture is founded on a key design philosophy: the **separation of concerns**. By decoupling different parts of the system, we can achieve greater scalability, resilience, and operational efficiency.

### Design Philosophy

1.  **Separation of Storage and Compute**: Traditional message brokers often colocate storage and compute on the same nodes. Krust separates these, using a dedicated object storage service (like S3) for data persistence and a separate set of stateless compute nodes (Agents) for processing. This allows compute resources to be scaled independently based on load, without the need for complex and costly data rebalancing.

2.  **Separation of Data and Metadata**: The actual message data (the log segments) is stored in the object storage layer. The metadata—information about topics, partitions, offsets, and the location of data files—is managed by a dedicated metadata service. This separation enhances security, as the control plane does not need access to the raw message data, and allows for specialized, highly-optimized stores for each type of information.

### System Components

The Krust architecture consists of three primary components:

| Component | Description | Technology | Responsibilities |
| :--- | :--- | :--- | :--- |
| **Agent** | A stateless Rust binary that acts as the broker. | Rust, Tokio | - Handle client connections (producers/consumers)<br>- Batch and write messages to the Storage Layer<br>- Serve read requests by fetching data from storage<br>- Communicate with the Metadata Layer for offset management | 
| **Storage Layer** | A durable, highly-available object store. | S3-compatible APIs | - Persist all log segment data<br>- Ensure data durability and availability | 
| **Metadata Layer** | A service responsible for all system metadata. | Pluggable (e.g., Etcd, PostgreSQL, or embedded DB) | - Manage topic and partition information<br>- Track consumer group offsets<br>- Maintain the mapping between log offsets and files in the Storage Layer | 

![Krust Architecture Diagram](https://i.imgur.com/example.png)  
*A high-level diagram illustrating the separation of Agent (compute), S3 (storage), and the Metadata Layer.*

### Data Flow

*   **Write Path**: A producer sends a message to an Agent. The Agent batches incoming messages for a given partition and writes them as a single object to the Storage Layer. Upon successful write, it updates the Metadata Layer with the new offset range for that object.

*   **Read Path**: A consumer requests messages from a specific offset. The Agent queries the Metadata Layer to find the object(s) in the Storage Layer that contain the requested offset range. The Agent then fetches these objects and streams the data back to the consumer.

## Key Features

- **Cloud-Native**: Built to run seamlessly in modern cloud environments, leveraging the scalability and durability of object storage.
- **Cost-Effective**: Dramatically reduces operational costs by using cheap, elastic object storage and avoiding expensive cross-AZ data replication.
- **Highly Scalable**: Stateless agents can be auto-scaled based on CPU and network load, without data rebalancing.
- **Durable & Available**: Data durability and availability are offloaded to the underlying object storage provider (e.g., S3's 99.999999999% durability).
- **Simple to Operate**: The stateless nature of the agents makes deployment, management, and failure recovery trivial.
- **Performance & Safety**: Built in Rust for memory safety, concurrency, and high performance.

## Project Roadmap

This project is in its initial stages. The following is a proposed roadmap for development:

1.  **Core Log Structure**: Implement the fundamental append-only log data structure.
2.  **Stateless Agent**: Develop the initial version of the stateless agent with basic produce/consume logic.
3.  **S3 Storage Backend**: Integrate with S3-compatible object storage for data persistence.
4.  **Pluggable Metadata Layer**: Design and implement the metadata service with an initial backend (e.g., an embedded database).
5.  **Consumer Groups**: Implement Kafka-style consumer groups with offset tracking.
6.  **Protocol Compatibility**: Add a compatibility layer for the Apache Kafka protocol to allow existing clients to connect.

---

*This README outlines the initial vision and architecture for Krust. As the project evolves, this document will be updated to reflect the latest design and implementation details.*
