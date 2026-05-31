# 10 State Management And Performance

## Scope

This spec defines the boundaries between global UI state and local component state, as well as rules for maintaining high performance with large HOD files (which can have thousands of joints and meshes).

## State Boundaries

- **Global State**: `App.tsx` owns the canonical `model` state (parsed from the HOD file). No other component should duplicate this state as an independent source of truth. Mutations must propagate upwards to `App.tsx` which will distribute changes downwards.
- **Local State**: UI toggles, expand/collapse states, and ephemeral modal states should live as close to their usage as possible (e.g., inside `HierarchyTree.tsx` or `Inspector.tsx`).

## Performance Rules

- **Memoization**: Heavy calculations, such as traversing the entire joint tree for diagnostics or filtering nodes, **must** be wrapped in `useMemo` hooks.
- **Avoid Extraneous Renders**: Do not trigger top-level React renders for high-frequency events (e.g., mouse dragging in the viewport or hovering). Use refs (`useRef`) and direct DOM manipulation or localized state updates for such interactions.
- **Large Lists**: If the hierarchy tree becomes extremely large, consider windowing (virtualization) solutions.

## Best Practices

- Always prefer small, focused components over massive monolithic files where possible.
- If a component does not need to re-render when the `model` changes (e.g., static toolbars), use `React.memo` to skip reconciliation.
