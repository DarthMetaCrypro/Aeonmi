use std::{
    collections::HashMap,
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use crate::encryption::{
    quantum_decrypt, quantum_encrypt, vault_keygen, EncryptedPayload, VaultKeyMaterial,
};
use anyhow::{anyhow, Context, Result};
use base64::{engine::general_purpose, Engine as _};
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct VaultDomainProfile {
    pub domain: String,
    pub registrar: String,
    pub expiration: String,
    pub auto_renew: bool,
    pub dnssec_enabled: bool,
    pub registrar_lock: bool,
    pub blockchain_registry: Option<String>,
    pub metadata: serde_json::Value,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct VaultRecord {
    pub id: String,
    pub created_at: String,
    pub updated_at: String,
    pub profile: VaultDomainProfile,
    pub encrypted_blob: EncryptedPayload,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct VaultAuditEntry {
    pub timestamp: String,
    pub category: String,
    pub detail: serde_json::Value,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct VaultState {
    pub version: u32,
    pub created_at: String,
    pub updated_at: String,
    pub keys: VaultKeyMaterial,
    pub records: Vec<VaultRecord>,
    pub audits: Vec<VaultAuditEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultStatusReport {
    pub total_domains: usize,
    pub dnssec_enabled: usize,
    pub registrar_locked: usize,
    pub auto_renew_enabled: usize,
    pub blockchain_backed: usize,
    pub merkle_root: String,
    pub upcoming_expirations: Vec<(String, String)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultSimulationReport {
    pub domain: String,
    pub attack_surface: Vec<String>,
    pub mitigation_actions: Vec<String>,
    pub qube_script: Option<String>,
    pub titan_resilience_score: f64,
}

pub struct DomainQuantumVault {
    storage_path: PathBuf,
    state: VaultState,
}

impl DomainQuantumVault {
    pub fn open_default() -> Result<Self> {
        let root = default_vault_path()?;
        if let Some(parent) = root.parent() {
            fs::create_dir_all(parent).context("create vault directory")?;
        }
        if !root.exists() {
            let now = current_timestamp();
            let keys = vault_keygen()?;
            let state = VaultState {
                version: 1,
                created_at: now.clone(),
                updated_at: now,
                keys,
                records: Vec::new(),
                audits: Vec::new(),
            };
            let mut vault = Self {
                storage_path: root,
                state,
            };
            vault.persist()?;
            Ok(vault)
        } else {
            Self::from_path(root)
        }
    }

    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let data = fs::read_to_string(&path).context("read vault file")?;
        let state: VaultState = serde_json::from_str(&data).context("parse vault state")?;
        Ok(Self {
            storage_path: path.as_ref().to_path_buf(),
            state,
        })
    }

    pub fn register_domain(
        &mut self,
        profile: VaultDomainProfile,
        secret_material: serde_json::Value,
    ) -> Result<VaultRecord> {
        if self
            .state
            .records
            .iter()
            .any(|r| r.profile.domain == profile.domain)
        {
            return Err(anyhow!("domain already exists in vault"));
        }
        let secret_bytes = serde_json::to_vec(&secret_material)?;
        let aad = profile.domain.as_bytes();
        let encrypted_blob = quantum_encrypt(&self.state.keys, &secret_bytes, aad)?;

        let now = current_timestamp();
        let record = VaultRecord {
            id: random_record_id(),
            created_at: now.clone(),
            updated_at: now,
            profile,
            encrypted_blob,
        };
        self.state.records.push(record.clone());
        self.append_audit(
            "register",
            json!({
                "domain": record.profile.domain,
                "registrar": record.profile.registrar,
                "expiration": record.profile.expiration,
            }),
        );
        self.persist()?;
        Ok(record)
    }

    pub fn renew_domain(&mut self, domain: &str, new_expiration: &str) -> Result<()> {
        let mut found = false;
        for record in &mut self.state.records {
            if record.profile.domain == domain {
                record.profile.expiration = new_expiration.to_string();
                record.updated_at = current_timestamp();
                found = true;
            }
        }
        if !found {
            return Err(anyhow!("domain not found"));
        }
        self.append_audit(
            "renew",
            json!({
                "domain": domain,
                "new_expiration": new_expiration,
            }),
        );
        self.persist()
    }

    pub fn lock_domain(&mut self, domain: &str, lock: bool) -> Result<()> {
        let mut found = false;
        for record in &mut self.state.records {
            if record.profile.domain == domain {
                record.profile.registrar_lock = lock;
                record.updated_at = current_timestamp();
                found = true;
            }
        }
        if !found {
            return Err(anyhow!("domain not found"));
        }
        self.append_audit(
            "lock",
            json!({
                "domain": domain,
                "locked": lock,
            }),
        );
        self.persist()
    }

    pub fn enable_dnssec(&mut self, domain: &str) -> Result<()> {
        let mut found = false;
        for record in &mut self.state.records {
            if record.profile.domain == domain {
                record.profile.dnssec_enabled = true;
                record.updated_at = current_timestamp();
                found = true;
            }
        }
        if !found {
            return Err(anyhow!("domain not found"));
        }
        self.append_audit("dnssec", json!({ "domain": domain }));
        self.persist()
    }

    pub fn attach_blockchain_registry(&mut self, domain: &str, registry: &str) -> Result<()> {
        let mut found = false;
        for record in &mut self.state.records {
            if record.profile.domain == domain {
                record.profile.blockchain_registry = Some(registry.to_string());
                record.updated_at = current_timestamp();
                found = true;
            }
        }
        if !found {
            return Err(anyhow!("domain not found"));
        }
        self.append_audit(
            "blockchain",
            json!({ "domain": domain, "registry": registry }),
        );
        self.persist()
    }

    pub fn transfer_domain(
        &mut self,
        domain: &str,
        new_registrar: &str,
        token: Option<serde_json::Value>,
    ) -> Result<()> {
        let mut found = false;
        for record in &mut self.state.records {
            if record.profile.domain == domain {
                record.profile.registrar = new_registrar.to_string();
                record.updated_at = current_timestamp();
                if !record.profile.metadata.is_object() {
                    record.profile.metadata = json!({});
                }
                if let serde_json::Value::Object(map) = &mut record.profile.metadata {
                    map.insert(
                        "last_transfer".to_string(),
                        json!({
                            "ts": current_timestamp(),
                            "registrar": new_registrar,
                            "token_hash": token
                                .as_ref()
                                .map(|t| hash_json_value(t))
                                .unwrap_or_default(),
                        }),
                    );
                }
                found = true;
            }
        }
        if !found {
            return Err(anyhow!("domain not found"));
        }
        self.append_audit(
            "transfer",
            json!({
                "domain": domain,
                "new_registrar": new_registrar,
            }),
        );
        self.persist()
    }

    pub fn audit(&mut self, category: &str, detail: serde_json::Value) -> Result<()> {
        self.append_audit(category, detail);
        self.persist()
    }

    pub fn retrieve_secret(&self, domain: &str) -> Result<serde_json::Value> {
        let record = self
            .state
            .records
            .iter()
            .find(|r| r.profile.domain == domain)
            .ok_or_else(|| anyhow!("domain not found"))?;
        let plaintext =
            quantum_decrypt(&self.state.keys, &record.encrypted_blob, domain.as_bytes())?;
        let value: serde_json::Value = serde_json::from_slice(&plaintext)?;
        Ok(value)
    }

    pub fn generate_status(&self) -> Result<VaultStatusReport> {
        let total_domains = self.state.records.len();
        let dnssec_enabled = self
            .state
            .records
            .iter()
            .filter(|r| r.profile.dnssec_enabled)
            .count();
        let registrar_locked = self
            .state
            .records
            .iter()
            .filter(|r| r.profile.registrar_lock)
            .count();
        let auto_renew_enabled = self
            .state
            .records
            .iter()
            .filter(|r| r.profile.auto_renew)
            .count();
        let blockchain_backed = self
            .state
            .records
            .iter()
            .filter(|r| r.profile.blockchain_registry.is_some())
            .count();
        let upcoming_expirations = self.upcoming_expirations();
        let merkle_root = self.calculate_merkle_root();
        Ok(VaultStatusReport {
            total_domains,
            dnssec_enabled,
            registrar_locked,
            auto_renew_enabled,
            blockchain_backed,
            merkle_root,
            upcoming_expirations,
        })
    }

    pub fn run_simulation(&self, domain: &str) -> Result<VaultSimulationReport> {
        let record = self
            .state
            .records
            .iter()
            .find(|r| r.profile.domain == domain)
            .ok_or_else(|| anyhow!("domain not found"))?;
        let titan_report = crate::core::titan::quantum_vault::simulate_resilience(
            &record.profile.domain,
            record.profile.dnssec_enabled,
            record.profile.registrar_lock,
            record.profile.blockchain_registry.is_some(),
        );
        Ok(VaultSimulationReport {
            domain: record.profile.domain.clone(),
            attack_surface: titan_report.attack_surface,
            mitigation_actions: titan_report.mitigation_actions,
            qube_script: titan_report.qube_policy,
            titan_resilience_score: titan_report.resilience_score,
        })
    }

    pub fn run_qube_policy(&self, policy: &str) -> Result<String> {
        crate::core::titan::quantum_vault::execute_qube_policy(policy, &self.state.records)
    }

    pub fn export_merkle_tree(&self) -> Result<String> {
        let root = self.calculate_merkle_root();
        let leaves: Vec<_> = self
            .state
            .records
            .iter()
            .map(|r| {
                json!({
                    "domain": r.profile.domain,
                    "hash": general_purpose::STANDARD.encode(hash_record(r)),
                })
            })
            .collect();
        let tree = json!({
            "root": root,
            "leaves": leaves,
        });
        Ok(serde_json::to_string_pretty(&tree)?)
    }

    pub fn sync(&mut self) -> Result<()> {
        self.append_audit(
            "sync",
            json!({ "detail": "local vault sync placeholder complete" }),
        );
        self.persist()
    }

    pub fn backup_to<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let serialized = serde_json::to_string_pretty(&self.state)?;
        fs::write(path.as_ref(), serialized).context("write backup")
    }

    pub fn recover_from<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let data = fs::read_to_string(path).context("read backup")?;
        let mut state: VaultState = serde_json::from_str(&data).context("parse backup")?;
        state.updated_at = current_timestamp();
        self.state = state;
        self.persist()
    }

    fn upcoming_expirations(&self) -> Vec<(String, String)> {
        let mut expirations: Vec<_> = self
            .state
            .records
            .iter()
            .map(|r| (r.profile.domain.clone(), r.profile.expiration.clone()))
            .collect();
        expirations.sort_by(|a, b| a.1.cmp(&b.1));
        expirations.into_iter().take(5).collect()
    }

    fn calculate_merkle_root(&self) -> String {
        let mut leaves: Vec<Vec<u8>> = self
            .state
            .records
            .iter()
            .map(|r| hash_record(r).to_vec())
            .collect();
        if leaves.is_empty() {
            return String::from("0");
        }
        while leaves.len() > 1 {
            let mut next = Vec::with_capacity((leaves.len() + 1) / 2);
            for chunk in leaves.chunks(2) {
                let combined = if chunk.len() == 2 {
                    [chunk[0].as_slice(), chunk[1].as_slice()].concat()
                } else {
                    chunk[0].clone()
                };
                let mut hasher = Sha256::new();
                hasher.update(&combined);
                next.push(hasher.finalize().to_vec());
            }
            leaves = next;
        }
        general_purpose::STANDARD.encode(&leaves[0])
    }

    fn append_audit(&mut self, category: &str, detail: serde_json::Value) {
        self.state.updated_at = current_timestamp();
        self.state.audits.push(VaultAuditEntry {
            timestamp: current_timestamp(),
            category: category.to_string(),
            detail,
        });
    }

    fn persist(&mut self) -> Result<()> {
        let serialized = serde_json::to_string_pretty(&self.state)?;
        let parent = self
            .storage_path
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("."));
        let mut tmp = tempfile::NamedTempFile::new_in(&parent).context("create temp vault file")?;
        tmp.write_all(serialized.as_bytes())
            .context("write vault temp file")?;
        tmp.flush().ok();
        tmp.persist(&self.storage_path)
            .map_err(|e| anyhow!("persist vault state: {e}"))?;
        Ok(())
    }
}

fn default_vault_path() -> Result<PathBuf> {
    let mut path = dirs_next::home_dir().ok_or_else(|| anyhow!("home directory not found"))?;
    path.push(".aeonmi");
    path.push("vault");
    fs::create_dir_all(&path).ok();
    path.push("domain_quantum_vault.json");
    Ok(path)
}

fn current_timestamp() -> String {
    chrono::Utc::now().to_rfc3339()
}

fn random_record_id() -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(12)
        .map(char::from)
        .collect()
}

fn hash_record(record: &VaultRecord) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(&record.id);
    hasher.update(&record.profile.domain);
    hasher.update(&record.profile.registrar);
    hasher.update(&record.profile.expiration);
    hasher.finalize().to_vec()
}

fn hash_json_value(value: &serde_json::Value) -> String {
    let mut hasher = Sha256::new();
    let bytes = serde_json::to_vec(value).unwrap_or_default();
    hasher.update(bytes);
    general_purpose::STANDARD.encode(hasher.finalize())
}

pub fn describe_hijack_resilience(
    vault: &DomainQuantumVault,
) -> Result<HashMap<String, VaultSimulationReport>> {
    let mut map = HashMap::new();
    for record in &vault.state.records {
        let report = vault.run_simulation(&record.profile.domain)?;
        map.insert(record.profile.domain.clone(), report);
    }
    Ok(map)
}
