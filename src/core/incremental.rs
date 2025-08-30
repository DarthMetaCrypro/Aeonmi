//! Incremental parsing manager (initial scaffold).
//! Currently caches last full parse; future work will diff edited regions.

use crate::core::lexer::Lexer;
use crate::core::parser::{Parser as AeParser, ParserError};
use crate::core::ast::ASTNode;
use std::sync::Mutex;
use once_cell::sync::Lazy;
use sha1::{Sha1, Digest};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Instant, Duration};

#[derive(Debug, Clone)]
pub struct NodeSpan { pub start_line: usize, pub end_line: usize }

#[derive(Clone)]
pub struct CachedParse {
    pub hash: String,
    pub ast: ASTNode,
    pub source: String,
    pub top_spans: Vec<NodeSpan>,
}

#[derive(Debug, Clone, Default)]
pub struct DirtyInfo {
    pub changed: bool,
    pub first_changed_line: usize,
    pub last_changed_line: usize,
}

static CACHE: Lazy<Mutex<Option<CachedParse>>> = Lazy::new(|| Mutex::new(None));

// Cache of last semantic diagnostics grouped per top-level node (index aligned with Program children)
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct TopLevelDiagCache { pub per_node: Vec<Vec<crate::core::semantic_analyzer::SemanticDiagnostic>> }
#[allow(dead_code)]
pub static DIAG_CACHE: Lazy<Mutex<TopLevelDiagCache>> = Lazy::new(|| Mutex::new(TopLevelDiagCache::default()));
pub static LAST_REPLACED_INDEX: Lazy<Mutex<Option<usize>>> = Lazy::new(|| Mutex::new(None));
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct TopLevelTypeDiagCache { pub per_node: Vec<Vec<crate::core::types::TypeDiagnostic>> }
#[allow(dead_code)]
pub static TYPE_DIAG_CACHE: Lazy<Mutex<TopLevelTypeDiagCache>> = Lazy::new(|| Mutex::new(TopLevelTypeDiagCache::default()));

// Call graph + variable dependency metrics (global, for inspection & incremental invalidation stats)
#[derive(Debug, Default, Clone)]
pub struct CallGraphMetrics {
    pub functions: usize,
    pub edges: usize,
    pub reinfer_events: usize,
    pub variable_edges: usize, // variable -> function or function -> variable
}
pub static CALL_GRAPH_METRICS: Lazy<Mutex<CallGraphMetrics>> = Lazy::new(|| Mutex::new(CallGraphMetrics::default()));

// Variable dependency tracking: which functions read or write which top-level variables.
#[derive(Debug, Default, Clone)]
pub struct VarDeps { pub reads: HashMap<String, HashSet<usize>>, pub writes: HashMap<String, HashSet<usize>> }
pub static VAR_DEPS: Lazy<Mutex<VarDeps>> = Lazy::new(|| Mutex::new(VarDeps::default()));
// Deep propagation toggle (controls whether we compute full transitive caller set regardless of size)
pub static DEEP_PROPAGATION: AtomicBool = AtomicBool::new(false);

