# Changelog

All notable changes to this project will be documented in this file.

## Unreleased

### Added
- Exported `io` module (atomic) to support direct IO integration and library consumers.
- Snapshot updates for glyph formatting examples.

### Changed
- Removed library-related references and cleanup from internal modules (work done on branch `fix/remove-librs`).
- Improved formatting stability for example outputs (snapshots updated).

### Fixed
- Various fixes that surfaced during removal/cleanup of library references.

### Tests
- All unit and integration tests passing locally:
  - 34 unit tests for core library passed.
  - Multiple integration/golden tests passed.
  - Snapshot review performed and updated for `glyph.ai.snap`.
- Note: Some non-critical compiler warnings remain (unused variables and a dead method). These do not affect correctness but should be cleaned up in a follow-up.

### Notes
- Recommend a minor version bump (e.g. v0.2.0 -> v0.2.1) after merge if you consider these changes a patch/minor release.
- CI should be allowed to run on the PR to verify platform-specific behavior.