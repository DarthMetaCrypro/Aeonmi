# Domain Quantum Vault Architecture & Integration Plan

## Executive Summary
The Domain Quantum Vault (DQV) transforms Aeonmi into a zero-trust domain governance platform hardened for post-quantum threats. The module encrypts registrar tokens, DNSSEC keys, and vault metadata using an AES-256 + Kyber/Sphincs+ hybrid. Aeonmi CLI gains a full **`vault`** command set (register, transfer, fortify, qube-run, status, analyze, add, renew, lock, audit, watch, sync, backup, recover, export-merkle) with optional Ratatui dashboards and QUBE-driven simulations. Titan exposes helper APIs (`vault_keygen`, `quantum_encrypt`, `hybrid_sign`, `simulate_resilience`, `execute_qube_policy`) so future AI agents and symbolic runtimes orchestrate policy checks and hijack drills. Persistent state is Merkle-hashed, auditable, and ready for decentralized replication (Handshake/ENS mirrors and social recovery hooks).

## Architecture Overview
```mermaid
flowchart TD
    A[Aeonmi CLI
    `vault ...`] -->|Clap parser| B(cli_vault::VaultCommand)
    B --> C{commands::vault::dispatch}
    C -->|mutate| D[DomainQuantumVault
    (vault.rs)]
    C -->|simulate| E[Titan::quantum_vault]
    D -->|encrypt/decrypt| F[encryption.rs
    AES-256-GCM + Kyber/Sphincs+]
    D -->|state| G[~/.aeonmi/vault/domain_quantum_vault.json]
    E -->|policy out| H[QUBE scripts
    (policy text)]
    H -->|feedback| C
    D -->|status| I[Ratatui dashboard]
    D -->|Merkle export| J[Decentralized registry / backups]
```

### Core Modules
- **`src/encryption.rs`** – defines `VaultKeyMaterial`, `EncryptedPayload`, and hybrid primitives wrapping AES-256-GCM with Kyber1024 KEM and Sphincs+ signatures. Functions include `vault_keygen`, `quantum_encrypt`, `quantum_decrypt`, `hybrid_sign`, `hybrid_verify`.
- **`src/vault.rs`** – `DomainQuantumVault` orchestrates storage, audit trails, Merkle proofs, registrar lifecycle management, and blockchain bindings.
- **`src/cli_vault.rs`** – Clap definitions for the `vault` subcommand family and daemon/watch helpers.
- **`src/commands/vault.rs`** – high-level command dispatcher with TUI rendering, QUBE policy execution, audits, and automation hooks.
- **`src/core/titan/quantum_vault.rs`** – Titan façade exposing hybrid crypto helpers and resilience simulations for QUBE/Titan integrations.

## Rust Integration Highlights
### `vault.rs`
- `DomainQuantumVault::register_domain` encrypts JSON secrets with `quantum_encrypt`, appends audit proofs, and persists via atomic writes.
- `DomainQuantumVault::transfer_domain`, `renew_domain`, `enable_dnssec`, `attach_blockchain_registry`, and `lock_domain` enforce registrar best practices while logging Merkle-auditable events.
- `run_simulation` and `run_qube_policy` delegate to Titan for symbolic hijack modeling and QUBE directives.

### `encryption.rs`
- `vault_keygen` seeds AES-256 keys, Kyber1024 KEM pairs, and Sphincs+ signing keys.
- `quantum_encrypt` derives tamper-evident bindings from Kyber shared secrets, signs them with Sphincs+, and stores all artifacts as base64 for JSON compatibility.
- `quantum_decrypt` verifies Sphincs+ signatures, recomputes Kyber bindings, and returns plaintext secrets with integrity guarantees.

### `core/titan/quantum_vault.rs`
- `simulate_resilience` outputs TitanVaultSimulation (resilience score, attack vectors, mitigation actions, recommended QUBE policy) based on DNSSEC, locks, and blockchain anchoring.
- `execute_qube_policy` interprets symbolic policy directives (`check-expiration`, `enforce-lock`, `require-dnssec`, `emit-json`) over current vault records, enabling automated domain hygiene checks.

### CLI & UX
- `vault register` ingests metadata + secrets JSON, encrypts, logs, and stores.
- `vault fortify --dnssec --lock --blockchain handshake` toggles security controls and blockchain mirrors.
- `vault vault-status --json` or `--tui` surfaces real-time dashboards; `vault watch --interval 600 --tui` runs a live Ratatui cockpit.
- `vault qube-run policy.qube --profile harden` evaluates symbolic policies, while `vault vault-analyze <domain>` simulates hijack attempts with Titan scoring.

## Titan & QUBE Extensions
- Titan exposes wrappers for Aeonmi runtime to invoke hybrid cryptography without duplicating primitives.
- QUBE policies can now introspect expiration horizons, registrar locks, and DNSSEC posture; results feed back into CLI (alerts, JSON exports) or future automation (Mother AI alignment checks).