// Per-function inference timing metrics (index-based; names resolved on query)
// ema_ns tracks an exponential moving average (recent-weighted) of inference duration.
#[derive(Debug, Clone)]
pub struct FunctionInferenceMetric { pub total_ns: u128, pub runs: u64, pub last_ns: u128, pub ema_ns: u128, pub window: VecDeque<u128>, pub last_run_epoch_ms: u64 }
impl Default for FunctionInferenceMetric { fn default() -> Self { Self { total_ns:0, runs:0, last_ns:0, ema_ns:0, window:VecDeque::new(), last_run_epoch_ms:0 } } }
pub static FUNCTION_METRICS: Lazy<Mutex<HashMap<usize, FunctionInferenceMetric>>> = Lazy::new(|| Mutex::new(HashMap::new()));
static LAST_PERSIST: Lazy<Mutex<Option<Instant>>> = Lazy::new(|| Mutex::new(None));
const PERSIST_DEBOUNCE: Duration = Duration::from_millis(500);
// Runtime configurable EMA alpha (1..=100) via env AEONMI_EMA_ALPHA (default 20)
pub static EMA_ALPHA_RUNTIME: once_cell::sync::Lazy<std::sync::atomic::AtomicU64> = once_cell::sync::Lazy::new(|| {
    let v = std::env::var("AEONMI_EMA_ALPHA").ok().and_then(|s| s.parse::<u64>().ok()).filter(|v| (1..=100).contains(v)).unwrap_or(20);
    std::sync::atomic::AtomicU64::new(v)
});
// Rolling window capacity via env AEONMI_METRICS_WINDOW (default 16, min 4, max 256)
pub static WINDOW_CAP_RUNTIME: once_cell::sync::Lazy<std::sync::atomic::AtomicUsize> = once_cell::sync::Lazy::new(|| {
    let v = std::env::var("AEONMI_METRICS_WINDOW").ok().and_then(|s| s.parse::<usize>().ok()).filter(|v| *v>=4 && *v<=256).unwrap_or(16);
    std::sync::atomic::AtomicUsize::new(v)
});
static SESSION_START_EPOCH_MS: once_cell::sync::Lazy<u64> = once_cell::sync::Lazy::new(|| current_epoch_ms());

fn current_epoch_ms() -> u64 { use std::time::{SystemTime, UNIX_EPOCH}; SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as u64).unwrap_or(0) }
pub fn session_start_epoch_ms() -> u64 { *SESSION_START_EPOCH_MS }
pub fn set_ema_alpha(pct: u64) { if (1..=100).contains(&pct) { EMA_ALPHA_RUNTIME.store(pct, Ordering::Relaxed); } }
pub fn set_window_capacity(n: usize) { if n>=4 && n<=256 { WINDOW_CAP_RUNTIME.store(n, Ordering::Relaxed); } }

// Partial vs estimated full inference savings metrics with recent window history
#[derive(Debug, Clone)]
pub struct SavingsSample { pub partial_ns: u128, pub estimated_full_ns: u128, pub savings_ns: u128 }
#[derive(Debug, Clone)]
pub struct SavingsMetrics { pub cumulative_savings_ns: u128, pub cumulative_partial_ns: u128, pub cumulative_estimated_full_ns: u128, pub history: VecDeque<SavingsSample>, pub window_partial_ns: u128, pub window_est_full_ns: u128, pub history_cap: usize }
impl Default for SavingsMetrics { fn default() -> Self { Self { cumulative_savings_ns:0, cumulative_partial_ns:0, cumulative_estimated_full_ns:0, history:VecDeque::new(), window_partial_ns:0, window_est_full_ns:0, history_cap:32 } } }
impl SavingsMetrics { pub fn push_sample(&mut self, partial: u128, est_full: u128) { let savings = est_full.saturating_sub(partial); self.cumulative_partial_ns += partial; self.cumulative_estimated_full_ns += est_full; self.cumulative_savings_ns += savings; self.window_partial_ns += partial; self.window_est_full_ns += est_full; let sample = SavingsSample { partial_ns: partial, estimated_full_ns: est_full, savings_ns: savings }; if self.history.len() == self.history_cap { if let Some(old) = self.history.pop_front() { // adjust window counters removing old sample
 self.window_partial_ns = self.window_partial_ns.saturating_sub(old.partial_ns); self.window_est_full_ns = self.window_est_full_ns.saturating_sub(old.estimated_full_ns); }
 } self.history.push_back(sample); } }
