# Manual Testing for HODEditorJS inputs and outputs

## Objective

The objective of this document is to log happy path testing to make sure everything works as it should, any agent reading this document will be able to look at the testing history at each snapshot (by timestamp generated after changes were done). So when a change is applied, a new test section must be added following the current template.

Agent would create this file in the testing subdirectory for a HOD, adding test snapshots from the template for the user to fill in after manual testing (snapshot date should be added by the agent for when it changed something and asks the user to test). On every change there must be a commit to track the snapshots.

## Resources

All test resources (original files being edited and output files resulting from editing) are located here:

`/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld 347380/GBXTools/WorkshopTool/mod-tools/HODEditorJS/testing/`

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
For HOD 1.0 files loaded onto editor from `/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld 347380/GBXTools/WorkshopTool/uncompressed_bigs/freespace_remastered/ship/`
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

SHA: `pending`
Timestamp: `01/06/2026 22:02`
Test hod: `ter_centaur`

**Goal**: Verify follow-up fix for the singleton COL context menu showing Delete when the COL node is named `Root`.

Automated checks:

1. Frontend verification:
    - `npm run build`: [x] PASS - existing Vite large chunk warning only.

Manual COL Context Menu Retest:

1. Open a HOD with a parsed COL node named `Root`:
    - Right-click COL opens a non-empty context menu: [ ] PENDING
    - Rename is still hidden for COL: [ ] PENDING
    - Delete option is visible for COL: [ ] PENDING
    - Delete removes the COL node: [ ] PENDING
