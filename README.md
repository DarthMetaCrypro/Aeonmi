<p align="center">
  <img src="https://img.shields.io/github/actions/workflow/status/DarthMetaCrypro/Aeonmi/release.yml?label=build" />
  <img src="https://img.shields.io/github/v/release/DarthMetaCrypro/Aeonmi?include_prereleases&sort=semver" />
  <img src="https://img.shields.io/badge/license-Proprietary-red" />
  <img src="https://img.shields.io/badge/language-Rust-informational" />
</p>

# Aeonmi v0.2.0 – Closed Source Pre-Release.

> **Notice**
> Aeonmi is a closed-source project. No redistribution, modification, reverse engineering, or unauthorized use is permitted without explicit written consent from the author. All rights reserved. This pre-release is provided for demonstration, evaluation, and controlled collaboration.

## Overview

Aeonmi is an advanced programming language and compiler framework designed for AI‑native, secure, and multi‑dimensional computing. It introduces **QUBE**, a symbolic/hieroglyphic inner‑core language aimed at adaptive, self‑modifying operations with quantum‑resistant security and deep AI integration.

> New: A comprehensive language + tooling manual is now available: see **[Aeonmi Language Guide](docs/Aeonmi_Language_Guide.md)** for a structured, beginner→expert path, patterns, limitations, roadmap hints, and quantum examples.

## Branch Status

| Role | Branch | Description | Guidance |
|------|--------|-------------|----------|
| Stable / Recommended | `main` | Most recent validated release-quality code; CI green; safe for downstream usage. | Default clone target. File issues against this unless they concern in‑flight features. |
| Active Development | `ci/tauri-bridge-matrix-cache` | Ongoing integration work (CI matrix, repo hygiene, LFS enablement, editor/search & exec enhancements) prior to merge. May rebased / force-pushed. | Pin only for testing new features; expect occasional instability. |

Pull requests should generally target `main` unless explicitly coordinating work slated for the active development branch. After merge of the development branch, this table will be updated or reduced back to a single stable line.

## What’s in v0.2.0

* **Core compiler pipeline**: lexer → parser → semantic analyzer → code generator.
* **Diagnostics** with rich, contextual error reporting.
* **QUBE integration layer** foundations (symbolic / glyph parsing).
* **Examples** showing control flow, functions, glyphs, and basics.
* **Strict proprietary licensing**.

## Binaries

This workspace builds two executables:

* **`Aeonmi`** – primary CLI (default run target)
* **`aeonmi_project`** – legacy/test binary kept for compatibility

> Tip: Use `cargo run` for the default target, or `cargo run --bin aeonmi_project` to run the legacy binary.

### Windows Executable (Aeonmi.exe)

To produce a standalone optimized Windows binary:

```powershell
git clone https://github.com/DarthMetaCrypro/Aeonmi.git
cd Aeonmi
powershell -ExecutionPolicy Bypass -File .\build_windows.ps1
```

Output will be at `target\release\Aeonmi.exe`.

Include optional features (example: quantum):

```powershell
powershell -ExecutionPolicy Bypass -File .\build_windows.ps1 -Features "quantum"
```

You can then copy `Aeonmi.exe` to a directory in your PATH. Run with:

```powershell
Aeonmi.exe --help
```

#### Custom Icon / Logo

The Windows build embeds `assets/icon.ico` via `build.rs` (uses the `winres` crate).

Update the icon:
1. Replace `assets/icon.ico` with a valid multi-resolution ICO (16,32,48,64,128,256 recommended).
2. Rebuild: `powershell -ExecutionPolicy Bypass -File .\build_windows.ps1`
3. If Explorer still shows the old icon, clear the Windows icon cache or rename the file once.

Optional: set additional metadata (FileDescription, ProductName) by editing `build.rs`.

## Install & Build

```bash
# 1) Clone
git clone https://github.com/DarthMetaCrypro/Aeonmi.git
cd Aeonmi

# 2) Build
cargo build --release

# 3) Run (help)
cargo run -- --help

# Legacy/test binary
cargo run --bin aeonmi_project -- --help

### Optional Features

| Feature | Purpose | Notes |
|---------|---------|-------|
| quantum | Enable quantum backend operations | Pulls in `nalgebra`, `num-complex`. |
| qiskit | Qiskit bridge (Python) | Requires Python environment; builds with `pyo3`, `numpy`. |
| kdf-argon2 | Stronger Argon2id-based key derivation for API key encryption | Fallback is SHA256(user||host||salt); enable with `--features kdf-argon2`. |
```

