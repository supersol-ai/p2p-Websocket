// src/network.rs
use crate::message::{BootstrapMessage, MessageType};
use crate::peer::Peer;
use futures::{SinkExt, StreamExt};
use rand::Rng as _;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{accept_async, connect_async, tungstenite::protocol::Message};
pub struct Network {
    pub id: usize,
    pub addr: String,
    pub peers: Vec<Peer>,
    pub leader_id: Option<usize>,
}

impl Network {
    pub fn new(id: usize, addr: String) -> Self {
        Network {
            id,
            addr,
            peers: Vec::new(),
            leader_id: None,
        }
    }

    pub async fn run(&mut self, bootstrap_addr: String) {
        // Connect to bootstrap node
        self.register_with_bootstrap(&bootstrap_addr).await;

        // Start server to accept incoming peer connections
        let addr_clone = self.addr.clone();
        let id_clone = self.id;
        tokio::spawn(async move {
            start_server(&addr_clone, id_clone).await;
        });

        // Handle incoming messages
        // In a real implementation, you would have a message handling loop here

        // Leader election logic
        // For simplicity, we'll generate a random number and send to all peers
        self.leader_election().await;
    }

    async fn register_with_bootstrap(&mut self, bootstrap_addr: &str) {
        let url = format!("ws://{}", bootstrap_addr);
        match connect_async(&url).await {
            Ok((ws_stream, _)) => {
                let (mut write, mut read) = ws_stream.split();

                // Send Register message
                let register_msg = BootstrapMessage::Register {
                    id: self.id,
                    addr: self.addr.clone(),
                };
                let msg_text = serde_json::to_string(&register_msg).unwrap();
                write.send(Message::Text(msg_text)).await.unwrap();

                // Await PeerList
                if let Some(Ok(Message::Text(text))) = read.next().await {
                    let message: BootstrapMessage = serde_json::from_str(&text).unwrap();
                    match message {
                        BootstrapMessage::PeerList { peers } => {
                            println!("Received peer list: {:?}", peers);
                            self.peers = peers;
                        },
                        _ => {
                            println!("Unexpected message from bootstrap node");
                        }
                    }
                }

                // Handle incoming messages from bootstrap node
                tokio::spawn(async move {
                    while let Some(Ok(msg)) = read.next().await {
                        if let Message::Text(text) = msg {
                            let message: BootstrapMessage = serde_json::from_str(&text).unwrap();
                            match message {
                                BootstrapMessage::NewPeer { peer } => {
                                    println!("New peer joined: {:?}", peer);
                                    // Connect to the new peer
                                    connect_to_peer(peer).await;
                                },
                                BootstrapMessage::RemovePeer { id } => {
                                    println!("Peer {} disconnected", id);
                                    // Handle peer removal
                                    // In this simplified example, we're not maintaining a connection list
                                },
                                _ => {
                                    println!("Unknown message from bootstrap node");
                                }
                            }
                        }
                    }
                });
            },
            Err(e) => {
                eprintln!("Failed to connect to bootstrap node: {}", e);
            }
        }
    }

    pub async fn remove_peer(&mut self, id: usize, bootstrap_addr: String) {
        let url = format!("ws://{}", bootstrap_addr);
        match connect_async(&url).await {
            Ok((ws_stream, _)) => {
                let (mut write, mut read) = ws_stream.split();
                let remove_msg = BootstrapMessage::Deregister { id };
                let msg_text = serde_json::to_string(&remove_msg).unwrap();
                write.send(Message::Text(msg_text)).await.unwrap();
                println!("Peer {} removed from bootstrap node", id);
            },
            Err(e) => {
                eprintln!("Failed to connect to bootstrap node: {}", e);
            }
        }
    }

