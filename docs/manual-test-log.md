# Manual Testing for HODEditorJS inputs and outputs

## Objective

The objective of this document is to log happy path testing to make sure everything works as it should, any agent reading this document will be able to look at the testing history at each snapshot (by timestamp generated after changes were done). So when a change is applied, a new test section must be added following the current template.

Agent would create this file in the testing subdirectory for a HOD, adding test snapshots from the template for the user to fill in after manual testing (snapshot date should be added by the agent for when it changed something and asks the user to test). On every change there must be a commit to track the snapshots.

## Resources

All test resources (original files being edited and output files resulting from editing) are located here:

`testing/`

The files to look for contain the following naming conventions:

`*_2.0_original.hod` -> HOD 2.0 file that currently works 100% in game
`*_1.0_original.hod` -> For original HOD 1.0 file of the same ship (may contain differences from 2.0 original file)
`*_from_2.0_to_2.0.hod` -> original HOD 2.0 File opened in editor and saved directly to test similarity and compatibility to game engine
`*_from_1.0_to_2.0.hod` -> original HOD 1.0 file opened in editor and saved directly as HOD 2.0 to test compatibility with game engine (might be different to original HOD 2.0 file in terms of collision mesh and nodes, importance is compatibility in main meshes and textures with game)

## Test Template (steps and results)

---
SHA: ``
Timestamp: `DD/MM/YYYY HH:MM`
Test hod: `ter_zephyrus`

2.0 HOD Test:

1. Opened `*_2.0_original.hod` in editor:
    - No loading errors: PASS
    - Textures orientation: PASS
    - Textures assigned to correct materials: PASS
    - Full meshes shown: PASS
    - Collision mesh loaded: PASS
    - All expected nodes loaded: PASS (this depends on user knowledge of the HOD file on which nodes should be present)
2. Saved as `*_from_2.0_to_2.0.hod`:
    - No saving errors: PASS
3. Opened `_from_2.0_to_2.0.hod` in editor again:
    - No loading errors: PASS
    - Textures orientation: PASS
    - Textures assigned to correct materials: PASS
    - Full meshes shown: PASS
    - Collision mesh loaded: PASS
    - All expected nodes loaded: PASS (this depends on user knowledge of the HOD file on which nodes should be present)
4. Loaded `_from_2.0_to_2.0.hod` in game:
    - No crash on loading: PASS
    - Textures orientation: PASS
    - Textures assigned to correct materials: PASS
    - Full meshes shown: PASS
    - Correct Ship Orientation: PASS
    - All expected nodes working: PASS (this depends on user knowledge of the HOD file on which nodes should be present)

1.0 HOD Test:

1. Opened `*_2.0_original.hod` in editor:
    - No loading errors: PASS
    - Textures orientation: PASS
    - Textures assigned to correct materials: PASS
    - Full meshes shown: PASS
    - Collision mesh loaded: PASS
    - All expected nodes loaded: PASS (this depends on user knowledge of the HOD file on which nodes should be present)
2. Saved as `*_from_2.0_to_2.0.hod`:
    - No saving errors: PASS
3. Opened `_from_2.0_to_2.0.hod` in editor again:
    - No loading errors: PASS
    - Textures orientation: PASS
    - Textures assigned to correct materials: PASS
    - Full meshes shown: PASS
    - Collision mesh loaded: PASS
    - All expected nodes loaded: PASS (this depends on user knowledge of the HOD file on which nodes should be present)
4. Loaded `_from_2.0_to_2.0.hod` in game:
    - No crash on loading: PASS
    - Textures orientation: PASS
    - Textures assigned to correct materials: PASS
    - Full meshes shown: PASS
    - Correct Ship orientation [x] PASS
    - All expected nodes working: PASS (this depends on user knowledge of the HOD file on which nodes should be present)

---

## Test Runs

---
SHA: `3dc8ed8b292cb0dbf0edafa17fd3979fc8ce79a7`
Timestamp: `01/06/2026 14:05`
Test hod: `ter_zephyrus`

**Goal**: Verify the restoration of texture flipping on load/save, fallback generation to fix material mismatches, and `sx/sy/sz` rotation fixes.

2.0 HOD Test:

1. Opened `*_2.0_original.hod` in editor:
    - No loading errors: [x] PASS
    - Textures orientation: [x] PASS
    - Textures assigned to correct materials: [x] PASS
    - Full meshes shown: [x] PASS
    - Collision mesh loaded: [x] PASS
    - All expected nodes loaded: [x] PASS
2. Saved as `*_from_2.0_to_2.0.hod`:
    - No saving errors: [x] PASS
3. Opened `_from_2.0_to_2.0.hod` in editor again:
    - No loading errors: [x] PASS
    - Textures orientation: [x] PASS
    - Textures assigned to correct materials: [x] PASS
    - Full meshes shown: [x] PASS
    - Collision mesh loaded: [x] PASS
    - All expected nodes loaded: [x] FAIL - Docking path and points nodes are missing
4. Loaded `_from_2.0_to_2.0.hod` in game:
    - No crash on loading: [x] PASS
    - Textures orientation: [x] PASS
    - Textures assigned to correct materials: [x] PASS
    - Full meshes shown: [x] PASS
    - Correct Ship orientation [x] PASS
    - All expected nodes working: [x] FAIL - Due to missing docking paths and points, ships can't dock

1.0 HOD Test:

1. Opened `*_1.0_original.hod` in editor:
    - No loading errors: [x]
    - Textures orientation: [x] FAIL - Textures are Y Flipped (Loading error here)
    - Textures assigned to correct materials: [x] PASS
    - Full meshes shown: [x] PASS
    - Collision mesh loaded: [x] PASS
    - All expected nodes loaded: [x] PASS
2. Saved as `*_from_1.0_to_2.0.hod`:
    - No saving errors: [x]
3. Opened `_from_1.0_to_2.0.hod` in editor again:
    - No loading errors: [x]
    - Textures orientation: [x] FAIL - Textures are Y Flipped (no change from saving 1.0)
    - Textures assigned to correct materials: [x] FAIL - Some Textures are assigned to the wrong materials
    - Full meshes shown: [x] PASS
    - Collision mesh loaded: [x] PARTIAL PASS - Apparently a new COL node was added when there was already one (this might be an effect from saving as HOD 2.0)
    - All expected nodes loaded: [x] FAIL - Docking path and points nodes are missing
4. Loaded `_from_1.0_to_2.0.hod` in game:
    - No crash on loading: [x] PASS
    - Textures orientation: [x] FAIL - Textures are Y Flipped
    - Textures assigned to correct materials: [x] FAIL - Some Textures are assigned to the wrong materials
    - Full meshes shown: [X] PASS
    - Correct Ship Orientation: [x] FAIL - Ship not looking towards their forward vector
    - All expected nodes working: [x] FAIL - Due to missing docking paths and points, ships can't dock

---

SHA: `22ea37e51c6a138e61770e6e1a1ab6a747e445ec`
Timestamp: `01/06/2026 21:30`
Test hod: `ter_zephyrus`

**Goal**: Verify POOL mesh crash fix and DOCK chunk loss fix during save.

2.0 HOD Test:

1. Opened `*_2.0_original.hod` in editor:
    - No loading errors: [x] PASS 
    - Textures orientation: [x] PASS
    - Textures assigned to correct materials: [x] PASS
    - Full meshes shown: [x] PASS
    - Collision mesh loaded: [x] PASS
    - All expected nodes loaded: [x] PASS
2. Saved as `*_from_2.0_to_2.0.hod`:
    - No saving errors: [x] PASS
3. Opened `_from_2.0_to_2.0.hod` in editor again:
    - No loading errors: [x] PASS
    - Textures orientation: [x] PASS
    - Textures assigned to correct materials: [x] PASS
    - Full meshes shown: [x] PASS
    - Collision mesh loaded: [x] PARTIAL PASS - I see two COL nodes now, where is this being added? there should be no new node being added at any moment, only load if there is an existing one
    - All expected nodes loaded: [x ] PASS, docking nodes and points shown
4. Loaded `_from_2.0_to_2.0.hod` in game:
    - No crash on loading: [x] PASS
    - Textures orientation: [x] PASS
    - Textures assigned to correct materials: [x] PASS
    - Full meshes shown: [x] PASS
    - Correct Ship Orientation: [x] PASS
    - All expected nodes working: [x] PASS

---
SHA: `47c71bd`
Timestamp: `01/06/2026 18:00`
Test hod: `ter_zephyrus`

**Goal**: Verify STAT material shader parameters (glossiness, specular colors, team colors) are preserved during HOD 2.0 round-trip. Materials should render correctly in-game with proper shader uniforms.

2.0 HOD Test:

1. Opened `*_2.0_original.hod` in editor:
    - No loading errors: [x] PASS
    - Textures orientation: [x] PASS
    - Textures assigned to correct materials: [x] PASS
    - Full meshes shown: [x] PASS
    - Collision mesh loaded: [x] PASS
    - All expected nodes loaded: [x] PASS
2. Saved as `*_from_2.0_to_2.0.hod`:
    - No saving errors: [x] PASS
