use tokio::net::TcpListener;
use tokio_tungstenite::{accept_async, WebSocketStream};
use tokio_tungstenite::tungstenite::Message;
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::sync::Mutex;
use serde_json::{json, Value};
use std::collections::HashMap;
use sha2::{Sha256, Digest};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let port = std::env::var("PORT").unwrap_or_else(|_| "8001".to_string());
    let addr = format!("0.0.0.0:{}", port);

    let listener = TcpListener::bind(&addr).await?;
    println!("WebSocket server is running on ws://0.0.0.0:{}", port);

    // Store waiting clients by their shared hash
    let waiting_clients: Arc<Mutex<HashMap<String, WebSocketStream<tokio::net::TcpStream>>>> = Arc::new(Mutex::new(HashMap::new()));

    loop {
        let (tcp_stream, addr) = listener.accept().await?;
        println!("New connection from: {}", addr);
        
        let waiting_clients = waiting_clients.clone();
        
        // Accept the WebSocket connection
        match accept_async(tcp_stream).await {
            Ok(ws_stream) => {
                println!("WebSocket connection established from: {}", addr);
                
                tokio::spawn(async move {
                    if let Err(e) = handle_client_connection(ws_stream, waiting_clients).await {
                        eprintln!("Client connection error: {}", e);
                    }
                });
            }
            Err(e) => {
                eprintln!("WebSocket handshake failed for {}: {}", addr, e);
            }
        }
    }
}

async fn handle_client_connection(
    ws_stream: WebSocketStream<tokio::net::TcpStream>,
    waiting_clients: Arc<Mutex<HashMap<String, WebSocketStream<tokio::net::TcpStream>>>>
) -> Result<(), Box<dyn std::error::Error>> {
    let (mut write, mut read) = ws_stream.split();

    // Send welcome message
    let welcome_msg = json!({
        "type": "welcome",
        "message": "Connected to chat server. Send your username and target username to start chatting."
    });
    write.send(Message::Text(welcome_msg.to_string())).await?;

    while let Some(msg) = read.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                match serde_json::from_str::<Value>(&text) {
                    Ok(json_msg) => {
                        if let Some(msg_type) = json_msg.get("type").and_then(|v| v.as_str()) {
                            match msg_type {
                                "init_chat" => {
                                    if let (Some(username), Some(target_username)) = (
                                        json_msg.get("username").and_then(|v| v.as_str()),
                                        json_msg.get("target_username").and_then(|v| v.as_str())
                                    ) {
                                        let shared_hash = generate_shared_hash(username, target_username);
                                        println!("Client {} wants to chat with {} (hash: {})", username, target_username, shared_hash);
                                        
                                        // Check if target client is waiting
                                        let mut clients = waiting_clients.lock().await;
                                        if let Some(target_socket) = clients.remove(&shared_hash) {
                                            // Found matching client, start chat session
                                            println!("Starting chat session between {} and {}", username, target_username);
                                            
                                            // Send success message to both clients
                                            let success_msg = json!({
                                                "type": "chat_started",
                                                "message": format!("Chat session started with {}", target_username)
                                            });
                                            write.send(Message::Text(success_msg.to_string())).await?;
                                            
                                            // Reconstruct the WebSocket stream for the chat session
                                            let ws_stream = write.reunite(read)?;
                                            
                                            // Start the chat session
                                            tokio::spawn(async move {
                                                if let Err(e) = handle_chat_session(ws_stream, target_socket).await {
                                                    eprintln!("Chat session error: {}", e);
                                                }
                                            });
                                            
                                            return Ok(());
                                        } else {
                                            // No matching client found, add this client to waiting list
                                            println!("No matching client found for {}. Adding {} to waiting list.", shared_hash, username);
                                            
                                            let waiting_msg = json!({
                                                "type": "waiting",
                                                "message": format!("Waiting for {} to connect...", target_username),
                                                "shared_hash": shared_hash
                                            });
                                            write.send(Message::Text(waiting_msg.to_string())).await?;
                                            
                                            // Reconstruct the WebSocket stream for storage
                                            let ws_stream = write.reunite(read)?;
                                            clients.insert(shared_hash.clone(), ws_stream);
                                            
                                            return Ok(());
                                        }
                                    } else {
                                        let error_msg = json!({
                                            "type": "error",
                                            "message": "Missing username or target_username"
                                        });
                                        write.send(Message::Text(error_msg.to_string())).await?;
                                    }
                                }
                                "message" => {
                                    // Handle encrypted messages (relay between clients)
                                    // This will be handled in the chat session
                                    println!("Received message in waiting state, ignoring");
                                }
                                _ => {
                                    let error_msg = json!({
                                        "type": "error",
                                        "message": "Unknown message type"
                                    });
                                    write.send(Message::Text(error_msg.to_string())).await?;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to parse JSON: {}", e);
                        let error_msg = json!({
                            "type": "error",
                            "message": "Invalid JSON format"
                        });
                        write.send(Message::Text(error_msg.to_string())).await?;
                    }
                }
            }
            Ok(Message::Close(_)) => {
                println!("Client disconnected");
                break;
            }
            Err(e) => {
                eprintln!("Error reading from client: {}", e);
                break;
            }
            _ => {}
        }
    }
    Ok(())
}

