use clap::Parser;
use dirs::{config_dir, home_dir};
use std::fs;
use std::io::Write;

mod args;
mod handlers;
mod utils;

fn main() {
    let dvault_config_path = config_dir().unwrap().join("DVault");
    if !dvault_config_path.exists() {
        fs::create_dir_all(&dvault_config_path).unwrap();
        let mut config_file = fs::File::create(dvault_config_path.join("dvault_home")).unwrap();
        let default_home = home_dir().unwrap().join("DVault");
        let default_home = default_home.to_str().unwrap();
        write!(config_file, "{}", default_home).unwrap();
        fs::create_dir_all(default_home).unwrap();
    }

    let dvault_home = fs::read_to_string(dvault_config_path.join("dvault_home")).unwrap();
    let dvault_args = args::DVaultArgs::parse();
    match dvault_args.action {
        args::Action::New(new) => handlers::handle_new(dvault_home, new),
        args::Action::List => handlers::handle_list(dvault_home),
        args::Action::Lock(lock) => handlers::handle_lock(dvault_home, lock),
        args::Action::Unlock(unlock) => handlers::handle_unlock(dvault_home, unlock),
        args::Action::Setup(setup) => handlers::handle_setup(dvault_home, setup),
    }
}