3. Opened `_from_2.0_to_2.0.hod` in editor again:
    - No loading errors: [x] PASS
    - Textures orientation: [x] PASS
    - Textures assigned to correct materials: [x] PASS
    - Full meshes shown: [x] PASS
    - Collision mesh loaded: [x] PASS
    - All expected nodes loaded: [x] PASS
4. Loaded `_from_2.0_to_2.0.hod` in game:
    - No crash on loading: [x] PASS
    - Textures orientation: [x] PASS
    - Textures assigned to correct materials: [x] PASS
    - Full meshes shown: [x] PASS
    - Correct Ship Orientation: [x] PASS
    - Material shader parameters correct (glossiness, specular, team colors): [x] PASS
    - All expected nodes working: [x] PASS

1.0 HOD Test:

1. Opened `*_1.0_original.hod` in editor:
    - No loading errors: [x] PASS
    - Textures orientation: [x] PASS
    - Textures assigned to correct materials: [x] PASS
    - Full meshes shown: [x] PASS
    - Collision mesh loaded: [x] PASS
    - All expected nodes loaded: [x] PASS
2. Saved as `*_from_1.0_to_2.0.hod`:
    - No saving errors: [x] PASS
3. Opened `_from_1.0_to_2.0.hod` in editor again:
    - No loading errors: [x] PASS
    - Textures orientation: [x] PASS
    - Textures assigned to correct materials: [x] PASS
    - Full meshes shown: [x] PASS
    - Collision mesh loaded: [x] PASS
    - All expected nodes loaded: [x] PASS
4. Loaded `_from_1.0_to_2.0.hod` in game:
    - No crash on loading: [x] PASS
    - Textures orientation: [x] PASS
    - Textures assigned to correct materials: [x] PASS
    - Full meshes shown: [x] PASS
    - Correct Ship Orientation: [x] PASS
    - All expected nodes working: [x] PASS

---
SHA: `21c1324`
Timestamp: `01/06/2026 16:15`
Test hod: `ter_zephyrus`

**Goal**: Verify HOD 1.0 -> 2.0 material assignment fix and correct scaling extraction. Also fix correct creation of proxy engine burn nodes with correct EngineNozzle# naming convention

2.0 HOD Test:

1. Opened `*_2.0_original.hod` in editor:
    - No loading errors: [x]  PASS
    - Textures orientation: [x] PASS
    - Textures assigned to correct materials: [x] PASS
    - Full meshes shown: [x] PASS
    - Collision mesh loaded: [x] PASS
    - All expected nodes loaded: [x] PASS
2. Saved as `*_from_2.0_to_2.0.hod`:
    - No saving errors: [x] PASS
3. Opened `_from_2.0_to_2.0.hod` in editor again:
    - No loading errors: [x] PASS
    - Textures orientation: [x] PASS
    - Textures assigned to correct materials: [x] PASS
    - Full meshes shown: [x] PASS
    - Collision mesh loaded: [x] PASS
    - All expected nodes loaded: [x] PASS
4. Loaded `_from_2.0_to_2.0.hod` in game:
    - No crash on loading: [x] PASS
    - Textures orientation: [x] PASS
    - Textures assigned to correct materials: [x] PASS
    - Full meshes shown: [x] PASS
    - Correct Ship Orientation: [x] PASS
    - All expected nodes working: [x] PASS

1.0 HOD Test:

1. Opened `*_1.0_original.hod` in editor:
    - No loading errors: [x] PASS
    - Textures orientation: [x] PASS
    - Textures assigned to correct materials: [x] PASS
    - Full meshes shown: [x] PASS
    - Collision mesh loaded: [x] PASS
    - All expected nodes loaded: [x] PARTIAL PASS - Something I skipped on noticing before, the engine nozzle nodes created are not following standard naming conventions (showing name `BurnProxy_EngineBurn4_from_e4` instead of `EngineNozzle#`)
2. Saved as `*_from_1.0_to_2.0.hod`:
    - No saving errors: [x] PASS
3. Opened `_from_1.0_to_2.0.hod` in editor again:
    - No loading errors: [x] PASS
    - Textures orientation: [x] PASS
    - Textures assigned to correct materials: [x] FAIL - Even thought when opening the original HOD 1.0 the editor was showing the correct textures in materials, when opening this edited version the materials are reassigned wrongly.
    - Full meshes shown: [x] PASS
    - Collision mesh loaded: [x] PASS
    - All expected nodes loaded: [x] PASS
4. Loaded `_from_1.0_to_2.0.hod` in game:
    - No crash on loading: [x] PASS
    - Textures orientation: [x] PASS
    - Textures assigned to correct materials: [x] FAIL (reason stated on previous step)
    - Full meshes shown: [x] PASS
    - Correct Ship Orientation: [x] FAIL, needs same correction for scaling I believe, it is oriented the same way as the `from_2.0_to_2.0` was before it got fixed on its creation.
    - All expected nodes working: [x] PASS

---
SHA: `c3ca158`
Timestamp: `01/06/2026 16:40`
Test hod: `ter_zephyrus`

**Goal**: Verify HOD 1.0 -> 2.0 material assignment fix and correct scaling extraction. Also fix correct creation of proxy engine burn nodes with correct EngineNozzle# naming convention

2.0 HOD Test:

1. Opened `*_2.0_original.hod` in editor:
    - No loading errors: [x]  PASS
    - Textures orientation: [x] PASS
    - Textures assigned to correct materials: [x] PASS
    - Full meshes shown: [x] PASS
    - Collision mesh loaded: [x] PASS
    - All expected nodes loaded: [x] PASS
2. Saved as `*_from_2.0_to_2.0.hod`:
    - No saving errors: [x] PASS
3. Opened `_from_2.0_to_2.0.hod` in editor again:
    - No loading errors: [x] PASS
    - Textures orientation: [x] PASS
    - Textures assigned to correct materials: [x] PASS
    - Full meshes shown: [x] PASS
    - Collision mesh loaded: [x] PASS
    - All expected nodes loaded: [x] PASS
4. Loaded `_from_2.0_to_2.0.hod` in game:
    - No crash on loading: [x] PASS
    - Textures orientation: [x] PASS
    - Textures assigned to correct materials: [x] PASS
    - Full meshes shown: [x] PASS
    - Correct Ship Orientation: [x] PASS
    - All expected nodes working: [x] PASS

1.0 HOD Test:

1. Opened `*_1.0_original.hod` in editor:
    - No loading errors: [x] PASS
    - Textures orientation: [x] PASS
    - Textures assigned to correct materials: [x] PASS
    - Full meshes shown: [x] PASS
    - Collision mesh loaded: [x] PASS
    - All expected nodes loaded: [x] PASS
2. Saved as `*_from_1.0_to_2.0.hod`:
    - No saving errors: [x] PASS
3. Opened `_from_1.0_to_2.0.hod` in editor again:
    - No loading errors: [x] PASS
    - Textures orientation: [x] PASS
    - Textures assigned to correct materials: [x] FAIL - Even thought when opening the original HOD 1.0 the editor was showing the correct textures in materials, when opening this edited version the materials are reassigned wrongly.
    - Full meshes shown: [x] PASS
    - Collision mesh loaded: [x] PASS
    - All expected nodes loaded: [x] PASS
4. Loaded `_from_1.0_to_2.0.hod` in game:
    - No crash on loading: [x] PASS
    - Textures orientation: [x] PASS
    - Textures assigned to correct materials: [x] FAIL (reason stated on previous step)
    - Full meshes shown: [x] PASS
    - Correct Ship Orientation: [x] FAIL, needs same correction for scaling I believe, it is oriented the same way as the `from_2.0_to_2.0` was before it got fixed on its creation.
    - All expected nodes working: [x] PASS

---
SHA: `eef4d2b`
Timestamp: `01/06/2026 17:08`
Test hod: `ter_zephyrus`

**Goal**: Verify HOD 1.0 -> 2.0 conversion scaling and orientation fix. Check if the ship is properly scaled down (not 100x bigger) and not rotated sideways in game.

2.0 HOD Test:

1. Opened `*_2.0_original.hod` in editor:
    - No loading errors: [x] PASS
    - Textures orientation: [x] PASS
    - Textures assigned to correct materials: [x] PASS
    - Full meshes shown: [x] PASS
    - Collision mesh loaded: [x] PASS
    - All expected nodes loaded: [x] PASS
2. Saved as `*_from_2.0_to_2.0.hod`:
    - No saving errors: [x] PASS
3. Opened `_from_2.0_to_2.0.hod` in editor again:
    - No loading errors: [x] PASS
    - Textures orientation: [x] PASS
    - Textures assigned to correct materials: [x] PASS
    - Full meshes shown: [x] PASS
    - Collision mesh loaded: [x] PASS
    - All expected nodes loaded: [x] PASS
4. Loaded `_from_2.0_to_2.0.hod` in game:
    - No crash on loading: [x] PASS
    - Textures orientation: [x] PASS
    - Textures assigned to correct materials: [x] PASS
    - Full meshes shown: [x] PASS
    - Correct Ship Orientation: [x] PASS
    - All expected nodes working: [x] PASS

1.0 HOD Test:

1. Opened `*_1.0_original.hod` in editor:
    - No loading errors: [x] PASS
    - Textures orientation: [x] PASS
    - Textures assigned to correct materials: [x] PASS
    - Full meshes shown: [x] FAIL - The mesh is scaled down! it used to show in proper size in renderer
    - Collision mesh loaded: [x] PARTIAL PASS - mesh loads, but also scaled down in size
    - All expected nodes loaded: [x] PASS
