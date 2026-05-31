import re

with open("src/components/Inspector.tsx", "r", encoding="utf-8") as f:
    content = f.read()

# 1. Inject GlowLODInspector at the top (right after MeshLODInspector)
glow_lod_code = """
interface GlowLODInspectorProps {
  model: HODModel;
  baseName: string;
  onModelChange?: (updatedModel: HODModel) => void;
  setIsLoading?: React.Dispatch<React.SetStateAction<boolean>>;
  setStatusMsg?: React.Dispatch<React.SetStateAction<string>>;
}

const GlowLODInspector: React.FC<GlowLODInspectorProps> = ({ model, baseName, onModelChange, setIsLoading, setStatusMsg }) => {
  const [selectedLodIdx, setSelectedLodIdx] = useState(0);
  const glowLods = (model.engine_glows || []).filter(g => g.name === baseName).sort((a, b) => a.lod - b.lod);
  const selectedLodGlow = glowLods[selectedLodIdx] || glowLods[0];

  const handleImportEngineGlowOBJ = async () => {
    if (!selectedLodGlow) return;
    try {
      const selectedPath = await invoke<string | null>("open_file_dialog", {
        title: "Select OBJ File for Engine Glow",
        filters: ["obj"]
      });
      if (!selectedPath) return;

      setIsLoading?.(true);
      setStatusMsg?.("Parsing Engine Glow OBJ...");
      const fileContents = await invoke<string>("read_text_file", { path: selectedPath });

      const { OBJLoader } = await import("three/examples/jsm/loaders/OBJLoader.js");
      const loader = new OBJLoader();
      const objGroup = loader.parse(fileContents);

      const newParts: any[] = [];
      let totalTris = 0;
      let totalVerts = 0;

      objGroup.traverse((child) => {
        if ((child as THREE.Mesh).isMesh) {
          const mesh = child as THREE.Mesh;
          const geom = mesh.geometry;
          if (!geom.attributes.position) return;

          const positions = geom.attributes.position.array;
          const normals = geom.attributes.normal?.array;
          const uvs = geom.attributes.uv?.array;

          const indices = geom.index ? Array.from(geom.index.array) : Array.from({ length: positions.length / 3 }, (_, i) => i);

          const vertices = [];
          for (let i = 0; i < positions.length; i += 3) {
            vertices.push({
              position: { x: positions[i], y: positions[i + 1], z: positions[i + 2] },
              normal: normals ? { x: normals[i], y: normals[i + 1], z: normals[i + 2] } : { x: 0, y: 1, z: 0 },
              uv: uvs ? { u: uvs[(i / 3) * 2], v: 1 - uvs[(i / 3) * 2 + 1] } : { u: 0, v: 0 }
            });
          }

          totalTris += indices.length / 3;
          totalVerts += vertices.length;

          newParts.push({ material_index: 0, vertices, indices });
        }
      });

      if (newParts.length === 0) {
        alert("No valid mesh found in OBJ.");
        setIsLoading?.(false);
        return;
      }

      const updatedGlows = model.engine_glows.map(g => {
        if (g.name === selectedLodGlow.name && g.lod === selectedLodGlow.lod) {
          return { ...g, mesh: { ...g.mesh, parts: newParts } };
        }
        return g;
      });

      onModelChange?.({ ...model, engine_glows: updatedGlows });
      setIsLoading?.(false);
      invoke("log_event", { level: "INFO", message: `Imported OBJ into Engine Glow: ${selectedLodGlow.name} (LOD ${selectedLodGlow.lod})` }).catch(console.error);
    } catch (e) {
      console.error(e);
      setIsLoading?.(false);
      alert("Failed to import Engine Glow OBJ.");
    }
  };

  const handleExportEngineGlowOBJ = async () => {
    if (!selectedLodGlow) return;
    try {
      setIsLoading?.(true);
      setStatusMsg?.("Exporting Glow OBJ...");
      const { OBJExporter } = await import("three/examples/jsm/exporters/OBJExporter.js");
      const exporter = new OBJExporter();
      const group = new THREE.Group();
      group.name = selectedLodGlow.name;
      selectedLodGlow.mesh.parts.forEach((part: any, pIdx: number) => {
        const geometry = new THREE.BufferGeometry();
        const vertices: number[] = [];
        part.vertices.forEach((v: any) => vertices.push(v.position.x, v.position.y, v.position.z));
        geometry.setAttribute("position", new THREE.Float32BufferAttribute(vertices, 3));
        geometry.setIndex(part.indices);
        const meshObj = new THREE.Mesh(geometry, new THREE.MeshBasicMaterial());
        meshObj.name = `${selectedLodGlow.name}_part_${pIdx}`;
        group.add(meshObj);
      });
      const objResult = exporter.parse(group);
      const objFilename = `${selectedLodGlow.name}_lod${selectedLodGlow.lod}`;
      await invoke<string | null>("save_text_file", { defaultName: `${objFilename}.obj`, filters: ["obj"], contents: objResult });
      setIsLoading?.(false);
      invoke("log_event", { level: "INFO", message: `Exported Engine Glow OBJ: ${objFilename}` }).catch(console.error);
    } catch (e) {
      console.error(e);
      setIsLoading?.(false);
      alert("Failed to export Glow OBJ.");
    }
  };

  if (!selectedLodGlow) return <div style={{ color: "var(--text-muted)", textAlign: "center" }}>Engine Glow not found</div>;
  const totalVerts = selectedLodGlow.mesh.parts.reduce((sum, p) => sum + p.vertices.length, 0);
  const totalTris = selectedLodGlow.mesh.parts.reduce((sum, p) => sum + p.indices.length, 0) / 3;

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: "16px" }}>
      <div>
        <div style={{ display: "flex", alignItems: "center", gap: "8px", fontSize: "10px", textTransform: "uppercase", letterSpacing: "0.15em", color: "var(--text-muted)", marginBottom: "4px" }}>
          <span style={{ fontSize: "12px" }}>🔥</span>
          <span>Selected Engine Glow</span>
        </div>
        <div style={{ fontSize: "16px", fontWeight: "600", color: "var(--accent-cyan)", wordBreak: "break-all" }}>
          {baseName}
        </div>
      </div>
      <hr style={{ border: "none", borderTop: "1px solid var(--border-color)", margin: 0 }} />
      <div>
        <div style={{ fontSize: "11px", color: "var(--text-secondary)", fontWeight: "500", marginBottom: "4px" }}>Parent Joint</div>
        <div style={{ background: "rgba(255,255,255,0.03)", padding: "8px 10px", borderRadius: "4px", fontSize: "12px", border: "1px solid var(--border-color)" }}>
          {selectedLodGlow.parent_name}
        </div>
      </div>

      <div>
        <div style={{ fontSize: "11px", color: "var(--text-secondary)", fontWeight: "500", marginBottom: "8px" }}>LOD Levels</div>
        <div style={{ display: "flex", flexWrap: "wrap", gap: "6px" }}>
          {glowLods.map((m, idx) => (
            <button
              key={m.lod}
              onClick={() => setSelectedLodIdx(idx)}
              style={{
                background: selectedLodIdx === idx ? "var(--accent-cyan)" : "var(--bg-lighter)",
                color: selectedLodIdx === idx ? "#000" : "var(--text-primary)",
                border: "1px solid",
                borderColor: selectedLodIdx === idx ? "var(--accent-cyan)" : "var(--border-color)",
                padding: "4px 10px",
                borderRadius: "4px",
                fontSize: "11px",
                cursor: "pointer"
              }}
            >
              LOD {m.lod}
            </button>
          ))}
        </div>
      </div>

      <div>
        <div style={{ fontSize: "11px", color: "var(--text-secondary)", fontWeight: "500", marginBottom: "8px" }}>Mesh Statistics</div>
        <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: "8px", background: "rgba(22, 160, 255, 0.03)", border: "1px solid var(--border-color)", borderRadius: "4px", padding: "12px" }}>
          <div>
            <div style={{ fontSize: "10px", color: "var(--text-muted)", marginBottom: "2px" }}>TRIANGLES</div>
            <div style={{ fontSize: "15px", fontWeight: "600", color: "var(--text-primary)" }}>{totalTris}</div>
          </div>
          <div>
            <div style={{ fontSize: "10px", color: "var(--text-muted)", marginBottom: "2px" }}>VERTICES</div>
            <div style={{ fontSize: "15px", fontWeight: "600", color: "var(--text-primary)" }}>{totalVerts}</div>
          </div>
        </div>
      </div>

      <div style={{ display: "flex", gap: "8px" }}>
        <button onClick={handleImportEngineGlowOBJ} style={{ height: "32px", fontSize: "12px", display: "flex", alignItems: "center", justifyContent: "center", gap: "6px", flex: 1 }}>
          <Download size={14} /> Import OBJ
        </button>
        <button onClick={handleExportEngineGlowOBJ} style={{ height: "32px", fontSize: "12px", display: "flex", alignItems: "center", justifyContent: "center", gap: "6px", flex: 1 }}>
          <Upload size={14} /> Export OBJ
        </button>
      </div>
    </div>
  );
};
"""

