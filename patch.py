import os

hierarchy_file = "/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld 347380/GBXTools/WorkshopTool/mod-tools/HODEditorJS/src/components/HierarchyTree.tsx"
app_file = "/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld 347380/GBXTools/WorkshopTool/mod-tools/HODEditorJS/src/App.tsx"

with open(hierarchy_file, "r") as f:
    hierarchy = f.read()

# 1. Turret template
turret_old = """      const base = name.startsWith("Weapon_") || name.startsWith("weapon_") ? name : `Weapon_${name}`;
      const posName = `${base}_Position`;
      const latName = `${base}_Latitude`;
      const dirName = `${base}_Direction`;
      const barName = `${base}_Barrel`;
      const muzName = `${base}_Muzzle`;
      const restName = `${base}_Rest`;

      const templateJoints = [
        {
          name: posName,
          parent_name: parent === "(None)" ? undefined : parent,
          local_transform: {
            m: [
              [1, 0, 0, 0],
              [0, 1, 0, 0],
              [0, 0, 1, 0],
              [0, 0, 0, 1]
            ]
          }
        },
        {
          name: dirName,
          parent_name: posName,
          local_transform: {
            m: [
              [1, 0, 0, 0],
              [0, 1, 0, 0],
              [0, 0, 1, 0],
              [0, 5.0, 0, 1]
            ]
          }
        },
        {
          name: latName,
          parent_name: posName,
          local_transform: {
            m: [
              [1, 0, 0, 0],
              [0, 1, 0, 0],
              [0, 0, 1, 0],
              [0, 0, 5.0, 1]
            ]
          }
        },
        {
          name: barName,
          parent_name: latName,
          local_transform: {
            m: [
              [1, 0, 0, 0],
              [0, 1, 0, 0],
              [0, 0, 1, 0],
              [0, 5.0, 0, 1]
            ]
          }
        },
        {
          name: muzName,
          parent_name: barName,
          local_transform: {
            m: [
              [1, 0, 0, 0],
              [0, 1, 0, 0],
              [0, 0, 1, 0],
              [0, 5.0, 0, 1]
            ]
          }
        },
        {
          name: restName,
          parent_name: posName,
          local_transform: {
            m: [
              [1, 0, 0, 0],
              [0, 1, 0, 0],
              [0, 0, 1, 0],
              [0, 0, 5.0, 1]
            ]
          }
        }
      ];"""
turret_new = """      const base = name.startsWith("Weapon_") || name.startsWith("weapon_") ? name : `Weapon_${name}`;
      const posName = `${base}_Position`;
      const latName = `${base}_Latitude`;
      const dirName = `${base}_Direction`;
      const muzName = `${base}_Muzzle`;
      const restName = `${base}_Rest`;

      const templateJoints = [
        {
          name: posName,
          parent_name: parent === "(None)" ? undefined : parent,
          local_transform: {
            m: [
              [1, 0, 0, 0],
              [0, 1, 0, 0],
              [0, 0, 1, 0],
              [0, 0, 0, 1]
            ]
          }
        },
        {
          name: dirName,
          parent_name: posName,
          local_transform: {
            m: [
              [1, 0, 0, 0],
              [0, 1, 0, 0],
              [0, 0, 1, 0],
              [0, 5.0, 0, 1]
            ]
          }
        },
        {
          name: latName,
          parent_name: posName,
          local_transform: {
            m: [
              [1, 0, 0, 0],
              [0, 1, 0, 0],
              [0, 0, 1, 0],
              [0, 0, 5.0, 1]
            ]
          }
        },
        {
          name: muzName,
          parent_name: latName,
          local_transform: {
            m: [
              [1, 0, 0, 0],
              [0, 1, 0, 0],
              [0, 0, 1, 0],
              [0, 5.0, 0, 1]
            ]
          }
        },
        {
          name: restName,
          parent_name: posName,
          local_transform: {
            m: [
              [1, 0, 0, 0],
              [0, 1, 0, 0],
              [0, 0, 1, 0],
              [0, 0, 5.0, 1]
            ]
          }
        }
      ];"""
hierarchy = hierarchy.replace(turret_old, turret_new)

# 2. Salvage point offset
salvage_old = """        {
          name: leftName,
          parent_name: base,
          local_transform: {
            m: [
              [1, 0, 0, 0],
              [0, 1, 0, 0],
              [0, 0, 1, 0],
              [0, 0, 5.0, 1]
            ]
          }
        },
        {
          name: upName,"""
salvage_new = """        {
          name: leftName,
          parent_name: base,
          local_transform: {
            m: [
              [1, 0, 0, 0],
              [0, 1, 0, 0],
              [0, 0, 1, 0],
              [5.0, 0, 0, 1]
            ]
          }
        },
        {
          name: upName,"""
hierarchy = hierarchy.replace(salvage_old, salvage_new)

# 3. Rename Protection
rename_old = """    if (type.endsWith("_group")) {
      const groupJoints = model.joints.filter(j => j.name.toLowerCase().startsWith(oldName.toLowerCase() + "_") || j.name.toLowerCase() === oldName.toLowerCase());"""
rename_new = """    if (type.endsWith("_group")) {
      if (type === "capture_point_group" || type === "repair_point_group" || type === "salvage_point_group" || type === "hardpoint_group") {
         window.alert("Cannot rename this type of assembly arbitrarily. It must follow the auto-generated naming convention.");
         return;
      }
      const groupJoints = model.joints.filter(j => j.name.toLowerCase().startsWith(oldName.toLowerCase() + "_") || j.name.toLowerCase() === oldName.toLowerCase());"""
