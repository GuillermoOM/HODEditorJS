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

See `README.md` for full build and packaging instructions for any developer and platform. Builds need the Tauri prerequisites, including GTK/WebKit on Linux. On this machine a container is used; specifics live in the machine-local environment file.

Machine-specific paths, container names, and build quirks live in your untracked local-environment file (this machine: `.omo/local-environment.md`). Read it if it exists before assuming any absolute path or build command variant. See `docs/local-environment.md` for the convention and template.

For parser-only verification: `cargo check --lib --manifest-path parser/Cargo.toml` (or `--lib` to skip unrelated `validation_suite.rs`).
For frontend-only verification: `npm run build`.

See `README.md` for the generic equivalents, full artifact paths, and details.

## Parser Verification

**CRITICAL PARSER RULE:** If you change any serialization logic in `parser/src/serialize.rs`, parsing logic in `parser/src/parse.rs`, DXT/texture codec logic in `parser/src/texture_codec.rs`, or logic in `parser/src/compiler.rs`, you **MUST** run `verify_lossless` to prove you did not break the game engine format.

Fixture paths are read from the machine-local environment file (this machine: `.omo/local-environment.md`); `verify_lossless` exits with an actionable message when it is absent.

If the game installation required by `verify_lossless` is absent, run `cargo test --manifest-path parser/Cargo.toml` instead and state in the session record that `verify_lossless` was **not run**. Do not claim a lossless pass when its external inputs are unavailable.

### Useful Paths for Testing

Fixture files are external inputs extracted from your own game/mod installation. See `docs/testing-fixtures.md` for what is needed and `docs/local-environment.md` for where to record paths.
- HOD 1.3 ship directory
- HOD 2.0 mod ship directory
- DAE conversion directory
- Vanilla HOD 2.0 ship directory
- Vanilla HOD 2.0 pebble directory

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
