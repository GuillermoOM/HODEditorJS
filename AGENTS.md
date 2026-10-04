# Entry Point for Agents

Welcome to HODEditorJS!

If you are an AI agent picking up a task in this repository, please immediately read the full handbook located at:

**`docs/agent-handbook/README.md`**

This handbook will explain:
1. The 3-tier architecture (React, Tauri, Rust Core).
2. The rules for modifying the UI (using the UI Source of Truth).
3. The rules for modifying the binary parser (Running `verify_lossless` is MANDATORY).
4. The historical context of HOD 2.0 reverse engineering.

*Do not begin modifying files until you have read the handbook.*

---

## Build & Packaging

**All release builds (`tauri build`) must be performed inside the `mesa-dev` distrobox.** Native host builds fail due to missing GTK/WebKit dependencies and AppImage FUSE issues. The Windows cross-compile also requires `mesa-dev` for `mingw-w64` and `makensis`.

```bash
distrobox enter mesa-dev

# Linux
NO_STRIP=1 npm run tauri build

# Windows NSIS
CARGO_TARGET_DIR=/tmp/cargo_target npm run tauri build -- --target x86_64-pc-windows-gnu --bundles nsis
```

For parser-only verification: `cargo check --lib --manifest-path parser/Cargo.toml` (or `--lib` to skip unrelated `validation_suite.rs`).
For frontend-only verification: `npm run build`.

See `README.md` for full artifact paths and details.

---

## Documentation Update Rule

**MANDATORY**: Before ending any session, update the reverse engineering project documentation so another agent can pick up the work without context loss.

### What to Update

1. **`docs/hod2-reverse-engineering/PROGRESS.md`** — Update the "Current Status" section, "Current Issues" list, and "Planned Tasks" with what was fixed, what broke, and what's next. This is the **ONLY** file you must update for tracking daily progress!
2. **`docs/README.md` (Knowledge Graph)** — If you create any *new* permanent specification document or directory, you MUST add a link to it in this file so other agents can find it. Do not link temporary or stale logs here.
   The mandated manual test snapshots are recorded in `docs/manual-test-log.md`.

3. **`docs/hod2-reverse-engineering/hod2_reverse_engineering_knowledge_base.md`** — If you discover a new binary format quirk, endianness trap, or undocumented engine constraint, you MUST add it here as a permanent caveat.

### What to Document

- **What was fixed / succeeded** — specific file:line references and what the fix does. Include exact success messages or metric improvements.
- **What failed / remains broken** — specific error messages, file:line locations, and root cause analysis if known. Never leave broken code without logging the error.
- **Caveats & Quirks found** — Any strange behavior, unexpected limitations, or non-standard workarounds used to satisfy the game engine.
- **Test results** — exact command output (pass/fail counts, error messages).
- **Decisions made** — why a particular approach was chosen over alternatives.
- **Blockers** — what prevents progress and what would unblock it.

### Commit Checkpoint Rule

Before ending a session with significant changes, create a git commit as a checkpoint:
- Use a descriptive commit message summarizing what changed and why.
- Stage all modified and untracked files (`git add -A`).
- The commit message should be useful to the next agent reading `git log --oneline`.
