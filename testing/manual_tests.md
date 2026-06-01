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
    - All expected nodes loaded: [ ] FAIL - Docking path and points nodes are missing
4. Loaded `_from_2.0_to_2.0.hod` in game:
    - No crash on loading: [x] PASS
    - Textures orientation: [x] PASS
    - Textures assigned to correct materials: [x] PASS
    - Full meshes shown: [x] PASS
    - Correct Ship orientation [x] PASS
    - All expected nodes working: [ ] FAIL - Due to missing docking paths and points, ships can't dock

1.0 HOD Test:

1. Opened `*_1.0_original.hod` in editor:
    - No loading errors: [x]
    - Textures orientation: [ ] FAIL - Textures are Y Flipped (Loading error here)
    - Textures assigned to correct materials: [x] PASS
    - Full meshes shown: [x] PASS
    - Collision mesh loaded: [x] PASS
    - All expected nodes loaded: [x] PASS
2. Saved as `*_from_1.0_to_2.0.hod`:
    - No saving errors: [x]
3. Opened `_from_1.0_to_2.0.hod` in editor again:
    - No loading errors: [x]
    - Textures orientation: [ ] FAIL - Textures are Y Flipped (no change from saving 1.0)
    - Textures assigned to correct materials: [ ] FAIL - Some Textures are assigned to the wrong materials
    - Full meshes shown: [x] PASS
    - Collision mesh loaded: [x] PARTIAL PASS - Apparently a new COL node was added when there was already one (this might be an effect from saving as HOD 2.0)
    - All expected nodes loaded: [ ] FAIL - Docking path and points nodes are missing
4. Loaded `_from_1.0_to_2.0.hod` in game:
    - No crash on loading: [x] PASS
    - Textures orientation: [ ] FAIL - Textures are Y Flipped
    - Textures assigned to correct materials: [ ] FAIL - Some Textures are assigned to the wrong materials
    - Full meshes shown: [X] PASS
    - Correct Ship Orientation: [ ] FAIL - Ship not looking towards their forward vector
    - All expected nodes working: [ ] FAIL - Due to missing docking paths and points, ships can't dock
---
---
Timestamp: `01/06/2026 21:30`
Test hod: `ter_zephyrus`

**Goal**: Verify POOL mesh crash fix and DOCK chunk loss fix during save.

2.0 HOD Test:

1. Opened `*_2.0_original.hod` in editor:
    - No loading errors: [ ] 
    - Textures orientation: [ ] 
    - Textures assigned to correct materials: [ ] 
    - Full meshes shown: [ ] 
    - Collision mesh loaded: [ ] 
    - All expected nodes loaded: [ ] 
2. Saved as `*_from_2.0_to_2.0.hod`:
    - No saving errors: [ ] 
3. Opened `_from_2.0_to_2.0.hod` in editor again:
    - No loading errors: [ ] 
    - Textures orientation: [ ] 
    - Textures assigned to correct materials: [ ] 
    - Full meshes shown: [ ] 
    - Collision mesh loaded: [ ] 
    - All expected nodes loaded: [ ] (Please verify docking path nodes are now loaded)
4. Loaded `_from_2.0_to_2.0.hod` in game:
    - No crash on loading: [ ] 
    - Textures orientation: [ ] 
    - Textures assigned to correct materials: [ ] 
    - Full meshes shown: [ ] 
    - Correct Ship Orientation: [ ] 
    - All expected nodes working: [ ] (Please verify docking works now)

1.0 HOD Test:

1. Opened `*_1.0_original.hod` in editor:
    - No loading errors: [ ]
    - Textures orientation: [ ] 
    - Textures assigned to correct materials: [ ] 
    - Full meshes shown: [ ] 
    - Collision mesh loaded: [ ] 
    - All expected nodes loaded: [ ] 
2. Saved as `*_from_1.0_to_2.0.hod`:
    - No saving errors: [ ]
3. Opened `_from_1.0_to_2.0.hod` in editor again:
    - No loading errors: [ ] (This should no longer crash!)
    - Textures orientation: [ ] 
    - Textures assigned to correct materials: [ ] 
    - Full meshes shown: [ ] 
    - Collision mesh loaded: [ ] 
    - All expected nodes loaded: [ ] (Please verify docking path nodes are now loaded)
4. Loaded `_from_1.0_to_2.0.hod` in game:
    - No crash on loading: [ ] 
    - Textures orientation: [ ] 
    - Textures assigned to correct materials: [ ] 
    - Full meshes shown: [ ] 
    - Correct Ship Orientation: [ ] 
    - All expected nodes working: [ ] (Please verify docking works now)
