use tokio::sync::mpsc::UnboundedSender;
use tokio_tungstenite::tungstenite::Message;

#[derive(Debug, Clone)]
pub struct Peer {
    pub id: u64,
    pub addr: String,
    pub tx: UnboundedSender<Message>,
}