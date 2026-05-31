# 13 Undo Redo History

## Scope

This document sets the architectural foundation required to support an Undo/Redo history stack in the future.

## Command Pattern

Currently, state mutations often directly modify deeply nested properties of the `model` state. To support Undo/Redo:

1. **Serializable Actions**:
   - All mutations to the `model` state should ideally route through a serializable command or action (e.g., `RENAME_NODE`, `MOVE_NODE`, `DELETE_NODE`, `UPDATE_TRANSFORM`).

2. **Immutability**:
   - When an action is dispatched, the `model` should be updated immutably (or via a controlled immer-like proxy) so that previous snapshots can be stored in a history stack without deep-cloning the entire HOD model every time.

3. **History Stack Limitation**:
   - Because HOD models can be large (many meshes, vertices, and animations), the history stack should only track structural changes (hierarchy, names, transforms). Vertex-level edits or texture replacements might need specialized diffing or limits to prevent memory bloat.

## Future Implementation

When history is fully implemented, `App.tsx` will maintain a `past`, `present`, and `future` state stack. Components must not bypass this stack by mutating `present` directly, or history integrity will be lost.
