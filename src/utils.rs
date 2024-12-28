use aes_gcm::aead::rand_core::RngCore;
use aes_gcm::aead::{Aead, KeyInit, OsRng};
use aes_gcm::{Aes256Gcm, Nonce};
use base64::engine::general_purpose::URL_SAFE;
use base64::Engine;
use rpassword::read_password;
use sha2::{Digest, Sha256};

use std::{
    fs::{self, OpenOptions},
    io::{Error, Write},
    path::Path,
};

pub fn generate_key(passphrase: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(passphrase);
    let result = hasher.finalize();
    let mut key = [0u8; 32];
    key.copy_from_slice(&result[..]);
    key
}

pub fn input(prompt: &str) -> String {
    print!("{prompt}");
    std::io::stdout().flush().unwrap();
    let out = read_password().unwrap();
    print!("\x1B[F\x1B[K");
    std::io::stdout().flush().unwrap();
    out.trim().to_string()
}

pub fn encode(_in: [u8; 32]) -> String {
    URL_SAFE.encode(_in)
}

pub fn decode(_in: String) -> [u8; 32] {
    let mut out = [0u8; 32];
    let buf = URL_SAFE.decode(_in).unwrap();
    out.copy_from_slice(&buf[..]);
    out
}

pub fn is_valid_vault(name: &str, path: &Path) -> bool {
    if get_metadata(name, path) == "NOT FOUND" {
        return false;
    };
    true
}

pub fn lock_unlock_vault(name: &str, path: &Path, is_locked: bool) {
    let file_contents =
        fs::read_to_string(path).expect("dvault:utils: failed to open `.dvault/dvaultdb`!");
    let mut new = String::new();
    for line in file_contents.lines() {
        let mut line = line.to_string();
        if line.starts_with(name) {
            if is_locked {
                line = line.replace("lock", "unlk")
            } else {
                line = line.replace("unlk", "lock");
            }
        }
        new.push_str(line.as_str());
        new.push('\n');
    }
    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(path)
        .unwrap();

    write!(file, "{}", new).expect("dvault: utils: failed to write to `.dvault/dvaultdb`!");
}

pub fn get_metadata(name: &str, path: &Path) -> String {
    let file_contents =
        fs::read_to_string(path).expect("dvault: utils: failed to open `.dvault/dvaultdb`!");
    for line in file_contents.lines() {
        if line.starts_with("!!") {
            continue;
        }
        if line.starts_with(name) {
            return line.to_string();
        }
    }
    "NOT FOUND".to_string()
}

pub fn get_password_hash(vault_name: &str, path_dvaultdb: &Path) -> std::io::Result<[u8; 32]> {
    // Get password from user and hash it>>
    let passwd = input("Password❯ ");
    let hash = generate_key(passwd.as_bytes());
    let hashed_hash = generate_key(&hash[..]);

    // Check entered password
    let check_hash = {
        let metadata: String = get_metadata(vault_name, path_dvaultdb);
        let encoded_hash = metadata
            .split("|")
            .map(|s| s.trim())
            .nth(1)
            .unwrap()
            .to_string();
        decode(encoded_hash)
    };

    if check_hash != hashed_hash {
        return Err(Error::new(
            std::io::ErrorKind::InvalidInput,
            "Password Incorrect.",
        ));
        // return an error here
    }

    Ok(hash)
}
pub fn encrypt_file(hash: &[u8], fpath: &Path, epath: &Path) {
    let cipher = Aes256Gcm::new(hash.into());

    // Generate a random 12-byte nonce
    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let plaintext = fs::read_to_string(fpath).expect("dvault: lock: error reading file!");

    // Encrypt the message
    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .expect("encryption failure!");

    // Concatenate nonce and ciphertext
    let encrypted_message = [nonce_bytes.to_vec(), ciphertext].concat();

    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(epath)
        .unwrap();
    file.write_all(&encrypted_message[..])
        .expect("dvault: utils: error writing to file!");
}

pub fn decrypt_file(hash: &[u8], fpath: &Path, epath: &Path) {
    let cipher = Aes256Gcm::new(hash.into());

    let ciphertext = fs::read(fpath).expect("dvault: unlock: error reading file!");

    // Extract the nonce (first 12 bytes)
    let nonce = Nonce::from_slice(&ciphertext[..12]);

    // Extract the actual ciphertext (after the nonce)
    let ciphertext = &ciphertext[12..];

    // Decrypt the message
    let decrypted = cipher.decrypt(nonce, ciphertext).unwrap();

    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(epath)
        .unwrap();

    file.write_all(&decrypted[..]).unwrap();
}
