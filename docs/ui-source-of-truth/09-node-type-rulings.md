# 09 Node Type Rulings

## Scope

This spec is the editable ruling matrix for what each UI node type is allowed to be and do. Use it when changing creation, tree rendering, selection, inspector behavior, transforms, rename/delete, or validation.

Primary sources:

- `src/components/HierarchyTree.tsx`
- `src/components/Inspector.tsx`
- `src/components/Viewport.tsx`
- `src/App.tsx`
- `src/components/AnimationDock.tsx` for animation/keyframe nodes
- `src-tauri/src/lib.rs` and `parser/src/hod.rs` for persistence details when needed

## How To Use This File

- Treat each row as a ruling that can be revised by humans.
- If a human changes a ruling here, update the app to match the ruling.
- If current app behavior differs from this file and no human explicitly changed the ruling, inspect source and ask before changing behavior.
- Keep wording precise: distinguish a real HOD node, a UI-only assembly row, and a visual helper row.

## Ruling Columns

- **Tree Type**: `selectedNode.type` or UI grouping type.
- **Persistent Data**: model collection or backing HOD chunk.
- **Has Transform**: whether this type owns transform-like data.
- **Transform Owner**: exact field or backing node that controls position/orientation.
- **Can Have Children**: whether users may attach hierarchy children beneath it.
- **Required Children**: children that must exist for a valid template/assembly.
- **Optional Children**: allowed extra child nodes or attachments.
- **Rename Rule**: where rename is allowed and what must cascade.
- **Delete Rule**: where delete is allowed and what must cascade.
- **Enforcement**: current code or expected enforcement point.

## Basic Node Rulings

