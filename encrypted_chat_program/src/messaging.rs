use aes_gcm::{Aes256Gcm, Key, Nonce}; // Or import as-needed
use aes_gcm::aead::{Aead, KeyInit};
use rand::Rng;


pub fn encrypt_message(msg: &[u8], key_bytes: &[u8; 32]) -> (Vec<u8>, [u8; 12]) {
    let key = aes_gcm::Key::<Aes256Gcm>::from_slice(key_bytes);

    let cipher = Aes256Gcm::new(key);
    let mut nonce = [0u8; 12];
    rand::thread_rng().fill(&mut nonce);
    let nonce_obj = Nonce::from_slice(&nonce);

    let ciphertext = cipher.encrypt(nonce_obj, msg).expect("encryption error");
    (ciphertext, nonce)
}

pub fn decrypt_message(ciphertext: &[u8], nonce: [u8; 12], key_bytes: &[u8; 32]) -> Vec<u8> {
    let key = aes_gcm::Key::<Aes256Gcm>::from_slice(key_bytes);
    let cipher = Aes256Gcm::new(key);
    let nonce_obj = Nonce::from_slice(&nonce);

    cipher.decrypt(nonce_obj, ciphertext).expect("decryption error")
}
