# 02 Node Creation And Types

## Scope

This spec covers supported node creation flows, node types, default names, and template-generated structures.

Primary sources:

- `src/components/HierarchyTree.tsx`
- `src/components/Viewport.tsx` for model interfaces/rendering expectations
- `README.md` only as secondary context for template intent

## Add Node Modal

Node creation is driven by the Add Node modal in `HierarchyTree.tsx`. The modal supports these `addNodeType` values:

- `joint`
- `marker`
- `navlight`
- `dockpath`
- `collision`
- `weapon_template`
- `turret_template`
- `engine_nozzle`
- `mesh`
- `repair_point_template`
- `capture_point_template`
- `hardpoint_template`
- `salvage_point_template`

Default names are generated from current model counts and naming conventions in `HierarchyTree.tsx`. Agents must copy defaults from code when updating this spec.

Some node types generate their name automatically and therefore hide the Node Name field: `collision`, `engine_nozzle`, `repair_point_template`, `capture_point_template`, and `salvage_point_template`. The Add Node button must be enabled for those types without any user-entered name, because `handleAddNode` supplies the generated name. Types that still show the Node Name field require a non-empty value before submission.

## Basic Nodes

Joint creation adds a joint under the selected parent with an identity transform.

Marker creation adds a marker associated to the selected parent joint.

Mesh creation adds a mesh/LOD entry associated to the selected parent.

NavLight creation creates nav light data and an associated position joint when required by current code.

Dockpath creation adds a dockpath with dockpoint data according to current `handleAddNode` behavior.

Collision creation creates/restores the single engine-defined COL entry. The user does not choose a name or parent: the editor always creates it as `Root`, blocks creation when a collision already exists, and keeps save/import normalization aligned with that rule.

Engine nozzle creation adds an engine burn/nozzle node and its associated joint data according to current code.

## Template Nodes

Template node types create multi-joint structures with standard offsets and parent/child relationships.

Current template families are:

- Weapon assembly.
- Turret assembly.
- Hardpoint.
- Capture point.
- Repair point.
- Salvage point.

Agents must treat `handleAddNode` in `HierarchyTree.tsx` as canonical for exact joint names, offsets, parent names, and optional components.

UI Tree should show those structures as a main Assembly node (not specifically a HOD node) to represent the assembly subtree, with the position of the node in HOD and render being the main "*_Position" joint node. While they children nodes can be editable and can add other nodes inside as they are still joint nodes. The rules for renaming should be for all the nodes in the assembly, but can only rename via the assembly. The only way of deleting these set of nodes is by deleting the assembly, it should delete all child nodes.

## Node Identity

Names are significant because many collections reference joints by string name. Rename and delete behavior must maintain or intentionally update these cross-references.

Duplicate names are strictly restricted across the entire tree during node creation and renaming. The UI enforces uniqueness for all newly created Nodes or Meshes to prevent internal naming collisions. The single collision node is the exception: it must be named `Root` even though the root joint also uses that display name, because the node type disambiguates it.
