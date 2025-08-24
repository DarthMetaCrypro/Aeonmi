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

### Troubleshooting

| Issue | Resolution |
|-------|------------|
| `qsim` says quantum not built | Re-run with `--features quantum` |
| Node not found when running JS | Install Node.js and ensure `node` is in PATH |
| Colors missing on Windows | Use Windows Terminal or VS Code integrated terminal |
| Mouse selection blocked in TUI | Press F9 to disable mouse capture |
| Unsaved changes warning on exit | Press Ctrl+S then Esc again |

### Roadmap Notes (Preview)

* Hieroglyphic (QUBE) execution semantics expansion
* Additional quantum backends / remote execution
* Enhanced diagnostics & AI-assisted refactors
* Plugin architecture for custom tokens and transformations

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

## License

This software is licensed under the Aeonmi Proprietary Software License Agreement. See **LICENSE** for details.
