Minimal Tauri wrapper prototype

This folder contains a minimal Tauri wrapper that launches the existing
`gui/tauri_bridge` demo (if present) and exposes a small command the
frontend can call to start the demo and return the bridge URL.

Usage (developer):

- Ensure `gui/tauri_bridge` has been built: `cargo build -p tauri_bridge --manifest-path gui/tauri_bridge/Cargo.toml`
- In this prototype you can run the tauri wrapper with `cargo run` from `gui/tauri_tauri/src-tauri` (requires Tauri toolchain installed).

This is intentionally minimal: it prefers launching the built `tauri_bridge` binary under `target/debug` and falls back to spawning `cargo run --manifest-path gui/tauri_bridge/Cargo.toml` if the binary is missing.
