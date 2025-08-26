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
```

## CLI Usage (subject to change)

High‑level subcommands currently wired into the CLI:

```text
run <file.ai> [--out FILE] [--pretty-errors] [--no-sema]
# compile to JS and try executing with Node

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

Roadmap (native): semantic analysis integration, richer type coercions, quantum op execution hooks, glyph intrinsic dispatch, performance optimizations (bytecode / arena), and deterministic random with seed control.

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
