// src/network.rs
use crate::message::MessageType;
use crate::peer::Peer;
use futures::{SinkExt, StreamExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{accept_async, connect_async, tungstenite::protocol::Message};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn start_server(addr: &str, peers: Arc<Mutex<Vec<Peer>>>, peer_id: u64) {
    let listener = TcpListener::bind(addr).await.expect("Failed to bind");
    println!("Server listening on: {}", addr);

    while let Ok((stream, _)) = listener.accept().await {
        let peer_list = peers.clone();
        tokio::spawn(async move {
            handle_connection(stream, peer_list, peer_id).await;
        });
    }
}

async fn handle_connection(stream: TcpStream, peers: Arc<Mutex<Vec<Peer>>>, peer_id: u64) {
    let addr = stream.peer_addr().expect("Connected streams should have a peer address");
    println!("Peer connected: {}", addr);

    let ws_stream = accept_async(stream).await.expect("Failed to accept");
    let (mut write, mut read) = ws_stream.split();

    while let Some(msg) = read.next().await {
        let msg = msg.expect("Failed to read message");
        if msg.is_text() {
            let text = msg.into_text().unwrap();
            let message: MessageType = serde_json::from_str(&text).unwrap();
            // Handle the message accordingly
            println!("Received message from {}: {:?}", addr, message);
        }
    }
}

pub async fn connect_to_peers(
    peer_addrs: Vec<String>,
    peers: Arc<Mutex<Vec<Peer>>>,
    peer_id: u64,
) {
    for addr in peer_addrs {
        let url = format!("ws://{}", addr);
        match connect_async(&url).await {
            Ok((ws_stream, _)) => {
                let (write, read) = ws_stream.split();
                let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

                // Spawn tasks to read and write messages
                tokio::spawn(write_messages(rx, write));
                tokio::spawn(read_messages(read));

                // Add the peer to the peer list
                let mut peer_list = peers.lock().await;
                peer_list.push(Peer {
                    id: 0, // Assign proper IDs as needed
                    addr: addr.clone(),
                    tx,
                });

                println!("Connected to peer at {}", addr);
            }
            Err(e) => {
                eprintln!("Failed to connect to {}: {}", addr, e);
            }
        }
    }
}

async fn write_messages(
    mut rx: tokio::sync::mpsc::UnboundedReceiver<Message>,
    mut write: impl SinkExt<Message> + Unpin,
) {
    while let Some(msg) = rx.recv().await {
        if let Err(e) = write.send(msg).await {
            // eprintln!("Error sending message: {:?}", e);
        }
    }
}

async fn read_messages(mut read: impl StreamExt<Item = Result<Message, tokio_tungstenite::tungstenite::Error>> + Unpin) {
    while let Some(msg) = read.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                let message: MessageType = serde_json::from_str(&text).unwrap();
                println!("Received message: {:?}", message);
                // Handle the message accordingly
            }
            Ok(_) => {}
            Err(e) => {
                eprintln!("Error receiving message: {}", e);
                break;
            }
        }
    }
}

pub async fn leader_election(
    peers: &Arc<Mutex<Vec<Peer>>>,
    peer_id: u64,
    my_random_number: u64,
) -> u64 {
    // Broadcast your random number to all peers
    let message = MessageType::RandomNumber(my_random_number);
    broadcast_message(peers, message).await;

    // Collect random numbers from all peers including yourself
    let mut random_numbers = vec![(peer_id, my_random_number)];

    // Assume we have a way to collect random numbers from other peers
    // For simplicity, we can simulate receiving numbers here

    // TODO: Implement actual collection of random numbers

    // Determine the leader (peer with the highest random number)
    random_numbers.sort_by(|a, b| b.1.cmp(&a.1)); // Sort descending
    let leader_id = random_numbers.first().unwrap().0;
    println!("Leader elected: Peer {}", leader_id);

    // Announce the leader to all peers
    let announcement = MessageType::LeaderAnnouncement { leader_id };
    broadcast_message(peers, announcement).await;

    leader_id
}

async fn broadcast_message(peers: &Arc<Mutex<Vec<Peer>>>, message: MessageType) {
    let msg_text = serde_json::to_string(&message).unwrap();
    let msg = Message::Text(msg_text);

    let peer_list = peers.lock().await;
    for peer in peer_list.iter() {
        let _ = peer.tx.send(msg.clone());
    }
}

pub async fn send_message_to_leader(
    peers: &Arc<Mutex<Vec<Peer>>>,
    leader_id: u64,
    peer_id: u64,
    content: String,
) {
    let message = MessageType::PeerMessage {
        from_id: peer_id,
        content,
    };
    let msg_text = serde_json::to_string(&message).unwrap();
    let msg = Message::Text(msg_text);

    let peer_list = peers.lock().await;
    if let Some(leader_peer) = peer_list.iter().find(|peer| peer.id == leader_id) {
        let _ = leader_peer.tx.send(msg);
    } else {
        eprintln!("Leader not found");
    }
}

pub async fn leader_broadcast(
    peers: &Arc<Mutex<Vec<Peer>>>,
    content: String,
) {
    let message = MessageType::LeaderMessage { content };
    broadcast_message(peers, message).await;
}