pub static SAVINGS_METRICS: Lazy<Mutex<SavingsMetrics>> = Lazy::new(|| Mutex::new(SavingsMetrics::default()));
#[allow(dead_code)]
pub fn record_savings(partial_ns: u128, estimated_full_ns: u128) { if partial_ns == 0 || estimated_full_ns == 0 { return; } if let Ok(mut sm) = SAVINGS_METRICS.lock() { sm.push_sample(partial_ns, estimated_full_ns); } }
/// Back-compat wrapper (tests expect this name). Records a partial inference duration and
/// the estimated full duration. Ignores samples where either is zero or estimated is less
/// than partial (invalid / inverted measurement).
pub fn record_partial_savings(partial_ns: u128, estimated_full_ns: u128) {
    if partial_ns == 0 || estimated_full_ns == 0 || estimated_full_ns < partial_ns { return; }
    record_savings(partial_ns, estimated_full_ns);
}
pub fn set_history_cap(n: usize) { if n>=8 && n<=256 { if let Ok(mut sm)=SAVINGS_METRICS.lock() { sm.history_cap = n; while sm.history.len()>sm.history_cap { if let Some(old)=sm.history.pop_front() { sm.window_partial_ns = sm.window_partial_ns.saturating_sub(old.partial_ns); sm.window_est_full_ns = sm.window_est_full_ns.saturating_sub(old.estimated_full_ns); } } } } }

#[allow(dead_code)]
pub fn record_function_infer(idx: usize, dur: u128) {
    if let Ok(mut m) = FUNCTION_METRICS.lock() {
        let entry = m.entry(idx).or_insert_with(FunctionInferenceMetric::default);
        entry.total_ns += dur; entry.runs += 1; entry.last_ns = dur;
    entry.last_run_epoch_ms = current_epoch_ms();
    let alpha = EMA_ALPHA_RUNTIME.load(Ordering::Relaxed) as u128;
    if entry.runs == 1 { entry.ema_ns = dur; } else { entry.ema_ns = (entry.ema_ns * (100 - alpha) + dur * alpha) / 100; }
    let cap = WINDOW_CAP_RUNTIME.load(Ordering::Relaxed);
    if entry.window.len()==cap { entry.window.pop_front(); }
    entry.window.push_back(dur);
    }
}

pub fn set_deep_propagation(v: bool) { DEEP_PROPAGATION.store(v, Ordering::Relaxed); }
pub fn get_deep_propagation() -> bool { DEEP_PROPAGATION.load(Ordering::Relaxed) }

fn metrics_file_path() -> std::path::PathBuf {
    // Use user config dir (~/.config/aeonmi/aeonmi_metrics.json) for consistency with keys
    let base = dirs_next::config_dir().unwrap_or(std::env::temp_dir()).join("aeonmi");
    if let Err(e) = std::fs::create_dir_all(&base) { eprintln!("metrics dir create error: {e}"); }
    base.join("aeonmi_metrics.json")
}
const METRICS_FILE: &str = "aeonmi_metrics.json"; // kept for legacy; actual path computed dynamically
const METRICS_VERSION: u32 = 6; // bumped for rolling window & history

pub fn metrics_file_location() -> std::path::PathBuf { metrics_file_path() }

