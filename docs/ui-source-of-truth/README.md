# UI Source Of Truth

This directory is the agent-facing source of truth for how the current UI is expected to behave. These documents describe the behavior implemented in the React/Tauri app today, not aspirational behavior from TODOs.

## Agent Rules

- Read the narrowest spec before editing related UI code.
- Verify behavior against the listed source files before changing it.
- If a spec and source disagree, treat source as canonical unless the task explicitly asks to change behavior.
- When changing behavior, update the relevant spec in the same work item.
- Do not use `README.md`, `TODO.md`, or `docs/hod2-reverse-engineering/archive_logs/*` as authoritative without checking source code.
- Do not broaden behavior across other assembly types unless the user asks for that scope.

## Specs

- [01 App Shell And File Flow](./01-app-shell-and-file-flow.md)
- [02 Node Creation And Types](./02-node-creation-and-types.md)
- [03 Hierarchy Tree Interactions](./03-hierarchy-tree-interactions.md)
- [04 Rename Delete Reparent](./04-rename-delete-reparent.md)
- [05 Inspector Behavior](./05-inspector-behavior.md)
- [06 Viewport Interactions](./06-viewport-interactions.md)
- [07 Validation Warnings](./07-validation-warnings.md)
- [08 Assemblies And Groups](./08-assemblies-and-groups.md)
- [09 Node Type Rulings](./09-node-type-rulings.md)

## Source Priority

1. Current component code in `src/`.
2. Tauri commands in `src-tauri/src/lib.rs` when UI invokes backend actions.
3. Existing docs such as `README.md`, `TODO.md`, `agents.md`, and `docs/hod2-reverse-engineering/archive_logs/*` only as secondary context.

## Verification Pattern

For any UI behavior change, agents should record:

- Given: initial model/UI state.
- When: user action or callback.
- Then: required UI/model result.
- Source: component/function checked.
- Verification: build, diagnostics, test, or manual surface check.
