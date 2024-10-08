use base64::engine::general_purpose::URL_SAFE;
use base64::Engine;
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
    let mut out = String::new();
    std::io::stdin().read_line(&mut out).unwrap();
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
        let metadata: String = get_metadata(vault_name, &path_dvaultdb);
        let encoded_hash = metadata
            .split("|")
            .map(|s| s.trim())
            .skip(1)
            .next()
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