## CLI Usage (subject to change)

High‑level subcommands currently wired into the CLI:

```text
run <file.ai> [--out FILE] [--pretty-errors] [--no-sema]
# compile to JS and try executing with Node
  Bytecode / Optimization (feature: bytecode):
    --bytecode           Execute via internal bytecode VM instead of JS/native lowering
    --disasm             Print disassembly of compiled chunk (implies --bytecode)
  --opt-stats          Print human-readable optimization stats summary (implies --bytecode)
  --opt-stats-json     Emit optimization stats as pretty JSON (implies --bytecode; if built with feature debug-metrics merges into metrics JSON under compileOptStats)
  Environment:
    AEONMI_BYTECODE=1    Implicitly enable bytecode VM without passing --bytecode
    AEONMI_MAX_FRAMES=N  Set max call frame depth for bytecode recursion guard (default 256, clamped 4..65536)

tokens <file.ai>
# emit lexer tokens

ast <file.ai>
# emit parsed AST

edit [--tui] [FILE]
# open editor (TUI with --tui)

repl
# interactive REPL

format [--check] <inputs...>
# formatter (WIP)

* **New Features**:
  * `metrics-config --set-history-cap N` – adjust savings sample history (8–256). Reset restores to 32.
  * Optional: `--seed SEED` for reproducible jitter/distribution.
  * `metrics-export FILE.csv` – export current function metrics to CSV (always available; no debug-metrics feature required).
lint [--fix] <inputs...>
# linter (WIP)

cargo <args...>
# pass-through to system Cargo (e.g. `aeonmi cargo build --release`)

python <script.py> [args...]
# pass-through to system Python

node <file.js> [args...]
# pass-through to Node.js

exec <file.(ai|js|py|rs)> [args...]
# auto-detect by extension: .ai compiles then runs with node; .js via node; .py via python; .rs via rustc temp build
# Flags:
#   --watch       Re-run automatically on file change (poll 500ms)
#   --keep-temp   Preserve temporary outputs (__exec_tmp.js / __exec_tmp_rs.exe) for inspection
#   --no-run      (Internal/testing) Compile only; skip executing runtime (used when Node/Python absent)

native <file.ai> [--emit-ai FILE] [--watch]
# Run an .ai file directly on the Aeonmi native VM (equivalent to setting AEONMI_NATIVE=1 with run). Optional --emit-ai writes canonical form first.

metrics-dump
# Pretty-print the current aggregated metrics JSON (call graph, variable deps, function timings, savings).

metrics-flush
# Force an immediate metrics persistence (bypasses debounce) and echo the JSON.

metrics-path
# Print the absolute path to the persisted metrics file.
`metrics-top` – show hottest functions by recent (EMA) and lifetime average inference time. Sorts by `ema_ns` (recently expensive). Supports `--limit N` and `--json` (now includes `ema_ns`). Metrics schema v5 also exposes cumulative_savings_pct and cumulative_partial_pct derived from estimated full cost.
metrics-top [--limit N] [--json]
# Display top N slowest functions by average inference time (default 10). Use --json for machine-readable output.
key-rotate
# Re-encrypt all stored API keys with the current derivation (e.g., after enabling `--features kdf-argon2`). Shows per-provider results and preserves existing keys.

Ctrl-C / Shutdown Persistence
# Metrics are flushed on normal shutdown and also on Ctrl-C via a signal handler calling force_persist_metrics to reduce loss of recent timing samples.
```

## Interactive Shell (experimental)

An **Aeonmi Shard** interactive shell is available for quick file navigation and build actions (e.g., `compile`, `run`, `ls`, `cd`, `edit --tui`). Use the CLI help to discover the entrypoint and available commands.

### Quick Start (Shard)

```powershell
git clone https://github.com/DarthMetaCrypro/Aeonmi.git
cd Aeonmi
cargo run            # launches the Aeonmi Shard prompt
```

At the prompt:

```text
help                 # list commands
compile examples/hello.ai
run examples/hello.ai
edit --tui examples/hello.ai
exit
```

### Core Shell Commands

| Category  | Commands |
|-----------|----------|
| Navigation | `pwd`, `cd <dir>`, `ls [dir]` |
| Files | `cat <file>`, `mkdir <path>`, `rm <path>`, `mv <src> <dst>`, `cp <src> <dst>` |
| Build / Run | `compile <file.ai> [--emit js|ai] [--out FILE]`, `run <file.ai> [--out FILE]` |
| Editor | `edit [--tui] [FILE]` (opens TUI if `--tui`) |
| Quantum (feature gated) | `qsim`, `qstate`, `qgates`, `qexample` |
| Misc | `help`, `exit` |

