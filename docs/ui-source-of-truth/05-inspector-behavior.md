# 05 Inspector Behavior

## Scope

This spec covers the right-side inspector for selected nodes, assembly repair, field editing, and import/export controls exposed from the inspector.

Primary sources:

- `src/components/Inspector.tsx`
- `src/components/Viewport.tsx` for model types
- `src/App.tsx` for transform/update callbacks

## Empty State

When no node is selected or no model is loaded, Inspector should show the corresponding empty state rather than editing stale data.

## Assembly Inspector

Inspector recognizes special group selections for weapon, hardpoint, capture, repair, and salvage assemblies.

For assemblies, Inspector shows completion state, missing component rows, and a repair action when required joints are missing. Repair recreates missing joints with default offsets and parent relationships from current code.

Assembly rename behavior must preserve related joint references and avoid duplicate names.

## Joint And Marker Editing

Joint and marker editing updates transform-related fields through the model update callbacks. Transform edits should remain synchronized with viewport gizmo behavior.

Marker edits update marker position/rotation data and parent association according to current Inspector code.

## NavLight Editing

NavLight editor fields include size, phase, frequency, style, distance, color, and enabled/toggle fields currently implemented in `Inspector.tsx`.

NavLight UI edits must update navlight data and any associated joint data through `onModelChange` or the relevant callback path.

## Dockpath And Dockpoint Editing

Dockpath/dockpoint inspector supports editing dockpoint position and properties such as tolerance and max speed according to current code.

Dockpoint selection names are derived from dockpath name and point index.

## Collision Editing

Collision inspector supports editing center/radius/extents and selecting or calculating from a source mesh when current code exposes that action.

Collision mesh parent access may be optional in imported/malformed data; UI code should preserve null-safe handling.

Collision creation/removal controls belong to the COL/collision flow, not engine glow, engine nozzle, engine shape, or generic joint inspectors.

## Mesh Editing And OBJ Flow

Mesh inspector supports editing mesh name, LOD, parent, and materials according to current code. OBJ import/export controls are exposed from Inspector and route through app/Tauri callbacks.

Mesh OBJ import preserves `usemtl` groups as mesh parts. Each `usemtl` name reuses an existing HOD material with the same name (case-insensitive) or creates a placeholder material using the selected LOD's current shader. The mesh inspector also exposes **Import MTL Materials**: `newmtl` names are matched to HOD materials (created if absent), and the `map_Kd` path is matched by filename to textures already loaded in the model, ignoring folders, extensions, known texture-type suffixes, and compression suffixes. MTL import never reads image files from disk and must not overwrite an existing texture assignment when no matching loaded texture exists.

## Engine Nodes

Inspector contains editors for engine burn/glow/shape data. Agents must verify exact fields in `Inspector.tsx` before adding or documenting new engine-specific controls.

Engine Glow LOD rows may expose per-LOD visibility toggles that write through the shared `visibleMeshes` state using `engine_glow:<glow name>` keys, matching Viewport rendering. Mesh and Engine Glow LOD visibility is mutually exclusive per base node: enabling one LOD hides sibling LODs for the same node.

## Materials And Textures

Material and texture editing lives in Inspector and the Hierarchy tree's Materials tab. Do not duplicate material state outside model updates. 
The Hierarchy tree's Materials tab provides a comprehensive list of all loaded materials and textures.
Texture items in this list have a right-click context menu that can toggle the `legacy_storage_y_flipped` property, which directly controls the WebGL `flipY` parameter in the Viewport for accurate in-game orientation previews.
