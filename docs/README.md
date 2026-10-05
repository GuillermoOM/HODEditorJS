# HODEditorJS Knowledge Graph & Documentation Hub

Welcome to the central documentation hub for HODEditorJS. This document connects the dots between our various documentation directories, clearly defining what is an active source of truth and what is an archived historical log.

## 🧭 Navigation Graph

### 1. 🤖 Agent Instructions (START HERE)
If you are an AI agent or a new human developer, you must read these first.
- [AGENTS.md](../AGENTS.md) - The strict rules of engagement for modifying this repository.
- [docs/agent-handbook/README.md](agent-handbook/README.md) - The central handbook on architecture (React, Tauri, Rust), UI rules, and backend parsing rules.
- [Knowledge Graph](README.md) - This documentation hub and its active/archive navigation rules.

### 2. UI Source of Truth 🖥️
Documentation defining the frontend behaviors, node trees, and the React architecture.
- [UI Source of Truth Directory](ui-source-of-truth/)
- [UI Source of Truth Guide](ui-source-of-truth/README.md) - How to use and maintain the frontend behavior specifications.
- Start with [01-app-shell-and-file-flow.md](ui-source-of-truth/01-app-shell-and-file-flow.md)
- [02-node-creation-and-types.md](ui-source-of-truth/02-node-creation-and-types.md)
- [03-hierarchy-tree-interactions.md](ui-source-of-truth/03-hierarchy-tree-interactions.md)
- [04-rename-delete-reparent.md](ui-source-of-truth/04-rename-delete-reparent.md)
- [05-inspector-behavior.md](ui-source-of-truth/05-inspector-behavior.md)
- [06-viewport-interactions.md](ui-source-of-truth/06-viewport-interactions.md)
- [07-validation-warnings.md](ui-source-of-truth/07-validation-warnings.md)
- [08-assemblies-and-groups.md](ui-source-of-truth/08-assemblies-and-groups.md)
- [09-node-type-rulings.md](ui-source-of-truth/09-node-type-rulings.md)
- [10-state-management-and-performance.md](ui-source-of-truth/10-state-management-and-performance.md)
- [11-error-boundaries-and-resilience.md](ui-source-of-truth/11-error-boundaries-and-resilience.md)
- [12-accessibility-and-keybinds.md](ui-source-of-truth/12-accessibility-and-keybinds.md)
- [13-undo-redo-history.md](ui-source-of-truth/13-undo-redo-history.md)

### 3. Backend & Format Specifications ⚙️ (Active Source of Truth)
This directory contains the **current**, active specifications for the `.HOD` formats and `.DAE` pipelines. If you need to understand how bytes are structured or how pipelines work *today*, read these:
- [HOD 2.0 Reverse Engineering Knowledge Base](hod2-reverse-engineering/hod2_reverse_engineering_knowledge_base.md) - The foundational reverse-engineering knowledge (Flat file structure, Endianness inconsistencies, MULT padding). *(Migrated from legacy `agents_info/` directory).*
- [Release Process](RELEASE_PROCESS.md) - Step-by-step documentation covering release builds and publishing.
- [Local Environment](local-environment.md) - Convention and template for untracked machine-specific development profiles.
- [HOD 2.0 Creation Specification](hod2-reverse-engineering/hod2-creation-specification.md) - How we construct HOD 2.0 files.
- [POOL Chunk Spec](hod2-reverse-engineering/pool-chunk-specification.md) - How vertices, faces, and materials are packed.
- [HOD 1.3 vs HOD 2.0 Comparison](hod2-reverse-engineering/hod1-vs-hod2-comparison.md) - Key differences between legacy and modern formats.
- [DAEnerys Pipeline Spec](hod2-reverse-engineering/daenerys-obj-to-dae-pipeline.md) - How DAEnerys generates `.DAE` nodes (`ROOT_LOD[0]`, `JNT`, etc.) and how our parser reads them.
- [Testing Guide](hod2-reverse-engineering/testing-guide.md) - How to run the `cargo run --bin` tests.
- [Testing Fixtures](testing-fixtures.md) - How to obtain the external, untracked fixture corpus.
- [HOD 2.0 Reverse Engineering README](hod2-reverse-engineering/README.md) - Orientation for the active reverse-engineering documentation.
- [Quickstart](hod2-reverse-engineering/QUICKSTART.md) - Fast path for understanding and working with the HOD format tooling.
- [Architecture Overview](hod2-reverse-engineering/architecture-overview.md) - Current parser, model, and save architecture.

### 4. Progress Tracking 📈
This single file is the source of truth for current project status, blockers, and completed milestones.
- [**PROGRESS.md**](hod2-reverse-engineering/PROGRESS.md)
- [Manual Test Log](manual-test-log.md) - Dated manual verification snapshots.
- [Progress Reporting Standard](hod2-reverse-engineering/progress-reporting-standard.md) - Required format for recording project progress.

### 5. Historical Archive 🗄️ (Stale/Logs)
During the reverse engineering of HODOR's MS-XCA LZ77 compression and Homeworld's tangents, agents produced dozens of analysis documents, phase summaries, and fix plans. 
These have been safely moved here to prevent context confusion, along with the legacy `agents_info/` walkthroughs. **Do not treat these as active instructions.**
Archived plans and completed investigations are retained for provenance only and must not be treated as active instructions.
- [Archived Logs Directory](hod2-reverse-engineering/archive_logs/)

---
*Note to Agents: When updating documentation, ALWAYS update `PROGRESS.md` first. If you create a new specification document, you must link it in this Knowledge Graph!*