    async fn leader_election(&mut self) {
        let my_random_number = rand::thread_rng().gen_range(1..100);
        println!("Peer {} generated random number: {}", self.id, my_random_number);

        // Send RandomNumber to all peers
        for peer in &self.peers {
            send_message(&peer.addr, MessageType::RandomNumber(my_random_number)).await;
        }

        // For demonstration, let's assume we collected random numbers
        // In a real implementation, you would collect these from incoming messages
        let mut random_numbers = vec![(self.id, my_random_number)];
        // Simulate receiving random numbers from peers
        random_numbers.push((2, 50)); // Example peer
        random_numbers.push((3, 75)); // Example peer

        // Determine the leader
        random_numbers.sort_by(|a, b| b.1.cmp(&a.1));
        let leader_id = random_numbers.first().unwrap().0;
        self.leader_id = Some(leader_id);
        println!("Leader elected: Peer {}", leader_id);

        // Announce the leader to all peers
        for peer in &self.peers {
        let announcement = MessageType::LeaderAnnouncement { leader_id };
            send_message(&peer.addr, announcement).await;
        }

        if leader_id == self.id {
            println!("I am the leader!");
            // Leader-specific actions
        } else {
            println!("I am a regular peer.");
        }
    }
}

async fn send_message(addr: &str, message: MessageType) {
    let url = format!("ws://{}", addr);
    match connect_async(&url).await {
        Ok((mut ws_stream, _)) => {
            let msg_text = serde_json::to_string(&message).unwrap();
            ws_stream.send(Message::Text(msg_text)).await.unwrap();
        },
        Err(e) => {
            eprintln!("Failed to connect to {}: {}", addr, e);
        }
    }
}

async fn connect_to_peer(peer: Peer) {
    let url = format!("ws://{}", peer.addr);
    match connect_async(&url).await {
        Ok((mut ws_stream, _)) => {
            println!("Connected to peer {}", peer.id);
            // Send a welcome message or perform handshake
            let welcome_msg = MessageType::PeerMessage {
                from_id: 0, // Or your own ID
                content: "Hello from new peer!".to_string(),
            };
            let msg_text = serde_json::to_string(&welcome_msg).unwrap();
            ws_stream.send(Message::Text(msg_text)).await.unwrap();

            // Handle incoming messages from the peer
            tokio::spawn(async move {
                let (_, mut read) = ws_stream.split();
                while let Some(Ok(Message::Text(text))) = read.next().await {
                    let message: MessageType = serde_json::from_str(&text).unwrap();
                    println!("Received from peer: {:?}", message);
                    // Handle message accordingly
                }
            });
        },
        Err(e) => {
            eprintln!("Failed to connect to peer {}: {}", peer.id, e);
        }
    }
}

pub async fn start_server(addr: &str, id: usize) {
    let listener = TcpListener::bind(addr).await.expect("Failed to bind");
    println!("Peer {} listening on: {}", id, addr);

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(async move {
            handle_peer_connection(stream, id).await;
        });
    }
}

async fn handle_peer_connection(stream: TcpStream, id: usize) {
    let ws_stream = accept_async(stream).await.expect("Failed to accept");
    let (mut write, mut read) = ws_stream.split();

    while let Some(Ok(msg)) = read.next().await {
        if let Message::Text(text) = msg {
            let message: MessageType = serde_json::from_str(&text).unwrap();
            println!("Peer {} received: {:?}", id, message);

            // Handle message based on type
            match message {
                MessageType::PeerMessage { from_id, content } => {
                    println!("Peer {} received message from Peer {}: {}", id, from_id, content);
                    // Optionally, respond or perform actions
                },
                MessageType::LeaderAnnouncement { leader_id } => {
                    println!("Peer {} acknowledges Leader: Peer {}", id, leader_id);
                    // Update leader status if needed
                },
                _ => {
                    println!("Unknown message type");
                }
            }
        }
    }
}

pub fn generate_peer_id() -> usize {
    let mut rng = rand::thread_rng();
    rng.gen_range(1..=1_000_000) // Generates a peer ID between 1 and 1,000,000
}

/// Assigns a unique peer address by selecting an available port.
///
/// Returns:
/// - `Option<String>`: Some(address) if successful, None otherwise.
pub fn generate_peer_addr() -> Option<String> {
    let base_ip = "127.0.0.1";
    let base_port = 9000;
    let max_attempts = 100;

    let mut rng = rand::thread_rng();

    for _ in 0..max_attempts {
        // Randomly select a port within a range to minimize conflicts
        let port = rng.gen_range(base_port..base_port + 10_000);
        let addr = format!("{}:{}", base_ip, port);
        // Attempt to bind to the address to ensure it's available
        if std::net::TcpListener::bind(&addr).is_ok() {
            // Successfully bound, so the port is available
            return Some(addr);
        }
        // If binding fails, the port is likely in use. Retry.
    }

    // Failed to find an available port after max_attempts
    None
}