async fn handle_chat_session(
    socket1: WebSocketStream<tokio::net::TcpStream>,
    socket2: WebSocketStream<tokio::net::TcpStream>
) -> Result<(), Box<dyn std::error::Error>> {
    let (mut write1, mut read1) = socket1.split();
    let (mut write2, mut read2) = socket2.split();

    let client1_to_client2 = async {
        while let Some(msg) = read1.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    // Parse the JSON message to validate format
                    match serde_json::from_str::<Value>(&text) {
                        Ok(json_msg) => {
                            if let Some(msg_type) = json_msg.get("type").and_then(|v| v.as_str()) {
                                if msg_type == "message" {
                                    println!("Relaying encrypted message from client 1 to client 2");
                                    
                                    // Simply relay the encrypted message without decrypting
                                    if write2.send(Message::Text(text)).await.is_err() {
                                        eprintln!("Error writing to client 2");
                                        break;
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Failed to parse JSON from client 1: {}", e);
                        }
                    }
                }
                Ok(Message::Close(_)) => {
                    println!("Client 1 disconnected");
                    break;
                }
                Err(e) => {
                    eprintln!("Error reading from client 1: {}", e);
                    break;
                }
                _ => {}
            }
        }
        Ok::<(), Box<dyn std::error::Error>>(())
    };

    let client2_to_client1 = async {
        while let Some(msg) = read2.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    // Parse the JSON message to validate format
                    match serde_json::from_str::<Value>(&text) {
                        Ok(json_msg) => {
                            if let Some(msg_type) = json_msg.get("type").and_then(|v| v.as_str()) {
                                if msg_type == "message" {
                                    println!("Relaying encrypted message from client 2 to client 1");
                                    
                                    // Simply relay the encrypted message without decrypting
                                    if write1.send(Message::Text(text)).await.is_err() {
                                        eprintln!("Error writing to client 1");
                                        break;
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Failed to parse JSON from client 2: {}", e);
                        }
                    }
                }
                Ok(Message::Close(_)) => {
                    println!("Client 2 disconnected");
                    break;
                }
                Err(e) => {
                    eprintln!("Error reading from client 2: {}", e);
                    break;
                }
                _ => {}
            }
        }
        Ok::<(), Box<dyn std::error::Error>>(())
    };

    tokio::select! {
        _ = client1_to_client2 => Ok(()),
        _ = client2_to_client1 => Ok(()),
    }
}

fn generate_shared_hash(username1: &str, username2: &str) -> String {
    // Create a deterministic hash by sorting usernames alphabetically
    let (first, second) = if username1 < username2 {
        (username1, username2)
    } else {
        (username2, username1)
    };
    
    // Combine usernames with a separator
    let combined = format!("{}:{}", first, second);
    
    // Generate SHA-256 hash
    let mut hasher = Sha256::new();
    hasher.update(combined.as_bytes());
    let result = hasher.finalize();
    
    // Return hex string of the hash
    format!("{:x}", result)
}
