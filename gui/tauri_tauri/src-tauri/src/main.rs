#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use anyhow::Result;
use std::path::{PathBuf, Path};
use serde_json::json;
// import aeonmi tauri bridge commands for direct invoke (diagnostics, compile, etc.)
use aeonmi_project::gui::tauri_bridge::commands::{aeonmi_compile_ai, aeonmi_run_native, aeonmi_diagnostics, aeonmi_ai, aeonmi_quantum_run, aeonmi_version, aeonmi_symbols, aeonmi_code_actions, aeonmi_types, aeonmi_quantum_circuit, aeonmi_rename_symbol, aeonmi_metrics};
use aeonmi_project::core::incremental::{load_metrics, reset_metrics_session, reset_metrics_full, set_deep_propagation, get_deep_propagation};
use std::process::{Command, Stdio};
use std::fs;
use tauri;
use std::sync::{Mutex, Arc};
use portable_pty::{CommandBuilder, MasterPty, PtySize};
use std::io::Write;
use std::collections::HashMap;
use once_cell::sync::Lazy;
use aeonmi_project::core::ai_provider::ProviderRegistry;
use aeonmi_project::core::api_keys::{set_api_key, get_api_key, delete_api_key};
use aeonmi_project::core::artifact_cache::{set_cache_logging, cache_stats}; // logging toggle + stats

// We'll reuse the ai registry by depending on the workspace crate if accessible; placeholder simplified dynamic dispatch copied if not.

static AI_ENABLED: Lazy<Mutex<Option<Vec<String>>>> = Lazy::new(|| Mutex::new(None));
static AI_REGISTRY: Lazy<Mutex<ProviderRegistry>> = Lazy::new(|| Mutex::new(ProviderRegistry::new()));
static PTY_REGISTRY: Lazy<Mutex<HashMap<String, Arc<PtyEntry>>>> = Lazy::new(|| Mutex::new(HashMap::new()));
static PREFS: Lazy<Mutex<HashMap<String, String>>> = Lazy::new(|| {
    // Attempt to load from disk (prefs.json in cwd)
    let mut map = HashMap::new();
    if let Ok(data) = std::fs::read_to_string("prefs.json") {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&data) {
            if let Some(obj) = json.as_object() {
                for (k,v) in obj.iter() { if let Some(s) = v.as_str() { map.insert(k.clone(), s.to_string()); } }
            }
        }
    }
    Mutex::new(map)
});
const MAX_PTY_BUFFER: usize = 200_000; // characters retained per PTY buffer

struct PtyEntry {
    child: Mutex<Box<dyn portable_pty::Child + Send>>,
    master: Mutex<Box<dyn MasterPty + Send>>,
    writer: Mutex<Box<dyn std::io::Write + Send>>,
    buffer: Mutex<String>,
    title: Mutex<String>,
}

fn save_prefs() {
    if let Ok(p) = PREFS.lock() {
        let obj: serde_json::Value = serde_json::json!(*p);
        let _ = std::fs::write("prefs.json", serde_json::to_string_pretty(&obj).unwrap_or_default());
    }
}

#[tauri::command]
fn prefs_get_all() -> Result<HashMap<String,String>, String> { Ok(PREFS.lock().unwrap().clone()) }

#[tauri::command]
fn prefs_set(key: String, value: String) -> Result<(), String> { let mut p = PREFS.lock().unwrap(); p.insert(key, value); save_prefs(); Ok(()) }

