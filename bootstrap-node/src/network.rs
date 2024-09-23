use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;


// src/network.rs
use crate::message::BootstrapMessage;
use crate::peer::Peer;
use futures::{SinkExt, StreamExt};
use tokio_tungstenite::{accept_async, tungstenite::protocol::Message};

pub async fn handle_connection(stream: tokio::net::TcpStream, peers: Arc<Mutex<HashMap<usize, Peer>>>) {
    let ws_stream = accept_async(stream).await.expect("Failed to accept");
    let (mut write, mut read) = ws_stream.split();

    while let Some(msg) = read.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                let message: BootstrapMessage = serde_json::from_str(&text).unwrap();
                match message {
                    BootstrapMessage::Register { id, addr } => {
                        println!("Registering peer {} at {}", id, addr);
                        // Add to peer list
                        let peer = Peer {
                            id,
                            addr: addr.clone(),
                        };
                        let mut peers_guard = peers.lock().await;
                        peers_guard.insert(id, peer.clone());
                        // Send current peer list to the new peer
                        let current_peers: Vec<Peer> = peers_guard.values().cloned().collect();
                        let peer_list_msg = BootstrapMessage::PeerList {
                            peers: current_peers,
                        };
                        let msg_text = serde_json::to_string(&peer_list_msg).unwrap();
                        write.send(Message::Text(msg_text)).await.unwrap();

                        // Notify existing peers about the new peer
                        let new_peer_msg = BootstrapMessage::NewPeer { peer };
                        let msg_text = serde_json::to_string(&new_peer_msg).unwrap();
                        broadcast(&peers, &msg_text, id).await;
                    }
                    BootstrapMessage::Deregister { id } => {
                        println!("Deregistering peer {}", id);
                        let mut peers_guard = peers.lock().await;
                        peers_guard.remove(&id);
                        // Notify existing peers to remove this peer
                        let remove_peer_msg = BootstrapMessage::RemovePeer { id };
                        let msg_text = serde_json::to_string(&remove_peer_msg).unwrap();
                        drop(peers_guard); // Release the lock before calling broadcast
                        broadcast(&peers, &msg_text, id).await;
                    }
                    _ => {
                        println!("Unknown message from peer");
                    }
                }
            }
            Ok(Message::Close(_)) => {
                println!("Connection closed");
                break;
            }
            Err(e) => {
                eprintln!("Error receiving message: {}", e);
                break;
            }
            _ => {}
        }
    }
}

pub async fn broadcast(peers: &Arc<Mutex<HashMap<usize, Peer>>>, msg: &str, exclude_id: usize) {
    let peers_guard = peers.lock().await;
    for (id, peer) in peers_guard.iter() {
        if *id == exclude_id {
            continue;
        }

        // Connect to the peer and send the message
        let addr = &peer.addr;
        let url = format!("ws://{}", addr);
        match tokio_tungstenite::connect_async(&url).await {
            Ok((ws_stream, _)) => {
                let (mut write, _) = ws_stream.split();
                if let Err(e) = write.send(Message::Text(msg.to_string())).await {
                    eprintln!("Failed to send message to {}: {}", addr, e);
                }
            }
            Err(e) => {
                eprintln!("Failed to connect to {}: {}", addr, e);
            }
        }
    }
}