2. Saved as `*_from_1.0_to_2.0.hod`:
    - No saving errors: [x] PASS
3. Opened `_from_1.0_to_2.0.hod` in editor again:
    - No loading errors: [x] PASS
    - Textures orientation: [x] PASS
    - Textures assigned to correct materials: [x] PASS
    - Full meshes shown: [x] PASS
    - Collision mesh loaded: [x] PASS
    - All expected nodes loaded: [x] PASS
4. Loaded `_from_1.0_to_2.0.hod` in game:
    - No crash on loading: [x] PASS
    - Textures orientation: [x] PASS
    - Textures assigned to correct materials: [x] PASS
    - Full meshes shown: [x] PASS
    - Correct Ship Orientation: [x] PASS
    - All expected nodes working: [x] PASS

---
SHA: `1e882cd1`
Timestamp: `01/06/2026 17:18`
Test hod: `ter_zephyrus`

**Goal**: Verify HOD 1.0 -> 2.0 conversion scaling fix (without making it an ant) and dynamic STAT/MATT generation for dropdown assignments.

2.0 HOD Test:

1. Opened `*_2.0_original.hod` in editor:
    - No loading errors: [x] PASS
    - Textures orientation: [x] PASS
    - Textures assigned to correct materials: [x] PASS
    - Full meshes shown: [x] PASS
    - Collision mesh loaded: [x] PASS
    - All expected nodes loaded: [x] PASS
2. Saved as `*_from_2.0_to_2.0.hod`:
    - No saving errors: [x] PASS
