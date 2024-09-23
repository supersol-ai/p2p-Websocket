use serde::{Deserialize, Serialize};
use crate::peer::Peer;

#[derive(Serialize, Deserialize, Debug)]
pub enum MessageType {
    RandomNumber(u64),
    LeaderAnnouncement { leader_id: usize },
    PeerMessage { from_id: usize, content: String },
    LeaderMessage { content: String },
}

#[derive(Serialize, Deserialize, Debug)]
pub enum BootstrapMessage {
    Register { id: usize, addr: String },
    Deregister { id: usize },
    PeerList { peers: Vec<Peer> },
    NewPeer { peer: Peer },
    RemovePeer { id: usize },
}