If the quantum feature is not enabled, `qsim` / `qexample` will inform you how to build with the feature.

### TUI Editor

Launch via:

```powershell
cargo run -- edit --tui examples/hello.ai
```

Key bindings:

| Key | Action |
|-----|--------|
| Enter | Append current input line to buffer |
| Ctrl+S | Save file |
| F4 | Toggle emit target (JS / AI) |
| F5 | Compile (writes `output.js` or `output.ai`) |
| F6 | Compile then run (JS only) |
| F9 | Toggle mouse capture (free terminal selection when OFF) |
| F1 | Toggle key/mouse debug overlay in status line |
| Esc / Ctrl+Q | Quit (warns if unsaved) |
| Tab | Insert 4 spaces |
| Ctrl+F / Search button | Activate incremental search |
| Enter / n | Next match while searching |
| Shift+N | Previous match while searching |
| Esc (in search) | Cancel search (first Esc exits search, second Esc may quit) |

Improved search UX:
* Inline status shows `/query [current/total]`.
* Dynamic highlight of matches; counts update as you type.
* `n` / `Enter` move forward, `Shift+N` moves backward.
* First Esc exits search mode; second Esc handles quit logic.
* Last search query persists across sessions in `.aeonmi_last_search` (auto-loaded on next launch when pressing Ctrl+F).

The status line shows contextual results (save, compile success, errors, etc.).

### Quantum Feature (Optional)

To enable quantum commands:

```powershell
cargo run --features quantum
```

Available quantum shell commands (when built with the feature):

| Command | Purpose |
|---------|---------|
| `qsim <file.ai> [--shots N] [--backend titan|qiskit]` | Run quantum simulation (currently native `titan` backend; `qiskit` if compiled with that feature) |
| `qstate` | Show available quantum backends |
| `qgates` | List symbolic / glyph gate representations |
| `qexample list` | List bundled quantum examples |
| `qexample teleport|grover|error_correction|qube` | Run an example program |

Example:

```powershell
cargo run --features quantum
qexample list
qexample grover
qsim examples/grover_search.ai --shots 512 --backend titan
```

### Example Workflow

```powershell
cargo run
compile examples/hello.ai
cat output.js
run examples/hello.ai
edit --tui examples/hello.ai   # make changes, F5 to compile, F6 to run
```

### Fast Create Workflow

You can now create a starter file and optionally open/compile/run in one command:

```powershell
# Create a new file with template (does not open)
cargo run -- new --file demo.ai

# Create + open line editor
cargo run -- new --file demo.ai --open

# Create + open TUI editor
cargo run -- new --file demo.ai --open --tui

# Create + emit AI canonical form immediately (default emits AI now)
cargo run -- new --file demo.ai --compile

# Create + emit AI + also run (compiles JS second for execution)
cargo run -- new --file demo.ai --compile --run
```

Flags:
* `--open` open editor after creation (line mode unless `--tui`).
* `--tui` open the TUI editor (implies `--open`).
* `--compile` emits `output.ai` (AI canonical) by default now.
* `--run` compiles AI then JS and executes via Node (JS backend still required to run).

Rationale: AEONMI focuses on its native `.ai` form first; JS emission is still available for execution and interoperability.

### Troubleshooting

| Issue | Resolution |
|-------|------------|
| `qsim` says quantum not built | Re-run with `--features quantum` |
| Node not found when running JS | Install Node.js and ensure `node` is in PATH |
| `cargo` not found (Windows) | See Cargo PATH section below |
| Colors missing on Windows | Use Windows Terminal or VS Code integrated terminal |
#### Cargo PATH / Execution Issues (Windows)

If PowerShell cannot run `cargo` inside Aeonmi passthrough commands:

```powershell
where cargo              # should show something like C:\Users\<you>\.cargo\bin\cargo.exe
& "$env:USERPROFILE\.cargo\bin\cargo.exe" --version  # force direct execution
[Environment]::GetEnvironmentVariable('Path','User')    # ensure it contains ;%USERPROFILE%\.cargo\bin
```

If `where cargo` returns nothing, (re)install Rust via <https://rustup.rs/>. After installation, restart PowerShell or run:

```powershell
$env:Path += ";$env:USERPROFILE\.cargo\bin"
```

