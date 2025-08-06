use aes_gcm::{Aes256Gcm, Key, Nonce}; 
use aes_gcm::aead::{Aead, KeyInit}; // Import Aead (Authenticated Encryption with Associated Data) trait and KeyInit trait for cipher initialization.
use rand::Rng;

// Encrypts a message using AES-256 GCM.
// `msg`: The plaintext message as a byte slice.
// `key_bytes`: The 32-byte (256-bit) encryption key.
// Returns a tuple: (ciphertext, nonce). The ciphertext is the encrypted message, and the nonce is the random value used for this encryption.
pub fn encrypt_message(msg: &[u8], key_bytes: &[u8; 32]) -> (Vec<u8>, [u8; 12]) {
    // Create an AES-GCM Key from the provided key bytes.
    let key = aes_gcm::Key::<Aes256Gcm>::from_slice(key_bytes);

    let cipher = Aes256Gcm::new(key);

    // Generate a random 12-byte Nonce (Number used once).
    let mut nonce = [0u8; 12];
    rand::thread_rng().fill(&mut nonce); // Fill the nonce array with random bytes.
    let nonce_obj = Nonce::from_slice(&nonce); // Convert the byte array into a Nonce object.

    // Encrypt the message using the cipher and nonce.
    let ciphertext = cipher.encrypt(nonce_obj, msg).expect("encryption error");

    // Return the encrypted message (ciphertext) and the nonce used.
    (ciphertext, nonce)
}

// Decrypts a message using AES-256 GCM.
// `ciphertext`: The encrypted message as a byte slice.
// `nonce`: The 12-byte nonce that was used during encryption.
// `key_bytes`: The 32-byte (256-bit) decryption key (must be the same as the encryption key).
// Returns the decrypted plaintext message as a Vec<u8>.
pub fn decrypt_message(ciphertext: &[u8], nonce: [u8; 12], key_bytes: &[u8; 32]) -> Vec<u8> {
    // Create an AES-GCM Key from the provided key bytes, similar to encryption.
    let key = aes_gcm::Key::<Aes256Gcm>::from_slice(key_bytes);

    // Initialize the AES-256 GCM cipher with the key.
    let cipher = Aes256Gcm::new(key);

    // Convert the nonce byte array back into a Nonce object.
    let nonce_obj = Nonce::from_slice(&nonce);

    // Decrypt the ciphertext using the cipher and nonce.
    cipher.decrypt(nonce_obj, ciphertext).expect("decryption error")
}
