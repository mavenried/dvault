use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::process::exit;

use crate::args::{LockVault, NewVault, SetupDVault, UnlockVault};
use crate::utils::*;

pub fn handle_new(home_path: String, new: NewVault) {
    // Initialise paths >>
    let path_home = Path::new(&home_path);
    let path_dvault = path_home.join(".dvault");
    let path_dvaultdb = path_dvault.join("dvaultdb");

    // Check if paths exist and create otherwise >>
    if !path_dvault.exists() {
        fs::create_dir_all(&path_dvault).expect("dvault: new: error creating .dvault directory!");
    }
    if !path_dvaultdb.exists() {
        let mut f = File::create(&path_dvaultdb)
            .expect("dvault: new: error creating .dvault/dvaultdb file!");
        write!(f, "!! DO NOT EDIT !!\n").expect("dvault: utils: failed to write to `.dvault`!");
    }

    // Get a file handle to .dvault/dvaultdb >>
    let mut f = OpenOptions::new()
        .append(true)
        .write(true)
        .open(&path_dvaultdb)
        .expect("dvault: new: Failed to open `.dvault/dvaultdb`!");

    // Check if a vault by that name already exists >>
    if !is_valid_vault(&new.vault_name, &path_dvaultdb) {
        println!("dvault: new: a vault of this name already exists!");
        exit(1);
    }

    // Create a passwd for the vault >>
    let passwd = input("Vault Password❯ ");
    let passwd_confirmed = input("Repeat Password❯ ");
    if passwd != passwd_confirmed {
        println!("dvault: new: passwords do not match! aborting...");
        exit(1);
    }

    // Obtain the hash of the hash >>
    let hash = generate_key(passwd.as_bytes());
    let hashed_hash = generate_key(&hash[..]);

    // Update the .dvault/dvaultdb file >>
    let data = format!("{} | {} | unlk\n", &new.vault_name, encode(hashed_hash));
    println!("{data}");
    f.write_all(data.as_bytes())
        .expect("dvault: new: Failed to write to file `.dvault/dvaultdb`!");

    // Create the vault directory and temp directory >>
    fs::create_dir(path_home.join(&new.vault_name))
        .expect("dvault: new: error creating temp directory!");

    fs::create_dir(path_dvault.join(&new.vault_name))
        .expect("dvault: new: error creating vault directory");

    // Show message >>
    println!(
        "Your new vault `{}` is all set up and ready to go! Remember to lock it up once done!",
        new.vault_name
    );
}

pub fn handle_lock(home_path: String, lock: LockVault) {
    println!("I'm handling shit [from handle_lock]");
}

pub fn handle_unlock(home_path: String, unlock: UnlockVault) {
    // Initialise paths >>
    let path_home = Path::new(&home_path);
    let path_dvault = path_home.join(".dvault");
    let path_dvaultdb = path_dvault.join("dvaultdb");

    // Check if vault is valid
    if !is_valid_vault(&unlock.vault_name, &path_dvaultdb) {
        println!("dvault: unlock: no such vault!");
        exit(1);
    }

    // Get password from user and hash it>>
    let passwd = input("Password❯ ");
    let hash = generate_key(passwd.as_bytes());
    let hashed_hash = generate_key(&hash[..]);

    // Check entered password
    let check_hash = {
        let metadata: String = get_metadata(&unlock.vault_name, &path_dvaultdb);
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
        println!("dvault: unlock: auth failure, wrong password!");
        exit(1);
    }

    println!("{}", encode(hash));
}
pub fn handle_setup(home_path: String, setup: SetupDVault) {
    println!("{home_path}");
}
pub fn handle_list(home_path: String) {
    // Initialise paths >>
    let path_home = Path::new(&home_path);
    let path_dvault = path_home.join(".dvault");
    let path_dvaultdb = path_dvault.join("dvaultdb");

    // Check for dvaultdb file >>
    if !path_dvaultdb.exists() {
        eprintln!("dvault: list: no dvaultdb file, create a vault first!");
        exit(1);
    }

    // Get all vaults >>
    let db = fs::read_to_string(path_dvaultdb).unwrap();
    for line in db.lines() {
        if line.starts_with("!!") {
            continue;
        }
        let metadata = line
            .split("|")
            .map(|s| s.trim().to_string())
            .collect::<Vec<String>>();
        println!("{metadata:#?}");

        let name = metadata[0].clone();
        let icon = {
            if metadata[2] == "unlk" {
                "󱉽 "
            } else {
                "󱉼 "
            }
        };
    }
}