If double‑clicking `.rs` files launches the wrong program, it does not affect CLI usage, but you can reset file associations in Windows Settings > Default Apps.

Test passthrough:

```powershell
aeonmi cargo --version
aeonmi python --version
aeonmi node --version
```

If only `aeonmi cargo` fails while plain `cargo` works, confirm no alias/shim interference (`Get-Command cargo`).

### Exec Subcommand Details

`aeonmi exec` offers quick single-file execution across multiple ecosystems:

| Extension | Behavior |
|-----------|----------|
| `.ai` | Compiles to temporary `__exec_tmp.js` then runs with Node (removed unless `--keep-temp`) |
| `.js` | Direct Node execution (skip with `--no-run`) |
| `.py` | Python interpreter execution (skip with `--no-run`) |
| `.rs` | One-off `rustc -O` build to `__exec_tmp_rs(.exe)` then run (artifact removed unless `--keep-temp`) |

Flags:
* `--watch` — poll source and re-run automatically when timestamp changes.
* `--keep-temp` — retain generated artifacts for debugging.
* `--no-run` — compile only (hidden; primarily for CI/tests without Node/Python). You can also simulate via: `aeonmi exec file.ai --no-run`.

Watch loop can be limited to a single iteration for testing by setting environment variable:

```powershell
$env:AEONMI_WATCH_ONCE = "1"; aeonmi exec script.ai --watch --no-run
```

Artifacts cleanup: By default temporary files are deleted after successful execution. On failure they are left in place for inspection.

| Mouse selection blocked in TUI | Press F9 to disable mouse capture |
| Unsaved changes warning on exit | Press Ctrl+S then Esc again |

### Native Interpreter (Node-less Fallback / Opt-In)

Aeonmi now ships an initial native interpreter implementing a tree-walk VM over the lowered IR. By default execution of `.ai` still prefers the historical JS emission + Node.js runtime. The native path is used when either:

1. Node.js is not detected on PATH, or
2. You explicitly request native mode with the environment variable `AEONMI_NATIVE=1`.

Supported today: literals, variable declarations & assignment, arithmetic / comparison / logical operators, functions & calls, `if` / `while` / `for`, returns, and built-ins `print`, `log`, `time_ms`, `rand`. Quantum and hieroglyphic operations currently lower to placeholder function names (no physical simulation yet).

Opt-in examples:

```bash
AEONMI_NATIVE=1 aeonmi run examples/hello.ai
AEONMI_NATIVE=1 aeonmi exec examples/hello.ai
```

PowerShell:

```powershell
$env:AEONMI_NATIVE = "1"; aeonmi run examples/hello.ai
```

When native mode executes with `--no-sema` you'll see a note mirroring the JS path. Errors are reported with the same pretty diagnostics when `--pretty-errors` is enabled.

---

## Aeonmi Language Guide (Beginner ➜ Intermediate ➜ Advanced)

> Status: This reflects the CURRENT implemented subset in the native VM path (tree‑walk interpreter) and JS lowering. Some features mentioned as “Planned” are not yet parsed (attempting them now will raise a lexing or parsing error). This guide focuses on what you can reliably use today plus patterns to work around missing constructs.

### 1. First Program

```ai
log("Hello, Aeonmi!");
```

Run (native):
```powershell
Aeonmi.exe run --native hello.ai
```

### 2. Lexical Basics
| Item | Supported Now | Notes |
|------|---------------|-------|
| Whitespace | Yes | Spaces, tabs, newlines separate tokens. |
| Comments | Yes | Line comments start with `#` (put comment on its own line). |
| Identifiers | Yes | Start with letter or `_`, then letters / digits / `_`. Case sensitive. |
| Strings | Yes | Double quotes `"..."`. Escape handling currently minimal (prefer plain text). |
| Numbers | Yes | Integer literals (no fractional parsing yet unless already implemented in your branch). |
| Booleans | Yes | `true`, `false` (if lexer currently recognizes; else represent with 1 / 0). |
| Arrays / `[]` | Not yet | Using `[` causes a lexing error today. See “Sequences Without Arrays”. |
| `%` (modulo) | Not yet | Use division + subtraction patterns. |
| `&&`, `||` | If implemented | If absent, emulate with nested `if`. |

