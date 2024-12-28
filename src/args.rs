use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(author = "Joseph Chacko <josephchacko2006@gmail.com>")]
#[command(version = "0.1.0")]
#[command(
    help_template = "{name} v{version}\n{author-section} {about-section}\n{usage-heading} {usage} \n\n{all-args}"
)]
#[command(about, long_about = None)]
pub struct DVaultArgs {
    #[clap(subcommand)]
    pub action: Action,
}

#[derive(Debug, Subcommand)]
pub enum Action {
    New(NewVault),
    Lock(LockVault),
    Unlock(UnlockVault),
    List,
    Setup(SetupDVault),
}

#[derive(Debug, Args)]
pub struct UnlockVault {
    /// Name of vault to unlock
    pub vault_name: String,
}

#[derive(Debug, Args)]
pub struct LockVault {
    /// Save and lock up specified vault
    pub vault_name: String,
}

#[derive(Debug, Args)]
pub struct NewVault {
    /// Name of new vault
    pub vault_name: String,
}

#[derive(Debug, Args)]
pub struct SetupDVault {
    /// Set vault home path (without this option dvaults will get the path instead)
    pub vault_home_path: Option<String>,
}
