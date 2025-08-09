use tokio::net::TcpStream;
use tokio::io::{self, AsyncWriteExt, AsyncReadExt, AsyncBufReadExt};

pub mod messaging;

const AES_KEY: &[u8; 32] = b"anexampleverysecurekey32bytes!!!";

#[tokio::main]
async fn main() -> io::Result<()> {
    let stream = TcpStream::connect("127.0.0.1:8080").await?;
    println!("Connected to server. Type your message:");

    let (mut reader, mut writer) = io::split(stream);

    // Task for receiving messages
    tokio::spawn(async move {
        let mut buffer = [0u8; 1024];
        loop {
            let n = match reader.read(&mut buffer).await {
                Ok(n) if n == 0 => {
                    println!("Server disconnected.");
                    break;
                },
                Ok(n) => n,
                Err(e) => {
                    eprintln!("Error reading from socket: {}", e);
                    break;
                }
            };

            if n < 12 {
                eprintln!("Received data too short to contain a nonce.");
                continue;
            }
            let mut received_nonce = [0u8; 12];
            received_nonce.copy_from_slice(&buffer[0..12]);
            let received_ciphertext = &buffer[12..n];

            let decrypted_msg = messaging::decrypt_message(received_ciphertext, received_nonce, AES_KEY);
            let decrypted_msg_str = String::from_utf8_lossy(&decrypted_msg);
            println!("Received: {}", decrypted_msg_str);
        }
    });

    // Main loop for sending messages
    let mut stdin = io::BufReader::new(io::stdin());
    loop {
        let mut buf = String::new();
        stdin.read_line(&mut buf).await?;

        let trimmed_msg = buf.trim();
        if trimmed_msg.is_empty() {
            continue;
        }

        let (encrypted_msg, nonce) = messaging::encrypt_message(trimmed_msg.as_bytes(), AES_KEY);

        let mut data_to_send = Vec::with_capacity(nonce.len() + encrypted_msg.len());
        data_to_send.extend_from_slice(&nonce);
        data_to_send.extend_from_slice(&encrypted_msg);

        if writer.write_all(&data_to_send).await.is_err() {
            eprintln!("Error writing to socket.");
            break;
        }
    }

    Ok(())
}