### 3. Statements
| Construct | Form | Example |
|-----------|------|---------|
| Variable Declaration | `let name = expr;` | `let x = 10;` |
| Assignment | `name = expr;` | `x = x + 1;` |
| If | `if (condition) { ... } else { ... }` | `if (x == 0) { log("zero"); } else { log("non-zero"); }` |
| While | `while (condition) { ... }` | `while (i < 10) { log(i); i = i + 1; }` |
| For (if implemented) | `for (init; cond; step) { ... }` | (Check existing examples) |
| Function Decl (if implemented) | `fn name(params) { ... }` | `fn add(a, b) { return a + b; }` |
| Return | `return expr;` | `return x;` |
| Log / Print | `log(expr);` / `print(expr);` | `log("Value:" + v);` |

> Parentheses around `if` / `while` conditions ARE required (otherwise: “Parsing error: Expected '(' after if”).

### 4. Expressions & Operators
Currently safe core:
```
Arithmetic: +  -  *  /
Comparison: == != < <= > >=
Logical (if present): ! (unary not), &&, || (verify availability) 
Grouping: (expr)
Concatenation: String + String/Number (the `+` operator does double duty)
```

Missing / Not Yet: `%`, `++`, `--`, `?:`, bitwise ops.

### 5. Built‑ins (Native VM subset)
| Name | Purpose | Example |
|------|---------|---------|
| `log` | Print value (newline) | `log(x);` |
| `print` | (Alias if implemented) | `print("raw");` |
| `time_ms` | Millisecond timestamp | `let t = time_ms();` |
| `rand` | Pseudo random integer | `let r = rand();` |

Planned / Extended (Quantum etc.) show up as identifiers but may be stubs in native mode.

### 6. Control Flow Patterns

Loop with manual counter:
```ai
let i = 0;
while (i < 5) {
  log("i = " + i);
  i = i + 1;
}
```

Early return in a function (if functions enabled):
```ai
fn abs(n) {
  if (n < 0) { return -n; }
  return n;
}
log(abs(-5));
```

### 7. Random Selection Without Arrays or Modulo

Problem: Need one item from N choices; arrays and `%` not available.

Pattern:
```ai
let r = rand();
let r10 = (r / 10);       # shrink
let r100 = (r10 / 10);    # shrink again
let pick = (r100 / 10);   # coarse bucket

if (pick == 0) { log("Choice A"); }
else if (pick == 1) { log("Choice B"); }
else if (pick == 2) { log("Choice C"); }
else { log("Fallback"); }
```

### 8. Emulating Lists (Sequences Without `[]`)
Until literal arrays exist, store enumerated constants in a function chain:
```ai
fn say_fact(n) {
  if (n == 0) { log("Honey never spoils."); return; }
  if (n == 1) { log("Octopuses have three hearts."); return; }
  if (n == 2) { log("Bananas are berries; strawberries aren't."); return; }
  if (n == 3) { log("A day on Venus is longer than a year on Venus."); return; }
  log("Wombat poop is cube-shaped.");
}

let r = rand();
let a = (r / 10);
let b = (a / 10);
let pick = (b / 10);
say_fact(pick);
```

### 9. String Building
Use `+` to concatenate:
```ai
let name = "Traveler";
log("Hello, " + name + "!");
```
No template literals yet; just chain `+`.

### 10. Debugging & Errors
| Error Kind | Likely Cause | Fix |
|------------|-------------|-----|
| `Lexing error: Unexpected character '['` | Arrays not implemented | Remove `[` / use pattern in §8 |
| `Lexing error: Unexpected character '%'
` | Modulo not implemented | Use division + subtraction buckets |
| `Parsing error: Expected '(' after if` | Missing parentheses | Add `( )` around condition |
| Runtime error (if any) | Built‑in misuse / unhandled state | Add logging around variables |

Pretty diagnostics (with caret spans) appear when you pass `--pretty-errors` to CLI or enable pretty mode in shell environment.

### 11. Semantic Analyzer Skips
You can bypass semantic analysis (some checks) for fast iteration:
```powershell
Aeonmi.exe run --native --no-sema demo.ai
```
You may lose early detection of type/usage mistakes in that run.

### 12. Native vs JS Backend Differences
| Aspect | JS Path | Native VM |
|--------|---------|-----------|
| Execution | Transpiles to JS then Node | Direct interpretation |
| Speed (small scripts) | Node startup overhead | No external process |
| Feature Gaps | Potential broader syntax (future) | Emerging parity; arrays/modulo pending |
| Debug Env | Use JS tooling | Set `AEONMI_DEBUG=1` for internal debug logs |

Force native:
```powershell
Aeonmi.exe run --native file.ai
```