pub fn build_metrics_json() -> serde_json::Value {
    let m = CALL_GRAPH_METRICS.lock().ok().map(|g| g.clone()).unwrap_or_default();
    let v = VAR_DEPS.lock().ok().map(|g| g.clone()).unwrap_or_default();
    let fm = FUNCTION_METRICS.lock().ok().map(|g| g.clone()).unwrap_or_default();
    let sm = SAVINGS_METRICS.lock().ok().map(|g| g.clone()).unwrap_or_default();
    let session_start = session_start_epoch_ms();
    let mut pruned = 0usize;
    let function_metrics: HashMap<String, serde_json::Value> = fm.iter().filter_map(|(idx, fm)| {
        if fm.last_run_epoch_ms>0 && fm.last_run_epoch_ms < session_start { pruned +=1; return None; }
        let window_avg_ns = if !fm.window.is_empty() { fm.window.iter().copied().sum::<u128>() / fm.window.len() as u128 } else { 0 };        
        Some((idx.to_string(), serde_json::json!({
            "runs": fm.runs,
            "total_ns": fm.total_ns,
            "last_ns": fm.last_ns,
            "avg_ns": if fm.runs>0 { fm.total_ns / fm.runs as u128 } else { 0 },
            "ema_ns": fm.ema_ns,
            "window_avg_ns": window_avg_ns,
            "last_run_epoch_ms": fm.last_run_epoch_ms
        })))
    }).collect();
    let savings_pct = if sm.cumulative_estimated_full_ns>0 { (sm.cumulative_savings_ns as f64 / sm.cumulative_estimated_full_ns as f64) * 100.0 } else { 0.0 };
    let partial_pct = if sm.cumulative_estimated_full_ns>0 { (sm.cumulative_partial_ns as f64 / sm.cumulative_estimated_full_ns as f64) * 100.0 } else { 0.0 };
    let recent_window_savings_pct = if sm.window_est_full_ns>0 { (sm.history.iter().map(|s| s.savings_ns).sum::<u128>() as f64 / sm.window_est_full_ns as f64) *100.0 } else { 0.0 };
    let ema_alpha = EMA_ALPHA_RUNTIME.load(Ordering::Relaxed);
    let window_cap = WINDOW_CAP_RUNTIME.load(Ordering::Relaxed);
    serde_json::json!({
        "version": METRICS_VERSION,
        "metrics": {"functions": m.functions, "edges": m.edges, "reinfer_events": m.reinfer_events, "variable_edges": m.variable_edges},
        "varReads": v.reads.iter().map(|(k, set)| (k, set.iter().collect::<Vec<_>>())).collect::<std::collections::HashMap<_,_>>(),
        "varWrites": v.writes.iter().map(|(k, set)| (k, set.iter().collect::<Vec<_>>())).collect::<std::collections::HashMap<_,_>>(),
        "functionMetrics": function_metrics,
        "functionMetricsPruned": pruned,
        "emaAlphaPct": ema_alpha,
        "windowCapacity": window_cap,
        "deepPropagation": get_deep_propagation(),
        "savings": {"cumulative_savings_ns": sm.cumulative_savings_ns, "cumulative_partial_ns": sm.cumulative_partial_ns, "cumulative_estimated_full_ns": sm.cumulative_estimated_full_ns, "cumulative_savings_pct": savings_pct, "cumulative_partial_pct": partial_pct, "recent_window_partial_ns": sm.window_partial_ns, "recent_window_estimated_full_ns": sm.window_est_full_ns, "recent_window_savings_pct": recent_window_savings_pct, "recent_samples": sm.history.iter().map(|s| serde_json::json!({"partial_ns": s.partial_ns, "estimated_full_ns": s.estimated_full_ns, "savings_ns": s.savings_ns})).collect::<Vec<_>>() }
    })
}

/// Ensure a stub metrics file exists even if no metrics recorded yet (CLI tooling friendliness).
pub fn ensure_metrics_file_exists() {
    let path = metrics_file_path();
    if path.exists() { return; }
    let json = build_metrics_json();
    let _ = std::fs::write(&path, serde_json::to_string_pretty(&json).unwrap_or_default());
}

pub fn persist_metrics() {
    // Debounce check
    {
        if let Ok(mut last) = LAST_PERSIST.lock() {
            let now = Instant::now();
            if let Some(prev) = *last { if now.duration_since(prev) < PERSIST_DEBOUNCE { return; } }
            *last = Some(now);
        }
    }
    if CALL_GRAPH_METRICS.lock().is_ok() { // cheap check; build JSON anyway
        let json = build_metrics_json();
    let path = metrics_file_path();
    if let Err(e) = std::fs::write(&path, serde_json::to_string_pretty(&json).unwrap_or_default()) { eprintln!("persist_metrics write error: {e}"); }
    }
}