| Tree Type | Persistent Data | Has Transform | Transform Owner | Can Have Children | Required Children | Optional Children | Rename Rule | Delete Rule | Enforcement |
|---|---|---:|---|---|---|---|---|---|---|
| `joint` | `model.joints[]` / HIER | Yes | `joint.local_transform`; optional raw `position/rotation/scale` may exist | Yes | None | any type of node / assembly | Allowed for ordinary joints; must update child parents and attached references. Forbidden when the joint belongs to a protected assembly subtree. | Allowed for ordinary joints; deletes the full descendant subtree and attached data. Forbidden when the joint belongs to a protected weapon assembly subtree (must delete via assembly). | `HierarchyTree.tsx`, `Inspector.tsx`, `App.tsx`, `Viewport.tsx` |
| `marker` | `model.markers[]` / MRKS | Yes | `marker.position` and `marker.rotation`; parent can only be `marker.parent_joint` | No | None | None | Allowed; must preserve parent link unless explicitly changed. | Allowed; removes marker only. | `HierarchyTree.tsx`, `Inspector.tsx`, `App.tsx` |
| `mesh` | `model.meshes[]` / MULT mesh data | Inherits parent; mesh vertices have geometry positions | `mesh.parent_name` determines hierarchy parent; geometry positions live in mesh parts | No | None | Materials/textures through mesh parts | Allowed for mesh name; parent references are separate. | Allowed for selected mesh LOD part. | `HierarchyTree.tsx`, `Inspector.tsx`, `Viewport.tsx` |
| `navlight` | `model.nav_lights[]` / NAVL plus same-name HIER joint | Yes, through backing joint (HOD node will have its own transform) | NAVL stores light parameters only; same-name `joint.local_transform` stores position/transform | No, the navlight node should never have children. (On load, any children are automatically decoupled into proxy joints to preserve world transform) | Same-name HIER joint should exist for transform placement | Light parameters: section, size, phase, frequency, style, color, distance, sprite visibility, high-end-only | Allowed from NavLight row; must rename NAVL data and same-name joint together. | Allowed from NavLight row; removes NAVL data and same-name joint. | `HierarchyTree.tsx`, `Inspector.tsx`, `App.tsx` |
| `dockpath` | `model.dockpaths[]` / DOCK | Yes, through path points and parent | `dockpath.parent_name`; each dockpoint owns `position` and `rotation` | Has dockpoint rows, not arbitrary hierarchy children | At least one dockpoint when created by current UI | Dockpoints | Allowed for dockpath name; point names are derived. | Allowed; removes the dockpath and all dockpoints. | `HierarchyTree.tsx`, `Inspector.tsx` |
| `dockpoint` | Entry inside `dockpath.points[]` | Yes | `dockpoint.position` and `dockpoint.rotation` | No | None | None | Not independently renamed; name is derived as `dockpath:index`. | Removable from the Dockpath inspector; deleting a dockpath removes all of its dockpoints. | `HierarchyTree.tsx`, `Inspector.tsx`, `App.tsx` |
| `collision` | At most one `model.collision_meshes[]` entry / COLD+KDOP, normalized as `Root` | Yes | `collision.center`, extents, radius; `collision.mesh.parent_name` is normalized to `Root` for editor hierarchy placement | No | None | Source mesh(es) for convex hull generation and auto-bounds calculation | Forbidden. COL nodes are engine-defined and must remain named `Root`; context menu must not expose Rename. | Allowed; removes the single collision hull only, and save/import must not recreate it unless the user adds one. | `HierarchyTree.tsx`, `Inspector.tsx`, `Viewport.tsx`, `parser/src/hod.rs`, `parser/src/dae.rs`, `src-tauri/src/lib.rs` |
| `engine_burn` | `model.engine_burns[]` / BURN | Inherits parent plus plume vertices | `engine_burn.parent_name`; plume vertices define burn shape | No | Backing parent joint/nozzle should exist for placement | Vertices, divisions, flames | Allowed; must update burn name and same-name/backing joint if applicable. | Allowed; removes burn plume only unless deleting its parent subtree. | `HierarchyTree.tsx`, `Inspector.tsx`, `App.tsx` |
| `engine_glow` | `model.engine_glows[]` / GLOW | Inherits parent plus mesh | `engine_glow.parent_name`; embedded mesh geometry defines glow shape | No | Parent joint should exist for placement | Embedded mesh | Allowed if UI exposes it; must preserve parent link. | Delete support should remove glow only unless deleting parent subtree. | `HierarchyTree.tsx`, `Inspector.tsx`, `Viewport.tsx` |
| `engine_shape` | `model.engine_shapes[]` / ETSH | Inherits parent plus mesh | `engine_shape.parent_name`; embedded mesh geometry defines shape | No | Parent joint should exist for placement | Embedded mesh | Allowed if UI exposes it; must preserve parent link. | Delete support should remove shape only unless deleting parent subtree. | `HierarchyTree.tsx`, `Inspector.tsx`, `Viewport.tsx` |
| `material` | `model.materials[]` | No | None | No | None | Texture slots/maps | Allowed in material inspector; mesh material indices must remain valid. | Allowed; mesh part material indices must be remapped. | `HierarchyTree.tsx`, `Inspector.tsx` |
| `keyframe` | `model.animations[].tracks[].keyframes[]` | Yes, as animation delta/key data | Keyframe `position`, `rotation`, `rotation_euler`, and `scale` fields | No | Animation track and joint must exist | None | Not a hierarchy rename target. | Delete/edit through animation UI only. | `AnimationDock.tsx`, `Inspector.tsx`, `Viewport.tsx` |

## Template And Assembly Rulings