### 13. Suggested Progression Path
1. Output & variables (`log`, `let`).
2. Arithmetic & comparisons.
3. Conditionals (`if` / `else`).
4. Loops (`while`).
5. Functions (once enabled in your build) & returns.
6. Randomness patterns with `rand`.
7. Refactor repeated chains into helper functions.
8. (Later) Adopt arrays, modulo, richer types when released.

### 14. Idiomatic Patterns (Today)
| Goal | Pattern |
|------|---------|
| Clamp value to max 4 | `if (n > 4) { n = 4; }` |
| Ensure non‑negative | `if (n < 0) { n = 0; }` |
| Select one of N strings | Cascading `if (pick == k)` |
| Loop fixed times | Counter + `while (i < N)` |

### 15. Example: Mini “DailyMotivate”
```ai
let user = "Traveler";
log("Hello, " + user + "!");

let r1 = rand(); let r2 = rand();
let a1 = (r1 / 10); let a2 = (a1 / 10); let fact_pick = (a2 / 10);
let b1 = (r2 / 10); let b2 = (b1 / 10); let quote_pick = (b2 / 10);

if (fact_pick == 0) { log("Fun fact: Honey never spoils."); }
else if (fact_pick == 1) { log("Fun fact: Octopuses have three hearts."); }
else if (fact_pick == 2) { log("Fun fact: Bananas are berries; strawberries aren't."); }
else if (fact_pick == 3) { log("Fun fact: A day on Venus is longer than a year on Venus."); }
else { log("Fun fact: Wombat poop is cube-shaped."); }

if (quote_pick == 0) { log("Motivation: Keep going; you're closer than you think."); }
else if (quote_pick == 1) { log("Motivation: Small steps add up to big change."); }
else if (quote_pick == 2) { log("Motivation: Focus, commit, grow."); }
else if (quote_pick == 3) { log("Motivation: Progress > perfection."); }
else { log("Motivation: Your future is built today."); }
```

### 16. Roadmap Hints (Not Yet Available In Your Build)
Planned / emerging (watch release notes):
* Array literals `[...]` with indexing `name[i]`.
* `%` operator and possibly other arithmetic.
* Enhanced string escapes / interpolation.
* Structured data (records / objects) & pattern matching (experimental design stage).

If you experiment early and hit lexing errors, fall back to the emulation patterns above.

### 17. Checklist for New .ai Files
1. Start with `let` declarations and a `log` to confirm execution.
2. Introduce one new construct at a time.
3. If you see a lexing error: remove unsupported symbol (`[`, `%`, etc.).
4. Always wrap `if` / `while` conditions in parentheses.
5. Keep comments on their own lines starting with `#`.

### 18. Quick Troubleshooting Flow
| Symptom | Step 1 | Step 2 | Step 3 |
|---------|--------|--------|--------|
| Unexpected character `[` | Replace with cascaded `if` | Move data to helper fn | Track roadmap |
| Unexpected character `%` | Replace with divide/shrink pattern | Use subtraction loop | Await operator support |
| Stuck variable value | Insert `log("DEBUG:" + v);` lines | Re-run with native | Isolate minimal snippet |
| Program silent | Add first line `log("START");` | Check file path | Ensure run command order: `run file.ai --native` |

---

End of Language Guide. Contributions (suggested clarifications, new pattern examples) are welcome via issue tickets.


Roadmap (native): semantic analysis integration, richer type coercions, quantum op execution hooks, glyph intrinsic dispatch, performance optimizations (bytecode / arena).

### Roadmap Notes (Preview)

* Hieroglyphic (QUBE) execution semantics expansion
* Additional quantum backends / remote execution
* Enhanced diagnostics & AI-assisted refactors
* Plugin architecture for custom tokens and transformations

### Repository Hygiene & Large File Policy

To keep the repository lean:

* Build outputs (`target/`, `node_modules/`, temporary exec artifacts `__exec_tmp*`) are ignored and must not be committed.
* Helper scripts:
  * `scripts/scan_large_files.ps1` — scan current tracked files over a size threshold (default 1MB). Exit code 1 signals findings.
  * `scripts/scan_history_large_files.ps1` — scan full Git history for large blobs (default >1MB) for potential purge/LFS migration.
* Optional pre-commit hook (PowerShell): copy `scripts/git-hooks/pre-commit` into `.git/hooks/` to block large or disallowed files.
* Git LFS is enabled for `.pdb` and `.rmeta` artifacts (see `.gitattributes`). Collaborators must run `git lfs install` after pulling.
* For existing clones without LFS configured, run the migration snippet below to re-fetch LFS objects.
* The TUI search persistence file `.aeonmi_last_search` remains local only (ignored).