#[tauri::command]
#[tauri::command]
fn pty_create(window: tauri::Window, repl: bool, title: Option<String>) -> Result<String, String> {
    let system = portable_pty::native_pty_system();
    let pty_pair = system.openpty(Default::default()).map_err(|e| e.to_string())?;
    let mut master = pty_pair.master; let slave = pty_pair.slave;
    let id = format!("pty-{}", uuid::Uuid::new_v4());
    let mut cmd = if repl {
        let mut base = PathBuf::from("target").join("debug").join("Aeonmi Shard");
        if cfg!(windows) { let mut exe = base.clone(); exe.set_extension("exe"); if exe.exists() { base = exe; } }
        if base.exists() { let mut cb = CommandBuilder::new(base.to_string_lossy().as_ref()); cb.arg("--repl"); cb }
        else { let mut cb = CommandBuilder::new("cargo"); cb.arg("run").arg("--").arg("--repl"); cb }
    } else { if cfg!(windows) { CommandBuilder::new("cmd") } else { CommandBuilder::new("bash") } };
    let child = slave.spawn_command(cmd).map_err(|e| e.to_string())?;
    let mut reader = master.try_clone_reader().map_err(|e| e.to_string())?;
    let writer = master.take_writer().ok_or("no writer")?;
    let default_title = if repl { "REPL" } else { if cfg!(windows) { "cmd" } else { "shell" } }.to_string();
    let entry = Arc::new(PtyEntry { child: Mutex::new(child), master: Mutex::new(master), writer: Mutex::new(writer), buffer: Mutex::new(String::new()), title: Mutex::new(title.unwrap_or(default_title)) });
    PTY_REGISTRY.lock().unwrap().insert(id.clone(), entry.clone());
    // Spawn reader task
    let win = window.clone();
    tokio::spawn(async move {
        let mut buf = [0u8; 1024];
        loop {
            match reader.read(&mut buf) {
                Ok(n) if n > 0 => {
                    let chunk = String::from_utf8_lossy(&buf[..n]).to_string();
                    if let Some(e) = PTY_REGISTRY.lock().unwrap().get(&id) {
                        let mut b = e.buffer.lock().unwrap();
                        b.push_str(&chunk);
                        if b.len() > MAX_PTY_BUFFER {
                            // trim from front (preserve ending). Drain oldest excess.
                            let excess = b.len() - MAX_PTY_BUFFER;
                            b.drain(0..excess);
                        }
                    }
                    if win.emit("pty-data", json!({"id": id, "data": chunk})).is_err() { break; }
                }
                Ok(_) => { tokio::time::sleep(std::time::Duration::from_millis(16)).await; }
                Err(_) => break,
            }
        }
        let _ = win.emit("pty-exit", json!({"id": id}));
    });
    Ok(id)
}

#[tauri::command]
fn pty_write(id: String, data: String) -> Result<(), String> {
    if let Some(entry) = PTY_REGISTRY.lock().unwrap().get(&id) {
        let mut w = entry.writer.lock().unwrap();
        w.write_all(data.as_bytes()).map_err(|e| e.to_string())?;
        w.flush().ok();
        Ok(())
    } else { Err("unknown pty".into()) }
}

#[tauri::command]
fn pty_close(id: String) -> Result<(), String> {
    if let Some(entry) = PTY_REGISTRY.lock().unwrap().remove(&id) { let _ = entry.child.lock().unwrap().kill(); }
    Ok(())
}

#[tauri::command]
fn pty_resize(id: String, cols: u16, rows: u16) -> Result<(), String> {
    if let Some(entry) = PTY_REGISTRY.lock().unwrap().get(&id) {
        let size = PtySize { rows, cols, pixel_width: 0, pixel_height: 0 };
        entry.master.lock().unwrap().resize(size).map_err(|e| e.to_string())?;
        Ok(())
    } else { Err("unknown pty".into()) }
}

#[tauri::command]
fn pty_list() -> Result<Vec<String>, String> { Ok(PTY_REGISTRY.lock().unwrap().keys().cloned().collect()) }
#[tauri::command]
fn pty_list_detailed() -> Result<Vec<serde_json::Value>, String> {
    let reg = PTY_REGISTRY.lock().unwrap();
    Ok(reg.iter().map(|(id, e)| {
        serde_json::json!({"id": id, "title": *e.title.lock().unwrap()})
    }).collect())
}

#[tauri::command]
fn pty_rename(id: String, title: String) -> Result<(), String> {
    if let Some(e) = PTY_REGISTRY.lock().unwrap().get(&id) { *e.title.lock().unwrap() = title; Ok(()) } else { Err("unknown pty".into()) }
}

#[tauri::command]
fn pty_buffer(id: String) -> Result<String, String> {
    if let Some(e) = PTY_REGISTRY.lock().unwrap().get(&id) { Ok(e.buffer.lock().unwrap().clone()) } else { Err("unknown pty".into()) }
}

#[tauri::command]
fn pty_export(id: String, path: Option<String>) -> Result<String, String> {
    let buf = pty_buffer(id.clone())?;
    if let Some(p) = path {
        std::fs::write(&p, &buf).map_err(|e| e.to_string())?;
        Ok(p)
    } else {
        Ok(buf)
    }
}

#[tauri::command]
fn ai_list_providers() -> Result<Vec<String>, String> {
    Ok(AI_REGISTRY.lock().unwrap().list())
}

#[tauri::command]
fn ai_set_provider(name: String) -> Result<(), String> { if AI_REGISTRY.lock().unwrap().set_active(&name) { Ok(()) } else { Err("unknown provider".into()) } }

