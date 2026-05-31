import re
import sys

with open("src/components/HierarchyTree.tsx", "r", encoding="utf-8") as f:
    content = f.read()

# 1. Update handleAddNode
old_handle_add = """  const handleAddNode = () => {
    if (!model || !newNodeName.trim()) return;
    const name = newNodeName.trim();
    const parent = newNodeParent === "Root" ? "Root" : newNodeParent;"""

new_handle_add = """  const handleAddNode = () => {
    if (!model) return;
    let autoName = newNodeName.trim();
    const isAutoNumbered = ["engine_nozzle", "repair_point_template", "capture_point_template", "salvage_point_template"].includes(addNodeType);
    
    if (isAutoNumbered) {
        let maxNum = -1;
        let prefix = "";
        if (addNodeType === "engine_nozzle") prefix = "EngineNozzle";
        else if (addNodeType === "repair_point_template") prefix = "RepairPoint";
        else if (addNodeType === "capture_point_template") prefix = "CapturePoint";
        else if (addNodeType === "salvage_point_template") prefix = "SalvagePoint";

        model.joints.forEach(j => {
            const match = j.name.match(new RegExp(`^${prefix}(\\\\d+)$`, "i"));
            if (match) {
                maxNum = Math.max(maxNum, parseInt(match[1]));
            }
        });
        autoName = `${prefix}${maxNum + 1}`;
    } else if (!autoName) {
        return; // Empty name not allowed for non-auto nodes
    }

    const name = autoName;
    const parent = newNodeParent === "Root" ? "Root" : newNodeParent;"""

content = content.replace(old_handle_add, new_handle_add)

# 2. Fix Duplicate checking to use `base` or computed names for assemblies so it actually catches duplicates
# Wait, checkDuplicate is run with `name` BEFORE the `Weapon_` prefix is added.
# Let's fix that by moving checkDuplicate INSIDE the specific branches or just doing it smartly.
# Actually, the user says "not being able to add a new assembly of the same type of the same name".
# We can just let checkDuplicate check the final base node name.

old_check_dup = """    if (checkDuplicate(name)) {
      window.alert(`A node with the name "${name}" already exists! Please choose a unique name.`);
      return;
    }"""

new_check_dup = """    let finalNodeName = name;
    if (addNodeType === "weapon_template") finalNodeName = name.startsWith("Weapon_") || name.startsWith("weapon_") ? name : `Weapon_${name}`;
    else if (addNodeType === "turret_template") finalNodeName = name.startsWith("Weapon_") || name.startsWith("Turret_") ? name : `Weapon_${name}_Turret`;
    else if (addNodeType === "repair_point_template" || addNodeType === "capture_point_template" || addNodeType === "salvage_point_template") finalNodeName = name; // already computed autoName
    else if (addNodeType === "engine_nozzle") finalNodeName = name;

    if (checkDuplicate(finalNodeName)) {
      window.alert(`A node with the name "${finalNodeName}" already exists! Please choose a unique name.`);
      return;
    }"""

content = content.replace(old_check_dup, new_check_dup)

# 3. Fix Turret template base name logic
old_turret = """    } else if (addNodeType === "turret_template") {
      const base = name.startsWith("Weapon_") || name.startsWith("weapon_") ? name : `Weapon_${name}`;"""

new_turret = """    } else if (addNodeType === "turret_template") {
      const base = finalNodeName;"""

content = content.replace(old_turret, new_turret)

old_weapon = """    if (addNodeType === "weapon_template") {
      const base = name.startsWith("Weapon_") || name.startsWith("weapon_") ? name : `Weapon_${name}`;"""

new_weapon = """    if (addNodeType === "weapon_template") {
      const base = finalNodeName;"""
content = content.replace(old_weapon, new_weapon)

old_repair = """    } else if (addNodeType === "repair_point_template") {
      const base = name.startsWith("RepairPoint") ? name : `RepairPoint${name}`;"""
new_repair = """    } else if (addNodeType === "repair_point_template") {
      const base = finalNodeName;"""
content = content.replace(old_repair, new_repair)

old_capture = """    } else if (addNodeType === "capture_point_template") {
      const base = name.startsWith("CapturePoint") ? name : `CapturePoint${name}`;"""
new_capture = """    } else if (addNodeType === "capture_point_template") {
      const base = finalNodeName;"""
content = content.replace(old_capture, new_capture)

old_salvage = """    } else if (addNodeType === "salvage_point_template") {
      const base = name.startsWith("SalvagePoint") ? name : `SalvagePoint${name}`;"""
new_salvage = """    } else if (addNodeType === "salvage_point_template") {
      const base = finalNodeName;"""
content = content.replace(old_salvage, new_salvage)