Quick usage:

```powershell
pwsh -NoProfile -File scripts\scan_large_files.ps1
copy scripts\git-hooks\pre-commit .git\hooks\pre-commit  # install hook
```

#### Git LFS Activation

We track debug symbol / metadata files via LFS:

```text
*.pdb filter=lfs diff=lfs merge=lfs -text
*.rmeta filter=lfs diff=lfs merge=lfs -text
```

First-time setup (per developer):

```powershell
git lfs install
git pull --force   # ensure LFS pointers materialize
```

If you previously pulled before LFS activation and have large blobs in normal history you may reclone for a clean state or run:

```powershell
git lfs fetch --all
git lfs checkout
```

#### Large Blob Remediation (Maintainers)

1. Identify historical large blobs:
  ```powershell
  pwsh -NoProfile -File scripts\scan_history_large_files.ps1 -ThresholdMB 2
  ```
2. If unintended, create a branch & use `git filter-repo` (after full backup) to remove them.
3. Force-push sanitized history and instruct collaborators to reclone.

Never rewrite history on `main` without explicit coordination.

If the hook blocks a file you believe is necessary, open a maintainer issue referencing the context.

## Directory Structure

```text
Aeonmi/
├─ .github/
│  └─ workflows/
│     └─ release.yml
├─ Cargo.toml
├─ Cargo.lock
├─ LICENSE
├─ SECURITY.md
├─ README.md
├─ aeonmi.run.js
├─ output.js
├─ test_output.js
├─ examples/
│  ├─ hello.ai
│  ├─ control_flow.ai
│  ├─ functions.ai
│  ├─ glyph.ai
│  └─ ...
└─ src/
   ├─ cli.rs
   ├─ config.rs
   ├─ lib.rs
   ├─ main.rs
   ├─ bin/
   │  ├─ aeonmi.rs
   │  └─ aeonmi_project.rs
   ├─ ai/
   ├─ blockchain/
   ├─ cli/
   ├─ commands/
   │  ├─ ast.rs
   │  ├─ compile.rs
   │  ├─ edit.rs
   │  ├─ format.rs
   │  ├─ lint.rs
   │  ├─ mod.rs
   │  ├─ repl.rs
   │  ├─ run.rs
   │  └─ tokens.rs
   ├─ core/
   │  ├─ ast.rs
   │  ├─ code_generator.rs
   │  ├─ compiler.rs
   │  ├─ diagnostics.rs
   │  ├─ error.rs
   │  ├─ lexer.rs
   │  ├─ lib.rs
   │  ├─ mod.rs
   │  ├─ parser.rs
   │  ├─ qpoly.rs
   │  ├─ semantic_analyzer.rs
   │  └─ token.rs
   ├─ io/
   ├─ physics/
   ├─ shell/
   │  └─ mod.rs
   ├─ titan/
   └─ tui/
      ├─ editor.rs
      └─ mod.rs
```

## Examples

```bash
# Token stream of a program
cargo run -- tokens examples/hello.ai

# AST of a program
cargo run -- ast examples/functions.ai

# Compile & run in one shot (JS target -> node)
cargo run -- run examples/hello.ai --out output.js
```

## AI Integration (Multi-Provider Chat)

Aeonmi includes an experimental "Mother AI Module" supporting multiple providers behind feature flags. Providers are opt-in to keep the default build lean.

### Enable Providers

Enable one or more AI providers at build/run time using Cargo features:

```powershell
cargo run --features ai-openai -- ai chat --list
cargo run --features ai-openai,ai-perplexity -- ai chat --list
```

Available feature flags:
* `ai-openai`
* `ai-copilot`
* `ai-perplexity`
* `ai-deepseek`

Each current implementation uses blocking HTTP (reqwest) for simplicity; future versions may introduce async + streaming.

### Environment Variables

Set required API keys (only the ones for the features you enabled):

```powershell
$env:OPENAI_API_KEY = "sk-your-key"          # for ai-openai
$env:GITHUB_COPILOT_TOKEN = "ghu_xxx"        # (planned) for ai-copilot
$env:PERPLEXITY_API_KEY = "pxp_your-key"     # for ai-perplexity
$env:DEEPSEEK_API_KEY = "ds_your-key"        # for ai-deepseek
```

