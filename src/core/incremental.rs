//! Incremental parsing manager (initial scaffold).
//! Currently caches last full parse; future work will diff edited regions.

use crate::core::lexer::Lexer;
use crate::core::parser::{Parser as AeParser, ParserError};
use crate::core::ast::ASTNode;
use std::sync::Mutex;
use once_cell::sync::Lazy;
use sha1::{Sha1, Digest};
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Debug, Clone)]
pub struct NodeSpan { pub start_line: usize, pub end_line: usize }

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
pub struct TopLevelDiagCache { pub per_node: Vec<Vec<crate::core::semantic_analyzer::SemanticDiagnostic>> }
pub static DIAG_CACHE: Lazy<Mutex<TopLevelDiagCache>> = Lazy::new(|| Mutex::new(TopLevelDiagCache::default()));
pub static LAST_REPLACED_INDEX: Lazy<Mutex<Option<usize>>> = Lazy::new(|| Mutex::new(None));
#[derive(Debug, Clone, Default)]
pub struct TopLevelTypeDiagCache { pub per_node: Vec<Vec<crate::core::types::TypeDiagnostic>> }
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
#[derive(Debug, Default, Clone)]
pub struct FunctionInferenceMetric { pub total_ns: u128, pub runs: u64, pub last_ns: u128 }
pub static FUNCTION_METRICS: Lazy<Mutex<HashMap<usize, FunctionInferenceMetric>>> = Lazy::new(|| Mutex::new(HashMap::new()));

// Partial vs estimated full inference savings metrics
#[derive(Debug, Default, Clone)]
pub struct SavingsMetrics { pub cumulative_savings_ns: u128, pub cumulative_partial_ns: u128, pub cumulative_estimated_full_ns: u128 }
pub static SAVINGS_METRICS: Lazy<Mutex<SavingsMetrics>> = Lazy::new(|| Mutex::new(SavingsMetrics::default()));

pub fn record_function_infer(idx: usize, dur: u128) {
    if let Ok(mut m) = FUNCTION_METRICS.lock() {
        let entry = m.entry(idx).or_insert_with(FunctionInferenceMetric::default);
        entry.total_ns += dur; entry.runs += 1; entry.last_ns = dur;
    }
}

pub fn set_deep_propagation(v: bool) { DEEP_PROPAGATION.store(v, Ordering::Relaxed); }
pub fn get_deep_propagation() -> bool { DEEP_PROPAGATION.load(Ordering::Relaxed) }

const METRICS_FILE: &str = "aeonmi_metrics.json";
const METRICS_VERSION: u32 = 3; // bumped for savings metrics addition

pub fn persist_metrics() {
    if let (Ok(m), Ok(v), Ok(fm), Ok(sm)) = (CALL_GRAPH_METRICS.lock(), VAR_DEPS.lock(), FUNCTION_METRICS.lock(), SAVINGS_METRICS.lock()) {
        let function_metrics: HashMap<String, serde_json::Value> = fm.iter().map(|(idx, fm)| (
            idx.to_string(),
            serde_json::json!({
                "runs": fm.runs,
                "total_ns": fm.total_ns,
                "last_ns": fm.last_ns,
                "avg_ns": if fm.runs>0 { fm.total_ns / fm.runs as u128 } else { 0 }
            })
        )).collect();
        let json = serde_json::json!({
            "version": METRICS_VERSION,
            "metrics": {"functions": m.functions, "edges": m.edges, "reinfer_events": m.reinfer_events, "variable_edges": m.variable_edges},
            "varReads": v.reads.iter().map(|(k, set)| (k, set.iter().collect::<Vec<_>>())).collect::<std::collections::HashMap<_,_>>(),
            "varWrites": v.writes.iter().map(|(k, set)| (k, set.iter().collect::<Vec<_>>())).collect::<std::collections::HashMap<_,_>>(),
            "functionMetrics": function_metrics,
            "deepPropagation": get_deep_propagation(),
            "savings": {"cumulative_savings_ns": sm.cumulative_savings_ns, "cumulative_partial_ns": sm.cumulative_partial_ns, "cumulative_estimated_full_ns": sm.cumulative_estimated_full_ns}
        });
        let _ = std::fs::write(METRICS_FILE, serde_json::to_string_pretty(&json).unwrap_or_default());
    }
}

