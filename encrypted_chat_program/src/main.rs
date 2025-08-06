use tokio::net::TcpListener; // Import TcpListener for creating a TCP server.
use tokio::io::{AsyncReadExt, AsyncWriteExt}; // Import necessary I/O traits for async read/write operations.

pub mod messaging;


const AES_KEY: &[u8; 32] = b"anexampleverysecurekey32bytes!!!";

#[tokio::main] // Marks the main function as the entry point for the Tokio runtime.
async fn main() -> std::io::Result<()> { // Defines the asynchronous main function, returning a Result for I/O operations.
    // Bind the TcpListener to the specified address and port, awaiting the operation.
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("server is running on 127.0.0.1:8080"); // Inform that the server is running.

    loop { // Start an infinite loop to continuously accept incoming connections.
        // Accept a new incoming connection, which returns a TcpStream and the client's address.
        let (mut socket, addr) = listener.accept().await?;
        println!("New connection : {}", addr); // Log the address of the new connection.

        // Spawn a new asynchronous task for each client connection.
        tokio::spawn(async move { // `async move` ensures the socket and address are moved into the new task.

            // This creates a buffer: an array with space for 1,024 elements, where each element is a byte.
            let mut buffer = [0u8; 1024]; // Create a buffer to read incoming data from the client.
            
            loop { // Loop to continuously read data from the client.
                // Read data from the socket into the buffer.
                let n = match socket.read(&mut buffer).await { // Await the read operation.
                    Ok(n) if n == 0 => break, // If 0 bytes are read, the connection is closed, so break the loop.
                    Ok(n) => n, // If bytes are read, `n` is the number of bytes read.
                    Err(_) => break, // If an error occurs during read, break the loop.
                };

                // Extract nonce (first 12 bytes) and ciphertext.
                if n < 12 {
                    eprintln!("Received data too short to contain a nonce.");
                    break;
                }
                let mut received_nonce = [0u8; 12];
                received_nonce.copy_from_slice(&buffer[0..12]);
                let received_ciphertext = &buffer[12..n];

                // Decrypt the message.
                let decrypted_msg = messaging::decrypt_message(received_ciphertext, received_nonce, AES_KEY);
                let decrypted_msg_str = String::from_utf8_lossy(&decrypted_msg);
                println!("Decrypted message from {}: {}", addr, decrypted_msg_str);

                // Prepare a response message.
                let response_msg = format!("Server received: {}", decrypted_msg_str);
                
                // Encrypt the response.
                let (encrypted_response, response_nonce) = messaging::encrypt_message(response_msg.as_bytes(), AES_KEY);

                // Combine nonce and ciphertext for sending.
                let mut data_to_send = Vec::with_capacity(response_nonce.len() + encrypted_response.len());
                data_to_send.extend_from_slice(&response_nonce);
                data_to_send.extend_from_slice(&encrypted_response);

                // Send the encrypted response back to the client.
                if socket.write_all(&data_to_send).await.is_err() {
                    eprintln!("Error writing to socket: {}", addr);
                    break;
                }
            }
        });
    }

}
