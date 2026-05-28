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

## Documentation Update Rule

**MANDATORY**: Before ending any session, update the reverse engineering project documentation so another agent can pick up the work without context loss.

### What to Update

1. **`docs/hod2-reverse-engineering/PROGRESS.md`** — Update the "Current Status" section, "Current Issues" list, and "Planned Tasks" with what was fixed, what broke, and what's next.
2. **`docs/hod2-reverse-engineering/from-assets-to-hod-analysis.md`** — Update "What is not complete" and "Next Targets" sections.
3. **`docs/hod2-reverse-engineering/hodor-replication-test-success.md`** — Update "Current Result" with latest test output and "Next Steps".

### What to Document

- **What was fixed** — specific file:line references and what the fix does.
- **What's still broken** — specific error messages, file:line locations, and root cause analysis if known.
- **Test results** — exact command output (pass/fail counts, error messages).
- **Decisions made** — why a particular approach was chosen over alternatives.
- **Blockers** — what prevents progress and what would unblock it.

### Commit Checkpoint Rule

Before ending a session with significant changes, create a git commit as a checkpoint:
- Use a descriptive commit message summarizing what changed and why.
- Stage all modified and untracked files (`git add -A`).
- The commit message should be useful to the next agent reading `git log --oneline`.