/// Force persistence ignoring debounce (used by metrics-flush CLI)
pub fn force_persist_metrics() {
    if CALL_GRAPH_METRICS.lock().is_ok() {
        let json = build_metrics_json();
    let path = metrics_file_path();
    if let Err(e) = std::fs::write(&path, serde_json::to_string_pretty(&json).unwrap_or_default()) { eprintln!("force_persist_metrics write error: {e}"); }
    }
}

// Drop guard to flush metrics on shutdown (best-effort, ignores errors)
pub struct MetricsFlushGuard;
impl Drop for MetricsFlushGuard {
    fn drop(&mut self) { force_persist_metrics(); }
}

pub fn install_shutdown_flush_guard() -> MetricsFlushGuard { MetricsFlushGuard }

/// Helper to compute caller propagation set given reverse edges (rev[target] -> callers)
#[allow(dead_code)]
pub fn compute_transitive_callers(changed: usize, rev: &Vec<Vec<usize>>, deep: bool, shallow_limit: usize) -> std::collections::HashSet<usize> {
    let mut set: std::collections::HashSet<usize> = std::collections::HashSet::new();
    for &caller in &rev[changed] { set.insert(caller); }
    if deep || set.len() < shallow_limit {
        let mut queue: std::collections::VecDeque<usize> = set.iter().copied().collect();
        while let Some(cur) = queue.pop_front() { for &c in &rev[cur] { if set.insert(c) { queue.push_back(c); } } }
    }
    set
}

pub fn load_metrics() {
    let path = metrics_file_path();
    if let Ok(data) = std::fs::read_to_string(path) {
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&data) {
            if let Some(mo) = val.get("metrics") { if let Ok(mut m) = CALL_GRAPH_METRICS.lock() {
                m.functions = mo.get("functions").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                m.edges = mo.get("edges").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                m.reinfer_events = mo.get("reinfer_events").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                m.variable_edges = mo.get("variable_edges").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
            }}
            if let Some(fr) = val.get("varReads") { if let Ok(mut vd)=VAR_DEPS.lock() { if let Some(obj)=fr.as_object() { for (k, arr) in obj { let mut set: HashSet<usize> = HashSet::new(); if let Some(a)=arr.as_array() { for v in a { if let Some(s)=v.as_str() { if let Ok(idx)=s.parse::<usize>() { set.insert(idx); } } } } vd.reads.insert(k.clone(), set); } } } }
            if let Some(fw) = val.get("varWrites") { if let Ok(mut vd)=VAR_DEPS.lock() { if let Some(obj)=fw.as_object() { for (k, arr) in obj { let mut set: HashSet<usize> = HashSet::new(); if let Some(a)=arr.as_array() { for v in a { if let Some(s)=v.as_str() { if let Ok(idx)=s.parse::<usize>() { set.insert(idx); } } } } vd.writes.insert(k.clone(), set); } } } }
                if let Some(fm) = val.get("functionMetrics") { if let Ok(mut map)=FUNCTION_METRICS.lock() { if let Some(obj)=fm.as_object() { for (k,v) in obj { if let Ok(idx)=k.parse::<usize>() { let mut metric=FunctionInferenceMetric::default(); metric.runs=v.get("runs").and_then(|x| x.as_u64()).unwrap_or(0); metric.total_ns=v.get("total_ns").and_then(|x| x.as_u64()).unwrap_or(0) as u128; metric.last_ns=v.get("last_ns").and_then(|x| x.as_u64()).unwrap_or(0) as u128; metric.ema_ns=v.get("ema_ns").and_then(|x| x.as_u64()).unwrap_or(metric.last_ns as u64) as u128; map.insert(idx, metric); } } } } }
            if let Some(dp)=val.get("deepPropagation") { if let Some(b)=dp.as_bool() { set_deep_propagation(b); } }
            if let Some(sv)=val.get("savings") { if let Ok(mut sm)=SAVINGS_METRICS.lock() { sm.cumulative_savings_ns = sv.get("cumulative_savings_ns").and_then(|x| x.as_u64()).unwrap_or(0) as u128; sm.cumulative_partial_ns = sv.get("cumulative_partial_ns").and_then(|x| x.as_u64()).unwrap_or(0) as u128; sm.cumulative_estimated_full_ns = sv.get("cumulative_estimated_full_ns").and_then(|x| x.as_u64()).unwrap_or(0) as u128; if let Some(arr)=sv.get("recent_samples").and_then(|x| x.as_array()) { for s in arr { let p = s.get("partial_ns").and_then(|x| x.as_u64()).unwrap_or(0) as u128; let e = s.get("estimated_full_ns").and_then(|x| x.as_u64()).unwrap_or(0) as u128; if p>0 && e>0 { sm.push_sample(p,e); } } } } }
        }
    }
}

