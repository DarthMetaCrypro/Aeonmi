## Contributing & Repository Hygiene

This project is presently a closed pre-release. External contributions require explicit authorization. If you have been granted access, follow these rules to keep the repository clean and efficient.

### 1. Do Not Commit Build Artifacts
Rust `target/`, Node `node_modules/`, generated temporary exec artifacts (`__exec_tmp*`), log files, and editor swap files must never be committed.

Run a quick audit before pushing:

```powershell
git status --short | Select-String -Pattern "target/|node_modules|__exec_tmp|.pdb$|.dll$"
```

### 2. Pre‑Commit Checks
Install the optional pre-commit hook to block large or unwanted files:

```powershell
copy scripts\git-hooks\pre-commit .git\hooks\pre-commit
```

### 3. Large Files (>1MB) Policy
Source files should rarely exceed 1MB. If you need to add a large asset, coordinate first and use Git LFS after approval.

Scan current working tree (tracked files only):

```powershell
pwsh -NoProfile -File scripts\scan_large_files.ps1
```

### 4. History Hygiene
If a large file was committed accidentally:
1. Remove it & commit.
2. Open an issue referencing the commit hash.
3. Await maintainer decision (Git LFS migration or history rewrite with `git filter-repo`).

### 5. Search Query Persistence File
The TUI stores last search text in `.aeonmi_last_search`. This is intentional and should remain untracked (ignored). Delete locally if you want to reset.

### 6. Temporary Exec Artifacts
`aeonmi exec` may create `__exec_tmp.js` or `__exec_tmp_rs(.exe)`; they are auto-removed unless `--keep-temp`. Never commit them.

### 7. Line Endings & Encoding
Follow `.gitattributes` (text normalized to LF). On Windows, use a capable terminal (Windows Terminal / VS Code) to avoid stray CRLF issues.

### 8. Commit Style
Conventional-ish prefixes help triage:
`feat:`, `fix:`, `refactor:`, `docs:`, `test:`, `chore:`, `perf:`, `build:`, `ci:`

### 9. Testing
Run the test suite before pushing significant changes:

```powershell
cargo test --all --quiet
```

Use hidden flags / env for faster CI-friendly runs where documented (`--no-run`, `AEONMI_WATCH_ONCE`).

### 10. Security & Proprietary Notice
Do not copy proprietary source outside approved channels. Report suspected leaks immediately.

---
For questions, contact the maintainer listed in `SECURITY.md` or open a private issue.