## Registrar Migration & Anti-Hijack Strategy
1. **Registrar Exit Plan:** `vault transfer --to namecheap` updates metadata, stores hashed transfer tokens, and ensures new registrar inherits DNSSEC + locks before old API credentials expire.
2. **DNSSEC Guardrails:** `vault fortify --dnssec` enforces DS key activation and logs state transitions for zero-trust audits.
3. **Blockchain Mirrors:** `vault fortify --blockchain handshake` seeds decentralized proofs so even ICANN/GoDaddy takedowns cannot erase chain-of-custody.
4. **Continuous Monitoring:** `vault watch` combined with QUBE `check-expiration 45` ensures advanced warning before renewal windows close, preventing social-engineering hijacks.
5. **Attack Simulation:** `vault vault-analyze domain.xyz --mitigate` surfaces Titan-generated mitigation steps; repeated scoring drives automated policies (auto-lock, backup rotation) to keep resilience score near 0.95.
6. **Decentralized Recovery:** `vault export-merkle` outputs a Merkle snapshot ready for Handshake/ENS notarization and social-trust recovery flows.

**Hijack Scenario Defence:** If GoDaddy/ICANN attempt unauthorized transfer, registrar locks and blockchain attestations block updates; Sphincs+-signed audit history plus Merkle proofs demonstrate custody. Automated QUBE policies detect DS record tampering, while `vault watch` triggers Matrix/Discord hooks (future CLI innovation) to alert operators. Social trust signatures and HSM binding prevent attacker from using stolen API keys.

## CLI Usage Examples
```shell
# Register domain with metadata + secret bundle
$ aeonmi vault register example.com --registrar namecheap --expiration 2025-06-01 \
    --auto-renew --dnssec --lock --secret secrets/example.com.json --metadata configs/example-meta.json

# Run symbolic audit and mitigation plan
$ aeonmi vault vault-analyze example.com --mitigate

# Live dashboard
$ aeonmi vault vault-status --tui

# Execute QUBE policy script
$ aeonmi vault qube-run policies/renewal.qube --profile harden

# Export Merkle tree for decentralized ledger anchoring
$ aeonmi vault export-merkle > merkle_snapshot.json
```

## Integration Notes
- **CLI Parser:** `src/cli_vault.rs` plugs into Clap; `src/cli.rs` registers the `vault` subcommand variant.
- **Command Dispatcher:** `src/commands/vault.rs` routes to `DomainQuantumVault`, renders Ratatui dashboards, and interacts with Titan/QUBE wrappers.
- **Runtime Modules:** `src/encryption.rs`, `src/vault.rs`, and `src/core/titan/quantum_vault.rs` are added to `lib.rs` and `main.rs` for broad access.
- **State Location:** `~/.aeonmi/vault/domain_quantum_vault.json` (auto-created) stores encrypted records, audits, and keys. `vault backup`/`recover` operate on JSON snapshots.

## Vault Roadmap
- **v1.0 (current sprint):** CLI vault tooling, hybrid encryption core, Ratatui dashboard, Titan resilience scoring, QUBE policy execution.
- **v2.0:** Decentralized backups (Handshake/ENS notarization), Matrix/Discord notification hooks, automated registrar lock rotation, AI-driven audit policies, quantum-safe sync to air-gapped replicas.
- **v3.0:** Atlas AI governance, zero-trust distributed vault shards, biometric + social-trust recovery, automated on-chain renewals, continuous Mother AI ethics oversight.

## CLI Innovation Ideas (Top 10)
1. `vault watch` daemon with webhook/Matrix/Discord alerts.
2. Ratatui "galaxy" view of domains + registrar posture.
3. Quantum-safe, air-gapped `vault sync --quorum 3` replication bundles.
4. QUBE policy registry with configurable audit profiles per domain.
5. Registrar threat simulation engine raising AI-driven alerts.
6. Notification hooks for email/SMS/Discord/Matrix per policy rule.
7. API key lifecycle automation (`vault api rotate --age 90`).
8. `vault export-merkle` for Merkle trees & ledger anchoring.
9. Social trust recovery signatures (`vault recover --guardian` flow).
10. Quantum-safe multi-sig for domain updates (threshold Sphincs+ signatures).

## Vault Security Enhancements (5)
1. Self-fusing encryption keys that zeroize on tamper detection.
2. Biometric challenge phrases stored as sealed secret shares.
3. Continuous registrar redirect monitoring + anomaly detection.
4. Timed encryption keys that require periodic keep-alive pings.
5. Local HSM binding of registrar tokens and DS keys.

## UX & CLI Enhancements (5)
1. Auto-complete and linting for QUBE scripts inside the CLI.
2. Guided onboarding wizard (`vault init`) with TUI-based walkthrough.
3. `vault doctor` for automated configuration/misconfiguration diagnosis.
4. `vault diff` showing state deltas across backups/merkle snapshots.
5. Clipboard-safe copy/paste with automatic clearance timers for secrets.
