# 08 Assemblies And Groups

## Scope

This spec covers semantic groups and assemblies as they appear in the hierarchy and inspector.

Primary sources:

- `src/components/HierarchyTree.tsx`
- `src/components/Inspector.tsx`
- `README.md` and `docs/hod2-reverse-engineering/archive_logs/walkthrough.md` only after confirming against source

## Weapon Assemblies

Weapon assemblies are identified from joint names matching the current weapon-group helper in `HierarchyTree.tsx`. The base assembly row is rendered as a `weapon_group`; component subnodes render under that group.

Weapon subnodes are protected structural children:

- They must not open the context menu.
- They must not be directly deleted.
- They must not be directly renamed.

The assembly/group row itself remains editable through group rename/delete behavior.

## Turret Assemblies

Turret templates are weapon-style assemblies with additional turret-related joints according to current template creation code. They share the weapon-group rendering and protected-subnode rules.

## Point Groups

Hardpoint, capture point, repair point, and salvage point templates create standard grouped joint structures. Current grouping behavior is primarily Inspector-side for completion, repair, and semantic editing.

Agents must not assume point-group subnodes have the same tree context-menu protections as weapon subnodes unless source code implements that behavior or the task asks to add it.

## Required Components

Required components vary by assembly type. Inspector is the canonical source for each group's required and optional components.

Current Inspector-backed required component keys are:

- Weapon group: `Position`, `Direction`, `Muzzle`, `Rest`.
- Turret group: `Position`, `Direction`, `Latitude`, `Barrel`, `Muzzle`, `Rest`.
- Hardpoint group: `Position`, `Direction`, `Rest`.
- Capture point, repair point, and salvage point groups: `Base`, `Heading`, `Left`, `Up`.

If hierarchy diagnostics and Inspector disagree on required components, do not guess. Read both implementations and either reconcile docs to source or ask for a behavior decision.

## Rename Boundaries

Assembly-level rename should update all related component names and references for that assembly. Subnode-level rename is forbidden for protected weapon assembly subnodes.

Point-group rename behavior must be checked in Inspector before making claims, because point-group semantics are not identical to weapon-group tree semantics.

## Delete Boundaries

Deleting a weapon group deletes the full collected subtree and attached data. Deleting an ordinary joint deletes its full subtree. Deleting protected weapon subnodes directly is forbidden.

For other assembly families, agents must inspect current delete code before assuming grouped subtree deletion or protection rules.
