use std::path::PathBuf;
use aeonmi_project::commands::compile::compile_pipeline;
use aeonmi_project::cli::EmitKind;
use aeonmi_project::core::lexer::{Lexer, LexerError};
use aeonmi_project::core::parser::{Parser as AeParser, ParserError};
use aeonmi_project::core::semantic_analyzer::{SemanticAnalyzer, Severity};
use aeonmi_project::core::symbols::{collect_symbols};
use aeonmi_project::core::code_actions::suggest_actions;
use aeonmi_project::core::types::TypeContext;
use aeonmi_project::core::incremental::{parse_or_cached, parse_or_partial, DIAG_CACHE, LAST_REPLACED_INDEX, TYPE_DIAG_CACHE, CALL_GRAPH_METRICS, VAR_DEPS, record_reinfer_event, persist_metrics, record_function_infer, get_deep_propagation};
use aeonmi_project::core::quantum_extract::{extract_circuit, circuit_to_json, circuit_to_pseudo_qasm};
use aeonmi_project::core::ast::ASTNode;
use aeonmi_project::core::incremental::{snapshot_call_graph_metrics, VAR_DEPS, FUNCTION_METRICS, get_deep_propagation, SAVINGS_METRICS};

#[tauri::command]
pub fn aeonmi_compile_ai(input: String, out: Option<String>) -> Result<String, String> {
    let output = out.unwrap_or_else(|| "output.ai".into());
    compile_pipeline(
        Some(PathBuf::from(&input)),
        EmitKind::Ai,
        PathBuf::from(&output),
        false,
        false,
        false,
        false,
        false,
    ).map_err(|e| e.to_string())?;
    Ok(output)
}

#[tauri::command]
pub fn aeonmi_run_native(input: String) -> Result<String, String> {
    std::env::set_var("AEONMI_NATIVE", "1");
    aeonmi_project::commands::run::main_with_opts(PathBuf::from(&input), None, false, false)
        .map_err(|e| e.to_string())?;
    Ok("ok".into())
}

#[tauri::command]
pub fn aeonmi_diagnostics(source: String) -> Result<serde_json::Value, String> {
    #[derive(serde::Serialize)]
    struct Diag { message: String, line: usize, column: usize, endLine: usize, endColumn: usize, severity: String }
    let mut lexer = Lexer::from_str(&source);
    let tokens = match lexer.tokenize() { Ok(t)=>t, Err(e)=> {
            let (line, col) = match e {
                LexerError::UnexpectedCharacter(_, l, c)
                | LexerError::UnterminatedString(l, c)
                | LexerError::InvalidNumber(_, l, c)
                | LexerError::InvalidQubitLiteral(_, l, c)
                | LexerError::UnterminatedComment(l, c) => (l, c),
                _ => (0,0)
            }; return Ok(serde_json::json!({"diagnostics": [Diag{ message: e.to_string(), line, column: col, endLine: line, endColumn: col+1, severity: "error".into() }]})); }};
    // Incremental: attempt partial reparse; fallback to cached/full parse
    let ast_opt: Option<(ASTNode,bool)> = match parse_or_partial(&source) { Ok(t)=>Some(t), Err(_)=>None };
    let mut diags: Vec<Diag> = Vec::new();
    if let Some((ast, partial)) = ast_opt {
            if partial {
                // Only re-analyze replaced node if available
                let replaced = *LAST_REPLACED_INDEX.lock().unwrap();
                if let (Some(r), ASTNode::Program(items)) = (replaced, &ast) {
                    // Ensure diag cache sized
                    {
                        let mut cache = DIAG_CACHE.lock().unwrap();
                        if cache.per_node.len() != items.len() { cache.per_node = vec![Vec::new(); items.len()]; }
                        // Recompute diagnostics for node r only
                        let mut sema = SemanticAnalyzer::new();
                        let node_diags = sema.analyze_with_spans(&items[r]);
                        cache.per_node[r] = node_diags;
                        // Merge all cached diags
                        for vecd in &cache.per_node { for d in vecd { diags.push(Diag { message: d.message.clone(), line: d.line, column: d.column, endLine: d.line, endColumn: d.column + d.len, severity: (if d.severity == Severity::Warning {"warning"} else {"error"}).into() }); } }
                    }
                } else {
                    // Fallback full analysis
                    let mut sema = SemanticAnalyzer::new();
                    let sema_diags = sema.analyze_with_spans(&ast);
                    let mut cache = DIAG_CACHE.lock().unwrap();
                    cache.per_node = if let ASTNode::Program(items) = &ast { items.iter().map(|_| Vec::new()).collect() } else { Vec::new() };
                    for d in sema_diags { diags.push(Diag { message: d.message, line: d.line, column: d.column, endLine: d.line, endColumn: d.column + d.len, severity: (if d.severity == Severity::Warning {"warning"} else {"error"}).into() }); }
                }
            } else {
                // Full analysis (cache rebuild)
                let mut sema = SemanticAnalyzer::new();
                let sema_diags = sema.analyze_with_spans(&ast);
                let mut cache = DIAG_CACHE.lock().unwrap();
                cache.per_node = if let ASTNode::Program(items) = &ast { items.iter().map(|_| Vec::new()).collect() } else { Vec::new() };
                for d in sema_diags { diags.push(Diag { message: d.message, line: d.line, column: d.column, endLine: d.line, endColumn: d.column + d.len, severity: (if d.severity == Severity::Warning {"warning"} else {"error"}).into() }); }
            }
    } else {
        // Fallback full parse path for error reporting
        let mut parser = AeParser::new(tokens.clone());
        match parser.parse() {
            Ok(ast) => { let mut sema = SemanticAnalyzer::new(); for d in sema.analyze_with_spans(&ast) { diags.push(Diag { message: d.message, line: d.line, column: d.column, endLine: d.line, endColumn: d.column + d.len, severity: (if d.severity == Severity::Warning {"warning"} else {"error"}).into() }); } }
            Err(ParserError { message, line, column }) => { diags.push(Diag { message: format!("Parsing error: {message}"), line, column, endLine: line, endColumn: column+1, severity: "error".into() }); }
        }
    }
    Ok(serde_json::json!({"diagnostics": diags}))
}