if "const MeshLODInspector" in content:
    content = content.replace("const MeshLODInspector", glow_lod_code + "\nconst MeshLODInspector")

# 2. Inject `engine_shape` export functionality
shape_export_fn = """
      const handleExportEngineShapeOBJ = async () => {
        try {
          setIsLoading?.(true);
          setStatusMsg?.("Exporting Shape OBJ...");
          const { OBJExporter } = await import("three/examples/jsm/exporters/OBJExporter.js");
          const exporter = new OBJExporter();
          const group = new THREE.Group();
          group.name = shape.name;
          shape.mesh.parts.forEach((part: any, pIdx: number) => {
            const geometry = new THREE.BufferGeometry();
            const vertices: number[] = [];
            part.vertices.forEach((v: any) => vertices.push(v.position.x, v.position.y, v.position.z));
            geometry.setAttribute("position", new THREE.Float32BufferAttribute(vertices, 3));
            geometry.setIndex(part.indices);
            const meshObj = new THREE.Mesh(geometry, new THREE.MeshBasicMaterial());
            meshObj.name = `${shape.name}_part_${pIdx}`;
            group.add(meshObj);
          });
          const objResult = exporter.parse(group);
          await invoke<string | null>("save_text_file", { defaultName: `${shape.name}.obj`, filters: ["obj"], contents: objResult });
          setIsLoading?.(false);
          invoke("log_event", { level: "INFO", message: `Exported Engine Shape OBJ: ${shape.name}` }).catch(console.error);
        } catch (e) {
          console.error(e);
          setIsLoading?.(false);
          alert("Failed to export Shape OBJ.");
        }
      };
"""