#[allow(dead_code)]
pub fn reset_metrics_session() { if let Ok(mut m)=CALL_GRAPH_METRICS.lock() { m.reinfer_events = 0; } }

#[allow(dead_code)]
pub fn reset_metrics_full() {
    if let Ok(mut cg)=CALL_GRAPH_METRICS.lock() { *cg = CallGraphMetrics::default(); }
    if let Ok(mut vd)=VAR_DEPS.lock() { *vd = VarDeps::default(); }
    if let Ok(mut fm)=FUNCTION_METRICS.lock() { fm.clear(); }
    if let Ok(mut sm)=SAVINGS_METRICS.lock() { *sm = SavingsMetrics::default(); }
    persist_metrics();
}

pub fn reset_runtime_metrics_config() { set_ema_alpha(20); set_window_capacity(16); set_history_cap(32); }

#[allow(dead_code)]
pub fn record_reinfer_event(count: usize) {
    if let Ok(mut m) = CALL_GRAPH_METRICS.lock() { m.reinfer_events += count; }
}

#[allow(dead_code)]
pub fn snapshot_call_graph_metrics() -> CallGraphMetrics { CALL_GRAPH_METRICS.lock().unwrap().clone() }

// Compute variable dependency map (reads/writes) for all top-level functions in an AST.
// This mirrors logic used in the GUI command for selective reinference.
#[allow(dead_code)]
pub fn compute_var_deps(ast: &ASTNode) -> VarDeps {
    let mut var_reads: HashMap<String, HashSet<usize>> = HashMap::new();
    let mut var_writes: HashMap<String, HashSet<usize>> = HashMap::new();
    use crate::core::ast::ASTNode as N;
    fn walk(idx: usize, n: &N, reads: &mut HashMap<String, HashSet<usize>>, writes: &mut HashMap<String, HashSet<usize>>) {
        match n {
            N::Assignment { name, value, .. } => { writes.entry(name.clone()).or_default().insert(idx); walk(idx, value, reads, writes); },
            N::VariableDecl { name, value, .. } => { writes.entry(name.clone()).or_default().insert(idx); walk(idx, value, reads, writes); },
            N::Identifier(name) | N::IdentifierSpanned { name, .. } => { reads.entry(name.clone()).or_default().insert(idx); },
            N::Function { body, .. } => { for c in body { walk(idx, c, reads, writes); } },
            N::Block(b) => { for c in b { walk(idx, c, reads, writes); } },
            N::If { condition, then_branch, else_branch } => { walk(idx, condition, reads, writes); walk(idx, then_branch, reads, writes); if let Some(e)=else_branch { walk(idx, e, reads, writes); } },
            N::While { condition, body } => { walk(idx, condition, reads, writes); walk(idx, body, reads, writes); },
            N::For { init, condition, increment, body } => { if let Some(i)=init { walk(idx, i, reads, writes); } if let Some(c)=condition { walk(idx, c, reads, writes); } if let Some(inc)=increment { walk(idx, inc, reads, writes); } walk(idx, body, reads, writes); },
            N::BinaryExpr { left, right, .. } => { walk(idx, left, reads, writes); walk(idx, right, reads, writes); },
            N::UnaryExpr { expr, .. } => { walk(idx, expr, reads, writes); },
            N::Call { callee, args } => { walk(idx, callee, reads, writes); for a in args { walk(idx, a, reads, writes); } },
            N::Return(e) | N::Log(e) => { walk(idx, e, reads, writes); },
            _ => {}
        }
    }
    if let ASTNode::Program(items) = ast { for (idx, node) in items.iter().enumerate() { if let N::Function { body, .. } = node { for c in body { walk(idx, c, &mut var_reads, &mut var_writes); } } } }
    VarDeps { reads: var_reads, writes: var_writes }
}

