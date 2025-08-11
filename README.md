Aeonmi v0.2.0
⚠ Proprietary Software – All Rights Reserved
This project is closed-source and is the exclusive property of DARK META Studios. Unauthorized copying, modification, distribution, or use is strictly prohibited. See the LICENSE file for full terms.

Overview
Aeonmi is a next-generation programming language and execution environment designed for AI-native development, quantum-resistant security, and high-performance computing.
Version 0.2.0 introduces the initial public binary release for limited testing, featuring the core compiler, parser, lexer, and semantic analysis pipeline.

Directory Structure
```
aeonmi_project/
├── .github/
│   └── workflows/
│       └── release.yml
├── .gitignore
├── Cargo.lock
├── Cargo.toml
├── LICENSE.txt
├── README.md
├── output.js
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
├── test_output.js
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
```
Release Notes – v0.2.0
🚀 Initial Code Drop – Full compiler pipeline implementation (Lexer → Parser → Semantic Analyzer → Code Generator).

🛡 Quantum-Resistant Security Layer integrated into the core design principles.

⚡ Optimized Tokenization & Parsing for complex syntax and AI-driven workflows.

🧪 Automated Test Suite with coverage for precedence rules, control flow, functions, and compiler pipelines.

📦 Binary Output Support – Generates JavaScript output for cross-platform execution.

License
This software is licensed under the Aeonmi Proprietary Software License Agreement.
See LICENSE.txt for details.
