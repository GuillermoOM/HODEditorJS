# Task Checklist: Fix `generate_v2_from_model` DTRM Serialization

> Last updated: 2026-05-26T16:24:00-07:00

## Bug Fixes

- [x] **Bug 1**: Fix HIER `first_val` — compute `0xFFFFFF00 | ((-joint_count) & 0xFF)` instead of hardcoded `0xFFFFFF00` (line 2902)
- [x] **Bug 2**: Fix HIER chunk type — change from `ChunkType::Normal` to `ChunkType::Form` (line 2929)
- [x] **Bug 3**: Fix joint rotation/scale — read from `joint.position`/`.rotation`/`.scale` with `decompose_matrix` fallback instead of hardcoded zeros/ones (lines 2918-2925)
- [x] **Bug 4**: Fix BURN chunks — write individual `ChunkType::Default` BURN chunks instead of one consolidated `NRML BURN` (lines 2966-2988)
- [x] **Bug 5**: Fix NAVL preservation — regenerate NAVL data from `model.nav_lights` instead of preserving stale original data (lines 2991-2999)

## Verification

- [x] Build passes (`cargo check` in parser/)
- [x] `verify_lossless` passes for all test files (pebble_0, ter_elysium, ter_fenris, asteroid_3, DAE)
- [x] Binary analysis of re-saved `ter_elysium_edited.hod` confirms correct `first_val`, rotation data, individual BURNs
- [x] Update walkthrough with results

## Documentation

- [x] Update `agents_info/generate_v2_dtrm_fixes/walkthrough.md` with changes made and verification results
- [ ] Update `agents_info/hod2_reverse_engineering_knowledge_base.md` if new structural knowledge was discovered