# Re-implement handleImportEngineShapeOBJ to match the new format
shape_block_match = re.search(r'if \(selectedNode\.type === "engine_shape"\) \{.*?const totalVerts = shape\.mesh\.parts\.reduce.*?;', content, re.DOTALL)
if shape_block_match:
    content = content.replace(shape_block_match.group(0), shape_block_match.group(0) + "\n" + shape_export_fn)

shape_buttons = """        <div style={{ display: "flex", gap: "8px" }}>
          <button onClick={handleImportEngineShapeOBJ} style={{ height: "32px", fontSize: "12px", display: "flex", alignItems: "center", justifyContent: "center", gap: "6px", flex: 1 }}>
            <Download size={14} style={{ color: "var(--accent-cyan)" }} /> Import OBJ
          </button>
          <button onClick={handleExportEngineShapeOBJ} style={{ height: "32px", fontSize: "12px", display: "flex", alignItems: "center", justifyContent: "center", gap: "6px", flex: 1 }}>
            <Upload size={14} style={{ color: "var(--accent-cyan)" }} /> Export OBJ
          </button>
        </div>"""

content = re.sub(r'<button onClick=\{handleImportEngineShapeOBJ\}.*?</button>', shape_buttons, content, flags=re.DOTALL)

# 3. Add Nozzle Support to the Joint Block
old_joint_block = """    if (selectedNode.type === "joint") {
      const joint = model.joints.find(j => j.name === selectedNode.name);
      if (!joint) return <div style={{ color: "var(--text-muted)", textAlign: "center" }}>Joint not found</div>;
      const m = joint.local_transform.m;
      const pos = { x: m[3][0], y: m[3][1], z: m[3][2] };
      const rot = getEulerRotation(m);
      const parentName = joint.parent_name || "(None)";"""

new_joint_block = """    if (selectedNode.type === "joint" || selectedNode.type === "engine_nozzle") {
      const isNozzle = selectedNode.type === "engine_nozzle";
      const joint = model.joints.find(j => j.name === selectedNode.name);
      if (!joint) return <div style={{ color: "var(--text-muted)", textAlign: "center" }}>Joint not found</div>;
      const m = joint.local_transform.m;
      const pos = { x: m[3][0], y: m[3][1], z: m[3][2] };
      const rot = getEulerRotation(m);
      const parentName = joint.parent_name || "(None)";
      
      const hasBurn = model.engine_burns?.some(b => b.parent_name === joint.name);
      const hasGlow = model.engine_glows?.some(g => g.parent_name === joint.name);
      const hasShape = model.engine_shapes?.some(s => s.parent_name === joint.name);

      const handleAddSubnode = (type: string) => {
        if (type === "burn") {
          const newBurn = { name: `burn_${joint.name}`, parent_name: joint.name, num_divisions: 16, num_flames: 4, vertices: [{x:0,y:0,z:0},{x:0,y:0,z:-1},{x:0,y:0,z:-2},{x:0,y:0,z:-3}] };
          onModelChange?.({ ...model, engine_burns: [...(model.engine_burns || []), newBurn] });
        } else if (type === "glow") {
          const newGlow = { name: `glow_${joint.name}`, parent_name: joint.name, lod: 0, mesh: { name: `glow_${joint.name}`, parts: [] } };
          onModelChange?.({ ...model, engine_glows: [...(model.engine_glows || []), newGlow] });
        } else if (type === "shape") {
          const newShape = { name: `shape_${joint.name}`, parent_name: joint.name, mesh: { name: `shape_${joint.name}`, parts: [] } };
          onModelChange?.({ ...model, engine_shapes: [...(model.engine_shapes || []), newShape] });
        }
      };
      const handleRemoveSubnode = (type: string) => {
        if (type === "burn") {
          onModelChange?.({ ...model, engine_burns: (model.engine_burns || []).filter(b => b.parent_name !== joint.name) });
        } else if (type === "glow") {
          onModelChange?.({ ...model, engine_glows: (model.engine_glows || []).filter(g => g.parent_name !== joint.name) });
        } else if (type === "shape") {
          onModelChange?.({ ...model, engine_shapes: (model.engine_shapes || []).filter(s => s.parent_name !== joint.name) });
        }
      };"""

