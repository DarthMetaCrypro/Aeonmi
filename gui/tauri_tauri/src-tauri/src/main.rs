#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use anyhow::Result;
use std::path::{PathBuf, Path};
use serde_json::json;
use std::process::{Command, Stdio};
use std::fs;
use tauri;
use std::sync::Mutex;
use once_cell::sync::Lazy;

// We'll reuse the ai registry by depending on the workspace crate if accessible; placeholder simplified dynamic dispatch copied if not.

static AI_ENABLED: Lazy<Mutex<Option<Vec<String>>>> = Lazy::new(|| Mutex::new(None));

#[tauri::command]
fn ai_list_providers() -> Result<Vec<String>, String> {
    // For now infer from env features markers (could be improved by exposing from core crate via feature)
    let mut cache = AI_ENABLED.lock().unwrap();
    if cache.is_none() {
        let mut v = Vec::new();
        if std::env::var("OPENAI_API_KEY").is_ok() { v.push("openai".to_string()); }
        if std::env::var("PERPLEXITY_API_KEY").is_ok() { v.push("perplexity".to_string()); }
        if std::env::var("DEEPSEEK_API_KEY").is_ok() { v.push("deepseek".to_string()); }
        if std::env::var("GITHUB_COPILOT_TOKEN").is_ok() { v.push("copilot".to_string()); }
        if v.is_empty() { v.extend(["openai","perplexity","deepseek","copilot"].iter().map(|s| s.to_string())); }
        *cache = Some(v);
    }
    Ok(cache.clone().unwrap())
}

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
    // For now reuse cargo run path; not true incremental provider-level stream due to process boundary.
    // Future: link core crate and call registry directly.
    let prov = provider.unwrap_or_default();
    // We will just call non-streaming to simulate until integrated with core streaming.
    let mut args = vec!["run","--","ai","chat","--stream"]; if !prov.is_empty() { args.push("--provider"); args.push(&prov); } args.push(&prompt);
    match run_capture_output(&args) {
        Ok((_code, stdout, stderr)) => {
            if !stdout.trim().is_empty() { window.emit("ai-stream", json!({"chunk": stdout})).ok(); }
            if !stderr.trim().is_empty() { window.emit("ai-stream", json!({"error": stderr})).ok(); }
        }
        Err(e) => { window.emit("ai-stream", json!({"error": e})).ok(); }
    }
    window.emit("ai-stream", json!({"done": true})).ok();
    Ok(())
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
fn run_js(path: String) -> Result<String, String> {
    // Assumes path already compiled to JS (or is JS). Use node.
    let output = Command::new("node").arg(&path).output().map_err(|e| format!("node spawn failed: {e}"))?;
    let code = output.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    Ok(json!({"exitCode": code, "stdout": stdout, "stderr": stderr}).to_string())
}

fn find_bridge_binary() -> Option<PathBuf> {
    let exe = if cfg!(windows) { "tauri_bridge.exe" } else { "tauri_bridge" };
    let candidate = std::env::current_dir().ok()?.join("target").join("debug").join(exe);
    if candidate.exists() {
        Some(candidate)
    } else {
        None
    }
}

#[tauri::command]
fn launch_bridge() -> Result<String, String> {
    if let Some(bin) = find_bridge_binary() {
        // spawn detached bridge process; let it print its chosen URL
        Command::new(bin)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| format!("failed to spawn bridge binary: {}", e))?;
        Ok("http://127.0.0.1:9001".into())
    } else {
        // fallback: spawn `cargo run --manifest-path gui/tauri_bridge/Cargo.toml` in a child
        Command::new("cargo")
            .args(["run","--manifest-path","gui/tauri_bridge/Cargo.toml"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| format!("failed to spawn cargo run: {}", e))?;
        Ok("http://127.0.0.1:9001".into())
    }
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

    tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![launch_bridge, tauri_compile, load_file, save_file, run_js, ai_list_providers, ai_chat, ai_chat_stream])
        .run(context)
        .expect("error while running tauri application");
}