| UI Assembly Type | Persistent Data | Has Transform | Transform Owner | Can Have Children | Required Children / Template | Optional Children | Rename Rule | Delete Rule | Enforcement |
|---|---|---:|---|---|---|---|---|---|---|
| `weapon_group` | UI-only row representing `Weapon_*` HIER subtree | Yes | `${base}_Position.local_transform` | Yes, through real child joints under the assembly subtree | `Position`, `Direction`, primary `Muzzle` (`${base}_Muzzle`), `Rest`; additional muzzles `${base}_Muzzle1`, `${base}_Muzzle2`, ... are optional | Extra joints and attachments under any assembly joint | Rename only through assembly row/inspector; must cascade to all assembly joints but not the rest of child nodes, also make sure not to show the new name on child nodes in the node tree when changed, renaming must always respect the naming conventions of all weapon assembly joint nodes. Direct subnode rename is forbidden (the Context Menu is natively blocked for these subnodes). | Delete only through assembly row; must delete full assembly subtree and attached data. Direct subnode delete is forbidden (the Context Menu is natively blocked for these subnodes). | `HierarchyTree.tsx`, `Inspector.tsx`, `Viewport.tsx`, `App.tsx` |
| Slave barrel assembly (`${main}Slave`, `${main}Slave1`, ...) | UI-only `weapon_group` row for a full parallel weapon chain | Yes | `${slaveBase}_Position.local_transform` | Yes, through its own child joints | `Position`, `Muzzle`; optional `Direction`, `Latitude`, `Rest`, `Barrel` | Same as a normal weapon assembly | Ordinary assembly rename; renaming the main weapon also cascades to `${main}SlaveN` chains | Deleting the main weapon also deletes its slave chains | `HierarchyTree.tsx`, `Inspector.tsx`, `weapon_utils.ts` |
| Turret assembly | `turret_group` assembly row (turret-template joints are currently grouped by the weapon-group name matcher) | Yes | `${base}_Position.local_transform` | Yes, through real child joints under the assembly subtree | Template creates `Position`, `Direction`, `Latitude`, `Muzzle` (child of `Latitude`), `Rest`; additional muzzles optional. `Barrel` is optional/unverified and not created by default. | Extra joints and attachments under any assembly joint; optional `Barrel` | Same as `weapon_group`. | Same as `weapon_group`. | `HierarchyTree.tsx`, `Inspector.tsx`, `Viewport.tsx`, `App.tsx` |
| `hardpoint_group` | UI-only row representing `Hardpoint_` HIER subtree | Yes | `${base}_Position.local_transform` | Yes, through real child joints under the assembly subtree | `Position`, `Direction`, `Rest` | Extra joints and attachments under any assembly joint | Same as `weapon_group`. | Same as `weapon_group`. | `HierarchyTree.tsx`, `Inspector.tsx`, `Viewport.tsx`, `App.tsx` |
| `capture_point_group` | UI-only row representing `CapturePoint` HIER subtree | Yes | Base joint local transform | Yes, through real child joints under the assembly subtree | Base, `Heading`, `Left`, `Up` | Extra joints and attachments under any assembly joint | Same as `weapon_group`. | Same as `weapon_group`. | `HierarchyTree.tsx`, `Inspector.tsx`, `Viewport.tsx`, `App.tsx` |
| `repair_point_group` | UI-only row representing `RepairPoint` HIER subtree | Yes | Base joint local transform | Yes, through real child joints under the assembly subtree | Base, `Heading`, `Left`, `Up` | Extra joints and attachments under any assembly joint | Same as `weapon_group`. | Same as `weapon_group`. | `HierarchyTree.tsx`, `Inspector.tsx`, `Viewport.tsx`, `App.tsx` |
| `salvage_point_group` | UI-only row representing `SalvagePoint` HIER subtree | Yes | Base joint local transform | Yes, through real child joints under the assembly subtree | Base, `Heading`, `Left`, `Up` | Extra joints and attachments under any assembly joint | Same as `weapon_group`. | Same as `weapon_group`. | `HierarchyTree.tsx`, `Inspector.tsx`, `Viewport.tsx`, `App.tsx` |

## Template Children And Offsets

Offsets are listed as local transform translation `[x, y, z]`. If `HierarchyTree.tsx` creation offsets and `Inspector.tsx` repair offsets disagree, treat it as an open ruling and reconcile before changing behavior.

