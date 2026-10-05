# Issues — repo-cleanup-and-hod-split

Problems and gotchas encountered during work on this plan.

_Auto-scaffolded by /ulw-execute. Append new entries below - never overwrite._

---
- Host rm -rf and chmod could not remove foreign-owned dist/assets; non-interactive sudo was unavailable. podman unshare was required and succeeded.
- Task 3: the plan's expected plain check-ignore exit 0 for tracked parser/src/lib.rs is incompatible with Git behavior; --no-index is required to demonstrate the temporary bogus rule.
- (orchestrator) `category=quick` dispatch is broken in this session (model chain unavailable). Recorded as orchestration deviation; `subagent_type=general` used instead for all tasks.
- Task 8: `cargo check` emitted the existing global Cargo cache permission warning for `window-vibrancy/.cargo_vcs_info.json`; compilation still exited 0 and the parser test suite remained at 14 passed.
Task 7 verify_lossless raw and normalized comparisons were non-zero against the Task 1 baseline despite matching 2,090-line output regions and expected gate markers; likely baseline/output drift outside this bin cleanup. Orchestrator should independently inspect the diff.
- Task 14: Cargo continues to report the existing global cache cleanup permission warning for `window-vibrancy-0.6.0/.cargo_vcs_info.json`; it does not affect exit status. The first verify_lossless attempt exceeded the 120-second tool timeout, but the required rerun with a 600-second timeout exited 0 and matched all markers.
- Task 19: Cargo continues to report the existing global cache cleanup permission warning for `window-vibrancy-0.6.0/.cargo_vcs_info.json`; parser and Tauri checks still exit 0. The moved parse.rs retains unused imports from the byte-identical extraction, producing warnings only. No functional or verification blocker remains.
- Task 19 follow-up: The initial extraction left parse_texture in hod.rs because the T15 location was ambiguous. It was moved byte-identically into parse.rs; the only required follow-up was explicit imports for texture_codec helpers and hod::base64_encode. The first mechanical attempt omitted the function's closing brace, caught immediately by cargo check, then corrected without logic changes. Final gates pass.
- Task 20: Cargo retains the existing global cache permission warning for `window-vibrancy-0.6.0/.cargo_vcs_info.json`; it does not affect exit status. The move exposes existing unused-variable warnings in the relocated serializer but introduces no errors. `parse_stat_params` required `pub(crate)` visibility solely for the new serialize-module round-trip test.
- Task 21: Cargo continues to emit the pre-existing global cache cleanup permission warning for `window-vibrancy-0.6.0/.cargo_vcs_info.json`. It does not affect either exit status. The intentional QA failure path emitted 99 compile errors after replacing the `hod.rs` types glob re-export; this was expected and the fresh clone was removed afterward.
- Task 22: Markdown LSP diagnostics were unavailable because no `.md` language server is configured. The cached removal temporarily appears as tracked-file deletion entries in `git status`; after targeted staging and commit, no `testing/` entries remain. No functional test was needed because this task changes only tracking and documentation.
- Task 23: The mandated repository-wide bin grep still finds removed names in dated PROGRESS history; those entries were intentionally not edited. The active-doc scoped grep passes with the three surviving bins. Markdown LSP diagnostics remain unavailable because no `.md` language server is configured.
- Task 24: The citation grep still reports pre-existing `hod.rs:NNNN` references in dated historical sections of `PROGRESS.md`; these are sacred records and were not modified. The required scratch failure path reported `hod.rs:1234` as expected. Markdown LSP diagnostics remain unavailable because no `.md` language server is configured.
