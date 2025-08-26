// Artifact compile cache scaffold
// Stores compiled artifacts (e.g., lowered IR / bytecode) keyed by hash of source segment.
// Placeholder implementation to be expanded.
use std::collections::HashMap;
use std::sync::Mutex;
use once_cell::sync::Lazy;
use std::time::{Instant, Duration};
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Clone, Debug)]
pub struct ArtifactEntry { pub hash: String, pub data: Vec<u8>, pub size: usize, pub last_used: Instant }

static ARTIFACT_CACHE: Lazy<Mutex<HashMap<String, ArtifactEntry>>> = Lazy::new(|| Mutex::new(HashMap::new()));
const MAX_ENTRIES: usize = 128;
const MAX_TOTAL_BYTES: usize = 10 * 1024 * 1024; // 10MB
static LOGGING_ENABLED: AtomicBool = AtomicBool::new(true);

pub fn get_artifact(hash: &str) -> Option<ArtifactEntry> { let mut g = ARTIFACT_CACHE.lock().ok()?; if let Some(e) = g.get_mut(hash) { e.last_used = Instant::now(); return Some(e.clone()); } None }

pub fn put_artifact(hash: String, data: Vec<u8>) {
    if let Ok(mut m) = ARTIFACT_CACHE.lock() {
        let size = data.len();
        m.insert(hash.clone(), ArtifactEntry { hash, data, size, last_used: Instant::now() });
        prune_locked(&mut m);
    }
}

fn prune_locked(m: &mut HashMap<String, ArtifactEntry>) {
    // Enforce max entries
    if m.len() > MAX_ENTRIES || total_bytes(m) > MAX_TOTAL_BYTES {
        let before_entries = m.len(); let before_bytes = total_bytes(m);
        // Collect keys sorted by last_used ascending
        let mut entries: Vec<_> = m.values().cloned().collect();
        entries.sort_by_key(|e| e.last_used);
        let mut bytes = total_bytes(m);
        for e in entries { if m.len() <= MAX_ENTRIES && bytes <= MAX_TOTAL_BYTES { break; } if m.remove(&e.hash).is_some() { bytes = total_bytes(m); } }
    if LOGGING_ENABLED.load(Ordering::Relaxed) { eprintln!("[artifact_cache] pruned: entries {}->{} bytes {}->{}", before_entries, m.len(), before_bytes, total_bytes(m)); }
    }
}

pub fn set_cache_logging(enabled: bool) { LOGGING_ENABLED.store(enabled, Ordering::Relaxed); }

pub fn cache_stats() -> (usize, usize) {
    if let Ok(m) = ARTIFACT_CACHE.lock() { (m.len(), total_bytes(&m)) } else { (0,0) }
}

fn total_bytes(m: &HashMap<String, ArtifactEntry>) -> usize { m.values().map(|e| e.size).sum() }

pub fn clear_artifacts() { let _ = ARTIFACT_CACHE.lock().map(|mut m| m.clear()); }
