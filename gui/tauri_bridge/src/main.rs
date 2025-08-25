use futures::{SinkExt, StreamExt};
use std::path::PathBuf;
use portable_pty::CommandBuilder;
use tracing::{error, info};
use warp::Filter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    info!("Starting tauri_bridge PTY server");

    // Serve static files
    let static_dir = warp::fs::dir("static");

    // WebSocket endpoint
    let ws_route = warp::path("pty")
        .and(warp::ws())
        .map(|ws: warp::ws::Ws| ws.on_upgrade(handle_ws));

    // Detach endpoint - spawn a detached native terminal running Aeonmi
    let detach_route = warp::path("detach").and(warp::post()).map(|| {
        // Determine binary or fallback to cargo
        let base = PathBuf::from("target/debug/Aeonmi Shard");
        let bin_path = if cfg!(windows) {
            let mut exe = base.clone();
            exe.set_extension("exe");
            if exe.exists() { exe } else if base.exists() { base } else { PathBuf::from("cargo") }
        } else {
            if base.exists() { base } else { PathBuf::from("cargo") }
        };

        let use_cargo = bin_path.file_name().and_then(|s| s.to_str()) == Some("cargo");

        // Build a command string to execute detached
        let cmd_string = if use_cargo {
            "cargo run -- --repl".to_string()
        } else {
            format!("\"{}\" --repl", bin_path.display())
        };

        // Spawn detached depending on platform
        if cfg!(windows) {
            // `start` opens a new window and returns immediately
            let _ = std::process::Command::new("cmd")
                .arg("/C")
                .arg("start")
                .arg("")
                .arg(cmd_string)
                .spawn();
        } else {
            // Use sh -c '<cmd> &' to background the process
            let _ = std::process::Command::new("sh")
                .arg("-c")
                .arg(format!("{} &", cmd_string))
                .spawn();
        }

        warp::reply::with_status("detached", warp::http::StatusCode::OK)
    });

    let routes = static_dir.or(ws_route).or(detach_route);

    info!("Listening on 127.0.0.1:9001");
    warp::serve(routes).run(([127, 0, 0, 1], 9001)).await;

    Ok(())
}

async fn handle_ws(ws: warp::ws::WebSocket) {
    let (ws_tx, mut ws_rx) = ws.split();

    // Spawn PTY system
    let pty_system = portable_pty::native_pty_system();
    let pair = pty_system; // native_pty_system returns Box<dyn PtySystem>

    let mut pty_pair = match pair.openpty(Default::default()) {
        Ok(p) => p,
        Err(e) => {
            error!("openpty failed: {}", e);
            return;
        }
    };

    // Prefer Aeonmi Shard binary if present (Windows: prefer .exe). Fall back to `cargo`.
    let bin_path = {
        let base = PathBuf::from("target/debug/Aeonmi Shard");
        if cfg!(windows) {
            // try with .exe extension first
            let mut exe = base.clone();
            exe.set_extension("exe");
            if exe.exists() {
                exe
            } else if base.exists() {
                base
            } else {
                PathBuf::from("cargo")
            }
        } else {
            if base.exists() { base } else { PathBuf::from("cargo") }
        }
    };

    let use_cargo = bin_path.file_name().and_then(|s| s.to_str()) == Some("cargo");
    if use_cargo {
        info!("Aeonmi binary not found, falling back to 'cargo run' route");
    } else {
        info!("Using Aeonmi binary: {}", bin_path.display());
    }

    // Build the portable-pty CommandBuilder
    let mut cmd_builder = if use_cargo {
        let mut cb = CommandBuilder::new("cargo");
        cb.arg("run");
        cb.arg("--");
        cb.arg("--repl");
        cb
    } else {
        // pass the binary path as program; if it contains spaces it should still work
        let prog = bin_path.to_string_lossy().to_string();
        let mut cb = CommandBuilder::new(&prog);
        cb.arg("--repl");
        cb
    };

    // portable-pty spawn on the slave side
    let child = match pty_pair.slave.spawn_command(cmd_builder) {
        Ok(h) => h,
        Err(e) => {
            error!("failed to spawn pty child: {}", e);
            return;
        }
    };

    // Use the master side for I/O
    let mut pty_reader = pty_pair.master.try_clone_reader().expect("clone reader");
    let mut pty_writer = pty_pair.master.take_writer().expect("take writer");

    // Forward PTY -> WebSocket
    let mut ws_tx = ws_tx;
    tokio::spawn(async move {
        let mut buf = [0u8; 1024];
        loop {
            match pty_reader.read(&mut buf) {
                Ok(n) if n > 0 => {
                    if ws_tx.send(warp::ws::Message::binary(buf[..n].to_vec())).await.is_err() {
                        break;
                    }
                }
                Ok(_) => {
                    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                }
                Err(e) => {
                    error!("pty read error: {}", e);
                    break;
                }
            }
        }
    });

    // Forward WebSocket -> PTY (text frames only)
    while let Some(msg) = ws_rx.next().await {
        match msg {
            Ok(m) => {
                if m.is_text() || m.is_binary() {
                    let data = if m.is_text() { m.to_str().unwrap().as_bytes() } else { &m.as_bytes() };
                    if let Err(e) = pty_writer.write_all(data) {
                        error!("pty write error: {}", e);
                        break;
                    }
                } else if m.is_close() {
                    break;
                }
            }
            Err(e) => {
                error!("ws recv error: {}", e);
                break;
            }
        }
    }

    info!("WebSocket connection closed");
}
