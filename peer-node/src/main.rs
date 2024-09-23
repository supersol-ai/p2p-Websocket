// src/main.rs
mod message;
mod network;
mod peer;

use network::{generate_peer_addr, generate_peer_id, Network};
use tokio::signal;
use std::env;

#[tokio::main]
async fn main() {
     // Collect command-line arguments
     let args: Vec<String> = env::args().collect();

     // Check if exactly one argument (bootstrap_addr) is provided
     if args.len() != 2 {
         eprintln!("Usage: cargo run -- <bootstrap_addr>");
         eprintln!("Example: cargo run -- 127.0.0.1:8080");
     }
 
     let bootstrap_addr = args[1].clone();
 
     // Automatically generate a unique peer_id
     let peer_id = generate_peer_id();
 
     // Automatically assign a unique peer_addr
     let peer_addr = match generate_peer_addr() {
         Some(addr) => addr,
         None => {
             eprintln!("Error: Unable to assign a unique peer address.");
             return;
         }
     };
 
     println!("Assigned Peer ID: {}", peer_id);
     println!("Assigned Peer Address: {}", peer_addr);
     println!("Bootstrap Address: {}", bootstrap_addr);

    let mut network = Network::new(peer_id, peer_addr.clone());

    let bootstrap_addr_clone = bootstrap_addr.clone();
    network.run(bootstrap_addr).await;

    let shutdown_handle = tokio::spawn(async move {
        // Wait for Ctrl+C
        signal::ctrl_c().await.expect("Failed to listen for Ctrl+C");

        println!("\nReceived Ctrl+C, shutting down...");

        network.remove_peer(peer_id, bootstrap_addr_clone).await;

        // Exit the program
        std::process::exit(0);
    });

    // Wait for the shutdown handler to complete
    shutdown_handle.await.expect("Failed to join shutdown handle");
    // Keep the main function alive
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
    }
}