Optional overrides:
* `AEONMI_OPENAI_MODEL` (default: `gpt-4o-mini`)
* `AEONMI_SEED` (u64) – deterministic global seed for native VM `rand()` and synthetic metrics generation (when a bench seed flag isn't provided)

### List Enabled Providers

```powershell
cargo run --features ai-openai,ai-perplexity -- ai chat --list
```

### Simple Chat

If you enable only one provider you can omit `--provider`:

```powershell
$env:OPENAI_API_KEY = "sk-your-key"
cargo run --features ai-openai -- ai chat "Explain QUBE in one sentence"
```

Streaming (OpenAI only currently):

```powershell
cargo run --features ai-openai -- ai chat --stream "Stream a short description of Aeonmi"
```

With multiple providers, specify `--provider`:

```powershell
cargo run --features ai-openai,ai-perplexity -- ai chat --provider perplexity "Compare QUBE to traditional ASTs"
```

You can also pipe a prompt from stdin (omit the prompt argument):

```powershell
"Summarize Aeonmi goals" | cargo run --features ai-openai -- ai chat
```

### Roadmap (AI)

* Streaming responses (server-sent events / chunked)
* Concurrent multi-provider fan-out
* Structured refactor suggestions and code patch proposals
* Context injection from local project files
* Secure key storage / keychain integration
* Rate limiting & caching layer


## License

This software is licensed under the Aeonmi Proprietary Software License Agreement. See **LICENSE** for details.

`metrics-config` – configure runtime metrics parameters. Examples:
```powershell
# Show current config (ema alpha %, window)
aeonmi metrics-config
# Set EMA alpha to 35%
aeonmi metrics-config --set-ema 35
# Set rolling window size to 32
 aeonmi metrics-config --set-window 32
# JSON output
 aeonmi metrics-config --json
# Reset to defaults
aeonmi metrics-config --reset
```
`metrics-deep` – enable/disable deep propagation (transitive caller expansion regardless of size):
```powershell
# Enable deep propagation
aeonmi metrics-deep --enable
# Disable
aeonmi metrics-deep --disable
# JSON status
aeonmi metrics-deep --json
```
`metrics-bench` (feature `debug-metrics`) – generate synthetic metrics.
Flags: `--functions N`, `--samples N`, `--base-ns`, `--step-ns`, `--jitter-pct P`, `--dist linear|exp`, `--sort ema|avg|last`, `--csv file.csv`, `--reset`, `--json`, `--seed S` (if omitted falls back to `AEONMI_SEED` or default 0xC0FFEE).
Example:
```powershell
cargo run --features debug-metrics -- metrics-bench --functions 8 --samples 12 --base-ns 500 --step-ns 50 --jitter-pct 15 --dist exp --csv bench.csv --json --reset
```
`metrics-debug` (feature `debug-metrics`) – dump raw internal metric structures (windows, EMA, savings history).
```powershell
cargo run --features debug-metrics -- metrics-debug --pretty
```

Metrics Schema Evolution (current version = 6):
| Version | Additions |
|---------|-----------|
| 3 | Initial persisted call graph + function timings |
| 4 | Exponential moving average (ema_ns) |
| 5 | Cumulative savings percentages (cumulative_savings_pct, cumulative_partial_pct) |
| 6 | Rolling window averages (window_avg_ns), recent window savings (recent_window_*), sample history (recent_samples), pruning (functionMetricsPruned), runtime config (emaAlphaPct, windowCapacity), deepPropagation flag |

Savings Metrics Fields:
- cumulative_savings_ns / cumulative_partial_ns / cumulative_estimated_full_ns
- cumulative_savings_pct = savings_ns / estimated_full_ns * 100
- cumulative_partial_pct = partial_ns / estimated_full_ns * 100
- recent_window_partial_ns / recent_window_estimated_full_ns / recent_window_savings_pct (rolling over last N samples; N = history_cap default 32)
- recent_samples: array of { partial_ns, estimated_full_ns, savings_ns }

Pruning: function metrics with last_run_epoch_ms before session start are excluded at build_json time and counted in functionMetricsPruned.

Runtime Configuration Sources:
1. Environment variables at process start:
   - AEONMI_EMA_ALPHA (1-100, default 20)
   - AEONMI_METRICS_WINDOW (4-256, default 16)
2. CLI: metrics-config setters override the atomic values for the life of the process.

Deterministic Randomness:
* Set `AEONMI_SEED` to fix the native interpreter `rand()` sequence. Absent this, a time-based seed initializes the LCG once. Seed value 0 is coerced to 1.