#[tauri::command]
fn ai_chat(provider: Option<String>, prompt: String, stream: bool) -> Result<String, String> {
    // Call into the main crate via executing cargo run for now (simplest boundary) – future refactor: extract into shared lib.
    if stream {
        // streaming not supported via exec fallback yet
    }
    let prov = provider.unwrap_or_default();
    let mut args = vec!["run","--","ai","chat"]; if !prov.is_empty() { args.push("--provider"); args.push(&prov); } args.push(&prompt);
    let (code, stdout, stderr) = run_capture_output(&args).map_err(|e| e)?;
    if code != 0 { return Err(format!("chat exited {code}: {stderr}")); }
    let combined = if stdout.trim().is_empty() { stderr } else { stdout };
    Ok(json!({"provider": prov, "output": combined}).to_string())
}
 
#[tauri::command]
fn ai_chat_stream(window: tauri::Window, provider: Option<String>, prompt: String) -> Result<(), String> {
    let prov = provider.unwrap_or_default();
    // Simulate streaming by chunking deterministic faux output
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    prompt.hash(&mut hasher); prov.hash(&mut hasher);
    let seed = hasher.finish();
    let base = format!("[{prov}] {} :: {}", seed % 10000, prompt.chars().take(120).collect::<String>());
    let window_clone = window.clone();
    tokio::spawn(async move {
        let chars: Vec<char> = base.chars().collect();
        let chunk_size = 16usize;
        for i in (0..chars.len()).step_by(chunk_size) {
            let chunk: String = chars[i..std::cmp::min(i+chunk_size, chars.len())].iter().collect();
            if window_clone.emit("ai-stream", json!({"chunk": chunk})).is_err() { break; }
            tokio::time::sleep(std::time::Duration::from_millis(60)).await;
        }
        let _ = window_clone.emit("ai-stream", json!({"done": true}));
    });
    Ok(())
}

#[tauri::command]
fn aeonmi_quantum_simulate(source: String) -> Result<String, String> {
    // Very small stub: derive number of (pseudo) qubits from distinct identifiers 'q' digits, build uniform or simple entangled state
    use aeonmi_project::core::lexer::Lexer;
    use aeonmi_project::core::token::TokenKind;
    let mut lexer = Lexer::from_str(&source);
    let mut qubits: Vec<String> = Vec::new();
    let mut superpose = 0; let mut entangle = 0; let mut measure = 0;
    if let Ok(tokens) = lexer.tokenize() {
        for t in tokens.iter() {
            match t.kind { TokenKind::Identifier => { let txt = t.lexeme.clone(); if txt.starts_with('q') && !qubits.contains(&txt) { qubits.push(txt); } },
                           TokenKind::Superpose => superpose+=1,
                           TokenKind::Entangle => entangle+=1,
                           TokenKind::Measure => measure+=1,
                           _=>{} }
        }
    }
    let n = qubits.len().cloned().max(1);
    let dim = 1usize << n;
    // build naive statevector
    let mut state: Vec<f64> = vec![0.0; dim];
    if superpose > 0 { let amp = 1.0 / (dim as f64).sqrt(); for v in state.iter_mut() { *v = amp; } } else { state[0] = 1.0; }
    if entangle > 0 && dim >= 4 { // simple bell-ish adjust first two significant basis states
        for v in state.iter_mut() { *v = 0.0; }
        state[0] = 1.0 / std::f64::consts::SQRT_2; state[dim-1] = 1.0 / std::f64::consts::SQRT_2;
    }
    // histogram (probabilities * shots) with default shots 1024 if measurement present
    let shots = 1024.0;
    let mut histogram = serde_json::Map::new();
    if measure > 0 {
        for (i, amp) in state.iter().enumerate() { let p = amp * amp; if p > 0.00001 { histogram.insert(format!("{:0width$b}", i, width=n), serde_json::json!((p*shots).round() as u64)); } }
    }
    Ok(json!({
        "qubits": n,
        "ops": {"superpose": superpose, "entangle": entangle, "measure": measure},
        "statevector": state,
        "histogram": histogram,
    }).to_string())
}
use serde_json::json;

// Simple compile helper reusing workspace binary via invoking `cargo run -- compile`
fn run_capture_output(args: &[&str]) -> std::result::Result<(i32,String,String), String> {
    let output = Command::new("cargo").args(args).output().map_err(|e| format!("spawn failed: {e}"))?;
    let code = output.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    Ok((code, stdout, stderr))
}

