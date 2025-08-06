use tokio::net::TcpStream; // Import TcpStream for establishing TCP connections.
use tokio::io::{self, AsyncWriteExt, AsyncReadExt, AsyncBufReadExt}; // Import necessary I/O traits for async operations.

pub mod messaging;

const AES_KEY: &[u8; 32] = b"anexampleverysecurekey32bytes!!!";

#[tokio::main] // Marks the main function as the entry point for the Tokio runtime.
async fn main() -> io::Result<()> { // Defines the asynchronous main function, returning a Result for I/O operations.
    // Connect to the server running on localhost at port 8080.
    let mut stream = TcpStream::connect("127.0.0.1:8080").await?;
    println!("Connected to server. Type your message:"); // Inform the user that the connection is established.

    let mut buf = String::new(); // Create a new mutable string to store user input.
    // Create a buffered reader for standard input to efficiently read lines.
    let mut stdin = io::BufReader::new(io::stdin());
    // Read a line from standard input into the buffer, awaiting the operation.
    stdin.read_line(&mut buf).await?;

    // Encrypt the message.
    let (encrypted_msg, nonce) = messaging::encrypt_message(buf.trim().as_bytes(), AES_KEY);

    // Combine nonce and ciphertext for sending.
    let mut data_to_send = Vec::with_capacity(nonce.len() + encrypted_msg.len());
    data_to_send.extend_from_slice(&nonce);
    data_to_send.extend_from_slice(&encrypted_msg);

    // Write the encrypted data to the TCP stream, awaiting the operation.
    stream.write_all(&data_to_send).await?;

    let mut resp = [0u8; 1024]; // Create a mutable byte array to store the server's response.
    // Read data from the TCP stream into the response buffer, awaiting the operation.
    let n = stream.read(&mut resp).await?;

    // Extract nonce (first 12 bytes) and ciphertext from the response.
    if n < 12 {
        eprintln!("Received data too short to contain a nonce.");
        return Ok(());
    }
    let mut received_nonce = [0u8; 12];
    received_nonce.copy_from_slice(&resp[0..12]);
    let received_ciphertext = &resp[12..n];

    // Decrypt the server's reply.
    let decrypted_reply = messaging::decrypt_message(received_ciphertext, received_nonce, AES_KEY);
    // Print the server's decrypted reply, converting the received bytes to a lossy UTF-8 string.
    println!("Server replied: {}", String::from_utf8_lossy(&decrypted_reply));

    Ok(()) // Return Ok to indicate successful execution.
}