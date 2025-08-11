<p align="center">
  <img src="https://img.shields.io/github/actions/workflow/status/DarthMetaCrypro/Aeonmi/release.yml?label=build" />
  <img src="https://img.shields.io/github/v/release/DarthMetaCrypro/Aeonmi?include_prereleases&sort=semver" />
  <img src="https://img.shields.io/badge/license-Proprietary-red" />
  <img src="https://img.shields.io/badge/language-Rust-informational" />
</p>

Aeonmi v0.2.0 – Closed Source Pre-Release
⚠️ Notice:
Aeonmi is currently a closed-source project. No redistribution, modification, reverse engineering, or unauthorized use of any kind is permitted without explicit written consent from the author. All rights reserved. This pre-release is provided for demonstration, evaluation, and controlled collaboration purposes only.

Overview
Aeonmi is an advanced programming language and compiler framework designed for AI-native, secure, and multi-dimensional computing. It introduces QUBE, a symbolic and hieroglyphic inner-core language capable of adaptive, self-modifying operations, quantum-resistant encryption, and deep AI integration.

Version 0.2.0 delivers the initial operational codebase, test scaffolding, and compiler infrastructure required to begin controlled internal evaluation.

Highlights in v0.2.0
Core Compiler Pipeline – Fully implemented lexer, parser, semantic analyzer, and code generator modules.

QUBE Integration Layer – Foundation for symbolic and quantum glyph parsing.

Robust Test Suite – Modular test files for compiler components:

compiler_pipeline.rs

control_flow.rs

functions.rs

quantum_glyph.rs

Diagnostics System – Advanced error handling with rich contextual output.

Security by Design – Quantum-resistant cryptographic stubs embedded in the architecture.

Strict Licensing – All code remains proprietary; public use is restricted.

Directory Structure
```
Aeonmi/
├── .github/workflows/release.yml       # Release automation
├── Cargo.toml                          # Rust project manifest
├── Cargo.lock
├── src/
│   ├── core/
│   │   ├── ast.rs
│   │   ├── code_generator.rs
│   │   ├── compiler.rs
│   │   ├── diagnostics.rs
│   │   ├── error.rs
│   │   ├── lexer.rs
│   │   ├── lib.rs
│   │   ├── mod.rs
│   │   ├── parser.rs
│   │   ├── semantic_analyzer.rs
│   │   └── token.rs
│   ├── lib.rs
│   └── main.rs
├── tests/
│   ├── assign_and_calls.rs
│   ├── cli_smoke.rs
│   ├── comparisons.rs
│   ├── compiler_pipeline.rs
│   ├── control_flow.rs
│   ├── diagnostics.rs
│   ├── errors_extra.rs
│   ├── functions.rs
│   ├── precedence.rs
│   └── quantum_glyph.rs
├── output.js
├── test_output.js
└── README.md
```
License
This release is governed under a proprietary license.
You may not:

Copy, distribute, or modify the source code

Reverse engineer, decompile, or attempt to derive the source

Use Aeonmi for commercial purposes without explicit permission

Next Steps
Controlled distribution to select collaborators

Begin alpha-stage evaluation of QUBE syntax

Expand compiler optimizations and AI integration layer