/// Parse source using cached AST when unchanged. Returns AST and dirty info.
pub fn parse_or_cached(source: &str) -> Result<ASTNode, String> {
    let mut hasher = Sha1::new(); hasher.update(source.as_bytes()); let hash = format!("{:x}", hasher.finalize());
    if let Some(cached) = CACHE.lock().unwrap().as_ref() {
        if cached.hash == hash { return Ok(cached.ast.clone()); }
    }
    let _dirty = compute_dirty_info(source);
    // For now we still reparse whole file; future: region-based reparse using token window around dirty lines.
    let mut lexer = Lexer::from_str(source);
    let tokens = lexer.tokenize().map_err(|e| e.to_string())?;
    let mut parser = AeParser::new(tokens);
    match parser.parse() {
        Ok(ast) => { let spans = index_top_level(&ast, source); *CACHE.lock().unwrap() = Some(CachedParse { hash, ast: ast.clone(), source: source.to_string(), top_spans: spans }); Ok(ast) },
        Err(ParserError { message, line, column }) => Err(format!("{message} at {line}:{column}"))
    }
}

/// Attempt simplified partial parse: if dirty region lies strictly between pre-indexed top-level nodes, we reuse AST.
#[allow(dead_code)]
pub fn parse_or_partial(source: &str) -> Result<(ASTNode,bool), String> {
    let cache_opt = CACHE.lock().unwrap().clone();
    if cache_opt.is_none() { return parse_or_cached(source).map(|a|(a,false)); }
    let prev = cache_opt.unwrap();
    let dirty = compute_dirty_info(source);
    if !dirty.changed { return Ok((prev.ast.clone(), false)); }
    // Count overlapping nodes
    let mut overlap_indices: Vec<usize> = Vec::new();
    for (i, sp) in prev.top_spans.iter().enumerate() { if overlaps(dirty.first_changed_line, dirty.last_changed_line, sp.start_line, sp.end_line) { overlap_indices.push(i); } }
    if !overlap_indices.is_empty() && overlap_indices.windows(2).all(|w| w[1]==w[0]+1) && overlap_indices.len() <= 8 {
        // Reparse only the slice covering that node span lines; simplistic: extract source subset by lines and parse as program fragment
        let first = overlap_indices[0]; let last = *overlap_indices.last().unwrap();
        let sp = &prev.top_spans[first];
        let end_span = &prev.top_spans[last];
        let lines: Vec<&str> = source.lines().collect();
        let fragment_src = lines[(sp.start_line.saturating_sub(1))..end_span.end_line.min(lines.len())].join("\n");
        let mut lexer = Lexer::from_str(&fragment_src);
        if let Ok(tokens) = lexer.tokenize() {
            let mut parser = AeParser::new(tokens);
            if let Ok(new_ast) = parser.parse() {
                // Expect Program root; splice children matched by position count 1: we take its children as replacement if exactly one top node else fallback
                if let ASTNode::Program(new_items) = new_ast.clone() {
                    if let ASTNode::Program(mut old_items) = prev.ast.clone() {
                        // Only proceed if counts match target replacement length
                        if new_items.len() == overlap_indices.len() {
                            for (offset, idx) in overlap_indices.iter().enumerate() { if *idx < old_items.len() { old_items[*idx] = new_items[offset].clone(); } }
                            let updated = ASTNode::Program(old_items); let spans = index_top_level(&updated, source); *CACHE.lock().unwrap() = Some(CachedParse { hash: String::new(), ast: updated.clone(), source: source.to_string(), top_spans: spans }); *LAST_REPLACED_INDEX.lock().unwrap() = Some(first); return Ok((updated,true));
                        }
                    }
                }
            }
        }
        // Fallback full parse on failure
        return parse_or_cached(source).map(|a|(a,false));
    }
    if overlap_indices.is_empty() {
        // treat as insertion between nodes; full parse for correctness
        let ast = parse_or_cached(source)?; return Ok((ast,true));
    }
    // Multiple nodes affected -> full parse
    parse_or_cached(source).map(|a|(a,false))
}

