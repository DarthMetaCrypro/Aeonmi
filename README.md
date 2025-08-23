# Aeonmi

Aeonmi is a compiler/runtime toolkit for quantum and AI-assisted code generation and execution. This repository contains the compiler, runtime pieces, and tooling used in development and testing.

## Quickstart

1. Install Rust (recommended stable toolchain).
2. Clone the repo:
   git clone https://github.com/DarthMetaCrypro/Aeonmi.git
3. Build:
   cd Aeonmi
   cargo build

## Running tests

- Run unit and integration tests:
  cargo test

- Run snapshot review (insta):
  cargo insta review
  - While reviewing snapshots you can:
    - a to accept the new snapshot
    - r to reject
    - s to skip and keep both versions

The recent branch `fix/remove-librs` required updating the glyph formatting snapshot; the new snapshot was accepted and included in the changelog.

## Notable changes in this branch
- Exported `io` module (atomic)
- Cleanups related to removing library references (branch `fix/remove-librs`)
- Snapshot updates for glyph formatting

## Developer notes / TODOs
- Address small warnings reported by the compiler:
  - `to_emit_kind` method is currently unused (dead_code).
  - A few unused variables in CLI command handling (prefix with `_` or remove).
- Decide on version bump strategy:
  - Patch release (0.2.1) recommended for these internal fixes and API stabilization.
- Verify CI matrix runs on Windows/Linux/macOS if not already present.

## Contributing
- Fork, branch, run tests locally, open PR against `main`.
- Ensure `cargo test` and `cargo insta review` pass before requesting merge.
- Keep commits small and descriptive; for small fixes prefer a squash-merge.