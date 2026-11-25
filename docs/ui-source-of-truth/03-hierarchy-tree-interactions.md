# 03 Hierarchy Tree Interactions

## Scope

This spec covers hierarchy rendering, selection, search, collapse, visibility, drag/drop, and context-menu behavior.

Primary sources:

- `src/components/HierarchyTree.tsx`
- `src/App.tsx` for `onReParentNode`
- `src/components/Viewport.tsx` for visibility effects

## Rendered Node Types

The hierarchy tree renders:

- Root and root-level joints.
- Child joints.
- Weapon group folders.
- Markers.
- Mesh LOD parts.
- Nav lights.
- Engine burns.
- Engine glows.
- Engine shapes.
- Collision hulls.
- Dockpaths and dockpoints.

Pure navlight position joints may be hidden from the general joint list unless they have meaningful children/attachments. Weapon assembly subnodes are hidden from the normal root/general joint list and shown under their weapon group folder.

## Selection

Clicking a tree row sets `selectedNode` with the row's logical type and name. Selection drives Inspector and Viewport behavior.

Weapon group rows use type `weapon_group`. Weapon subnodes rendered inside the group remain type `joint` for selection and viewport/inspector compatibility.

## Search And Collapse

Search filters tree rows using direct names and recursive descendant matches. Collapsible rows use `collapsedJoints` state. Weapon group collapse keys are prefixed with `weapon_group:`.

Hierarchy tab search row has **Collapse all** / **Expand all** buttons. Collapse all sets `collapsedJoints` for every joint that has children (except `Root`, so Root's direct children stay visible) and every weapon group key; Expand all clears it.

## Copy / Cut / Paste

Source: `HierarchyTree.tsx` (`keyHandlerRef`, module-level `nodeClipboard`), pure logic in `src/node_clipboard.ts`.

- Ctrl/Cmd+C copies the selected `joint`/`engine_nozzle` subtree (the joint, all descendant joints, and meshes, markers, nav lights, engine burns/glows/shapes and dockpaths parented to them). Root and weapon assembly subnodes are not copyable (same guard as delete). The clipboard is app-internal, not the OS clipboard.
- Selecting an assembly folder row (`weapon_group`, `hardpoint_group`, capture/repair/salvage point groups) and pressing Ctrl/Cmd+C copies the **whole assembly**: the main chain, all `Slave` chains, their descendants and attached data (`copyAssembly`). On paste the assembly gets a new numbered base (`Weapon_Gun0` -> `Weapon_Gun1`, using max existing number for that prefix + 1; `Foo` -> `Foo1`), keeping suffixes (`_Position`, `_Muzzle`, `Slave`...). Every chain root is parented to the paste target; local transforms are preserved. After Cut, pasting restores the original base name (a move). Non-joint names (meshes, etc.) that still collide get `_N`. Known limit: an assembly nested *inside* a plain joint subtree copy is not renumbered (its joints get `_N` suffixes and may stop grouping).
- Ctrl/Cmd+X copies and then removes the subtree and everything attached (descendants are removed, not re-parented, so cut+paste is a move).
- Ctrl/Cmd+V pastes under the selected joint, or under the root joint if the selection is not a plain joint. Local transforms are preserved. Names already in use get a `_N` suffix (same N for the whole paste); unused names are kept. The pasted root becomes selected. Each action is one undo step.

## Visibility

Eye toggles update `visibleMeshes` keys. Toggling a joint propagates visibility to descendant keys. Toggling a weapon group propagates visibility to all component joints and their descendants.

Visibility keys use prefixes such as `joint:`, `marker:`, `navlight:`, `engine_burn:`, `engine_glow:`, `engine_shape:`, `collision:`, `dockpath:`, and `weapon_group:`.

## Drag And Drop

Tree rows set drag data with node name and node type. Drops call `onReParentNode` from props.

`App.tsx` owns actual reparenting behavior. Current app-side reparenting supports joints, markers, meshes, engine burns, and navlights. Agents must check `App.tsx` before claiming support for other dropped types.

Root is not draggable.

## Context Menu

Right-click opens a portal-based context menu at cursor coordinates. The menu contains Rename and, when allowed, Delete for standard tree nodes.

For the Textures list in the Materials tab, the context menu provides options to:
- Toggle Y-Flip (inverts `legacy_storage_y_flipped` and instantly updates the WebGL `flipY` state for preview).
- Remove Texture.

Weapon assembly subnodes must not open the right-click menu. This is enforced by the shared protected-subnode guard in `HierarchyTree.tsx`.

The weapon assembly/group row itself must keep its context menu so the assembly can be renamed or deleted as a group.
