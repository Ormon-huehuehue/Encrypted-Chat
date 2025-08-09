use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::sync::Arc;
use tokio::sync::Mutex;

pub mod messaging;

const AES_KEY: &[u8; 32] = b"anexampleverysecurekey32bytes!!!";

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("server is running on 127.0.0.1:8080");

    let mut client_sockets: Vec<TcpStream> = Vec::with_capacity(2);

    loop {
        if client_sockets.len() < 2 {
            let (socket, addr) = listener.accept().await?;
            println!("New connection : {}", addr);
            client_sockets.push(socket);

            if client_sockets.len() == 2 {
                println!("Two clients connected. Starting chat session.");
                let socket1 = client_sockets.remove(0);
                let socket2 = client_sockets.remove(0);

                tokio::spawn(async move {
                    if let Err(e) = handle_chat(socket1, socket2).await {
                        eprintln!("Chat session error: {}", e);
                    }
                });
            }
        } else {
            // If 2 clients are already connected, we can choose to:
            // 1. Reject new connections (as implemented below)
            // 2. Wait for a client to disconnect before accepting a new one (more complex)
            let (mut socket, addr) = listener.accept().await?;
            println!("Connection limit reached. Rejecting new connection from {}", addr);
            let _ = socket.write_all(b"Server full. Try again later.\n").await;
        }
    }
}

async fn handle_chat(mut socket1: TcpStream, mut socket2: TcpStream) -> std::io::Result<()> {
    let (mut reader1, mut writer1) = socket1.split();
    let (mut reader2, mut writer2) = socket2.split();

    let client1_to_client2 = async {
        let mut buffer = [0u8; 1024];
        loop {
            let n = reader1.read(&mut buffer).await?;
            if n == 0 { break; }

            if n < 12 {
                eprintln!("Received data too short to contain a nonce from client 1.");
                continue;
            }
            let mut received_nonce = [0u8; 12];
            received_nonce.copy_from_slice(&buffer[0..12]);
            let received_ciphertext = &buffer[12..n];

            let decrypted_msg = messaging::decrypt_message(received_ciphertext, received_nonce, AES_KEY);
            let decrypted_msg_str = String::from_utf8_lossy(&decrypted_msg);
            println!("Decrypted message from client 1: {}", decrypted_msg_str);

            // Encrypt and send to client 2
            let (encrypted_response, response_nonce) = messaging::encrypt_message(decrypted_msg_str.as_bytes(), AES_KEY);
            let mut data_to_send = Vec::with_capacity(response_nonce.len() + encrypted_response.len());
            data_to_send.extend_from_slice(&response_nonce);
            data_to_send.extend_from_slice(&encrypted_response);

            if writer2.write_all(&data_to_send).await.is_err() {
                eprintln!("Error writing to socket");
                break;
            }
        }
        Ok::<(), std::io::Error>(())
    };

    let client2_to_client1 = async {
        let mut buffer = [0u8; 1024];
        loop {
            let n = reader2.read(&mut buffer).await?;
            if n == 0 { break; }

            if n < 12 {
                eprintln!("Received data too short to contain a nonce from client 2.");
                continue;
            }
            let mut received_nonce = [0u8; 12];
            received_nonce.copy_from_slice(&buffer[0..12]);
            let received_ciphertext = &buffer[12..n];

            let decrypted_msg = messaging::decrypt_message(received_ciphertext, received_nonce, AES_KEY);
            let decrypted_msg_str = String::from_utf8_lossy(&decrypted_msg);
            println!("Decrypted message from client 2: {}", decrypted_msg_str);

            // Encrypt and send to client 1
            let (encrypted_response, response_nonce) = messaging::encrypt_message(decrypted_msg_str.as_bytes(), AES_KEY);
            let mut data_to_send = Vec::with_capacity(response_nonce.len() + encrypted_response.len());
            data_to_send.extend_from_slice(&response_nonce);
            data_to_send.extend_from_slice(&encrypted_response);

            if writer1.write_all(&data_to_send).await.is_err() {
                eprintln!("Error writing to socket");
                break;
            }
        }
        Ok::<(), std::io::Error>(())
    };

    tokio::select! {
        _ = client1_to_client2 => Ok(()),
        _ = client2_to_client1 => Ok(()),
    }
}
