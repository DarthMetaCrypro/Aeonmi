use std::process::{Child, Command};
use std::time::Duration;
use std::{thread, io::Read, env};

fn build_bridge() {
    let status = Command::new("cargo")
        .arg("build")
        .arg("--bin")
        .arg("tauri_bridge")
        .current_dir(".")
        .status()
        .expect("cargo build failed");
    assert!(status.success(), "cargo build failed");
}

fn spawn_built_bridge() -> Child {
    let cwd = env::current_dir().expect("cwd");
    let mut bin = cwd.join("target").join("debug").join("tauri_bridge");
    if cfg!(windows) {
        bin.set_extension("exe");
    }
    // Create log files for stdout/stderr so CI failures are easier to debug
    let tmp = env::temp_dir();
    let stdout_path = tmp.join(format!("tauri_bridge_out_{}.log", std::process::id()));
    let stderr_path = tmp.join(format!("tauri_bridge_err_{}.log", std::process::id()));
    let stdout_file = std::fs::File::create(&stdout_path).expect("create stdout log");
    let stderr_file = std::fs::File::create(&stderr_path).expect("create stderr log");

    let mut cmd = Command::new(bin);
    cmd.current_dir(".")
        .stdout(std::process::Stdio::from(stdout_file))
        .stderr(std::process::Stdio::from(stderr_file));
    let child = cmd.spawn().expect("spawn built bridge");
    // attach the log paths to the child via its id in a temp file name; return child
    child
}

fn wait_for_ready() {
    let client = reqwest::blocking::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap();

    for _ in 0..60 {
        if let Ok(resp) = client.get("http://127.0.0.1:9001/").send() {
            if resp.status().is_success() {
                return;
            }
        }
        thread::sleep(Duration::from_millis(250));
    }
    panic!("bridge did not become ready")
}

#[test]
fn smoke_ws_and_detach() {
    // Build the bridge binary first and spawn it directly
    build_bridge();
    let mut bridge = spawn_built_bridge();

    wait_for_ready();

    // Connect websocket to /pty (ensure it accepts a connection). Try a few quick retries.
    let mut last_err = None;
    let mut socket = loop {
        match tungstenite::connect(url::Url::parse("ws://127.0.0.1:9001/pty").unwrap()) {
            Ok((s, resp)) => break s,
            Err(e) => {
                last_err = Some(e);
                thread::sleep(Duration::from_millis(200));
            }
        }
    };
    socket.write_message(tungstenite::Message::Text("hello\n".into())).unwrap();

    // Try to read a message from the websocket (PTY output forwarded). Use a slightly longer timeout.
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let msg = socket.read_message();
        let _ = tx.send(msg);
    });

    let received = rx.recv_timeout(std::time::Duration::from_secs(5)).expect("no message from pty/ws");
    match received {
        Ok(m) => {
            // Any non-empty payload is fine for the smoke test
            assert!(!m.into_data().is_empty(), "pty message empty");
        }
        Err(e) => panic!("ws read error: {}", e),
    }

    // POST /detach
    let client = reqwest::blocking::Client::new();
    let resp = client.post("http://127.0.0.1:9001/detach").send().unwrap();
    assert!(resp.status().is_success());

    // cleanup
    let _ = socket.close(None);
    let _ = bridge.kill();
}
