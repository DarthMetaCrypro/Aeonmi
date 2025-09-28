Tauri Bridge - PTY + WebSocket demo

This small demo runs a PTY-backed TUI (prefers `target/debug/Aeonmi Shard`) and exposes it over a WebSocket for an xterm.js frontend.

Requirements
- Rust toolchain (stable)
- On Windows: ensure `cargo` is on PATH and ConPTY is available (Windows 10+)

Run (PowerShell):

```powershell
cd gui/tauri_bridge
cargo run --color=always
```

Open http://127.0.0.1:9001/ in a browser and click "Start TUI". The server will try to spawn `target/debug/Aeonmi Shard`, falling back to `cargo run -- --repl` if not found.

Notes
- This is a demo to help migrate to a Tauri wrapper using `portable-pty`.
- For production Tauri integration, move the PTY code into the Tauri Rust backend and expose it via Tauri commands/events.