#[tauri::command]
pub fn aeonmi_ai(provider: Option<String>, prompt: String) -> Result<String, String> {
    let prov = provider.unwrap_or_else(|| "default".into());
    let enabled = match prov.as_str() {
        "openai" => std::env::var("OPENAI_API_KEY").is_ok(),
        "perplexity" => std::env::var("PERPLEXITY_API_KEY").is_ok(),
        "deepseek" => std::env::var("DEEPSEEK_API_KEY").is_ok(),
        "copilot" => std::env::var("GITHUB_COPILOT_TOKEN").is_ok(),
        _ => false,
    };
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    prompt.hash(&mut hasher); prov.hash(&mut hasher);
    let seed = hasher.finish();
    let excerpt: String = prompt.chars().take(48).collect();
    let faux = format!("{} response (seed {}): {}", prov, seed % 10000, excerpt);
    Ok(serde_json::json!({
        "provider": prov,
        "enabled": enabled,
        "inputTokensApprox": prompt.split_whitespace().count(),
        "output": faux
    }).to_string())
}

#[tauri::command]
pub fn aeonmi_quantum_run(source: String) -> Result<String, String> {
    let mut lexer = Lexer::from_str(&source);
    let mut superpose = 0; let mut entangle = 0; let mut measure = 0; let mut dod = 0;
    if let Ok(tokens) = lexer.tokenize() {
        use aeonmi_project::core::token::TokenKind::*;
        for t in tokens.iter() { match t.kind { Superpose => superpose+=1, Entangle => entangle+=1, Measure => measure+=1, Dod => dod+=1, _=>{} } }
    }
    let mut hist = serde_json::Map::new();
    if measure > 0 { hist.insert("00".into(), serde_json::json!(512)); hist.insert("01".into(), serde_json::json!(256)); hist.insert("10".into(), serde_json::json!(128)); hist.insert("11".into(), serde_json::json!(128)); }
    Ok(serde_json::json!({
        "bytes": source.len(),
        "ops": {"superpose": superpose, "entangle": entangle, "measure": measure, "dod": dod},
        "histogram": hist,
    }).to_string())
}

#[tauri::command]
pub fn aeonmi_version() -> Result<String, String> {
    Ok(env!("CARGO_PKG_VERSION").to_string())
}