pub fn load_metrics() {
    if let Ok(data) = std::fs::read_to_string(METRICS_FILE) {
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&data) {
            if let Some(mo) = val.get("metrics") { if let Ok(mut m) = CALL_GRAPH_METRICS.lock() {
                m.functions = mo.get("functions").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                m.edges = mo.get("edges").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                m.reinfer_events = mo.get("reinfer_events").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                m.variable_edges = mo.get("variable_edges").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
            }}
            if let Some(fr) = val.get("varReads") { if let Ok(mut vd)=VAR_DEPS.lock() { if let Some(obj)=fr.as_object() { for (k, arr) in obj { let mut set=HashSet::new(); if let Some(a)=arr.as_array() { for v in a { if let Some(s)=v.as_str() { set.insert(s.to_string()); } } } vd.reads.insert(k.clone(), set); } } } }
            if let Some(fw) = val.get("varWrites") { if let Ok(mut vd)=VAR_DEPS.lock() { if let Some(obj)=fw.as_object() { for (k, arr) in obj { let mut set=HashSet::new(); if let Some(a)=arr.as_array() { for v in a { if let Some(s)=v.as_str() { set.insert(s.to_string()); } } } vd.writes.insert(k.clone(), set); } } } }
            if let Some(fm) = val.get("functionMetrics") { if let Ok(mut map)=FUNCTION_METRICS.lock() { if let Some(obj)=fm.as_object() { for (k,v) in obj { if let Ok(idx)=k.parse::<usize>() { let mut metric=FunctionInferenceMetric::default(); metric.runs=v.get("runs").and_then(|x| x.as_u64()).unwrap_or(0); metric.total_ns=v.get("total_ns").and_then(|x| x.as_u64()).unwrap_or(0) as u128; metric.last_ns=v.get("last_ns").and_then(|x| x.as_u64()).unwrap_or(0) as u128; map.insert(idx, metric); } } } } }
            if let Some(dp)=val.get("deepPropagation") { if let Some(b)=dp.as_bool() { set_deep_propagation(b); } }
            if let Some(sv)=val.get("savings") { if let Ok(mut sm)=SAVINGS_METRICS.lock() { sm.cumulative_savings_ns = sv.get("cumulative_savings_ns").and_then(|x| x.as_u64()).unwrap_or(0) as u128; sm.cumulative_partial_ns = sv.get("cumulative_partial_ns").and_then(|x| x.as_u64()).unwrap_or(0) as u128; sm.cumulative_estimated_full_ns = sv.get("cumulative_estimated_full_ns").and_then(|x| x.as_u64()).unwrap_or(0) as u128; } }
        }
    }
}

pub fn reset_metrics_session() { if let Ok(mut m)=CALL_GRAPH_METRICS.lock() { m.reinfer_events = 0; } }

pub fn reset_metrics_full() {
    if let Ok(mut cg)=CALL_GRAPH_METRICS.lock() { *cg = CallGraphMetrics::default(); }
    if let Ok(mut vd)=VAR_DEPS.lock() { *vd = VarDeps::default(); }
    if let Ok(mut fm)=FUNCTION_METRICS.lock() { fm.clear(); }
    if let Ok(mut sm)=SAVINGS_METRICS.lock() { *sm = SavingsMetrics::default(); }
    persist_metrics();
}

pub fn record_reinfer_event(count: usize) {
    if let Ok(mut m) = CALL_GRAPH_METRICS.lock() { m.reinfer_events += count; }
}

pub fn snapshot_call_graph_metrics() -> CallGraphMetrics { CALL_GRAPH_METRICS.lock().unwrap().clone() }

// Compute variable dependency map (reads/writes) for all top-level functions in an AST.
// This mirrors logic used in the GUI command for selective reinference.
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
                if let ASTNode::Program(mut new_items) = new_ast.clone() {
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
