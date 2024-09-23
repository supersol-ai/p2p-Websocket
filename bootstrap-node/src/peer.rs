use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Peer {
    pub id: usize,
    pub addr: String,
}
