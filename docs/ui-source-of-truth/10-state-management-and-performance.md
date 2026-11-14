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
- **Tauri IPC & Loading Screens**: Tauri's `invoke` API serializes payloads to JSON. When passing massive objects (like the entire HOD `model`), `JSON.stringify` runs synchronously on the main thread, completely freezing the UI. If you trigger a loading state (`setIsLoading(true)`) immediately before an `await invoke(...)`, React's batched state updates will fail to paint the loading screen before the freeze occurs. **Rule:** You MUST wrap heavy `invoke` calls in a `setTimeout(() => { ... }, 50)` block to yield the main thread and guarantee the browser paints the loading screen overlay.

- **Viewport visibility is not a rebuild input**: `Viewport.tsx` builds every object once per model/material change and tags it with `userData.visKey`; visibility toggles only flip `.visible` in a dedicated effect. Never add `visibleMeshes` back to the scene-rebuild effect dependencies — that rebuilt the scene and re-uploaded every texture on each eye/layer click (~1 s on large ships).
- **Texture reuse**: GPU textures are cached across rebuilds (`textureCacheRef`, keyed `${name}:${yFlipped}`) and only entries unused by the new scene are disposed.
- **No IPC logging in hot paths**: `log_event` is a synchronous Tauri command (file append). Do not call it per joint/part/frame or inside selection/transform paths; one summary line per operation is the limit.
- **Indexed lookups in per-node renderers**: `HierarchyTree.tsx#renderJointNode` and the viewport matrix helpers must use the per-model indexes (`getTreeIndex`, `jointCacheRef`) instead of `model.joints.find/filter` per node. Indexes are keyed by `model` object identity, so model edits must keep producing new model objects.
- **Animation evaluation is gated**: the render loop calls `evaluateAnimation` only while playing/dragging or when the scene version, animation, time or selection changes.

## Best Practices

- Always prefer small, focused components over massive monolithic files where possible.
- If a component does not need to re-render when the `model` changes (e.g., static toolbars), use `React.memo` to skip reconciliation.