#[tauri::command]
pub fn aeonmi_symbols(source: String) -> Result<String, String> {
    let mut lexer = Lexer::from_str(&source);
    let tokens = lexer.tokenize().map_err(|e| e.to_string())?;
    let mut parser = AeParser::new(tokens);
    let ast = parser.parse().map_err(|e| e.to_string())?;
    let symbols = collect_symbols(&ast);
    Ok(serde_json::to_string(&symbols).unwrap())
}

#[tauri::command]
pub fn aeonmi_code_actions(source: String) -> Result<String, String> {
    let mut lexer = Lexer::from_str(&source);
    let tokens = lexer.tokenize().map_err(|e| e.to_string())?;
    let mut parser = AeParser::new(tokens);
    let ast = parser.parse().map_err(|e| e.to_string())?;
    let actions = suggest_actions(&ast);
    Ok(serde_json::to_string(&actions).unwrap())
}

#[tauri::command]
pub fn aeonmi_types(source: String) -> Result<String, String> {
    let (ast, partial) = parse_or_partial(&source).map_err(|e| e)?;
    if partial {
        let replaced = *LAST_REPLACED_INDEX.lock().unwrap();
        if let (Some(r), ASTNode::Program(items)) = (replaced, &ast) {
            let mut cache = TYPE_DIAG_CACHE.lock().unwrap();
            if cache.per_node.len() != items.len() { cache.per_node = vec![Vec::new(); items.len()]; }
            // Recompute only that node with a fresh TypeContext in a Program wrapper (to keep structure)
            let mut ctx = TypeContext::new();
            ctx.infer_program(&ASTNode::Program(vec![items[r].clone()]));
            cache.per_node[r] = ctx.diags.clone();
            // If changed node is a function, re-infer dependents that call it (shallow scan)
            if let ASTNode::Function { name: changed_name, .. } = &items[r] {
                // Build call graph (function index -> called function indices)
                let mut name_by_index: Vec<Option<String>> = Vec::with_capacity(items.len());
                for node in items.iter() { if let ASTNode::Function { name, .. } = node { name_by_index.push(Some(name.clone())); } else { name_by_index.push(None); } }
                let mut index_by_name: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
                for (idx, opt) in name_by_index.iter().enumerate() { if let Some(n)=opt { index_by_name.insert(n.clone(), idx); } }
                let mut calls: Vec<Vec<usize>> = vec![Vec::new(); items.len()];
                for (idx, node) in items.iter().enumerate() {
                    if let ASTNode::Function { body, .. } = node {
                        collect_calls(body, &index_by_name, &mut calls[idx]);
                    }
                }
                // Variable dependency collection (reads/writes) per function
                {
                    use aeonmi_project::core::ast::ASTNode as N;
                    let mut var_reads: std::collections::HashMap<String, std::collections::HashSet<usize>> = std::collections::HashMap::new();
                    let mut var_writes: std::collections::HashMap<String, std::collections::HashSet<usize>> = std::collections::HashMap::new();
                    fn walk(idx: usize, n: &N, reads: &mut std::collections::HashMap<String, std::collections::HashSet<usize>>, writes: &mut std::collections::HashMap<String, std::collections::HashSet<usize>>) {
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
                    for (idx, node) in items.iter().enumerate() { if let N::Function { body, .. } = node { for c in body { walk(idx, c, &mut var_reads, &mut var_writes); } } }
                    let mut vd = VAR_DEPS.lock().unwrap(); vd.reads.clear(); vd.writes.clear();
                    for (k,v) in var_reads { vd.reads.insert(k, v); }
                    for (k,v) in var_writes { vd.writes.insert(k, v); }
                }
                // Reverse edges for call-based deps
                let mut rev: Vec<Vec<usize>> = vec![Vec::new(); items.len()];
                for (i, outs) in calls.iter().enumerate() { for &t in outs { rev[t].push(i); } }
                let mut to_reinfer: std::collections::HashSet<usize> = std::collections::HashSet::new();
                if let Some(changed_idx) = index_by_name.get(changed_name) {
                    // Seed with direct callers only (not full transitive yet)
                    for &caller in &rev[*changed_idx] { to_reinfer.insert(caller); }
                }
                // Variable-level invalidation: functions that read vars written by changed function
                let vd = VAR_DEPS.lock().unwrap().clone();
                // Identify variables written by changed function
                let mut written: std::collections::HashSet<String> = std::collections::HashSet::new();
                for (var, writers) in vd.writes.iter() { if writers.contains(&r) { written.insert(var.clone()); } }
                // Add any function that reads those variables
                for (var, readers) in vd.reads.iter() { if written.contains(var) { for f in readers { if *f != r { to_reinfer.insert(*f); } } } }
                // (Optional) escalate to transitive callers only if set small (<8) for performance
                if get_deep_propagation() || to_reinfer.len() < 8 {
                    let mut queue: std::collections::VecDeque<usize> = to_reinfer.iter().cloned().collect();
                    while let Some(cur) = queue.pop_front() { for &caller in &rev[cur] { if to_reinfer.insert(caller) { queue.push_back(caller); } } }
                }
                let mut reinfer_count = 0usize;
                let mut partial_elapsed_ns: u128 = 0;
                for idx in to_reinfer { if let ASTNode::Function { .. } = &items[idx] { let start = std::time::Instant::now(); let mut dep_ctx = TypeContext::new(); dep_ctx.infer_program(&ASTNode::Program(vec![items[idx].clone()])); let dur = start.elapsed().as_nanos(); record_function_infer(idx, dur); cache.per_node[idx] = dep_ctx.diags.clone(); reinfer_count+=1; } }
                if reinfer_count>0 { record_reinfer_event(reinfer_count); }
                // After recording function metrics, compute partial elapsed (sum of last_ns for affected) and estimated full cost (sum of avg for all functions) then record savings
                {
                    let fm = FUNCTION_METRICS.lock().unwrap();
                    // Partial: sum last_ns for indices we just re-inferred
                    partial_elapsed_ns = fm.iter().filter(|(idx, _)| cache.per_node.get(**idx).is_some()).map(|(_, m)| m.last_ns).sum();
                    let total_est_full: u128 = fm.values().map(|m| if m.runs>0 { m.total_ns / m.runs as u128 } else { m.last_ns }).sum();
                    if partial_elapsed_ns>0 && total_est_full>0 { aeonmi_project::core::incremental::record_partial_savings(partial_elapsed_ns, total_est_full); }
                }
                // Update metrics (functions, edges, variable edges)
                {
                    let mut metrics = CALL_GRAPH_METRICS.lock().unwrap();
                    metrics.functions = name_by_index.iter().filter(|o| o.is_some()).count();
                    metrics.edges = calls.iter().map(|v| v.len()).sum();
                    // variable edges: total function reads + writes unique pairs
                    let vd = VAR_DEPS.lock().unwrap();
                    metrics.variable_edges = vd.reads.values().map(|s| s.len()).sum::<usize>() + vd.writes.values().map(|s| s.len()).sum::<usize>();
                }
                persist_metrics();
            }
            let merged: Vec<_> = cache.per_node.iter().flat_map(|v| v.clone()).collect();
            return Ok(serde_json::to_string(&merged).unwrap());
        }
    }
    let mut ctx = TypeContext::new(); ctx.infer_program(&ast);
    // Rebuild full cache
    if let ASTNode::Program(items) = &ast {
        let mut cache = TYPE_DIAG_CACHE.lock().unwrap(); cache.per_node = vec![Vec::new(); items.len()];
        // Rebuild call graph & var deps fully
        let mut name_by_index: Vec<Option<String>> = Vec::with_capacity(items.len());
        for node in items.iter() { if let ASTNode::Function { name, .. } = node { name_by_index.push(Some(name.clone())); } else { name_by_index.push(None); } }
        let mut index_by_name: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
        for (idx, opt) in name_by_index.iter().enumerate() { if let Some(n)=opt { index_by_name.insert(n.clone(), idx); } }
        let mut calls: Vec<Vec<usize>> = vec![Vec::new(); items.len()];
        for (idx, node) in items.iter().enumerate() { if let ASTNode::Function { body, .. } = node { collect_calls(body, &index_by_name, &mut calls[idx]); } }
        // variable deps optimized walk
        use aeonmi_project::core::ast::ASTNode as N;
        let mut var_reads: std::collections::HashMap<String, std::collections::HashSet<usize>> = std::collections::HashMap::new();
        let mut var_writes: std::collections::HashMap<String, std::collections::HashSet<usize>> = std::collections::HashMap::new();
        fn walk(idx: usize, n: &N, reads: &mut std::collections::HashMap<String, std::collections::HashSet<usize>>, writes: &mut std::collections::HashMap<String, std::collections::HashSet<usize>>) {
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
        for (idx, node) in items.iter().enumerate() { if let N::Function { body, .. } = node { for c in body { walk(idx, c, &mut var_reads, &mut var_writes); } } }
        {
            let mut vd = VAR_DEPS.lock().unwrap(); vd.reads.clear(); vd.writes.clear();
            for (k,v) in var_reads { vd.reads.insert(k, v); }
            for (k,v) in var_writes { vd.writes.insert(k, v); }
        }
        {
            let mut metrics = CALL_GRAPH_METRICS.lock().unwrap();
            metrics.functions = name_by_index.iter().filter(|o| o.is_some()).count();
            metrics.edges = calls.iter().map(|v| v.len()).sum();
            let vd = VAR_DEPS.lock().unwrap();
            metrics.variable_edges = vd.reads.values().map(|s| s.len()).sum::<usize>() + vd.writes.values().map(|s| s.len()).sum::<usize>();
        }
        persist_metrics();
    }
    Ok(serde_json::to_string(&ctx.diags).unwrap())
}

#[tauri::command]
pub fn aeonmi_quantum_circuit(source: String) -> Result<String, String> {
    let ast = parse_or_cached(&source).map_err(|e| e)?;
    let circ = extract_circuit(&ast);
    Ok(serde_json::to_string(&circ).unwrap())
}

#[tauri::command]
pub fn aeonmi_quantum_circuit_export(source: String) -> Result<String, String> {
    let ast = parse_or_cached(&source).map_err(|e| e)?;
    let circ = extract_circuit(&ast);
    let json = circuit_to_json(&circ);
    let qasm = circuit_to_pseudo_qasm(&circ);
    Ok(serde_json::json!({"json": json, "pseudo_qasm": qasm}).to_string())
}

#[tauri::command]
pub fn aeonmi_rename_symbol(source: String, line: usize, column: usize, new_name: String) -> Result<String, String> {
    use crate::core::scope_map::ScopeMap;
    let ast = parse_or_cached(&source).map_err(|e| e)?;
    let sm = ScopeMap::build(&ast);
    // Determine original name by scanning occurrences matching (line,column)
    let mut original: Option<String> = None;
    for (name, occs) in &sm.symbols { if occs.iter().any(|o| o.line==line && o.column==column) { original = Some(name.clone()); break; } }
    let Some(orig_name) = original else { return Err("symbol not found at position".into()); };
    if orig_name == new_name { return Ok(source); }
    let occs = sm.occurrences_in_same_scope(&orig_name, line, column);
    if occs.is_empty() { return Err("no occurrences in scope".into()); }
    // Replace occurrences carefully by line/column editing
    let mut lines: Vec<String> = source.lines().map(|s| s.to_string()).collect();
    for (ol, oc, _is_def) in occs {
        if let Some(line_str) = lines.get_mut(ol.saturating_sub(1)) {
            // columns are 1-based; ensure slice safety
            if oc>0 && oc-1 < line_str.len() {
                // simple word boundary check
                // Find token starting at oc
                if line_str[oc-1..].starts_with(&orig_name) {
                    let before = &line_str[..oc-1];
                    let after = &line_str[oc-1+orig_name.len()..];
                    *line_str = format!("{before}{new_name}{after}");
                }
            }
        }
    }
    Ok(lines.join("\n"))
}

fn function_body_calls(body: &Vec<ASTNode>, target: &str) -> bool {
    use aeonmi_project::core::ast::ASTNode;
    fn scan(n: &ASTNode, target: &str, found: &mut bool) {
        if *found { return; }
        match n {
            ASTNode::Call { callee, .. } => {
                if matches!(&**callee, ASTNode::Identifier(name) if name==target) || matches!(&**callee, ASTNode::IdentifierSpanned { name, .. } if name==target) { *found = true; return; }
                scan(callee, target, found);
            }
            ASTNode::Program(items) | ASTNode::Block(items) => { for it in items { scan(it,target,found); if *found {return;} } }
            ASTNode::Function { body, .. } => { for it in body { scan(it,target,found); if *found {return;} } }
            ASTNode::If { condition, then_branch, else_branch } => { scan(condition,target,found); scan(then_branch,target,found); if let Some(e)=else_branch { scan(e,target,found); } }
            ASTNode::While { condition, body } => { scan(condition,target,found); scan(body,target,found); }
            ASTNode::For { init, condition, increment, body } => { if let Some(i)=init { scan(i,target,found); } if let Some(c)=condition { scan(c,target,found); } if let Some(inc)=increment { scan(inc,target,found); } scan(body,target,found); }
            ASTNode::Assignment { value, .. } | ASTNode::VariableDecl { value, .. } => scan(value,target,found),
            ASTNode::Return(e) | ASTNode::Log(e) | ASTNode::UnaryExpr { expr: e, .. } => scan(e,target,found),
            ASTNode::BinaryExpr { left, right, .. } => { scan(left,target,found); scan(right,target,found); },
            ASTNode::Call { .. } | ASTNode::Identifier(_) | ASTNode::IdentifierSpanned { .. } | ASTNode::NumberLiteral(_) | ASTNode::StringLiteral(_) | ASTNode::BooleanLiteral(_) | ASTNode::QuantumOp { .. } | ASTNode::HieroglyphicOp { .. } | ASTNode::Error(_) => {}
        }
    }
    for stmt in body { let mut f=false; scan(stmt, target, &mut f); if f { return true; } }
    false
}

fn collect_calls(body: &Vec<ASTNode>, index_by_name: &std::collections::HashMap<String,usize>, out: &mut Vec<usize>) {
    use aeonmi_project::core::ast::ASTNode;
    fn scan(n: &ASTNode, map: &std::collections::HashMap<String,usize>, out: &mut Vec<usize>) {
        match n {
            ASTNode::Call { callee, .. } => {
                if let ASTNode::Identifier(name) | ASTNode::IdentifierSpanned { name, .. } = &**callee { if let Some(i)=map.get(name) { if !out.contains(i) { out.push(*i); } } }
                scan(callee, map, out);
            }
            ASTNode::Program(items) | ASTNode::Block(items) => { for it in items { scan(it,map,out); } }
            ASTNode::Function { body, .. } => { for it in body { scan(it,map,out); } }
            ASTNode::If { condition, then_branch, else_branch } => { scan(condition,map,out); scan(then_branch,map,out); if let Some(e)=else_branch { scan(e,map,out); } }
            ASTNode::While { condition, body } => { scan(condition,map,out); scan(body,map,out); }
            ASTNode::For { init, condition, increment, body } => { if let Some(i)=init { scan(i,map,out); } if let Some(c)=condition { scan(c,map,out); } if let Some(inc)=increment { scan(inc,map,out); } scan(body,map,out); }
            ASTNode::Assignment { value, .. } | ASTNode::VariableDecl { value, .. } => scan(value,map,out),
            ASTNode::Return(e) | ASTNode::Log(e) | ASTNode::UnaryExpr { expr: e, .. } => scan(e,map,out),
            ASTNode::BinaryExpr { left, right, .. } => { scan(left,map,out); scan(right,map,out); }
            _ => {}
        }
    }
    for stmt in body { scan(stmt, index_by_name, out); }
}

#[tauri::command]
pub fn aeonmi_metrics() -> Result<String, String> {
    let m = snapshot_call_graph_metrics();
    let vd = VAR_DEPS.lock().unwrap();
    let reads: std::collections::HashMap<String, usize> = vd.reads.iter().map(|(k,v)| (k.clone(), v.len())).collect();
    let writes: std::collections::HashMap<String, usize> = vd.writes.iter().map(|(k,v)| (k.clone(), v.len())).collect();
    Ok(serde_json::json!({
        "functions": m.functions,
        "callEdges": m.edges,
        "variableEdges": m.variable_edges,
        "reinferEvents": m.reinfer_events,
        "varReads": reads,
        "varWrites": writes,
    "deepPropagation": get_deep_propagation(),
    "functionInference": FUNCTION_METRICS.lock().unwrap().iter().map(|(idx, fm)| (idx.to_string(), serde_json::json!({"runs": fm.runs, "total_ns": fm.total_ns, "last_ns": fm.last_ns, "avg_ns": if fm.runs>0 { fm.total_ns / fm.runs as u128 } else { 0 }}))).collect::<serde_json::Value>(),
    "savings": { let sm = SAVINGS_METRICS.lock().unwrap(); serde_json::json!({"cumulative_savings_ns": sm.cumulative_savings_ns, "cumulative_partial_ns": sm.cumulative_partial_ns, "cumulative_estimated_full_ns": sm.cumulative_estimated_full_ns}) },
    }).to_string())
}