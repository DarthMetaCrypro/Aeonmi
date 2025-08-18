
### option B — unified diff (if you prefer `git apply`)
```diff
diff --git a/README.md b/README.md
index 0000000..0000000 100644
--- a/README.md
+++ b/README.md
@@
-<p align="center">
-  <img src="https://img.shields.io/github/actions/workflow/status/DarthMetaCrypro/Aeonmi/release.yml?label=build" />
-  <img src="https://img.shields.io/github/v/release/DarthMetaCrypro/Aeonmi?include_prereleases&sort=semver" />
-  <img src="https://img.shields.io/badge/license-Proprietary-red" />
-  <img src="https://img.shields.io/badge/language-Rust-informational" />
-</p>
-
-Aeonmi v0.2.0 – Closed Source Pre-Release
-⚠️ Notice:
-Aeonmi is currently a closed-source project. No redistribution, modification, reverse engineering, or unauthorized use of any kind is permitted without explicit written consent from the author. All rights reserved. This pre-release is provided for demonstration, evaluation, and controlled collaboration purposes only.
-
-Overview
-Aeonmi is an advanced programming language and compiler framework designed for AI-native, secure, and multi-dimensional computing. It introduces QUBE, a symbolic and hieroglyphic inner-core language capable of adaptive, self-modifying operations, quantum-resistant encryption, and deep AI integration.
-
-Version 0.2.0 delivers the initial operational codebase, test scaffolding, and compiler infrastructure required to begin controlled internal evaluation.
-
-Highlights in v0.2.0
-Core Compiler Pipeline – Fully implemented lexer, parser, semantic analyzer, and code generator modules.
-
-QUBE Integration Layer – Foundation for symbolic and quantum glyph parsing.
-
-Robust Test Suite – Modular test files for compiler components:
-
-compiler_pipeline.rs
-
-control_flow.rs
-
-functions.rs
-
-quantum_glyph.rs
-
-Diagnostics System – Advanced error handling with rich contextual output.
-
-Security by Design – Quantum-resistant cryptographic stubs embedded in the architecture.
-
-Strict Licensing – All code remains proprietary; public use is restricted.
-
-Directory Structure
-```
-[... old tree ...]
-```
-
-License
-This release is governed under a proprietary license.
-You may not:
+<p align="center">
+  <img src="https://img.shields.io/github/actions/workflow/status/DarthMetaCrypro/Aeonmi/release.yml?label=build" />
+  <img src="https://img.shields.io/github/v/release/DarthMetaCrypro/Aeonmi?include_prereleases&sort=semver" />
+  <img src="https://img.shields.io/badge/license-Proprietary-red" />
+  <img src="https://img.shields.io/badge/language-Rust-informational" />
+</p>
+
+# Aeonmi v0.2.0 – Closed Source Pre-Release
+
+> **Notice**  
+> Aeonmi is a closed-source project. No redistribution, modification, reverse engineering, or unauthorized use is permitted without explicit written consent from the author. All rights reserved. This pre-release is for demo/evaluation and controlled collaboration.
+
+## Overview
+Aeonmi is an advanced programming language and compiler framework for AI-native, secure, multi-dimensional computing. It introduces **QUBE**, a symbolic/hieroglyphic inner-core aimed at adaptive, self-modifying operations with quantum-resistant security and deep AI integration.
+
+## What’s in v0.2.0
+- **Core compiler pipeline**: lexer → parser → semantic analyzer → code generator.
+- **Diagnostics** with rich, contextual error reporting.
+- **QUBE integration layer** foundations (symbolic / glyph parsing).
+- **Examples** covering precedence, control flow, functions, and quantum/glyph basics.
+- **Strict proprietary licensing**.
+
+## Binaries
+This workspace builds two executables today:
+- **`Aeonmi`** – primary CLI (default run target).
+- **`aeonmi_project`** – legacy/test binary for back-compat.
+(Registered in `Cargo.toml` with `autobins=false`, `default-run="Aeonmi"`.)
+
+> Running `Aeonmi` **with no subcommand/legacy args** opens the **Aeonmi Shard** interactive shell. Use `help` inside the shell for commands like `ls`, `cd`, `edit --tui`, `compile`, `run`, `clear`, `exit`.
+
+## Install & Build
+```bash
+git clone https://github.com/DarthMetaCrypro/Aeonmi.git
+cd Aeonmi
+cargo build --release
+cargo run   # default opens the shell
+```
+
+## Quick Usage
+```bash
+cargo run -- run examples/hello.ai --out output.js
+cargo run
+```
+
+## Directory Structure
+```text
+[... tree from the “quick replace” version ...]
+```
+
+## License
+This software is licensed under the Aeonmi Proprietary Software License Agreement. See **LICENSE** for details.
