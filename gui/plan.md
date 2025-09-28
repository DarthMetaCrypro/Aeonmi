Goal (Expanded): Ship a cross‑platform, installable Aeonmi IDE (Editor + CLI + AI + Quantum tooling) that feels familiar like VS Code but opinionated for `.ai` / QUBE workflows, while remaining extensible for other languages.

High-Level Pillars
1. Core Editing: Monaco (initial) → Custom language server (syntax, semantic highlights, diagnostics, code actions) for `.ai`.
2. Integrated VM & Toolchain: Native interpreter + compile emit pipeline accessible via command palette & buttons.
3. Terminal & CLI Embedding: Persistent Aeonmi Shard/CLI PTY; multi-terminal tabs.
4. AI Assist: Inline completion, explain, optimize, refactor (wrapping existing provider registry).
5. Quantum & Glyph Panels: Visualize quantum states (when feature enabled) + hieroglyphic/glyph browsing.
6. Project/Workspace Management: File explorer, recent workspaces, tasks, build profiles.
7. Cross-Language Support: Basic syntax + run tasks for JS/Py/Rust with pluggable runners.
8. Distribution: Tauri app packaging (MSI / EXE for Windows, DMG for macOS, AppImage for Linux), auto-update channel.
9. Extensibility: Lightweight plugin manifest (JSON/TOML) to register new run tasks or code gens.
10. Security / Offline: Sandboxed provider calls; local-only mode.

Phase Breakdown
Phase 0 (DONE / IN PROGRESS):
- Minimal web prototype (Express + xterm.js) & native VM path.
- PTY bridging (tauri_bridge skeleton using `portable-pty`).

Phase 1 (Immediate Next):
- Consolidate: Migrate current Express prototype logic into Tauri frontend (remove duplication).
- Add unified command palette (hotkey Ctrl+Shift+P) calling backend commands.
- Implement robust binary discovery & build-on-missing (run `cargo build` automatically if missing).
- Basic Monaco tokenization for `.ai` (simple Monarch definition).

Phase 2:
- Language server (either custom or LSP stub) providing: diagnostics (reuse existing parser/lexer), go-to-definition, hover (token docs), formatting (existing formatter), and rename (symbol table pass).
- Side panel: Project file tree.
- Status bar: VM mode (native/JS), active backend (quantum), AI provider.

Phase 3:
- AI integration: Inline completions (ghost text), quick actions (Explain selection, Optimize), chat panel anchored in right sidebar.
- Quantum panel: If built with feature, subscribe to simulated circuit runs & show measurement histograms.
- Glyph browser: Searchable list of hieroglyphic ops with insertion.

Phase 4:
- Tasks system: JSON/TOML tasks (build, test, lint, run). Surface results in Problems panel.
- Multi-terminal tabs (shell, Aeonmi Shard, cargo build logs).
- Watch mode indicator + restart controls.

Phase 5:
- Packaging scripts: `cargo tauri build` pipelines for Windows/macOS/Linux.
- Auto-update channel (alpha/dev vs stable) using GitHub Releases.
- Crash reporting & telemetry (opt-in).

Phase 6:
- Plugin API: Register additional language runners, code transforms, AI prompt templates.
- Marketplace stub (local index file first).

Phase 7:
- Performance pass: Bytecode VM or JIT path, caching semantic model, incremental parsing.
- Memory / leak audits, large file handling.

Deliverables Inventory (Current vs Target)
Current:
- Native VM (tree-walk) in Rust.
- CLI + Shard + TUI editor.
- Minimal browser prototype (non-Tauri) with fallback pseudo-terminal.
- Tauri bridge skeleton using `portable-pty`.

Missing / To Build:
- Integrated Tauri UI with Monaco + xterm (shared code).
- Language syntax highlighting & semantic diagnostics in editor (Monaco Monarch + LSP adapter).
- Command palette & keybinding map.
- AI panel + inline completions.
- Quantum visualization widgets.
- Packaged installers.
- Automated end-to-end tests (Spectron-like or Playwright against Tauri).

Immediate Actionable Next Steps (Recommended Order)
1. Add a Monaco Monarch tokenizer for `.ai` inside Tauri frontend (keywords, literals, operators, glyph tokens).
2. Implement a thin JSON-RPC over stdio or in-process channel to reuse existing parser for diagnostics (produce line/col -> severity, message).
3. Replace Express prototype with Tauri window (retire `gui/server.js` once Tauri parity reached).
4. Integrate PTY launch: if Aeonmi binary missing, run a background build task & stream output to a “Build” panel.
5. Add run/compile buttons wired to invoke existing Rust functions (exposed as Tauri commands) for AI emit / JS emit / native run.
6. Introduce config persistence (~/.aeonmi/ide.json) remembering last opened files & layout.
7. Wrap AI providers: create command `ai.explainSelection` mapping to backend AI registry.
8. Add quantum feature detection (build flag) and stub panel when disabled.

Technical Notes
- Tauri Commands: Expose a subset of `commands::compile::compile_pipeline`, `commands::run::main_with_opts` via `tauri::command` functions.
- PTY: Already have portable-pty usage; integrate into a manager struct that supports multiple terminals.
- Diagnostics Flow: Frontend sends document text -> backend returns tokens + errors -> Monaco decorations.
- Security: Disable remote module loading; treat AI credentials via secure storage (Tauri keychain plugin) not plain env.
- Packaging: Provide feature flags to exclude AI/quantum for minimal build.

Risk / Mitigations
- node-pty install flakiness (web prototype) → move entirely to portable-pty inside Tauri (Rust side) early.
- Large file perf → incremental lex (later phase) + virtualization for file tree.
- AI latency → streaming responses / progressive insertion.

Success Criteria v1 (MVP IDE Release)
✔ Open/edit/save `.ai`
✔ Syntax highlight + diagnostics
✔ Run native & JS backend from UI
✔ Integrated terminal (Aeonmi Shard)
✔ AI explain + basic chat
✔ Windows + macOS + Linux installers

Stretch (v1.x)
✔ Quantum panel, glyph palette
✔ Inline completions + refactors
✔ Task runner + debugger hooks
✔ Plugin registration

Tracking: Convert above phases into GitHub issues / project board columns (Backlog, In Progress, Review, Done).

Next Commit Targets (if accepted):
- Add `monaco_ai_language.js` placeholder with token spec.
- Add `src-tauri/commands.rs` exposing compile/run.
- Replace Express server usage in docs with Tauri instructions.

End of expanded plan.