3. Opened `_from_2.0_to_2.0.hod` in editor again:
    - No loading errors: [x] PASS
    - Textures orientation: [x] PASS
    - Textures assigned to correct materials: [x] FAIL - Some materials got the wrong textures assigned after the save, instead of keeping the same (didn't read from editor values)
    - Full meshes shown: [x] PASS
    - Collision mesh loaded: [x] PASS
    - All expected nodes loaded: [x] PASS
4. Loaded `_from_2.0_to_2.0.hod` in game:
    - No crash on loading: [x] PASS
    - Textures orientation: [x] PASS
    - Textures assigned to correct materials: [x] FAIL
    - Full meshes shown: [x] PASS
    - Correct Ship Orientation: [x] PASS
    - All expected nodes working: [x] PASS

1.0 HOD Test:

1. Opened `*_1.0_original.hod` in editor:
    - No loading errors: [x] PASS
    - Textures orientation: [x] PASS
    - Textures assigned to correct materials: [x] PASS
    - Full meshes shown: [x] PASS
    - Collision mesh loaded: [x] PASS
    - All expected nodes loaded: [x] PASS
2. Saved as `*_from_1.0_to_2.0.hod`:
    - No saving errors: [x] PASS
3. Opened `_from_1.0_to_2.0.hod` in editor again:
    - No loading errors: [x] PASS
    - Textures orientation: [x] PASS
    - Textures assigned to correct materials: [x] PASS
    - Full meshes shown: [x] PASS
    - Collision mesh loaded: [x] PASS
    - All expected nodes loaded: [x] PASS
4. Loaded `_from_1.0_to_2.0.hod` in game:
    - No crash on loading: [x] PASS
    - Textures orientation: [x] PASS
    - Textures assigned to correct materials: [x] PASS
    - Full meshes shown: [x] PASS
    - Correct Ship Orientation: [x] PASS
    - All expected nodes working: [x] PASS

---
SHA: `0e89a5`
Timestamp: `01/06/2026 17:43`
Test hod: `ter_zephyrus`

**Goal**: Verify HOD 1.0 -> 2.0 conversion scaling fix (without making it an ant) and dynamic STAT/MATT generation for dropdown assignments.

2.0 HOD Test:

1. Opened `*_2.0_original.hod` in editor:
    - No loading errors: [x] PASS
    - Textures orientation: [x] PASS
    - Textures assigned to correct materials: [x] PASS
    - Full meshes shown: [x] PASS
    - Collision mesh loaded: [x] PASS
    - All expected nodes loaded: [x] PASS
2. Saved as `*_from_2.0_to_2.0.hod`:
    - No saving errors: [x] PASS
3. Opened `_from_2.0_to_2.0.hod` in editor again:
    - No loading errors: [x] PASS
    - Textures orientation: [x] PASS
    - Textures assigned to correct materials: [x] FAIL - Some materials got the wrong textures assigned after the save, instead of keeping the same (didn't read from editor values)
    - Full meshes shown: [x] PASS
    - Collision mesh loaded: [x] PASS
    - All expected nodes loaded: [x] PASS
4. Loaded `_from_2.0_to_2.0.hod` in game:
    - No crash on loading: [x] PASS
    - Textures orientation: [x] PASS
    - Textures assigned to correct materials: [x] FAIL - same reason as above
    - Full meshes shown: [x] PASS
    - Correct Ship Orientation: [x] PASS
    - All expected nodes working: [x] PASS

1.0 HOD Test:

1. Opened `*_1.0_original.hod` in editor:
    - No loading errors: [x] PASS
    - Textures orientation: [x] PASS
    - Textures assigned to correct materials: [x] PASS
    - Full meshes shown: [x] PASS
    - Collision mesh loaded: [x] PASS
    - All expected nodes loaded: [x] PASS
2. Saved as `*_from_1.0_to_2.0.hod`:
    - No saving errors: [x] PASS
3. Opened `_from_1.0_to_2.0.hod` in editor again:
    - No loading errors: [x] PASS
    - Textures orientation: [x] PASS
    - Textures assigned to correct materials: [x] PASS
    - Full meshes shown: [x] PASS
    - Collision mesh loaded: [x] PASS
    - All expected nodes loaded: [x] PASS
4. Loaded `_from_1.0_to_2.0.hod` in game:
    - No crash on loading: [x] PASS
    - Textures orientation: [x] PASS
    - Textures assigned to correct materials: [x] PASS
    - Full meshes shown: [x] PASS
    - Correct Ship Orientation: [x] PASS
    - All expected nodes working: [x] PASS

---
SHA: `9171f56`
Timestamp: `01/06/2026 18:09`
Test hod: `ter_zephyrus`

**Goal**: Verify HOD 2.0→2.0 texture assignment fix — `auto_assign_and_resize_textures` now correctly matches texture names with suffixes like `_GLOWDXT1`, `_TEAMDXT1` to shader slots, preserving all 3 textures per material.

2.0 HOD Test:

1. Opened `*_2.0_original.hod` in editor:
    - No loading errors: [x] PASS
    - Textures orientation: [x] PASS
    - Textures assigned to correct materials: [x] PASS
    - Full meshes shown: [x] PASS
    - Collision mesh loaded: [x] PASS
    - All expected nodes loaded: [x] PASS
2. Saved as `*_from_2.0_to_2.0.hod`:
    - No saving errors: [x] PASS
3. Opened `_from_2.0_to_2.0.hod` in editor again:
    - No loading errors: [x] PASS
    - Textures orientation: [x] PASS
    - Textures assigned to correct materials: [x] PASS
    - Full meshes shown: [x] PASS
    - Collision mesh loaded: [x] PASS
    - All expected nodes loaded: [x] PASS
4. Loaded `_from_2.0_to_2.0.hod` in game:
    - No crash on loading: [x] PASS
    - Textures orientation: [x] PASS
    - Textures assigned to correct materials: [x] PASS
    - Full meshes shown: [x] PASS
    - Correct Ship Orientation: [x] PASS
    - All expected nodes working: [x] PASS

1.0 HOD Test:

1. Opened `*_1.0_original.hod` in editor:
    - No loading errors: [x] PASS
    - Textures orientation: [x] PASS
    - Textures assigned to correct materials: [x] PASS
    - Full meshes shown: [x] PASS
    - Collision mesh loaded: [x] PASS
    - All expected nodes loaded: [x] PASS
2. Saved as `*_from_1.0_to_2.0.hod`:
    - No saving errors: [x] PASS
3. Opened `_from_1.0_to_2.0.hod` in editor again:
    - No loading errors: [x] PASS
    - Textures orientation: [x] PASS
    - Textures assigned to correct materials: [x] PASS
    - Full meshes shown: [x] PASS
    - Collision mesh loaded: [x] PASS
    - All expected nodes loaded: [x] PASS
4. Loaded `_from_1.0_to_2.0.hod` in game:
    - No crash on loading: [x] PASS
    - Textures orientation: [x] PASS
    - Textures assigned to correct materials: [x] PASS
    - Full meshes shown: [x] PASS
    - Correct Ship Orientation: [x] PASS
    - All expected nodes working: [x] PASS

---

SHA: `9171f56`
Timestamp: `01/06/2026 18:15`
Test hod: `ter_centaur`

**Goal**: Verify HOD 2.0→2.0 mesh is saved completely without issues.

2.0 HOD Test:

1. Opened `*_2.0_original.hod` in editor:
    - No loading errors: [x] PASS
    - Textures orientation: [x] PASS
    - Textures assigned to correct materials: [x] PASS
    - Full meshes shown: [x] PASS
    - Collision mesh loaded: [x] PASS
    - All expected nodes loaded: [x] PASS
2. Removed Extra LODs (left LOD0), Saved as `*_from_2.0_to_2.0.hod`:
    - No saving errors: [x] PASS
3. Opened `_from_2.0_to_2.0.hod` in editor again:
    - No loading errors: [x] PASS
    - Textures orientation: [x] PASS
    - Textures assigned to correct materials: [x] PASS
    - Full meshes shown: [ ] FAIL - Mesh is broken / incomplete with edges crossing on the inside. There was a fix for meshes missing faces before for ter_zephyrus before which could've affected smaller meshes like this one.
    - Collision mesh loaded: [x] PASS
    - All expected nodes loaded: [x] PASS
4. Loaded `_from_2.0_to_2.0.hod` in game:
    - No crash on loading: [x] PASS
    - Textures orientation: [x] PASS
    - Textures assigned to correct materials: [x] PASS
    - Full meshes shown: [x] PASS
    - Correct Ship Orientation: [x] PASS
    - All expected nodes working: [x] PASS

1.0 HOD Test:

1. Opened `*_1.0_original.hod` in editor:
    - No loading errors: [x] PASS
    - Textures orientation: [x] PASS
    - Textures assigned to correct materials: [x] PASS
    - Full meshes shown: [x] PASS
    - Collision mesh loaded: [x] PASS
    - All expected nodes loaded: [x] PASS
2. Removed Extra LODs (left LOD0), Saved as `*_from_1.0_to_2.0.hod`:
    - No saving errors: [x] PASS
3. Opened `_from_1.0_to_2.0.hod` in editor again:
    - No loading errors: [x] PASS
    - Textures orientation: [x] PASS
    - Textures assigned to correct materials: [ ] FAIL - the original glass material is gone and not parsed (didn't have DIFF suffix or similar, just shows as "transparentDXT5" in editor when loading HOD 1.0 original)
    - Full meshes shown: [x] PASS
    - Collision mesh loaded: [x] PASS
    - All expected nodes loaded: [x] PASS
4. Loaded `_from_1.0_to_2.0.hod` in game:
    - No crash on loading: [x] PASS
    - Textures orientation: [x] PASS
    - Textures assigned to correct materials: [x] PASS
    - Full meshes shown: [x] PASS
    - Correct Ship Orientation: [x] PASS
    - All expected nodes working: [x] PASS

---

SHA: `ab71c78`
Timestamp: `01/06/2026 16:45`
Test hod: `ter_centaur`

**Goal**: Verify MULT path mesh index fix and LMIP tiny chunk threshold fix.

2.0 HOD Test:

1. Opened `*_2.0_original.hod` in editor:
    - No loading errors: [x] PASS
    - Textures orientation: [x] PASS
    - Textures assigned to correct materials: [x] PASS
    - Full meshes shown: [x] PASS
    - Collision mesh loaded: [x] PASS
    - All expected nodes loaded: [x] PASS
2. Removed Extra LODs (left LOD0), Saved as `*_from_2.0_to_2.0.hod`:
    - No saving errors: [x] PASS
3. Opened `_from_2.0_to_2.0.hod` in editor again:
    - No loading errors: [x] PASS
    - Textures orientation: [x] PASS
    - Textures assigned to correct materials: [x] PASS
    - Full meshes shown: [x] PASS
    - Collision mesh loaded: [x] PASS
    - All expected nodes loaded: [x] PASS
4. Loaded `_from_2.0_to_2.0.hod` in game:
    - No crash on loading: [x] PASS
    - Textures orientation: [x] PASS
    - Textures assigned to correct materials: [x] PASS
    - Full meshes shown: [x] PASS
    - Correct Ship Orientation: [x] PASS
    - All expected nodes working: [x] PASS

1.0 HOD Test:

1. Opened `*_1.0_original.hod` in editor:
    - No loading errors: [x] PASS
    - Textures orientation: [x] PASS
    - Textures assigned to correct materials: [x] PASS
    - Full meshes shown: [x] PASS
    - Collision mesh loaded: [x] PASS
    - All expected nodes loaded: [x] PASS
2. Removed Extra LODs (left LOD0), Saved as `*_from_1.0_to_2.0.hod`:
    - No saving errors: [x] PASS
3. Opened `_from_1.0_to_2.0.hod` in editor again:
    - No loading errors: [x] PASS
    - Textures orientation: [x] PASS
    - Textures assigned to correct materials: [x] PASS
    - Full meshes shown: [x] PASS
    - Collision mesh loaded: [x] PASS
    - All expected nodes loaded: [x] PASS
4. Loaded `_from_1.0_to_2.0.hod` in game:
    - No crash on loading: [x] PASS
    - Textures orientation: [x] PASS
    - Textures assigned to correct materials: [x] PASS
    - Full meshes shown: [x] PASS
    - Correct Ship Orientation: [x] PASS
    - All expected nodes working: [x] PASS

---

SHA: `ab71c78`
Timestamp: `01/06/2026 16:45`
Test hod: `ter_fenris`

**Goal**: Verify MULT path mesh index fix and LMIP tiny chunk threshold fix.

2.0 HOD Test:

1. Opened `*_2.0_original.hod` in editor:
    - No loading errors: [x] PASS
    - Textures orientation: [x] PASS
    - Textures assigned to correct materials: [x] PASS
    - Full meshes shown: [x] PASS
    - Collision mesh loaded: [x] PASS
    - All expected nodes loaded: [ ] FAIL - No animations being loaded, this was working perfectly before!
2. Removed Extra LODs (left LOD0), Saved as `*_from_2.0_to_2.0.hod`:
    - No saving errors: [ ]
3. Opened `_from_2.0_to_2.0.hod` in editor again:
    - No loading errors: [ ]
    - Textures orientation: [ ]
    - Textures assigned to correct materials: [ ]
    - Full meshes shown: [ ]
    - Collision mesh loaded: [ ]
    - All expected nodes loaded: [ ]
4. Loaded `_from_2.0_to_2.0.hod` in game:
    - No crash on loading: [ ]
    - Textures orientation: [ ]
    - Textures assigned to correct materials: [ ]
    - Full meshes shown: [ ]
    - Correct Ship Orientation: [ ]
    - All expected nodes working: [ ]

1.0 HOD Test:

1. Opened `*_1.0_original.hod` in editor:
    - No loading errors: [x] PASS
    - Textures orientation: [x] PASS
    - Textures assigned to correct materials: [x] PASS
    - Full meshes shown: [x] PASS
    - Collision mesh loaded: [x] PASS
    - All expected nodes loaded: [ ] FAIL - No animations being loaded, this was working perfectly before!
2. Removed Extra LODs (left LOD0), Saved as `*_from_1.0_to_2.0.hod`:
    - No saving errors: [ ] PASS
3. Opened `_from_1.0_to_2.0.hod` in editor again:
    - No loading errors: [ ]
    - Textures orientation: [ ]
    - Textures assigned to correct materials: [ ]
    - Full meshes shown: [ ]
    - Collision mesh loaded: [ ]
    - All expected nodes loaded: [ ]
4. Loaded `_from_1.0_to_2.0.hod` in game:
    - No crash on loading: [ ]
    - Textures orientation: [ ]
    - Textures assigned to correct materials: [ ]
    - Full meshes shown: [ ]
    - Correct Ship Orientation: [ ]
    - All expected nodes working: [ ]

---

SHA: `5429cd4`
Timestamp: `01/06/2026 20:10`
Test hod: `ter_fenris`

**Goal**: Verify normalized companion MAD lookup restores animation loading for suffixed HOD test names.

2.0 HOD Test:

1. Opened `*_2.0_original.hod` in editor:
    - No loading errors: [x] PASS
    - Textures orientation: [x] PASS
    - Textures assigned to correct materials: [x] PASS
    - Full meshes shown: [x] PASS
    - Collision mesh loaded: [x] PASS
    - All expected nodes loaded: [x] PASS
    - Animations loading: [x] PASS
2. Removed Extra LODs (left LOD0), Saved as `*_from_2.0_to_2.0.hod`:
    - No saving errors: [x] PASS
3. Opened `_from_2.0_to_2.0.hod` in editor again:
    - No loading errors: [x] PASS
    - Textures orientation: [x] PASS
    - Textures assigned to correct materials: [x] PASS
    - Full meshes shown: [x] PASS
    - Collision mesh loaded: [x] PASS
    - All expected nodes loaded: [x] PASS
    - Animations Loading: [x] PASS
4. Loaded `_from_2.0_to_2.0.hod` in game:
    - No crash on loading: [x] PASS
    - Textures orientation: [x] PASS
    - Textures assigned to correct materials: [x] PASS
    - Full meshes shown: [x] PASS
    - Correct Ship Orientation: [x] PASS
    - All expected nodes working: [x] PASS
    - Animations Loading: [x] PASS

1.0 HOD Test:

1. Opened `*_1.0_original.hod` in editor:
    - No loading errors: [x] PASS
    - Textures orientation: [ ] FAILED - Textures loaded y flipped
    - Textures assigned to correct materials: [x] PASS
    - Full meshes shown: [x] PASS
    - Collision mesh loaded: [x] PASS
    - All expected nodes loaded: [x] PASS
    - Animations loading: [x] PASS
2. Removed Extra LODs (left LOD0), Saved as `*_from_1.0_to_2.0.hod`:
    - No saving errors: [x] PASS
3. Opened `_from_1.0_to_2.0.hod` in editor again:
    - No loading errors: [ ]
    - Textures orientation: [ ]
    - Textures assigned to correct materials: [ ]
    - Full meshes shown: [ ]
    - Collision mesh loaded: [ ]
    - All expected nodes loaded: [ ]
    - Animations Loading: [ ]
4. Loaded `_from_1.0_to_2.0.hod` in game:
    - No crash on loading: [ ]
    - Textures orientation: [ ]
    - Textures assigned to correct materials: [ ]
    - Full meshes shown: [ ]
    - Correct Ship Orientation: [ ]
    - All expected nodes working: [ ]
    - Animations Loading: [ ]

Additional findings on HOD 1.0 loading regarding textures:
For HOD 1.0 files loaded onto editor from the `freespace_remastered/ship` directory of the game install (see `docs/testing-fixtures.md`)
Something we need to take a look at, strictly for fixing loading of textures in HOD 1.0 files

- ter_centaur: DXT1 textures loaded correctly, DXT5 texture is broken
- ter_zephyrus: all textures loading correctly and not flipped (all DXT1)
- ter_fenris: textures loading at full, but y flipped (all DXT1)
- ter_leviathan: texture leviathan DIFF (DXT5) corrupted on load.
- ter_demon: Texture Capital04-01 loaded corrupted (DXT5), all textures y flipped
- shi_azrael: all textures loaded correctly (DXT1)
- shi_scorpion: all textures loaded correctly (DXT1)
- if material has DIFF selected as (none), the texture defaults to another DIFF instead of no texture. (this is a regression of existing functionality)

Pattern here is weird, all DXT5 textures load corrupted and flipped, but not all DXT1 textures looks correct in orientation

---

SHA: `e17c44d`
Timestamp: `01/06/2026 21:19`
Test hod: `ter_fenris`, `ter_centaur`, `ter_leviathan`

**Goal**: Verify HOD 1.0 inline LMIP/TEXM multi-mip texture offset fix and `(None)` material texture slot regression fix.

Automated checks:

1. Parser/build verification:
    - `cargo check --lib`: [x] PASS - 38 pre-existing warnings only.
    - `cargo run --bin verify_lossless`: [x] PASS - structural re-parse counts match; expected size differences remain from recompression.
    - `npm run build`: [x] PASS - existing Vite large chunk warning only.
2. Targeted HOD 1.0 texture probes before cleanup:
    - `ter_centaur transparentDXT5`: [x] PASS - legacy DXT5 data starts immediately after base dimensions; parsed PNG length changed 370 -> 210 after offset fix.
    - `ter_fenris nameplateDXT5`: [x] PASS - multi-mip DXT5 no longer skips 72 bytes into compressed data; parsed PNG length changed 127938 -> 1214.
    - `ter_leviathan leviathanDXT5`: [x] PASS - multi-mip DXT5 decodes from corrected base offset.
    - Material DIFF `(none)` slot: [x] PASS - Viewport and Inspector no longer fuzzy-match empty texture names to the first texture.

1.0 HOD Manual Retest:

1. Opened `ter_fenris_1.0_original.hod` in editor:
    - No loading errors: [x] PASS
    - Textures orientation: [x] PASS
    - Textures assigned to correct materials: [x] PASS
    - Full meshes shown: [x] PASS
    - Collision mesh loaded: [x] PASS
    - All expected nodes loaded: [x] PASS
    - Animations loading: [x] PASS
2. Opened `ter_centaur_1.0_original.hod` in editor:
    - DXT1 textures loaded correctly: [x] PASS
    - DXT5 transparent material loaded correctly: [x] PASS
    - Material `(none)` slot remains untextured: [x] PASS
3. Opened HOD 1.0 `ter_leviathan.hod` from uncompressed_bigs in editor:
    - DXT5 `leviathan DIFF` loaded correctly: [x] PASS
    - Textures orientation: [x] PASS

---

SHA: `289380c`
Timestamp: `01/06/2026 21:41`
Test hod: `ter_centaur`, `ter_fenris`, DAE import

**Goal**: Verify singleton COL node behavior: only one COL may exist, it must display/save as `Root`, users cannot rename it, and deleting it persists through save/reopen.

Automated checks:

1. Parser/build verification:
    - `cargo check --lib`: [x] PASS - 37 pre-existing warnings only.
    - `cargo run --bin verify_lossless`: [x] PASS - structural re-parse counts match; expected size differences remain from recompression.
    - `npm run build`: [x] PASS - existing Vite large chunk warning only.

Manual COL Retest:

1. Open HOD 2.0 `ter_centaur_2.0_original.hod` in editor:
    - Exactly one COL node appears: [ ] PENDING
    - COL node displays as `Root`: [ ] PENDING
    - Rename is not available for COL: [ ] PENDING
    - Delete removes the COL node: [ ] PENDING
    - Save/reopen after delete does not synthesize a new COL: [ ] PENDING
2. Open HOD 1.0 `ter_fenris_1.0_original.hod` in editor:
    - If a collision is parsed, exactly one COL node appears: [ ] PENDING
    - Parsed COL node displays as `Root`: [ ] PENDING
    - Save/reopen preserves singleton behavior: [ ] PENDING
3. Import a DAE with multiple `COL[...]` geometries:
    - Only one COL node is imported: [ ] PENDING
    - Imported COL node displays as `Root`: [ ] PENDING
4. Add Node modal:
    - Collision creation uses fixed `Root` name with no editable name field: [ ] PENDING
    - Adding a second collision is blocked until the existing COL is deleted: [ ] PENDING

---

SHA: `832ff97`
Timestamp: `01/06/2026 22:02`
Test hod: `ter_centaur`

**Goal**: Verify follow-up fix for the singleton COL context menu showing Delete when the COL node is named `Root`.

Automated checks:

1. Frontend verification:
    - `npm run build`: [x] PASS - existing Vite large chunk warning only.

Manual COL Context Menu Retest:

1. Open a HOD with a parsed COL node named `Root`:
    - Right-click COL opens a non-empty context menu: [x] PASS
    - Rename is still hidden for COL: [x] PASS
    - Delete option is visible for COL: [x] PASS
    - Delete removes the COL node: [x] PASS

---

SHA: `(working tree - STAT generic param fix)`
Timestamp: `03/10/2026 14:20`
Test hod: `testing/hgn_hft_hw2.hod`, `uncompressed_bigs` classic HW2 set

**Goal**: Verify `testing/hgn_hft_hw2.hod` contains (or does not contain) a hidden DIFF texture, and fix loading of all textures for classic HW2 / HOD 1.0 STAT materials.

File audit (no code change required):

1. IFF walk of `testing/hgn_hft_hw2.hod` (234988 B) covers the file exactly, no hidden chunks:
    - HVMD holds exactly 2 LMIP + 3 STAT + 1 MULT: [x] PASS
    - POOL texture stream (354944 B) equals exactly the 2 DXT5 mip chains: [x] PASS
    - Each STAT consumes its chunk exactly with `param_count=1`: [x] PASS
2. Texture content check (decoded + independent ImageMagick DDS decode):
    - `fth_exportDXT5`: R=0.00 / G=1.66 / B=201.55 -> glow-map look, bound as `$diffuse`: [x] CONFIRMED
    - `badge_exportDXT5`: R=0 / G=0 / B=181 flat blue: [x] CONFIRMED
    - Conclusion: the file has NO DIFF and no extra textures on `hw2Shader`. `INFO/OWNR` is `HODEditorJS` (both `hgn_hft.hod` and `hgn_hft_hw2.hod`, identical texture pools) -> already an editor export with the missing DIFF baked in.

Parser fix verification (generic STAT param records):

1. Parser/build verification:
    - `cargo check --lib`: [x] PASS - pre-existing warnings only.
    - `npx tsc --noEmit`: [x] PASS (note: `npm run build` fails on a pre-existing `dist/assets` EACCES permission issue, unrelated).
    - `cargo run --bin verify_lossless`: [x] PASS - all structural re-parse counts match. Size diffs vs original identical to stashed baseline (except `asteroid_3` -4 bytes: original `$glow` slot name is now preserved instead of being regenerated as `$specular`).
2. Previously crashing / misparsed files (`parser/src/bin/roundtrip_test.rs`, new diagnostic bin):
    - `FSFCv3/art/fx/bullet_splash/bullet_splash.hod` (nameless STAT): [x] PASS - used to die with `Failed to read param name: failed to fill whole buffer`; now loads `hw2Shader` -> `Bullet_splash3DXT5` and round-trips.
    - `FSFCv3/art/fx/ship_chunk01.hod` (nameless, type 5/5/4/4 mix): [x] PASS - both texture bindings and both 16-byte colour params parse.
    - `HWRM/resource/nebula/m05_dustcloud_nebula/m05_dustcloud_nebula.hod` (named, `Colour2`/`Colour3`): [x] PASS - type 4 colour params survive load -> save -> load byte-sized correctly.
    - `HWRM/resource/asteroid/asteroid4/asteroid4.hod` (11 params, mixed): [x] PASS - all 5 texture maps now load (`$diffuse`, `$glow`, `$spec`, `$progress`, `$normal`) instead of a truncated set.
    - `FSFCv3/ship/ter_loki/ter_loki.hod`, `FSFCv3/ship/shi_demon/shi_demon.hod`, `freespace_remastered/art/fx/debris_meshes/aeolus1.hod`, `freespace_remastered/resource/asteroid/asteroid_3/asteroid_3.hod`: [x] PASS - multi-param materials round-trip with original slot names (`$mask`, `$glow`, `$normal`) preserved.
3. Regression on the reported file:
    - `testing/hgn_hft_hw2.hod` still loads 2 textures / 3 materials / 1 map each: [x] PASS (unchanged, as expected - the file really has no more data).

---

SHA: `(working tree - HOD 1.0 texture name collision fix)`
Timestamp: `03/10/2026 16:40`
Test hod: `testing/hgn_hft.hod` (genuine HW2 classic original)

**Goal**: Analyze the genuine HW2 classic original `testing/hgn_hft.hod` (913,637 B, `OWNR` = "James O'Donnell", no POOL, 5 inline LMIP textures) and fix why the editor showed a GLOW texture as DIFF ("model looks blue").

File audit:

1. Genuine original confirmed: `OWNR` = "James O'Donnell" (Default-framed), no POOL, inline LMIP pixel data, `NAME` = "Homeworld2 Multi Mesh File": [x] PASS
2. Original contents (byte-exact):
    - `hw2Shader` [ship] `param_count=2`: `$diffuse`→idx0, `$glow`→idx1: [x] PASS
    - `BADGE1_MAT`/`BADGE0_MAT` [badge] `param_count=3`: `$mask`→idx2, `$diffuse`→idx3, `$glow`→idx4: [x] PASS
    - LMIP raw names are Photoshop layer exports: `fth_export.psd[1]`/`[2]`, `badge_export.psd[1]`/`[2]`/`[3]`: [x] PASS
3. Root cause identified in `parse_texture` name derivation: `rsplit_once('.')` on `fth_export.psd[1]` discards the trailing `[1]`, so DIFF and GLOW both became `fth_exportDXT5`; saves re-resolve bindings by name and dropped the DIFF (`fth_export.psd[1]`, R138/G110/B94) while keeping the GLOW (`fth_export.psd[2]`, R0/G2/B202) as `$diffuse`: [x] CONFIRMED (matches the reported symptom exactly)

Parser fix verification:

1. Parser/build verification:
    - `cargo check --lib`: [x] PASS - pre-existing warnings only.
    - `cargo run --bin verify_lossless`: [x] PASS - all structural re-parse counts match; size diffs identical to pre-existing baseline.
2. `testing/hgn_hft.hod` after fix (`parser/src/bin/dump_model.rs`, `roundtrip_test.rs`):
    - 5 uniquely named textures (`fth_export[1]DXT5`, `fth_export[2]DXT5`, `badge_export[1..3]DXT5`): [x] PASS
    - `hw2Shader` = `$diffuse`→`fth_export[1]DXT5` + `$glow`→`fth_export[2]DXT5`: [x] PASS
    - Badges = `$mask`→`badge_export[2]DXT5`, `$diffuse`→`badge_export[1]DXT5`, `$glow`→`badge_export[3]DXT5`: [x] PASS
    - Load → save → load keeps all 5 textures and all 8 param records: [x] PASS
    - Texture content now correct: DIFF is warm hull colour (R138/G110/B94), GLOW is blue (R0/G2/B202): [x] PASS
3. Regressions:
    - `shi_demon.hod` / `ter_zephyrus_1.3_original.hod` (`name[1].ext` convention): [x] PASS - texture names byte-identical to before the fix.
    - `testing/hgn_hft_hw2.hod` (HODEditorJS export with baked-in damage): [x] PASS - unchanged, still 2 textures (nothing more is recoverable from that file).

---

SHA: `1454668`
Timestamp: `16/10/2026 20:33`
Test hod: `testing/bst_mothership/bst_mothership.hod` (32,382,858 B)

**Goal**: Fix `IFF boundary read error: failed to fill whole buffer` when loading `testing/bst_mothership/bst_mothership.hod`.

Root cause:

1. The `POOL` size field at `0x3A` is big-endian `0x01E90E00` (~32 MB). BE is provably correct: `FORM ... HVMD` begins exactly at `0x3E + 0x01E90E00 = 0x1E90E3E`, and the BE chunk chain (`POOL` → `HVMD` → `DTRM` → `INFO`) ends exactly at EOF (`0x1EE1F8A`): [x] CONFIRMED
2. The old dynamic-endianness heuristic in `IffChunk::read_chunk` (`size > 0x00FFFFFF && size_le <= 0x00FFFFFF` → use little-endian) misread that size as `0x000EE901` (977,153 B) because the size's low byte is `0x00`. The parser resumed at `0xEE93F`, inside the Xpress-compressed texture pool, read a bogus chunk size (2,166,295,448) and failed the boundary read: [x] CONFIRMED
3. Fix: `IffChunk::resolve_chunk_size` now uses little-endian only when the big-endian size cannot fit in the bytes remaining in the stream while the little-endian size can; valid big-endian sizes can never be flipped: [x] DONE

Parser/build verification:

1. `cargo check --lib`: [x] PASS - pre-existing warnings only.
2. `cargo test --lib`: [x] PASS - 32 passed, including 4 new `iff::tests` (large BE size kept, LE fallback limited to overrun, LE `NRML BMSH` chunk read, 16 MiB `POOL` read).
3. `cargo run --bin verify_lossless`: [x] PASS - 4x `SUCCESS: Re-parsed generated file!`, size diffs `1316/-73438/-759998/-4817608`, all Original/Reparsed counts identical to the established baseline; DAE fallback `Output size: 140254 bytes`.

Previously failing file:

1. `testing/bst_mothership/bst_mothership.hod` (temporary `diag_load` bin, deleted before commit): loads with `meshes=39, joints=193, textures=53, materials=18, navlights=33, markers=45, glows=10, dockpaths=15, collisions=1`: [x] PASS
2. `dump_pool` reads all 6 top-level chunks (`VERS`, `NAME`, `POOL` len 32,050,688, `HVMD` 84 children, `DTRM` 28 children, `INFO`): [x] PASS
3. Regression sweep - real Rust `dump_pool` over all 505 HODs in `uncompressed_bigs` (290 vanilla ships, 4 pebbles, 114 HOD 1.3 ships), `HWRM_FSFC/source/ship` (92), and `testing/bst_mothership` (5): zero `Error reading chunk`: [x] PASS
4. Sibling files `bst_mothership2.hod`, `bst_mothership - current.hod`, `bst_mothership_wip2.hod`, `bst_mothership_old.hod`: load as before (35/35/35/39 meshes): [x] PASS

Editor manual test (load `testing/bst_mothership/bst_mothership.hod` in the app) - user-verified 16/10/2026:

1. Opened `bst_mothership.hod` in editor:
    - No loading errors: [x] PASS
    - Textures orientation: [x] PASS
    - Textures assigned to correct materials: [x] PASS
    - Full meshes shown: [x] PASS
    - Collision mesh loaded: [x] PASS
    - All expected nodes loaded: [x] PASS
2. In-game check of any exported file from this ship: [ ] NOT RUN (read-path-only fix; no output format changed)

---

SHA: `(working tree - coordinates, muzzles, barrels, orientation)`
Timestamp: `04/11/2026 14:23`
Test hod: `hgn_destroyer.hod`, `kus_defender.hod`, `hgn_battlecruiser.hod` (static audit); editor build verification

**Goal**: Implement the shared Local/Global coordinate-space toggle, multi-muzzle weapon assemblies, optional/unverified Barrel handling, and confirm that editor HOD coordinates match the game.

Static orientation audit:

1. `hgn_destroyer.hod` weapon joints point +Z (weapon positions around Z=+192): [x] CONFIRMED
2. `hgn_destroyer.hod` engine nozzle sits at -Z (around Z=-177): [x] CONFIRMED
3. Conclusion: HWRM HODs are Y-up/+Z-forward, the viewport consumes HOD coordinates directly, and no parser/viewport axis remap is needed: [x] CONFIRMED
4. DAE import intentionally discards the top-level export transform (Blender Z_UP→Y_UP wrapper); regression test `dae_top_level_export_transform_is_discarded_in_y_up_space` added: [x] PASS

Weapon corpus audit:

1. `kus_defender.hod` uses `Weapon_Gun0/1/2_Muzzle` under `_Latitude` at +Z ~7.64: [x] CONFIRMED
2. `hgn_battlecruiser.hod` uses one `_Muzzle` per weapon: [x] CONFIRMED
3. No `_Barrel` joints were found in the audited vanilla HWRM ship HODs; Barrel remains optional/unverified: [x] CONFIRMED

Automated verification:

1. `npx tsc --noEmit`: [x] PASS (no output)
2. `npm run build`: [x] PASS (1776 modules transformed, built in 2.22s; existing chunk-size warnings only)
3. `cargo test --manifest-path parser/Cargo.toml`: [x] PASS - 37 passed, 0 failed; includes the new DAE top-level transform regression test
4. `cargo run --manifest-path parser/Cargo.toml --bin verify_lossless`: [x] PASS - exit 0; all four generated files re-parsed with identical structural counts (Pebble 2/1/0/0/0, Elysium 3/30/16/5/8, Fenris 7/51/4/11/4, Asteroid 3 3/4/0/0/0); expected regenerated-pool size deltas +136, -74546, -761130, -5336577 bytes; DAE fallback 140254 bytes

Manual editor/in-game test (pending user):

1. Open a HOD with a nested joint; toggle Local/Global; confirm the viewport gizmo axes and Inspector numbers change together: [ ] NOT RUN
2. Edit position/rotation in Global, toggle back to Local, and confirm the local values and save round-trip: [ ] NOT RUN
3. Create a weapon assembly; add a 2nd/3rd muzzle; rename the assembly; confirm all muzzles follow: [ ] NOT RUN
4. Delete the assembly; confirm full muzzle cleanup: [ ] NOT RUN
5. Save/reload; confirm muzzles persist: [ ] NOT RUN
6. Load an exported ship in game; confirm orientation (Y-up, +Z forward) matches the editor: [ ] NOT RUN
7. If a test asset uses `_Barrel`, record the observed runtime behavior before changing the optional/unverified ruling: [ ] NOT RUN

---

SHA: `(working tree - static weapon audit + orientation probe)`
Timestamp: `04/11/2026 14:33`
Test hod: 290 vanilla HWRM ship HODs; `testing/orientation_probe/orientation_probe.{DAE,hod}`; `kus_defender.hod`

**Goal**: Complete the remaining static plan items: full vanilla muzzle/Barrel corpus sweep, editor-vs-HODOR turret comparison, and an asymmetric orientation test asset for the user's in-game check.

### 1. Full vanilla muzzle/Barrel corpus sweep

Command: `cargo run --manifest-path parser/Cargo.toml --bin sweep_weapon_joints -- "<...>/uncompressed_bigs/HWRM/ship"` (temporary diagnostic; deleted after the audit).

1. Files scanned: 290 HODs; parse failures: 0.
2. Weapon assemblies containing muzzle/Barrel joints: 397.
3. Muzzle count distribution: exactly 1 muzzle in all 397 assemblies.
4. Muzzle suffix distribution: `_Muzzle` = 397; `_MuzzleN` = 0.
5. Non-standard muzzle/Barrel names: 0.
6. `_Barrel` joints: 0 across 0 assemblies.
7. Muzzle parent distribution: `${base}_Position` = 313; `${base}_Latitude` = 83; `Root` = 1.
   - The `Root` exception is `vgr_commandcorvette/vgr_commandcorvette.hod` `Weapon_Gun` (`Weapon_Gun_Muzzle` at `(0,0,40.957)`), so diagnostics now accept direct `Root` parenting.
8. Conclusion: multi-muzzle numbering is an editor/mod extension, not a vanilla HWRM ship convention; Barrel is absent from the vanilla corpus.

### 2. Editor turret template vs vanilla reference

`HODOR.exe`/`RODOH.exe` are Windows PE binaries and no Wine runtime is installed on this host, so a fresh HODOR conversion could not be run. Official vanilla HWRM assets from the HODOR pipeline were used as the reference instead.

Comparison target: `kus_defender.hod` `Weapon_Gun0`.

| Joint | Parent | Vanilla | Editor template |
|---|---|---|---|
| `Position` | hull / none | `(-0.070, 0.000, -0.382)` on `HeavyDefenderArm.0` | `(0,0,0)` at chosen parent |
| `Direction` | `Position` | `(0, 10.000, 0)` | `(0, 5.000, 0)` |
| `Latitude` | `Position` | `(0,0,0)` | `(0,0,0)` |
| `Muzzle` | `Latitude` | `(0,0,7.639)` | `(0,0,7.640)` |
| `Rest` | `Position` | `(0,0,10.117)` | `(0,0,10.000)` |

1. Five-joint chain and parenting match exactly: [x] CONFIRMED
2. Muzzle/Latitude placement matches within 0.001 units: [x] CONFIRMED
3. Direction/Rest offsets vary per ship and are editor defaults, not structural mismatches: [x] CONFIRMED
4. Editor template round-trips through the HOD writer and re-parses as the same chain: [x] PASS

### 3. Asymmetric orientation probe

Prepared `testing/orientation_probe/`:

- `orientation_probe.DAE` — hand-authored Y-up/+Z-forward asset with an asymmetric box hull, axis joints, markers, a dockpath, and a collision hull.
- `orientation_probe.hod` — generated by parsing the DAE and serializing through the editor's HOD 2.0 writer (1,922 bytes).
- `orientation_probe.ship` — starting ship script copied from the Acolyte with weapon/engine-effect lines removed; register it with the usual test-mod pipeline.
- `README.md` — editor and in-game check steps.

Static verification:

1. `parse_dae` and reparse of the generated HOD both report `Nose_PX=(100,0,0)`, `Dorsal_PY=(0,100,0)`, `Forward_PZ=(0,0,100)`: [x] PASS
2. Markers `PX_Nose`, `PY_Dorsal`, `PZ_Forward` survive at the same positions: [x] PASS
3. `probe_dock[0]` survives at `(0,0,120)` with parent `world`: [x] PASS
4. Mesh `probe_hull` LOD 0 and one collision hull survive: [x] PASS
5. `validate_hod testing/orientation_probe/orientation_probe.hod`: [x] PASS - 0 errors (11 warnings/info: unused pool bytes, empty STAT params, normal COLD/KDOP/HIER/INFO findings)
6. `npm run build` after the diagnostics change: [x] PASS - TypeScript + Vite, built in 2.18s; existing chunk-size warnings only
7. `cargo test --manifest-path parser/Cargo.toml` after adding the temporary diagnostics: [x] PASS - 37 passed, 0 failed

### 4. Manual checks for the user

1. Open `testing/orientation_probe/orientation_probe.hod`; confirm the long hull axis points to `PZ_Forward` (+Z), the tall side is `+Y`, and the Inspector shows the axis positions above: [x] PASS (user)
2. Toggle Local/Global on the probe nodes and on a nested ship joint; confirm gizmo axes and Inspector numbers change together: [x] PASS (user)
3. Edit position/rotation in Global, toggle back to Local, save/reload, and confirm values persist: [x] PASS (user)
4. Create a weapon assembly; add a 2nd/3rd muzzle; rename the assembly; confirm all muzzles follow; delete and confirm cleanup: [ ] NOT RUN
5. Spawn/load `orientation_probe` in game and confirm the long axis is forward and `+Y` is up: [x] PASS (user: correct rendering confirmed)
6. If a test asset uses `_Barrel`, record the observed runtime behavior: [ ] NOT RUN

---

SHA: `(working tree - Slave barrel audit + implementation)`
Timestamp: `04/11/2026 15:03`
Test hod: `hgn_assaultcorvette.hod`, `hgn_destroyer.hod`, `kus_missiledestroyer.hod`; `HWCR/weapon/unh_ddmissile/unh_ddmissile.wepn`

**Goal**: Follow up the user's note that vanilla multi-barrel weapons use a `Slave` suffix rather than extra `_MuzzleN` joints, audit the corpus, and add editor support.

### 1. User-confirmed results from the previous snapshot

1. Direction verification: [x] PASS (user)
2. Correct rendering: [x] PASS (user)
3. Local/Global toggle: [x] PASS (user)
4. Saving/reloading: [x] PASS (user)
5. Multi-muzzle rename/delete: [x] PASS (user, covered by "tested everything")
6. Barrel runtime behavior: not reported (still open)

### 2. Slave barrel corpus audit

1. 290 vanilla HWRM ship HODs scanned; 45 slave assemblies across 16 weapon families: [x] CONFIRMED
2. Naming: 24 `Slave`, 11 `Slave1`, 10 `Slave2`: [x] CONFIRMED
3. Joint counts: 37 four-joint fixed chains, 8 five-joint turret chains: [x] CONFIRMED
4. Every scanned slave assembly has its own `Position` and `Muzzle`: [x] CONFIRMED
5. Examples:
    - `hgn_assaultcorvette.hod`: `Weapon_TurretTop` / `Weapon_TurretTopSlave`; main muzzle `+X 1.637`, slave `-X 1.626`.
    - `hgn_destroyer.hod`: `Weapon_Torpedo` / `Weapon_TorpedoSlave`; separate `Position` joints at `+X 9.328` and `-X 8.981`.
    - `kus_missiledestroyer.hod`: `Weapon_GunV0` plus `Slave`, `Slave1`, `Slave2`.
6. `.wepn` evidence: `setMiscValues(NewWeaponType, recoil, slaveDelay)`; vanilla `unh_ddmissile.wepn` comments call the second argument "Delay between inital firing and firing of slave barrels": [x] CONFIRMED
7. `.ship` binds only the main weapon; no `Slave` references were found in the scanned `.ship` files: [x] CONFIRMED
8. Conclusion: vanilla multi-barrel = parallel `${base}SlaveN` assembly; `_MuzzleN` is an editor extension for explicit multiple `StartShipWeaponConfig` bindings: [x] CONFIRMED

### 3. Editor implementation

1. `src/weapon_utils.ts` detects slave bases, selects the next free `SlaveN` name, identifies family joints, and maps family renames: [x] DONE
2. `WeaponGroupInspector.tsx` lists slave assemblies, adds a cloned slave chain (turret chains mirror the primary muzzle on X), removes a slave chain, and links a slave row back to its parent: [x] DONE
3. `HierarchyTree.tsx` labels slave rows, cascades main rename/delete/visibility to slaves, and warns when a slave is missing `Position`/`Muzzle`: [x] DONE
4. Docs updated: `02`, `07`, `08`, `09`, knowledge base, `README.md`, `PROGRESS.md`: [x] DONE
5. `npx tsc --noEmit`: [x] PASS
6. Helper behavior test (`getSlaveBases` returns 3 bases, `nextSlaveBase` returns `Slave3`, `Slave1` renames correctly, non-slave names excluded, clone produces the parallel chain with the muzzle mirrored from `+1.637` to `-1.637`): [x] PASS
7. `npm run build`: [x] PASS (TypeScript + Vite, 2.12s)
8. `cargo test --manifest-path parser/Cargo.toml`: [x] PASS - 37 passed, 0 failed

### 4. Manual checks for the user

1. Open `hgn_assaultcorvette.hod` (or any ship with a `Slave` chain); select `Weapon_TurretTop`; confirm the Slave Barrels section lists `Weapon_TurretTopSlave`: [x] PASS (user)
2. Add a slave to a weapon; confirm a full parallel chain is created; for a turret, confirm the primary muzzle is mirrored on X: [x] PASS (user)
3. Rename the main weapon; confirm all slave chains follow. Delete the main weapon; confirm slave chains are removed. Toggle main visibility; confirm slaves follow: [x] PASS (user)
4. Save/reload and confirm slave chains survive: [x] PASS (user)
5. In game, use a weapon with a non-zero `.wepn` `setMiscValues` slave delay; bind only the main weapon in the `.ship`; confirm both barrels fire: [x] PASS (user)
6. If the game does not fire the slave, record whether the `.ship` must bind the `Slave` joint explicitly: [x] NOT APPLICABLE - user confirmed the tested Slave workflow works with only the main weapon bound

---

SHA: `b5aab8c`
Timestamp: `04/11/2026 15:12`
Test hod: `hgn_assaultcorvette.hod`, `hgn_destroyer.hod`, `kus_missiledestroyer.hod` (Slave workflow); orientation probe (previous snapshot)

**Goal**: Record final user verification of the Slave barrel implementation and the completed orientation/coordinate checks.

### User verification results

1. Slave Barrels section lists the vanilla `Slave` chain: [x] PASS
2. Add slave creates a full parallel chain; turret muzzle mirroring works: [x] PASS
3. Main-weapon rename/delete/visibility cascades to slaves: [x] PASS
4. Save/reload preserves slave chains: [x] PASS
5. In-game firing with the `.wepn` `setMiscValues` slave delay works; only the main weapon is bound in the `.ship`: [x] PASS
6. Direction verification, rendering, Local/Global toggle, and save/reload from the previous snapshot: [x] PASS
7. `Barrel` runtime behavior: [ ] NOT RUN (remains optional/unverified)

**Conclusion**: The Slave barrel workflow is user-verified end-to-end and accepted. No further development is required for the vanilla multi-barrel convention. `Barrel` remains the only unverified weapon-joint behavior.

---

SHA: `(working tree - multi-muzzle audit correction)`
Timestamp: `04/11/2026 15:33`
Test hod: `vgr_heavyfusionmissilelauncherturret.hod`, `vgr_heavyfusionmissilelauncherturretleft.hod`, `kus_defender.hod`; full HWRM root

**Goal**: Check whether `_MuzzleN` has real vanilla usage before removing the multi-muzzle feature.

### Full HWRM root scan

1. Files scanned: 870 HODs (`ship`, `subsystem`, `resource`, and other HWRM roots), parse failures: 0.
2. Files with muzzle joints: 135.
3. Muzzle-bearing assemblies: 158.
4. Files with `_MuzzleN` (N > 0): **2**.
5. Index pattern: `[0, 2, 3]` in both files.
6. Exact users:
    - `subsystem/vgr_heavyfusionmissilelauncherturret/vgr_heavyfusionmissilelauncherturret.hod` — `Weapon_MissileRight_Muzzle`, `_Muzzle2`, `_Muzzle3`, all parented to `_Latitude`.
    - `subsystem/vgr_heavyfusionmissilelauncherturretleft/vgr_heavyfusionmissilelauncherturretleft.hod` — `Weapon_MissileLeft_Muzzle`, `_Muzzle2`, `_Muzzle3`, all parented to `_Latitude`.
7. `kus_defender.hod` contains the string `Weapon_Gun2_Muzzle1`, but it is not present in the HIER joint list; not a real weapon joint.
8. Conclusion: `_MuzzleN` is real but rare; the earlier "zero `_MuzzleN`" claim came from the ship-only sweep. The multi-muzzle feature is retained.

### Code correction

1. `getWarnings` no longer warns about missing primary `_Muzzle` or non-contiguous numbering: [x] DONE
2. Required `Muzzle` check accepts any `_MuzzleN`: [x] DONE
3. Add Muzzle uses `max(existing index) + 1`, so `0,2,3` adds `4`: [x] DONE
4. Non-standard suffix and parenting warnings remain (Root allowed): [x] DONE
5. `npx tsc --noEmit`: [x] PASS
6. Helper test (`muzzleIndex` returns `0,2,3`; `nextMuzzleIndex` returns `4`; `muzzleName` returns `_Muzzle4`): [x] PASS

### Manual checks for the user

1. Open `vgr_heavyfusionmissilelauncherturret.hod`; confirm the weapon inspector lists `Muzzle`, `Muzzle2`, `Muzzle3` and no gap warning: [ ] NOT RUN
2. Add a muzzle to that chain; confirm it creates `_Muzzle4`: [ ] NOT RUN
3. Save/reload; confirm numbering survives: [ ] NOT RUN

---

SHA: `98b6658`
Timestamp: `04/11/2026 16:25`
Test hod: `testing/acolyte/smt_fta_acolyte.hod` (material `page022`, shader `thruster`, five bound slots) or `testing/bst_mothership/bst_mothership.hod` (material `Bst_Mothership_Thruster`); vanilla `missile/tai_missile/tai_missile.hod` (material `Tai_Missile_Engine`) has all six slots for the full alias check

**Goal**: Verify that thruster STAT texture aliases survive a save and that the engine nozzle no longer falls back to white diffuse / black glow or a fake bump from the normal slot.

### Automated pre-verification (agent)

1. `cargo test --lib --manifest-path parser/Cargo.toml`: [x] PASS (43 tests, including canonical-alias, shared-binding, compact-layout, and auto-assign regression tests)
2. `cargo run --bin verify_lossless`: [x] PASS (all four fixtures reparse with identical structural counts; generated pebble_0/asteroid_3 bytes identical to the pre-change baseline; expected size deltas `136/-74546/-761130/-5336577`)
3. Vanilla `tai_missile.hod` load → `auto_assign_and_resize_textures` → save → reparse keeps the `thruster` material as `$diffuseOn,$diffuseOff,$glowOn,$glowOff,$team,$normal` with the matching `_DIFF/_DIFX/_GLOW/_GLOX/_TEAM/_NORM` values: [x] PASS
4. `testing/acolyte/smt_fta_acolyte.hod` load → auto-assign → save → reparse keeps all five aliases `$diffuseOn,$diffuseOff,$glowOn,$glowOff,$team` and preserves the intentionally shared DIFF/DIFX textures in GlowOff/Team instead of dropping them: [x] PASS

### Manual editor checks

1. Open `smt_fta_acolyte.hod`; Inspector → material `page022` shows the five slots the file actually binds (Diffuse On, Diffuse Off, Glow On, Glow Off, Team) with the expected textures; open vanilla `tai_missile.hod` to also see Normal: [x] PASS (user)
2. Save as `*_from_2.0_to_2.0.hod`; reopen the saved file and confirm the bound slots are unchanged and no slot gained another slot's texture: [x] PASS (user)
3. Inspect the saved STAT entries (`cargo run --bin dump_pool -- <saved.hod>`). For `smt_fta_acolyte.hod` the aliases must be `$diffuseOn`, `$diffuseOff`, `$glowOn`, `$glowOff`, `$team`; for vanilla `tai_missile.hod` they must be `$diffuseOn`, `$diffuseOff`, `$glowOn`, `$glowOff`, `$team`, `$normal`. No `$diffuse`, no under-`$glow`/`$normal` shifts, Team not dropped: [x] PASS (user)

### Manual in-game checks

1. Load the saved HOD in game; thruster on-state diffuse matches the authored texture (not `default_white`): [x] PASS (user)
2. Thruster glow on-state uses the authored glow map (not `default_black`): [x] PASS (user)
3. Off-state diffuse/glow (`_DIFX`/`_GLOX`) appear when the engine is inactive: [x] PASS (user)
4. For the six-slot vanilla test HOD, the Normal slot shows the authored normal map, with no fake bump from the glow texture: [x] PASS (user)
5. No crash on load and the ship renders normally: [x] PASS (user)

**User verification**: PASS — user reported the thruster fix works fine.
