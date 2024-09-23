## Overview

1. All peers can communicate with each other.
2. A leader is selected using the maximum random number strategy.
3. Peers can communicate with the leader and vice versa.
4. Messages are serialized/deserialized using Serde.

## Prerequisites

- `tokio`: Asynchronous runtime.
- `tungstenite`: For WebSocket communication.
- `serde` and `serde_json`: For serialization/deserialization.
- `rand`: For random number generation.

## Steps

### Defining the Message Structure

`src/message.rs`

This file defines the message types used in the network and handles serialization/deserialization using Serde.

```rust
// src/message.rs
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum MessageType {
    RandomNumber(u64),
    LeaderAnnouncement { leader_id: u64 },
    PeerMessage { from_id: u64, content: String },
    LeaderMessage { content: String },
}
```

`MessageType` is an enum representing different message types that can be exchanged between peers.