hierarchy = hierarchy.replace(rename_old, rename_new)

# 4. Context Menu
ctx_old = """  const handleContextMenu = (e: React.MouseEvent, name: string, type: string) => {
    if (type === "joint") {
      const wInfo = getWeaponGroupInfo(name);
      if (wInfo && name !== wInfo.baseName) {
        // It's a subnode of an assembly, prevent context menu
        e.preventDefault();
        e.stopPropagation();
        return;
      }
    }"""
ctx_new = """  const handleContextMenu = (e: React.MouseEvent, name: string, type: string) => {
    if (type === "joint") {
      if (getWeaponGroupInfo(name)) {
        // It's a subnode of an assembly, prevent context menu
        e.preventDefault();
        e.stopPropagation();
        return;
      }
    }"""
hierarchy = hierarchy.replace(ctx_old, ctx_new)

# 5. Draggable
drag_old = """          draggable={jointName !== "Root" && !(getWeaponGroupInfo(jointName) && jointName !== getWeaponGroupInfo(jointName)?.baseName) ? "true" : "false"}"""
drag_new = """          draggable={jointName !== "Root" && !getWeaponGroupInfo(jointName) ? "true" : "false"}"""
hierarchy = hierarchy.replace(drag_old, drag_new)

# 6. Delete Node
del_old = """    if (type === "joint") {
      const jointToDelete = model.joints.find(j => j.name === name);
      const parentJointName = jointToDelete ? jointToDelete.parent_name : "Root";

      updatedModel.joints = model.joints
        .filter(j => j.name !== name)
        .map(j => {
          if (j.parent_name === name) {
            return { ...j, parent_name: parentJointName };
          }
          return j;
        });

      updatedModel.meshes = model.meshes.map(m => {
        if (m.parent_name === name) return { ...m, parent_name: parentJointName || "Root" };
        return m;
      });
      updatedModel.markers = model.markers.map(mrk => {
        if (mrk.parent_joint === name) return { ...mrk, parent_joint: parentJointName || "Root" };
        return mrk;
      });
      updatedModel.engine_burns = model.engine_burns.filter(b => b.parent_name !== name);
      updatedModel.engine_glows = model.engine_glows.filter(g => g.parent_name !== name);
      updatedModel.engine_shapes = model.engine_shapes.filter(s => s.parent_name !== name);
      updatedModel.dockpaths = model.dockpaths.filter(dp => dp.parent_name !== name);
      updatedModel.nav_lights = model.nav_lights.filter(nav => nav.name !== name);
      invoke("log_event", { level: "INFO", message: `Deleted joint bone: ${name}. Children re-parented to: ${parentJointName}` }).catch(console.error);
    } else if (type.endsWith("_group")) {"""
del_new = """    if (type === "joint") {
      const getAllDescendants = (parentName: string): string[] => {
        const children = model.joints.filter(j => j.parent_name === parentName).map(j => j.name);
        let descendants = [...children];
        for (const child of children) {
          descendants = descendants.concat(getAllDescendants(child));
        }
        return descendants;
      };
      
      const descendants = getAllDescendants(name);
      if (descendants.length > 0) {
        if (!window.confirm(`Warning: Deleting joint "${name}" will also delete its ${descendants.length} descendant joints and all attached objects. Continue?`)) {
          return;
        }
      }
      
      const jointsToDelete = [name, ...descendants];
      
      updatedModel.joints = model.joints.filter(j => !jointsToDelete.includes(j.name));
      updatedModel.meshes = model.meshes.filter(m => !jointsToDelete.includes(m.parent_name));
      updatedModel.markers = model.markers.filter(mrk => !jointsToDelete.includes(mrk.parent_joint));
      updatedModel.engine_burns = model.engine_burns.filter(b => !jointsToDelete.includes(b.parent_name));
      updatedModel.engine_glows = model.engine_glows.filter(g => !jointsToDelete.includes(g.parent_name));
      updatedModel.engine_shapes = model.engine_shapes.filter(s => !jointsToDelete.includes(s.parent_name));
      updatedModel.dockpaths = model.dockpaths.filter(dp => !jointsToDelete.includes(dp.parent_name));
      updatedModel.nav_lights = model.nav_lights.filter(nav => !jointsToDelete.includes(nav.name));
      if (updatedModel.collision_meshes) {
        updatedModel.collision_meshes = updatedModel.collision_meshes.filter(c => !c.mesh || !jointsToDelete.includes(c.mesh.parent_name));
      }
      invoke("log_event", { level: "INFO", message: `Deleted joint bone subtree rooted at: ${name}.` }).catch(console.error);
    } else if (type.endsWith("_group")) {"""
hierarchy = hierarchy.replace(del_old, del_new)

with open(hierarchy_file, "w") as f:
    f.write(hierarchy)

with open(app_file, "r") as f:
    app = f.read()

app_old = """    } else if (nodeType === "engine_burn") {"""
app_new = """    } else if (nodeType === "navlight") {
      updatedModel.joints = model.joints.map((joint) => {
        if (joint.name === nodeName) {
          return { ...joint, parent_name: newParentName };
        }
        return joint;
      });
    } else if (nodeType === "engine_burn") {"""
app = app.replace(app_old, app_new)

with open(app_file, "w") as f:
    f.write(app)

print("Done")
