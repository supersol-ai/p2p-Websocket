use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum MessageType {
    RandomNumber(u64),
    LeaderAnnouncement { leader_id: u64 },
    PeerMessage { from_id: u64, content: String },
    LeaderMessage { content: String },
}