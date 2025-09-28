use anyhow::Result;
use chrono::Utc;
use serde_json::json;

use crate::encryption::{
    hybrid_sign as crate_hybrid_sign, quantum_encrypt as crate_quantum_encrypt,
    vault_keygen as crate_vault_keygen, EncryptedPayload, VaultKeyMaterial,
};

#[derive(Debug, Clone)]
pub struct TitanVaultSimulation {
    pub resilience_score: f64,
    pub attack_surface: Vec<String>,
    pub mitigation_actions: Vec<String>,
    pub qube_policy: Option<String>,
}

pub fn vault_keygen() -> Result<VaultKeyMaterial> {
    crate_vault_keygen()
}

pub fn quantum_encrypt(
    keys: &VaultKeyMaterial,
    plaintext: &[u8],
    aad: &[u8],
) -> Result<EncryptedPayload> {
    crate_quantum_encrypt(keys, plaintext, aad)
}

pub fn hybrid_sign(keys: &VaultKeyMaterial, message: &[u8]) -> Result<Vec<u8>> {
    crate_hybrid_sign(keys, message)
}

pub fn simulate_resilience(
    domain: &str,
    dnssec_enabled: bool,
    registrar_lock: bool,
    blockchain_backing: bool,
) -> TitanVaultSimulation {
    let mut attack_surface = vec![
        format!("{}: registrar credential reuse", domain),
        format!("{}: DNS spoofing from upstream TLD", domain),
    ];
    let mut mitigation_actions = vec![
        "Rotate registrar API tokens quarterly".to_string(),
        "Enforce hardware-backed MFA for registrar dashboard".to_string(),
    ];
    let mut score = 0.35; // baseline resilience

    if dnssec_enabled {
        attack_surface.retain(|item| !item.contains("DNS spoofing"));
        mitigation_actions.push("Audit DS records after registrar updates".to_string());
        score += 0.2;
    } else {
        mitigation_actions.push("Enable DNSSEC via registrar API".to_string());
    }

    if registrar_lock {
        attack_surface.retain(|item| !item.contains("credential reuse"));
        mitigation_actions.push("Monitor for unauthorized unlock requests".to_string());
        score += 0.25;
    } else {
        mitigation_actions
            .push("Set registrar clientTransferProhibited + clientUpdateProhibited".to_string());
    }

    if blockchain_backing {
        mitigation_actions
            .push("Periodically reconcile ENS/HNS mirror with ICANN root".to_string());
        score += 0.15;
    } else {
        attack_surface.push(format!("{}: centralized registry dependency", domain));
        mitigation_actions.push("Establish Handshake/ENS notarization of zone data".to_string());
    }

    if score > 0.95 {
        score = 0.95;
    }

    let qube_policy = Some(format!(
        "policy domain {}\n  require dnssec\n  enforce registrar-lock\n  mirror handshake quorum 3\n  alert if expiration < 45d",
        domain
    ));

    TitanVaultSimulation {
        resilience_score: score,
        attack_surface,
        mitigation_actions,
        qube_policy,
    }
}

pub fn execute_qube_policy(policy: &str, records: &[crate::vault::VaultRecord]) -> Result<String> {
    let mut output = Vec::new();
    for line in policy.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if trimmed.starts_with("check-expiration") {
            let threshold: i32 = trimmed
                .split_whitespace()
                .nth(1)
                .and_then(|v| v.parse().ok())
                .unwrap_or(45);
            for record in records {
                if record.profile.expiration <= Utc::now().format("%Y-%m-%d").to_string() {
                    output.push(format!("ALERT {} expired", record.profile.domain));
                } else {
                    output.push(format!(
                        "VERIFY {} expiration horizon {}d",
                        record.profile.domain, threshold
                    ));
                }
            }
        } else if trimmed.starts_with("enforce-lock") {
            for record in records {
                if !record.profile.registrar_lock {
                    output.push(format!("LOCK {}", record.profile.domain));
                }
            }
        } else if trimmed.starts_with("require-dnssec") {
            for record in records {
                if !record.profile.dnssec_enabled {
                    output.push(format!("DNSSEC {}", record.profile.domain));
                }
            }
        } else if trimmed.starts_with("emit-json") {
            let payload = json!({
                "domains": records.iter().map(|r| &r.profile.domain).collect::<Vec<_>>(),
                "policy": trimmed,
            });
            output.push(payload.to_string());
        } else {
            output.push(format!("INFO unknown directive: {}", trimmed));
        }
    }
    if output.is_empty() {
        output.push("INFO policy executed with no actions".to_string());
    }
    Ok(output.join("\n"))
}
