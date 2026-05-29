# HODEditorJS Knowledge Graph & Documentation Hub

Welcome to the central documentation hub for HODEditorJS. This document connects the dots between our various documentation directories, clearly defining what is an active source of truth and what is an archived historical log.

## 🧭 Navigation Graph

### 1. 🤖 Agent Instructions (START HERE)
If you are an AI agent or a new human developer, you must read these first.
- [AGENTS.md](../AGENTS.md) - The strict rules of engagement for modifying this repository.
- [docs/agent-handbook/README.md](agent-handbook/README.md) - The central handbook on architecture (React, Tauri, Rust), UI rules, and backend parsing rules.

### 2. UI Source of Truth 🖥️
Documentation defining the frontend behaviors, node trees, and the React architecture.
- [UI Source of Truth Directory](ui-source-of-truth/)
- Start with [01-app-shell-and-file-flow.md](ui-source-of-truth/01-app-shell-and-file-flow.md)

### 3. Backend & Format Specifications ⚙️ (Active Source of Truth)
This directory contains the **current**, active specifications for the `.HOD` formats and `.DAE` pipelines. If you need to understand how bytes are structured or how pipelines work *today*, read these:
- [HOD 2.0 Reverse Engineering Knowledge Base](hod2-reverse-engineering/hod2_reverse_engineering_knowledge_base.md) - The foundational reverse-engineering knowledge (Flat file structure, Endianness inconsistencies, MULT padding). *(Migrated from legacy `agents_info/` directory).*
- [HOD 2.0 Creation Specification](hod2-reverse-engineering/hod2-creation-specification.md) - How we construct HOD 2.0 files.
- [POOL Chunk Spec](hod2-reverse-engineering/pool-chunk-specification.md) - How vertices, faces, and materials are packed.
- [HOD 1.0 vs HOD 2.0 Comparison](hod2-reverse-engineering/hod1-vs-hod2-comparison.md) - Key differences between legacy and modern formats.
- [DAEnerys Pipeline Spec](hod2-reverse-engineering/daenerys-obj-to-dae-pipeline.md) - How DAEnerys generates `.DAE` nodes (`ROOT_LOD[0]`, `JNT`, etc.) and how our parser reads them.
- [Testing Guide](hod2-reverse-engineering/testing-guide.md) - How to run the `cargo run --bin` tests.

### 4. Progress Tracking 📈
This single file is the source of truth for current project status, blockers, and completed milestones.
- [**PROGRESS.md**](hod2-reverse-engineering/PROGRESS.md)

### 5. Historical Archive 🗄️ (Stale/Logs)
During the reverse engineering of HODOR's MS-XCA LZ77 compression and Homeworld's tangents, agents produced dozens of analysis documents, phase summaries, and fix plans. 
These have been safely moved here to prevent context confusion, along with the legacy `agents_info/` walkthroughs. **Do not treat these as active instructions.**
- [Archived Logs Directory](hod2-reverse-engineering/archive_logs/)

---
*Note to Agents: When updating documentation, ALWAYS update `PROGRESS.md` first. If you create a new specification document, you must link it in this Knowledge Graph!*
