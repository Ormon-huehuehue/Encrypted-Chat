use tokio::net::TcpListener; // Import TcpListener for creating a TCP server.
use tokio::io::{AsyncReadExt, AsyncWriteExt}; // Import necessary I/O traits for async read/write operations.

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
            let mut buffer = [0u8; 1024]; // Create a buffer to read incoming data from the client.
            loop { // Loop to continuously read data from the client.
                // Read data from the socket into the buffer.
                let n = match socket.read(&mut buffer).await { // Await the read operation.
                    Ok(n) if n == 0 => break, // If 0 bytes are read, the connection is closed, so break the loop.
                    Ok(n) => n, // If bytes are read, `n` is the number of bytes read.
                    Err(_) => break, // If an error occurs during read, break the loop.
                };

                // Echo received data back to the client (current functionality).
                if socket.write_all(&buffer[0..n]).await.is_err() { // Write the received bytes back to the client.
                    break; // If an error occurs during write, break the loop.
                }
            }
        });
    }

}