fn run_compile(input: &Path, emit_ai: bool) -> std::result::Result<serde_json::Value, String> {
    let emit_kind = if emit_ai { "ai" } else { "js" };
    let out_file = if emit_ai { "gui_output.ai" } else { "gui_output.js" };
    let (code, stdout, stderr) = run_capture_output(&["run","--","compile", input.to_string_lossy().as_ref(), "--emit", emit_kind, "--out", out_file])?;
    let success = code == 0;
    // Parse structured diagnostics: lines prefixed with @@DIAG:{json}
    let mut diagnostics = Vec::new();
    for line in stderr.lines() {
        if let Some(rest) = line.strip_prefix("@@DIAG:") {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(rest) {
                diagnostics.push(v);
            }
        }
    }
    Ok(json!({
        "success": success,
        "exitCode": code,
        "outputFile": out_file,
        "stdout": stdout,
        "stderr": stderr,
        "diagnostics": diagnostics,
    }))
}

#[tauri::command]
fn tauri_compile(path: String, emit: Option<String>) -> Result<String, String> {
    let p = PathBuf::from(&path);
    if !p.exists() { return Err(format!("file not found: {}", path)); }
    let ai = matches!(emit.as_deref(), Some("ai"));
    match run_compile(&p, ai) {
        Ok(v) => Ok(v.to_string()),
        Err(e) => Err(e),
    }
}

#[tauri::command]
fn load_file(path: String) -> Result<String, String> {
    fs::read_to_string(&path).map_err(|e| format!("load error: {e}"))
}

#[tauri::command]
fn save_file(path: String, contents: String) -> Result<bool, String> {
    fs::write(&path, contents).map_err(|e| format!("save error: {e}"))?;
    Ok(true)
}

#[tauri::command]
fn api_key_set(provider: String, key: String) -> Result<(), String> { set_api_key(&provider, &key) }
#[tauri::command]
fn api_key_get(provider: String) -> Result<Option<String>, String> { Ok(get_api_key(&provider)) }
#[tauri::command]
fn api_key_delete(provider: String) -> Result<(), String> { delete_api_key(&provider) }

#[tauri::command]
fn cache_logging(enable: bool) -> Result<(), String> { set_cache_logging(enable); Ok(()) }

#[tauri::command]
fn cache_stats_get() -> Result<String, String> { let (entries, bytes) = cache_stats(); Ok(json!({"entries": entries, "bytes": bytes}).to_string()) }

#[tauri::command]
fn metrics_reset() -> Result<(), String> { reset_metrics_session(); Ok(()) }

#[tauri::command]
fn metrics_reset_full() -> Result<(), String> { reset_metrics_full(); Ok(()) }

#[tauri::command]
fn metrics_set_deep(enable: bool) -> Result<(), String> { set_deep_propagation(enable); Ok(()) }
#[tauri::command]
fn metrics_get_deep() -> Result<bool, String> { Ok(get_deep_propagation()) }

#[tauri::command]
fn run_js(path: String) -> Result<String, String> {
    // Assumes path already compiled to JS (or is JS). Use node.
    let output = Command::new("node").arg(&path).output().map_err(|e| format!("node spawn failed: {e}"))?;
    let code = output.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    Ok(json!({"exitCode": code, "stdout": stdout, "stderr": stderr}).to_string())
}


fn main() {
    // generate_context!() can panic if the tauri config or assets are not present during
    // certain check environments. Try to provide a clearer error path.
    let context = std::panic::catch_unwind(|| tauri::generate_context!());
    let context = match context {
        Ok(ctx) => ctx,
        Err(_) => {
            eprintln!("tauri::generate_context!() failed: ensure src-tauri/tauri.conf.json and icons are present");
            std::process::exit(1);
        }
    };

    // Load persisted metrics early
    load_metrics();
    tauri::Builder::default()
    .on_window_event(|_w, e| { if let tauri::WindowEvent::CloseRequested { .. } = e { let mut reg = PTY_REGISTRY.lock().unwrap(); for (_id, entry) in reg.drain() { let _ = entry.child.lock().unwrap().kill(); } } })
    .invoke_handler(tauri::generate_handler![tauri_compile, load_file, save_file, run_js, ai_list_providers, ai_set_provider, ai_chat, ai_chat_stream, aeonmi_compile_ai, aeonmi_run_native, aeonmi_diagnostics, aeonmi_ai, aeonmi_quantum_run, aeonmi_quantum_simulate, aeonmi_version, aeonmi_symbols, aeonmi_code_actions, aeonmi_types, aeonmi_quantum_circuit, aeonmi_rename_symbol, aeonmi_metrics, metrics_reset, metrics_reset_full, metrics_set_deep, metrics_get_deep, pty_create, pty_write, pty_close, pty_resize, pty_list, pty_list_detailed, pty_rename, pty_buffer, pty_export, prefs_get_all, prefs_set, api_key_set, api_key_get, api_key_delete, cache_logging, cache_stats_get])
        .run(context)
        .expect("error while running tauri application");
}
