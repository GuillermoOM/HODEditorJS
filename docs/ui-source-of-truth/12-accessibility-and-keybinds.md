# 12 Accessibility And Keybinds

## Scope

This specification establishes a central standard for keyboard shortcuts and accessibility within the HODEditorJS interface.

## Standard Keybinds

To ensure a predictable user experience, the following keyboard shortcuts should be universally supported and respected across the application:

- **F2**: Rename the currently selected node.
- **Delete / Backspace**: Delete the currently selected node (with appropriate confirmation warnings).
- **Ctrl + S (Cmd + S on Mac)**: Save the current HOD model.
- **Ctrl + Z / Ctrl + Y**: Undo / Redo (if/when a history stack is implemented).
- **Escape**: Close any active modal, dialog, or cancel a drag/drop operation.

## UI Navigation

- Context menus should be navigable via keyboard arrows if possible, or easily dismissible.
- Ensure that forms and inputs inside `Inspector.tsx` are reachable via `Tab` indexing.
- When an input field in the Inspector has focus, hotkeys like `Delete` must be localized to the input field and not trigger a node deletion in the Hierarchy Tree.

## Focus Management

- After a node is deleted, focus should ideally fallback to its parent node or be cleared, preventing actions on a null selection.
- Modals should trap focus while open.