fn overlaps(a1: usize, a2: usize, b1: usize, b2: usize) -> bool { !(a2 < b1 || b2 < a1) }

fn index_top_level(ast: &ASTNode, source: &str) -> Vec<NodeSpan> {
    let mut spans = Vec::new();
    let total = source.lines().count().max(1);
    if let ASTNode::Program(items) = ast {
        let mut starts = Vec::new();
        for node in items { starts.push(node_start_line(node)); }
        for (i, st) in starts.iter().enumerate() { let end = if i+1<starts.len(){ starts[i+1].saturating_sub(1) } else { total }; spans.push(NodeSpan { start_line: *st, end_line: end.max(*st) }); }
    }
    spans
}

fn node_start_line(node: &ASTNode) -> usize { match node { ASTNode::Function { line, .. } => *line, ASTNode::VariableDecl { line, .. } => *line, ASTNode::Assignment { line, .. } => *line, _ => 0 } }

fn compute_dirty_info(new_src: &str) -> DirtyInfo {
    let cache = CACHE.lock().unwrap();
    if let Some(prev) = cache.as_ref() {
        if prev.source == new_src { return DirtyInfo { changed: false, first_changed_line: 0, last_changed_line:0 }; }
        let old_lines: Vec<&str> = prev.source.lines().collect();
        let new_lines: Vec<&str> = new_src.lines().collect();
        let mut first = 0usize; let mut last_old = old_lines.len().saturating_sub(1); let mut last_new = new_lines.len().saturating_sub(1);
        while first < old_lines.len() && first < new_lines.len() && old_lines[first] == new_lines[first] { first += 1; }
        if first == old_lines.len() && first == new_lines.len() { return DirtyInfo { changed: false, first_changed_line:0, last_changed_line:0 }; }
        while last_old>first && last_new>first && old_lines[last_old]==new_lines[last_new] { last_old-=1; last_new-=1; }
        DirtyInfo { changed: true, first_changed_line: first+1, last_changed_line: (last_new+1).max(first+1) }
    } else {
        DirtyInfo { changed: true, first_changed_line: 1, last_changed_line: new_src.lines().count() }
    }
}

#[allow(dead_code)]
pub fn dirty_region(source: &str) -> DirtyInfo { compute_dirty_info(source) }

pub fn debug_snapshot() -> serde_json::Value { let fm = FUNCTION_METRICS.lock().unwrap().clone(); let sm = SAVINGS_METRICS.lock().unwrap().clone(); serde_json::json!({ "functions": fm.iter().map(|(i,m)| (i.to_string(), serde_json::json!({"runs": m.runs, "last_ns": m.last_ns, "ema_ns": m.ema_ns, "window": m.window, "last_run_epoch_ms": m.last_run_epoch_ms}))).collect::<serde_json::Value>(), "savings_history_len": sm.history.len(), "savings_recent_samples": sm.history.iter().map(|s| { serde_json::json!({"partial": s.partial_ns, "est_full": s.estimated_full_ns, "savings": s.savings_ns}) }).collect::<Vec<_>>() }) }
