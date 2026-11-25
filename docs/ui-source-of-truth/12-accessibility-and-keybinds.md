# 12 Accessibility And Keybinds

## Scope

This specification establishes a central standard for keyboard shortcuts and accessibility within the HODEditorJS interface.

## Standard Keybinds

To ensure a predictable user experience, the following keyboard shortcuts should be universally supported and respected across the application:

Implemented (source: `App.tsx` global listener, `HierarchyTree.tsx` `keyHandlerRef`):

- **Ctrl/Cmd + S**: Save (App). Works even while a text field is focused.
- **Ctrl/Cmd + Z**: Undo. **Ctrl/Cmd + Y** or **Ctrl/Cmd + Shift + Z**: Redo (App). Ignored while a text field is focused so native text undo wins.
- **Escape**: Clears the selection (App).
- **F2**: Rename the selected node (Hierarchy tab only; uses `handleRenameNode`).
- **Delete / Backspace**: Delete the selected node after a `window.confirm` (Hierarchy tab only; types joint, engine_nozzle, marker, mesh, navlight, dockpath, collision, engine_burn; protected nodes ignored).
- **Ctrl/Cmd + C / X / V**: Copy / cut / paste a joint subtree (see `03-hierarchy-tree-interactions.md`). Not triggered when text is selected for native copy.

Tree/App key handlers all return early when the event target is an `input`, `textarea`, `select` or contenteditable element (except Ctrl+S).

## UI Navigation

- Context menus should be navigable via keyboard arrows if possible, or easily dismissible.
- Ensure that forms and inputs inside `Inspector.tsx` are reachable via `Tab` indexing.
- When an input field in the Inspector has focus, hotkeys like `Delete` must be localized to the input field and not trigger a node deletion in the Hierarchy Tree.

## Focus Management

- After a node is deleted, focus should ideally fallback to its parent node or be cleared, preventing actions on a null selection.
- Modals should trap focus while open.
