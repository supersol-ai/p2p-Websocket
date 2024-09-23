// src/main.rs
mod message;
mod network;
mod peer;

use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;

use crate::network::handle_connection;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    
    let addr = "127.0.0.1:8000";
    let listener = TcpListener::bind(addr).await.expect("Failed to bind");
    println!("Bootstrap node listening on: {}", addr);

    let peers = Arc::new(Mutex::new(HashMap::new()));

    while let Ok((stream, _)) = listener.accept().await {
        let peers = Arc::clone(&peers);
        tokio::spawn(async move {
            handle_connection(stream, peers).await;
        });
    }

    // Keep the main function alive
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
    }
}