content = content.replace(old_joint_block, new_joint_block)

old_joint_title = """            <div style={{ fontSize: "10px", textTransform: "uppercase", letterSpacing: "0.15em", color: "var(--text-muted)", marginBottom: "4px" }}>
              Selected Joint
            </div>"""
new_joint_title = """            <div style={{ fontSize: "10px", textTransform: "uppercase", letterSpacing: "0.15em", color: "var(--text-muted)", marginBottom: "4px", display: "flex", alignItems: "center", gap: "6px" }}>
              {isNozzle && <span style={{fontSize: "12px"}}>🚀</span>}
              <span>{isNozzle ? "Selected Engine Nozzle" : "Selected Joint"}</span>
            </div>"""
content = content.replace(old_joint_title, new_joint_title)

# Only replace the FIRST occurance of `<hr ... />` inside the joint block.
# We'll use a regex that specifically targets the joint block.
joint_block_start = content.find('if (selectedNode.type === "joint" || selectedNode.type === "engine_nozzle") {')
if joint_block_start != -1:
    hr_idx = content.find('<hr style={{ border: "none", borderTop: "1px solid var(--border-color)", margin: 0 }} />', joint_block_start)
    if hr_idx != -1:
        nozzle_buttons = """
          {isNozzle && (
            <div style={{ display: "flex", flexDirection: "column", gap: "8px", background: "rgba(255,255,255,0.02)", padding: "12px", borderRadius: "4px", border: "1px solid var(--border-color)" }}>
              <div style={{ fontSize: "11px", fontWeight: "600", color: "var(--text-secondary)", marginBottom: "4px" }}>Engine Subnodes</div>
              <div style={{ display: "flex", gap: "8px", flexWrap: "wrap" }}>
                {!hasBurn ? 
                  <button onClick={() => handleAddSubnode("burn")} style={{ fontSize: "11px", padding: "4px 8px" }}>+ Add Burn Plume</button> :
                  <button onClick={() => handleRemoveSubnode("burn")} style={{ fontSize: "11px", padding: "4px 8px", background: "rgba(255, 50, 50, 0.2)", borderColor: "rgba(255, 50, 50, 0.5)" }}>- Remove Burn</button>}
                {!hasGlow ? 
                  <button onClick={() => handleAddSubnode("glow")} style={{ fontSize: "11px", padding: "4px 8px" }}>+ Add Engine Glow</button> :
                  <button onClick={() => handleRemoveSubnode("glow")} style={{ fontSize: "11px", padding: "4px 8px", background: "rgba(255, 50, 50, 0.2)", borderColor: "rgba(255, 50, 50, 0.5)" }}>- Remove Glow</button>}
                {!hasShape ? 
                  <button onClick={() => handleAddSubnode("shape")} style={{ fontSize: "11px", padding: "4px 8px" }}>+ Add Engine Shape</button> :
                  <button onClick={() => handleRemoveSubnode("shape")} style={{ fontSize: "11px", padding: "4px 8px", background: "rgba(255, 50, 50, 0.2)", borderColor: "rgba(255, 50, 50, 0.5)" }}>- Remove Shape</button>}
              </div>
            </div>
          )}
          <hr style={{ border: "none", borderTop: "1px solid var(--border-color)", margin: 0 }} />"""
        content = content[:hr_idx] + nozzle_buttons + content[hr_idx + len('<hr style={{ border: "none", borderTop: "1px solid var(--border-color)", margin: 0 }} />'):]

with open("src/components/Inspector.tsx", "w", encoding="utf-8") as f:
    f.write(content)
print("patch_inspector_final applied")
