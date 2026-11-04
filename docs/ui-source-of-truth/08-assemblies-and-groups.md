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

### Multi-Muzzle Weapon Assemblies

Weapon assemblies support more than one muzzle. The primary muzzle is `${base}_Muzzle`; additional muzzles are numbered contiguously as `${base}_Muzzle1`, `${base}_Muzzle2`, and so on.

A full sweep of the 290 vanilla HWRM ship HODs found 397 weapon assemblies with muzzles: every one uses exactly one `_Muzzle`, with no `_MuzzleN`, no non-standard suffixes, and no `_Barrel` joints. Numbered multi-muzzle names are therefore an editor extension for mods that bind several muzzles through multiple `StartShipWeaponConfig` calls, not a vanilla HWRM convention.

- New muzzles are parented to `${base}_Latitude` when a Latitude joint exists, otherwise to `${base}_Position`. A direct `Root` parent is also accepted (vanilla `vgr_commandcorvette.hod` uses it for a fixed weapon).
- Adding or removing `Latitude` reparents all existing muzzles between `Latitude` and `Position`.
- The assembly inspector lists every muzzle joint and allows adding/removing them individually. Removing the primary muzzle does not renumber the remaining ones.
- Hierarchy diagnostics warn about non-standard muzzle names, missing primary `_Muzzle` numbering, numbering gaps, and muzzles parented outside the expected joint (Root excluded).
- Weapon assembly validation requires `Position`, `Direction`, `Muzzle`, and `Rest`. At least one muzzle is required; extra muzzles are optional.

### Slave Barrels (Vanilla Multi-Barrel)

Vanilla HWRM multi-barrel weapons use a **parallel weapon assembly** named `${base}Slave`, `${base}Slave1`, `${base}Slave2`, ... rather than extra `_MuzzleN` joints on the main chain. A full vanilla sweep found 45 slave assemblies across 16 weapon families (24 `Slave`, 11 `Slave1`, 10 `Slave2`); 37 are four-joint fixed chains and 8 are five-joint turret chains.

- Each slave chain has its own `Position` and `Muzzle` (and optional `Direction`/`Latitude`/`Rest`), just like a normal weapon assembly.
- The `.ship` file normally binds only the main weapon. The engine detects the `Slave` suffix on the parallel chain.
- The `.wepn` `setMiscValues(NewWeaponType, recoil, slaveDelay)` second argument is documented in vanilla scripts as the delay between initial firing and slave barrels. A non-zero value is the observable slave-fire control.
- The weapon inspector and hierarchy context menu can add a new slave chain (cloning the main chain; turret chains mirror the primary muzzle on X by default), and the inspector can remove a slave chain.
- Renaming or deleting the main assembly cascades to its slave assemblies. Visibility toggles on the main assembly also include its slaves.
- Hierarchy diagnostics warn when a slave assembly is missing its `Position` or `Muzzle`.

The `_MuzzleN` section above remains an editor extension for mods that bind several muzzles explicitly through multiple `StartShipWeaponConfig` calls; it is not the vanilla HWRM Slave mechanism.

## Turret Assemblies

Turret templates are weapon-style assemblies with additional turret-related joints according to current template creation code. They share the weapon-group rendering and protected-subnode rules.

The turret template creates five joints: `Position`, `Direction`, `Latitude`, `Muzzle`, and `Rest`. `Muzzle` is a child of `Latitude` at `+Z 7.64` (matching the vanilla `kus_defender.hod` turret reference), `Direction` is a `+Y 5.0` child of `Position`, and `Rest` is a `+Z 10.0` child of `Position`. Vanilla offsets vary per ship; these are editor defaults, not canonical replacements.

`Barrel` is **not** created by the template. No `_Barrel` joint exists in vanilla HWRM ship HODs, and its runtime behavior is unverified. The weapon assembly inspector exposes an optional **+ Add Barrel (Unverified)** toggle and documents this caveat; Barrel is never required by validation.

## Point Groups

Hardpoint, capture point, repair point, and salvage point templates create standard grouped joint structures. Current grouping behavior is primarily Inspector-side for completion, repair, and semantic editing.

Agents must not assume point-group subnodes have the same tree context-menu protections as weapon subnodes unless source code implements that behavior or the task asks to add it.

## Required Components

Required components vary by assembly type. Inspector is the canonical source for each group's required and optional components.

Current Inspector-backed required component keys are:

- Weapon group: `Position`, `Direction`, `Muzzle`, `Rest`.
- Turret group: `Position`, `Direction`, `Latitude`, `Muzzle`, `Rest`. `Barrel` is optional and unverified.
- Hardpoint group: `Position`, `Direction`, `Rest`.
- Capture point, repair point, and salvage point groups: `Base`, `Heading`, `Left`, `Up`.

If hierarchy diagnostics and Inspector disagree on required components, do not guess. Read both implementations and either reconcile docs to source or ask for a behavior decision.

## Rename Boundaries

Assembly-level rename should update all related component names and references for that assembly. Subnode-level rename is forbidden for protected weapon assembly subnodes.

Point-group rename behavior must be checked in Inspector before making claims, because point-group semantics are not identical to weapon-group tree semantics.

## Delete Boundaries

Deleting a weapon group deletes the full collected subtree and attached data. Deleting an ordinary joint deletes its full subtree. Deleting protected weapon subnodes directly is forbidden.

For other assembly families, agents must inspect current delete code before assuming grouped subtree deletion or protection rules.