| Template | Children Created By Add Node | Creation Offsets | Inspector Repair Offsets | Ruling Status |
|---|---|---|---|---|
| Weapon assembly | `${base}_Position`, `${base}_Direction`, `${base}_Muzzle`, `${base}_Rest` | Position `[0,0,0]`; Direction `[0,5,0]` (child of Position); Muzzle `[0,0,5]` (child of Position); Rest `[0,0,5]` (child of Position). Extra muzzles are parented to `${base}_Latitude` when present, otherwise Position. | Same as creation | Consistent. |
| Turret assembly | `${base}_Position`, `${base}_Direction`, `${base}_Latitude`, `${base}_Muzzle`, `${base}_Rest`; `Barrel` only when the user enables it | Position `[0,0,0]`; Direction `[0,5,0]` (child of Position); Latitude `[0,0,0]` (child of Position); Muzzle `[0,0,7.64]` (child of Latitude); Rest `[0,0,10]` (child of Position). Optional Barrel `[0,0,0]` (child of Latitude). | No separate turret repair template; missing joints are added through the weapon assembly inspector. | Consistent for creation. Vanilla `Direction`/`Rest` offsets vary per ship (e.g. `kus_defender.hod` uses `[0,10,0]` and `[0,0,10.117]`); the editor values are defaults, not canonical replacements. Barrel is optional and its runtime behavior is unverified. |
| Hardpoint | `${base}_Position`, `${base}_Direction`, `${base}_Rest` | Position `[0,0,0]`; Direction `[0,5,0]`; Rest `[0,0,5]` | Same as creation | Consistent. |
| Capture point | `${base}`, `${base}_Heading`, `${base}_Left`, `${base}_Up` | Base `[0,0,0]`; Heading `[0,0,5]`; Left `[5,0,0]`; Up `[0,5,0]` | Same as creation | Consistent. |
| Repair point | `${base}`, `${base}_Heading`, `${base}_Left`, `${base}_Up` | Base `[0,0,0]`; Heading `[0,0,5]`; Left `[5,0,0]`; Up `[0,5,0]` | Same as creation | Consistent. |
| Salvage point | `${base}`, `${base}_Heading`, `${base}_Left`, `${base}_Up` | Base `[0,0,0]`; Heading `[0,0,5]`; Left `[0,0,5]`; Up `[0,5,0]` | Same as creation | Consistent. |
| Engine nozzle | Nozzle joint plus burn plume | Nozzle joint `[0,0,0]`; burn plume vertices along negative Z | Not an assembly repair template | Consistent as a creation template, not a grouped assembly. |

## Validation Rulings

Current hierarchy diagnostics cover only these checks:

- Assemblies are incomplete and not following the related templates (missing nodes).
- Weapon assembly muzzle naming is non-standard (not `_Muzzle`/`_MuzzleN`) or is parented outside `${base}_Latitude`/`${base}_Position` (direct `Root` parenting is accepted). Numbering may be non-contiguous.
- A slave barrel assembly (`${main}SlaveN`) is missing its `Position` or `Muzzle`.
- A `Barrel` joint is present in a weapon assembly (reported as info: optional and unverified).
- Engine burn count at or above `12`.
- No collision meshes if mesh available.

Diagnostics are advisory unless save/load code explicitly blocks an operation.

## Human Rulings Decided

- All point groups became tree-level pseudo assembly nodes like `weapon_group`.
- Point-group subnodes received the same context-menu protection as weapon assembly subnodes.
- Creation offsets are authoritative for repair offsets.
- Turret assemblies have a distinct `turret_group` type.
- Every functional node with a same-name joint requires that joint to exist before save (validation on save must be there).
- NavLight cannot have children. During load, any children are automatically decoupled and reparented via proxy joints to maintain position.
- COL/collision is a singleton. Loading HOD 1.0, HOD 2.0, or DAE may preserve an existing collision, but parser/import/save/UI normalization must keep only one collision named `Root`; users may delete it but may not rename it.
