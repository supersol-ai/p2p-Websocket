// src/main.rs
mod message;
mod network;
mod peer;

use crate::network::{connect_to_peers, leader_election, start_server};
use rand::Rng;
use std::env;
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() {
    // Parse command-line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: cargo run <peer_id> <bind_addr> [peer_addr1] [peer_addr2] ...");
        return;
    }

    let peer_id: u64 = args[1].parse().expect("Invalid peer ID");
    let bind_addr = args[2].clone();
    let peer_addrs = args[3..].to_vec();

    // Shared state for peers
    let peers = Arc::new(Mutex::new(Vec::new()));

    // Start the server
    let server_peers = peers.clone();
    tokio::spawn(async move {
        start_server(&bind_addr, server_peers, peer_id).await;
    });

    // Connect to other peers
    connect_to_peers(peer_addrs, peers.clone(), peer_id).await;

    // Generate a random number for leader election
    let my_random_number = rand::thread_rng().gen_range(1..100);
    println!("Peer {} generated random number: {}", peer_id, my_random_number);

    // Leader election
    let leader_id = leader_election(&peers, peer_id, my_random_number).await;

    if leader_id == peer_id {
        println!("I am the leader!");
        // Leader-specific actions
    } else {
        println!("I am a regular peer.");
        // Send a message to the leader
        network::send_message_to_leader(&peers, leader_id, peer_id, "Hello, Leader!".to_string()).await;
    }

    // Keep the main function alive
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
    }
}
