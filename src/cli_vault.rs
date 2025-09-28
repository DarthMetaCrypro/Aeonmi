use clap::{Args, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Subcommand)]
pub enum VaultCommand {
    /// Register a new domain into the quantum vault
    Register(VaultRegisterArgs),
    /// Transfer an existing domain to a new registrar or registry
    Transfer(VaultTransferArgs),
    /// Apply layered security controls (DNSSEC, locks, blockchain bindings)
    Fortify(VaultFortifyArgs),
    /// Execute a QUBE symbolic policy script against the vault state
    #[command(name = "qube-run")]
    QubeRun(VaultQubeRunArgs),
    /// Emit a live status report (counts, expirations, Merkle root)
    #[command(name = "vault-status")]
    VaultStatus(VaultStatusArgs),
    /// Run deep analytics over the vault and produce mitigation plans
    #[command(name = "vault-analyze")]
    VaultAnalyze(VaultAnalyzeArgs),
    /// Low-level add command (alias for register when importing metadata)
    Add(VaultAddArgs),
    /// Renew an existing domain
    Renew(VaultRenewArgs),
    /// Toggle registrar lock state
    Lock(VaultLockArgs),
    /// Append a custom audit entry
    Audit(VaultAuditArgs),
    /// Continuously monitor expirations and registrar state
    Watch(VaultWatchArgs),
    /// Synchronize vault state with distributed storage
    Sync,
    /// Backup the vault file
    Backup(VaultBackupArgs),
    /// Recover the vault from a backup file
    Recover(VaultRecoverArgs),
    /// Export the Merkle tree view of the vault state
    ExportMerkle,
}

#[derive(Debug, Args)]
pub struct VaultRegisterArgs {
    #[arg(value_name = "DOMAIN")]
    pub domain: String,
    #[arg(long, value_name = "REGISTRAR")]
    pub registrar: String,
    #[arg(long, value_name = "YYYY-MM-DD")]
    pub expiration: String,
    #[arg(long)]
    pub auto_renew: bool,
    #[arg(long)]
    pub dnssec: bool,
    #[arg(long)]
    pub lock: bool,
    #[arg(long, value_name = "REGISTRY")]
    pub blockchain: Option<String>,
    #[arg(long = "metadata", value_name = "FILE")]
    pub metadata_path: Option<PathBuf>,
    #[arg(long = "secret", value_name = "FILE")]
    pub secret_path: PathBuf,
}

#[derive(Debug, Args)]
pub struct VaultTransferArgs {
    #[arg(value_name = "DOMAIN")]
    pub domain: String,
    #[arg(long = "to", value_name = "REGISTRAR")]
    pub new_registrar: String,
    #[arg(long = "token", value_name = "FILE")]
    pub auth_token_path: Option<PathBuf>,
}

#[derive(Debug, Args)]
pub struct VaultFortifyArgs {
    #[arg(value_name = "DOMAIN")]
    pub domain: String,
    #[arg(long)]
    pub dnssec: bool,
    #[arg(long)]
    pub lock: bool,
    #[arg(long, value_name = "REGISTRY")]
    pub blockchain: Option<String>,
}

#[derive(Debug, Args)]
pub struct VaultQubeRunArgs {
    #[arg(value_name = "SCRIPT")]
    pub script: PathBuf,
    #[arg(long, value_name = "PROFILE")]
    pub profile: Option<String>,
}

#[derive(Debug, Args)]
pub struct VaultStatusArgs {
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct VaultAnalyzeArgs {
    #[arg(value_name = "DOMAIN", required = false)]
    pub domain: Option<String>,
    #[arg(long = "mitigate", action = clap::ArgAction::SetTrue)]
    pub mitigate: bool,
}

#[derive(Debug, Args)]
pub struct VaultAddArgs {
    #[arg(value_name = "DOMAIN")]
    pub domain: String,
    #[arg(long, value_name = "REGISTRAR")]
    pub registrar: String,
    #[arg(long, value_name = "YYYY-MM-DD")]
    pub expiration: String,
    #[arg(long = "secret", value_name = "FILE")]
    pub secret_path: PathBuf,
    #[arg(long = "metadata", value_name = "FILE")]
    pub metadata_path: Option<PathBuf>,
}

#[derive(Debug, Args)]
pub struct VaultRenewArgs {
    #[arg(value_name = "DOMAIN")]
    pub domain: String,
    #[arg(long, value_name = "YYYY-MM-DD")]
    pub expiration: String,
}

#[derive(Debug, Args)]
pub struct VaultLockArgs {
    #[arg(value_name = "DOMAIN")]
    pub domain: String,
    #[arg(long = "unlock", action = clap::ArgAction::SetTrue)]
    pub unlock: bool,
}

#[derive(Debug, Args)]
pub struct VaultAuditArgs {
    #[arg(value_name = "CATEGORY")]
    pub category: String,
    #[arg(long = "detail", value_name = "JSON")]
    pub detail: Option<String>,
}

#[derive(Debug, Args)]
pub struct VaultWatchArgs {
    #[arg(long, value_name = "SECONDS", default_value = "300")]
    pub interval: u64,
    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub once: bool,
}

#[derive(Debug, Args)]
pub struct VaultBackupArgs {
    #[arg(value_name = "FILE")]
    pub path: PathBuf,
}

#[derive(Debug, Args)]
pub struct VaultRecoverArgs {
    #[arg(value_name = "FILE")]
    pub path: PathBuf,
}
