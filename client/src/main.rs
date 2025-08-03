use tokio::net::TcpStream; // Import TcpStream for establishing TCP connections.
use tokio::io::{self, AsyncWriteExt, AsyncReadExt, AsyncBufReadExt}; // Import necessary I/O traits for async operations.

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
    // Write the user's input (converted to bytes) to the TCP stream, awaiting the operation.
    stream.write_all(buf.as_bytes()).await?;

    let mut resp = [0u8; 1024]; // Create a mutable byte array to store the server's response.
    // Read data from the TCP stream into the response buffer, awaiting the operation.
    let n = stream.read(&mut resp).await?;
    // Print the server's reply, converting the received bytes to a lossy UTF-8 string.
    println!("Server replied: {}", String::from_utf8_lossy(&resp[0..n]));

    Ok(()) // Return Ok to indicate successful execution.
}
