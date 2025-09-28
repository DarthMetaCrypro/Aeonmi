# Incremental Parsing, Dependency Invalidation & Metrics

This document summarizes the architecture for Aeonmi's incremental compile pipeline, dependency tracking, and performance metrics.

## Overview

The system maintains a cached full AST plus per–top-level-node spans, semantic/type diagnostic caches, and multiple dependency graphs (call graph + variable access). Targeted re-parsing and selective re-inference minimize work after edits.

Key goals:
- Limit parsing to small dirty regions when possible (multi-node splice up to 8 contiguous items).
- Recompute types only for changed function + directly affected dependents (callers and variable readers), with optional deep propagation.
- Persist metrics & dependency stats for observability across sessions.

## Core Components

- CachedParse: stores last AST, hash (SHA1 of source), original source, and top-level line spans.
- DIAG_CACHE / TYPE_DIAG_CACHE: per-node diagnostic vectors.
- LAST_REPLACED_INDEX: index of last partially replaced node (for targeted type reinference).
- CallGraphMetrics: global counts (functions, edges, variable_edges, reinfer_events).
- VarDeps: maps variable -> sets of function indices (reads / writes).
- FUNCTION_METRICS: per-function inference timing (total_ns, runs, last_ns, avg_ns derived on query).
- SAVINGS_METRICS: cumulative time avoided by partial selective inference vs estimated full cost.
- DEEP_PROPAGATION flag: toggles transitive closure of caller expansion regardless of size limit.

## Partial Parsing Algorithm

1. Compute dirty region (first/last changed line) via line diff against cached source.
2. Identify overlapping top-level node spans.
3. If 1-8 contiguous nodes overlap, re-lex+parse only that slice; splice new nodes into cached Program.
4. Fallback to full parse if mismatch, non-contiguous, or parse error.

## Selective Type Reinference

When a partial parse replaces function at index R:
1. Re-infer that function in isolation (Program wrapper with single child) updating TYPE_DIAG_CACHE[R].
2. Build/refresh call graph + variable dependency maps.
3. Seed reinfer set with direct callers of changed function (reverse call edges).
4. Add functions that READ variables written by the changed function.
5. Optionally (if deepPropagation enabled or small set < 8) expand transitive callers (BFS over reverse edges).
6. Re-infer each selected function, timing each; update FUNCTION_METRICS and reinfer event counter.
7. Persist metrics (including savings if estimated full > partial).

## Variable Dependency Extraction

Walk each function body AST collecting:
- Writes: Assignment & VariableDecl names.
- Reads: Identifiers.
Nested constructs (blocks, if/while/for, expressions, calls) recursively traversed.

Helper `compute_var_deps(ast)` provided for testability.

## Metrics & Savings

Persisted file: `aeonmi_metrics.json` (versioned; METRICS_VERSION). Includes:
- metrics: function count, call edges, variable_edges, reinfer_events.
- varReads / varWrites: variable -> function index list.
- functionMetrics: per index timing aggregates.
- deepPropagation flag.
- savings: cumulative_savings_ns, cumulative_partial_ns, cumulative_estimated_full_ns.

Savings Calculation:
- After selective reinference, partial_elapsed = sum(last_ns) of re-inferred functions.
- Estimated full = sum(avg_ns or last_ns) across all functions.
- Accumulate difference if estimated_full >= partial.

## Reset Semantics

- metrics_reset: clears only reinfer_events counter (session reset).
- metrics_reset_full: clears call graph metrics, var deps, function timings, savings.

## API Key Storage Security

Keys encrypted with ChaCha20 using host+user derived 32-byte key (SHA256). Stored entries:
```json
{"v":1,"alg":"ChaCha20","nonce_ct_b64":"..."}
```
Backward compatibility: legacy plain base64 still accepted on read.

## UI Integration

Metrics panel (web UI) displays JSON snapshot, offers:
- Session Reset
- Full Reset (with confirmation)
- Deep Propagation toggle

Periodic refresh polls backend command `aeonmi_metrics`.

## Testing Coverage

Added tests:
- metrics_savings.rs: savings accumulation & negative guard
- metrics_function.rs: function timing + reinfer count + persistence round-trip
- var_deps.rs: variable dependency read/write extraction

## Future Improvements

1. More precise full-cost estimation (e.g. sum of per-function last_ns rather than avg for new functions).
2. Granular invalidation for non-function top-level items.
3. Batched persistence debounce to reduce disk writes.
4. Threaded background inference for large transitive updates.
5. Encryption key version migrations & optional KDF hardening (argon2) behind feature flag.

## Extension Points

- Add new dependency dimension: e.g., type alias usage, struct field accesses.
- Hook additional metrics by extending persisted JSON (increment METRICS_VERSION).
- Provide CLI subcommand to dump metrics file.

---
Document version 1.0