# 4. Engine Nozzle creation logic
old_nozzle = """    } else if (addNodeType === "engine_nozzle") {
      if (model.engine_burns.length >= 9) {
        alert("Maximum limit of 9 engine burns reached.");
        return;
      }
      const baseJointName = name.toLowerCase().includes("nozzle") ? name : `Nozzle_${name}`;
      const newJoint = {
        name: baseJointName,
        parent_name: parent === "(None)" ? undefined : parent,
        local_transform: {
          m: [
            [1, 0, 0, 0],
            [0, 1, 0, 0],
            [0, 0, 1, 0],
            [0, 0, 0, 1]
          ]
        }
      };

      const burnName = `burn_${name}`;
      const newBurn = {
        name: burnName,
        parent_name: baseJointName,
        num_divisions: 16,
        num_flames: 4,
        vertices: [
          { x: 0, y: 0, z: 0 },
          { x: 0, y: 0, z: -1.0 },
          { x: 0, y: 0, z: -2.0 },
          { x: 0, y: 0, z: -3.0 },
        ]
      };

      updatedModel.joints = [...model.joints, newJoint];
      updatedModel.engine_burns = [...model.engine_burns, newBurn];
      invoke("log_event", { level: "INFO", message: `Added new Engine Nozzle joint ${baseJointName} and fire plume ${burnName} parented under ${parent}` }).catch(console.error);"""

new_nozzle = """    } else if (addNodeType === "engine_nozzle") {
      const baseJointName = finalNodeName;
      const newJoint = {
        name: baseJointName,
        parent_name: parent === "(None)" ? undefined : parent,
        local_transform: {
          m: [
            [1, 0, 0, 0],
            [0, 1, 0, 0],
            [0, 0, 1, 0],
            [0, 0, 0, 1]
          ]
        }
      };
      // Don't auto-add engine_burn here, let the user add it via the inspector
      updatedModel.joints = [...model.joints, newJoint];
      invoke("log_event", { level: "INFO", message: `Added new Engine Nozzle joint ${baseJointName} parented under ${parent}` }).catch(console.error);"""
content = content.replace(old_nozzle, new_nozzle)

# 5. Hide Node Name input in UI
old_ui = """              {/* Node Name */}
              <div>
                <label style={{ display: "block", fontSize: "11px", fontWeight: "600", color: "var(--text-muted)", textTransform: "uppercase", marginBottom: "6px" }}>
                  {addNodeType === "weapon_template" || addNodeType === "turret_template" ? "Base Weapon Name" : "Node Name"}
                </label>
                <input
                  placeholder={addNodeType === "weapon_template" || addNodeType === "turret_template" ? "e.g. Laser_Turret" : "e.g. MyNewNode"}
                  value={newNodeName}
                  onChange={(e) => setNewNodeName(e.target.value)}
                  style={{ height: "36px", fontSize: "13px" }}
                />
                {(addNodeType === "weapon_template" || addNodeType === "turret_template") && (
                  <div style={{ fontSize: "10px", color: "var(--accent-cyan)", marginTop: "4px" }}>
                    ℹ️ This will auto-generate the complete compliant {addNodeType === "turret_template" ? "6-joint Turret" : "4-joint Weapon"} family!
                  </div>
                )}
              </div>"""

new_ui = """              {/* Node Name */}
              {!["engine_nozzle", "repair_point_template", "capture_point_template", "salvage_point_template"].includes(addNodeType) && (
                <div>
                  <label style={{ display: "block", fontSize: "11px", fontWeight: "600", color: "var(--text-muted)", textTransform: "uppercase", marginBottom: "6px" }}>
                    {addNodeType === "weapon_template" || addNodeType === "turret_template" ? "Base Weapon Name" : "Node Name"}
                  </label>
                  <input
                    placeholder={addNodeType === "weapon_template" || addNodeType === "turret_template" ? "e.g. Laser_Turret" : "e.g. MyNewNode"}
                    value={newNodeName}
                    onChange={(e) => setNewNodeName(e.target.value)}
                    style={{ height: "36px", fontSize: "13px" }}
                  />
                  {(addNodeType === "weapon_template" || addNodeType === "turret_template") && (
                    <div style={{ fontSize: "10px", color: "var(--accent-cyan)", marginTop: "4px" }}>
                      ℹ️ This will auto-generate the complete compliant {addNodeType === "turret_template" ? "6-joint Turret" : "4-joint Weapon"} family!
                    </div>
                  )}
                </div>
              )}
              {["engine_nozzle", "repair_point_template", "capture_point_template", "salvage_point_template"].includes(addNodeType) && (
                <div style={{ fontSize: "11px", color: "var(--text-muted)", fontStyle: "italic", marginBottom: "8px" }}>
                  Name will be auto-generated sequentially (e.g. {addNodeType === "engine_nozzle" ? "EngineNozzle0" : "Point0"}).
                </div>
              )}"""
content = content.replace(old_ui, new_ui)

with open("src/components/HierarchyTree.tsx", "w", encoding="utf-8") as f:
    f.write(content)

print("Patch 1 applied!")
