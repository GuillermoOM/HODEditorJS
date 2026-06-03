# TODO

## Things to complete:

- [x] Each joint should be toggeable
- [x] Joints (general use)
- [x] Quick visibility toggles
- [x] Collision Meshes (automatic)
- [x] Materials (with shader selection, format, using a main DIFF.TGA to load with the rest of texture files)
- [x] Ship Meshes (multiple LODs per Mesh, assigning materials)
- [x] Show something to let the user know a HOD is loading.
- [x] Scrollable value edit fields (coordinates, values, etc.)
- [x] Nodes should be draggable in the hierarchy to position under a parent node.
- [x] Exporting and importing of Materials and 3d models
- [x] Dockpaths loading and proper inspector (ter_orion and hgn_mothership are examples that use this)
- [x] Root node auto created on New HOD creation
- [x] Add Node Button (opens prompt to select type of node).
- [x] Nodes such as these: Weapon, Turret, Hardpoint, Capture Point, Repair Point, Salvage Point. Should be shown as a single node while the inspector shows the sub nodes, since moving the subnodes it can break the HOD with accidental changes, let's reserve them as special nodes and leave normal joint nodes as separate where we can drag or add nodes into. There are templates in the old repos (DAEnerys and CFHodEd) that show how they are added.
- [x] Engine Burns (mutiple flame points can be added)
- [x] Engine Glows are also meshes (untextured), added only to Engine Nozzle nodes
- [x] Engine Shape draggable to a joint node (also editable coordinates and rotation)
- [x] Errors and warnings (missing basic data like a Root node, broken or incomplete weapon nodes/joints)
- [x] make navlight nodes also draggable in the skeleton tree
- [x] Fix HOD 1.0 Material Loading (testing with `/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld 347380/GBXTools/WorkshopTool/uncompressed_bigs/freespace_remastered/ship/shi_cain/shi_cain.hod` which the ui isn't showing any materials. Material loading is working well when loading HOD 2.0)
- [x] tested ter_elysium.hod (2.0) loading and saving with the editor, the output file loads in game, but the ship is turned 90 degrees left and tilting a bit to the left about 30 degrees, maybe something to check out on (ter_elysium.hod (original) vs ter_elysium2.hod (created by editor, both located at `/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld/HWRM_FSFC/source/ship/ter_elysium/`).
- [x] Make the node spheres slightly bigger for improve visuals, same with axis helpers. It is hard to see in bigger ships. Should increase in size more if highlighted
- [x] Target Boxing editor (for adding to .ship files, DAEnerys had this, it is a separate tool that adds a viewable box that outputs the piece of code it represents, this isn't related to modifying the HOD at all)
- [x] Fix shaders for HOD 2.0 materials. I notice now that we load materials for HOD 1.0 properly, the shaders work correctly, so the shaders might be setup for those old material setups and not for HOD 2.0 materials? Something to investigate deeper here.
- [x] Collission box renderer hides navlights for some reason.
- [x] ter_elysium.hod created by editor fails to load in game, gives error log: `Unknown basich mesh version (2013593600)-- FATAL EXIT --basicmesh/489:!--stack trace--`.
- [x] Remove auto creation of collision mesh.
- [x] Repair auto loading and assignment of Texture mappings on materials (TEAM must align to TEAM mapping, GLOW to GLOW, NORM to NORM, etc.)
- [x] ter_zephyrus.hod has an existing docking path, but when loaded into editor, the path is rendered but not shown on the node tree.
- [x] loading and saving HOD 2.0 ter_zephyrus.hod crashes the game on load.
- [x] If a HOD loaded has any "assemblies" (weapon, capture, repair, etc.) sometimes they contain the proper structure but don't follow the naming conventions properly (es. ter_fenris.hod has CapturePoint0 and has children CapturePoint0Heading instead of CapturePoint0_Heading). We need to make sure to auto repair these, or when repairing we should auto clean up those nodes (assuming they have the Heading, Left, Up nodes).
- [x] We need to fix the editor alerts for missing components in assemblies, overall alerts in HOD being edited isn't following spec
- [x] Since Weapon and Turret assemblies are similar, we should be able to have an option to convert Weapon into Turret assembly in the inspector, which would add any missing nodes (must be repaired first).
- [x] Creation and Loading of Animations and proper inspector / editor (ter_orion and hgn_mothership are examples that use this) - Added Create Animation, Add Track, and Add Keyframe controls.
- [x] Make Hardpoint nodes assemblies
- [x] GLOW textures are rendering wrong in the texture shaded mode in editor, shader is interpreting wrong the colors, see here to compare files (original and edited) `/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld/HWRM_FSFC/source/ship/ter_elysium/`
- [x] Renaming an Assembly should only be done via the Assembly node (not position node), and make sure the assembly isn't broken when renamed (i.e. I renamed a Repair assembly node and it disbanded into normal joint nodes with the assembly gone!)
- [x] Some HOD 2.0 files (HWRM Vanilla) show their textures broken in the viewport
- [x] fix loading of HOD 1.0 files into the editor.
- [x] Fix the following HOD 1.0 loading issues:
    - Animations not being loaded properly (not shown in editor for `/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld 347380/GBXTools/WorkshopTool/uncompressed_bigs/freespace_remastered/ship/ter_orion/ter_orion.hod`)
    - Docking Paths not being loeaded properly (not shown in editor for `/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld 347380/GBXTools/WorkshopTool/uncompressed_bigs/freespace_remastered/ship/ter_orion/ter_orion.hod`)
- [x] Saving a HOD file (which creates a HOD 2.0) renders wrong in game, showing the whole mesh being turned some degrees to the left and down.
- [x] Loading HOD 1.3 file still needs y flipping on all textures, it happened only on the first one
- [x] Open HOD 2.0, "save it", compare with original HOD 2.0 file loaded to see if it similar
- [x] Open HOD 1.0, test how it loads and which nodes are incorrectly assigned or unknown to HOD 2.0 template (need a template for 2.0 based on all knowledge we gathered on nodes).
- [x] Create a new HOD and save it, test it in game.
- [x] Open files by double cliking on them
- [x] Rotation / scaling also seems to apply to all nodes! (engine burns looking sideways)
- [x] Loading HOD 1.3 Collosus breaks
- [x] Loading HOD 1.3 Hecate creates navlight nodes without coordinate editing
- [x] Many nodes show up on viewport that are missing on the node tree
- [x] DXT3 Texture compatiblity
- [x] Remove hardcoded nameplate.bmp texture
- [x] Make the Viewport Layer toggles also toggle visibility on all nodes but also have options to toggle markers and joint nodes
- [x] Need to make sure all texture names have their proper _TYPE suffix both in editor and on saved/loaded HODs and exported materials/textures
- [ ] Textures lose their "smoothness" when saving, in game they render blocky

- [ ] Editor works slow on bigger ships with many nodes, it seems to be rerendering everything on changing anything in the editor (node coordinate, node dragging, etc).


- [ ] Fix auto camera distance on loading model
- [ ] Drag based positioning / rotating in viewport (disable camera control with left click, leave it for right click)
- [ ] reintroduce Badges
- [ ] Proper GLOW and SPEC texture generation
- [ ] LOD mesh generation

- [ ] Render shaders same as in game
- [ ] Saving as HW2 Classic HOD (Actual HOD 1.0)
- [ ] Saving as HOD 1.3 (correct name, not HOD 1.0)
- [ ] Colorscheme manager, like a dropdown list that changes the collection of color samples to house the colorscheme per game (HW1, HW2, HWCata, HWMobile, Personal set, etc);
- [ ] Autobudget, meaning if the software can upscale according to the current size of the texture (or size of the ship).  Right now, I think 4K can be the limit, but you don't have to create a 4K texture for a ship engine, for example. Badges material don't need upscaling
