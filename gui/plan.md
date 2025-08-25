Goal: Create a Tauri-based GUI that embeds an xterm.js terminal to run the existing TUI (Aeonmi Shard) and a side editor (Monaco/CodeMirror).

Minimal prototype will include:
- Tauri backend (Rust) that spawns the Aeonmi Shard TUI binary in a PTY and forwards I/O.
- Frontend with an xterm.js pane and a toggle button to spawn the TUI.
- Save/Open hooks that call the Rust backend and use existing `commands::fs` logic.

Notes:
- We'll use portable-pty crate for cross-platform PTY support.
- For Windows, ensure ConPTY is used via portable-pty.
-- Start with a demo that runs `cargo run -- --repl` or `Aeonmi Shard --edit` inside the PTY.
