# 04 Rename Delete Reparent

## Scope

This spec covers structural mutations: rename, delete, context-menu availability, and reparenting.

Primary sources:

- `src/components/HierarchyTree.tsx`
- `src/App.tsx`
- `src/components/Inspector.tsx` for overlapping group rename/repair behavior

## Protected Nodes

The root node is not deletable.

Weapon assembly subnodes are protected from:

- Opening the right-click context menu.
- Direct delete through `isNodeDeletable` and `handleDeleteNode`.
- Direct rename through `handleRenameNode`.

The weapon assembly/group row itself is not a protected subnode. It may be renamed or deleted through the context menu.

## Delete Confirmation

All context-menu deletes route through `handleDeleteNode`, which owns confirmation.

Joint subtree deletes show a stronger warning when the joint has descendant joints or attached data. Weapon group deletes show a group-level warning. Simple leaf deletes use a simpler confirmation.

Do not add a second confirmation in menu rendering.

## Joint Delete

Deleting a joint deletes the full recursive joint subtree rooted at that joint. The operation removes descendant joints and attached data that references any deleted joint.

Current cascade removes:

- Joints.
- Meshes.
- Markers.
- Nav lights whose associated joint name is removed.
- Engine burns.
- Engine glows.
- Engine shapes.
- Collision meshes whose name or mesh parent belongs to the deleted subtree.
- Dockpaths.

Joint delete must not reparent child nodes to the deleted node's parent.

Engine nozzle rows use the `engine_nozzle` selection type even though they are backed by a joint. The context-menu delete for an engine nozzle must route through the same deletion branch as `joint`; otherwise the menu closes and the model is left unchanged. That branch also removes engine burns, glows, and shapes whose `parent_name` matches the nozzle and removes the matching nav light.

Deleting a collision row removes the single `model.collision_meshes[]` entry. Save/import code must not synthesize a replacement collision after deletion.

## Weapon Group Delete

Deleting a weapon group collects all group joints, then expands each group joint into its full subtree. It removes the same attachment types as ordinary joint subtree deletion.

Weapon group delete must not only delete prefix-matching joints, and must not reparent group meshes/markers to `Root`.

## Mesh Delete

Mesh rows are grouped by normalized base name, stripping trailing `_lod_#` and `_LOD#` suffixes.

Deleting a mesh group row removes every `model.meshes` entry whose normalized base name matches the selected row, not just one exact LOD key. Delete also clears the base visibility key and each removed LOD visibility key from `visibleMeshes`.

## Rename

Rename uses `window.prompt` and rejects empty/no-op inputs. Duplicate checks scan joints, meshes, nav lights, markers, engine burns, engine glows, engine shapes, collision meshes, and dockpaths.

Joint rename updates child joint parents and attached mesh/marker/engine parent references.

Weapon group rename batch-renames assembly joints by replacing the group prefix and updates affected parent references, mesh parents, and marker parents.

Special wrapped display names such as `NAVL[...]`, `MARK[...]`, `BURN[...]`, `MULT[...]`, `GLOW[...]`, and `SHAP[...]` are normalized before prompting and restored after input.

Collision rows are not rename targets. The context menu hides Rename for `collision`, and defensive rename handling must reject it because COL nodes are engine-defined and must remain named `Root`.

## Reparent

`HierarchyTree.tsx` only emits drag/drop intent. `App.tsx` implements mutation.

Current app-side reparenting:

- Prevents joint self-parenting and descendant cycles.
- Updates joint `parent_name`.
- Updates marker `parent_joint`.
- Updates mesh `parent_name`.
- Updates engine burn `parent_name`.
- Updates navlight-associated joint parent when applicable.

Agents must verify current `App.tsx` before extending reparent support to other node types.
