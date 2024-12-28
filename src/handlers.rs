use std::ffi::OsString;
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
        writeln!(f, "!! DO NOT EDIT !!").expect("dvault: utils: failed to write to `.dvault`!");
    }

    // Get a file handle to .dvault/dvaultdb >>
    let mut f = OpenOptions::new()
        .append(true)
        .open(&path_dvaultdb)
        .expect("dvault: new: Failed to open `.dvault/dvaultdb`!");

    // Check if a vault by that name already exists >>
    if is_valid_vault(&new.vault_name, &path_dvaultdb) {
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
        "Your new vault `{}` is all set up ad ready to go! Remember to lock it up once done!",
        new.vault_name
    );
}

pub fn handle_lock(home_path: String, lock: LockVault) {
    // Initialise paths >>
    let path_home = Path::new(&home_path);
    let path_dvault = path_home.join(".dvault");
    let path_dvaultdb = path_dvault.join("dvaultdb");

    // Check if vault is valid
    if !is_valid_vault(&lock.vault_name, &path_dvaultdb) {
        println!("dvault: unlock: no such vault!");
        exit(1);
    }
    // Get the password and hash it >>
    let hash = get_password_hash(&lock.vault_name, &path_dvaultdb).unwrap_or_else(|_e| {
        println!("dvault: lock: Wrong Password");
        exit(1)
    });

    // Encrypt all files in the vault and move them into the vault >>
    for file in fs::read_dir(path_home.join(&lock.vault_name)).unwrap() {
        let file = file.unwrap();
        let fpath = file.path();

        let mut filename = fpath.file_name().unwrap().to_os_string();
        filename.push(OsString::from(".vaulted"));
        let epath = path_dvault.join(&lock.vault_name).join(filename);
        encrypt_file(&hash, &fpath, &epath);
    }
    drop(fs::remove_dir_all(path_home.join(&lock.vault_name)));
    // Change the state of the vault
    lock_unlock_vault(&lock.vault_name, &path_dvaultdb, false);
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
    // Get the password and hash it >>
    let hash = get_password_hash(&unlock.vault_name, &path_dvaultdb).unwrap_or_else(|_e| {
        println!("dvault: lock: Wrong Password");
        exit(1)
    });
    if path_home.join(&unlock.vault_name).exists() {
        println!("dvault: unlock: temp dir exists, to abandon changes, delete it.");
        exit(1);
    }
    fs::create_dir(path_home.join(&unlock.vault_name))
        .expect("dvault: unlock: error creating temp");

    // Decrypt all files in the vault and move them into the temp >>
    for file in fs::read_dir(path_home.join(".dvault").join(&unlock.vault_name)).unwrap() {
        let file = file.unwrap();
        let fpath = file.path();

        let filename = fpath.file_name().unwrap();
        let filename = filename.to_str().unwrap().split('.').next().unwrap();

        let epath = path_home.join(&unlock.vault_name).join(filename);
        decrypt_file(&hash, &fpath, &epath);
    }
    // Change the state of the vault
    lock_unlock_vault(&unlock.vault_name, &path_dvaultdb, true);
}
pub fn handle_setup(home_path: String, dvault_config_path: &Path, setup: SetupDVault) {
    if let Some(path) = setup.vault_home_path {
        let mut config_file = fs::File::create(dvault_config_path.join("dvault_home")).unwrap();
        write!(config_file, "{}", path).unwrap();
    } else {
        println!("{home_path}");
    }
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

        let name = metadata[0].clone();
        let icon = {
            if metadata[2] == "unlk" {
                "󱉽 [unlk] "
            } else {
                "󱉼 [lock] "
            }
        };
        println!("{icon} {name}")
    }
}
