# Encrypted Chat Program

A secure WebSocket-based chat application with end-to-end encryption using AES-256-GCM.

## Features

- **WebSocket Protocol**: Modern WebSocket connections for browser compatibility
- **End-to-End Encryption**: AES-256-GCM encryption for message security
- **JSON Message Format**: Structured message protocol for easy client integration
- **Real-time Communication**: Bidirectional message relay between clients

## Server

The server is written in Rust using:
- `tokio` for async runtime
- `tokio-tungstenite` for WebSocket handling
- `aes-gcm` for encryption
- `serde_json` for JSON message formatting

### Running the Server

```bash
cd encrypted_chat_program
cargo run
```

The server will start on `ws://127.0.0.1:8080` and accept up to 2 concurrent clients.

## Message Protocol

Messages are sent as JSON with the following structure:

```json
{
  "type": "message",
  "data": "base64_encoded_encrypted_message",
  "nonce": "base64_encoded_nonce"
}
```

### Message Flow

1. Client encrypts message using AES-256-GCM with a random nonce
2. Client sends JSON message with encrypted data and nonce
3. Server decrypts the message using the shared key
4. Server re-encrypts the message with a new nonce
5. Server forwards the message to the other client

## TypeScript Client Example

Here's a simple TypeScript client for testing:

```typescript
class EncryptedChatClient {
    private ws: WebSocket;
    private key: Uint8Array;
    
    constructor(url: string, key: string) {
        this.ws = new WebSocket(url);
        this.key = new TextEncoder().encode(key);
        this.setupEventHandlers();
    }
    
    private setupEventHandlers() {
        this.ws.onopen = () => {
            console.log('Connected to chat server');
        };
        
        this.ws.onmessage = (event) => {
            const message = JSON.parse(event.data);
            if (message.type === 'message') {
                const decrypted = this.decryptMessage(message.data, message.nonce);
                console.log('Received:', decrypted);
            }
        };
        
        this.ws.onerror = (error) => {
            console.error('WebSocket error:', error);
        };
    }
    
    async sendMessage(text: string) {
        const encrypted = await this.encryptMessage(text);
        const message = {
            type: 'message',
            data: encrypted.data,
            nonce: encrypted.nonce
        };
        this.ws.send(JSON.stringify(message));
    }
    
    private async encryptMessage(text: string): Promise<{data: string, nonce: string}> {
        // Implementation would use Web Crypto API
        // This is a placeholder for the actual encryption logic
        const encoder = new TextEncoder();
        const data = encoder.encode(text);
        
        // Generate random nonce
        const nonce = crypto.getRandomValues(new Uint8Array(12));
        
        // Import key and encrypt
        const cryptoKey = await crypto.subtle.importKey(
            'raw',
            this.key,
            { name: 'AES-GCM' },
            false,
            ['encrypt']
        );
        
        const encrypted = await crypto.subtle.encrypt(
            { name: 'AES-GCM', iv: nonce },
            cryptoKey,
            data
        );
        
        return {
            data: btoa(String.fromCharCode(...new Uint8Array(encrypted))),
            nonce: btoa(String.fromCharCode(...nonce))
        };
    }
    
    private async decryptMessage(data: string, nonce: string): Promise<string> {
        // Implementation would use Web Crypto API
        // This is a placeholder for the actual decryption logic
        const encryptedData = new Uint8Array(atob(data).split('').map(c => c.charCodeAt(0)));
        const nonceData = new Uint8Array(atob(nonce).split('').map(c => c.charCodeAt(0)));
        
        const cryptoKey = await crypto.subtle.importKey(
            'raw',
            this.key,
            { name: 'AES-GCM' },
            false,
            ['decrypt']
        );
        
        const decrypted = await crypto.subtle.decrypt(
            { name: 'AES-GCM', iv: nonceData },
            cryptoKey,
            encryptedData
        );
        
        return new TextDecoder().decode(decrypted);
    }
}

// Usage
const client = new EncryptedChatClient('ws://127.0.0.1:8080', 'anexampleverysecurekey32bytes!!!');
client.sendMessage('Hello, encrypted world!');
```

## Security Notes

- The current implementation uses a hardcoded key for demonstration
- In production, implement proper key exchange (e.g., Diffie-Hellman)
- Consider adding message authentication and integrity checks
- Implement proper session management and user authentication

## Dependencies

- `tokio` - Async runtime
- `tokio-tungstenite` - WebSocket implementation
- `aes-gcm` - Encryption
- `serde_json` - JSON serialization
- `base64` - Base64 encoding/decoding
- `futures-util` - Async utilities 