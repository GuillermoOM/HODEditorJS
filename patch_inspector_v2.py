import re

file_path = "src/components/Inspector.tsx"

with open(file_path, "r") as f:
    content = f.read()

# 1. Add handlers for importing Glow and Shape OBJs
handlers = """
  const handleImportEngineGlowOBJ = async () => {
    if (!model || !selectedNode || selectedNode.type !== "engine_glow") return;
    try {
      const fileContent = await invoke<string | null>("load_text_file", { filters: ["obj"] });
      if (!fileContent) return;
      setIsLoading?.(true); setStatusMsg?.("Importing Engine Glow OBJ...");
      setTimeout(async () => {
        try {
          const { OBJLoader } = await import("three/examples/jsm/loaders/OBJLoader.js");
          const objGroup = new OBJLoader().parse(fileContent);
          const newParts: any[] = [];
          objGroup.traverse((child: any) => {
            if (child.isMesh) {
              const geo = (child as THREE.Mesh).geometry;
              if (geo?.attributes.position) {
                const posAttr = geo.attributes.position;
                const vertices: any[] = []; const indices: number[] = [];
                for (let i = 0; i < posAttr.count; i++) {
                  vertices.push({
                    position: { x: posAttr.getX(i), y: posAttr.getY(i), z: posAttr.getZ(i) },
                    normal: geo.attributes.normal ? { x: geo.attributes.normal.getX(i), y: geo.attributes.normal.getY(i), z: geo.attributes.normal.getZ(i) } : { x: 0, y: 1, z: 0 },
                    tangent: { x: 1, y: 0, z: 0 }, binormal: { x: 0, y: 0, z: 1 },
                    uv: geo.attributes.uv ? { u: geo.attributes.uv.getX(i), v: 1 - geo.attributes.uv.getY(i) } : { u: 0, v: 0 },
                    color: 0xFFFFFFFF, skinning_data: null,
                  });
                }
                if (geo.index) {
                  const idxArr = geo.index.array;
                  for (let i = 0; i < idxArr.length; i++) indices.push(idxArr[i]);
                } else {
                  for (let i = 0; i < posAttr.count; i++) indices.push(i);
                }
                newParts.push({ material_index: 0, vertex_mask: 0x01, vertices, indices });
              }
            }
          });
          if (newParts.length === 0) { alert("No geometry found in the OBJ file."); setIsLoading?.(false); return; }
          const updatedGlows = model.engine_glows.map(g => g.name === selectedNode.name ? { ...g, mesh: { ...g.mesh, parts: newParts } } : g);
          onModelChange?.({ ...model, engine_glows: updatedGlows });
          setIsLoading?.(false); alert(`Engine Glow mesh imported! Parts: ${newParts.length}`);
        } catch (e: any) { console.error(e); setIsLoading?.(false); alert(`Import failed: ${e.toString()}`); }
      }, 100);
    } catch (e: any) { console.error(e); alert(`Import dialog failed: ${e.toString()}`); }
  };

  const handleImportEngineShapeOBJ = async () => {
    if (!model || !selectedNode || selectedNode.type !== "engine_shape") return;
    try {
      const fileContent = await invoke<string | null>("load_text_file", { filters: ["obj"] });
      if (!fileContent) return;
      setIsLoading?.(true); setStatusMsg?.("Importing Engine Shape OBJ...");
      setTimeout(async () => {
        try {
          const { OBJLoader } = await import("three/examples/jsm/loaders/OBJLoader.js");
          const objGroup = new OBJLoader().parse(fileContent);
          const newParts: any[] = [];
          objGroup.traverse((child: any) => {
            if (child.isMesh) {
              const geo = (child as THREE.Mesh).geometry;
              if (geo?.attributes.position) {
                const posAttr = geo.attributes.position;
                const vertices: any[] = []; const indices: number[] = [];
                for (let i = 0; i < posAttr.count; i++) {
                  vertices.push({
                    position: { x: posAttr.getX(i), y: posAttr.getY(i), z: posAttr.getZ(i) },
                    normal: geo.attributes.normal ? { x: geo.attributes.normal.getX(i), y: geo.attributes.normal.getY(i), z: geo.attributes.normal.getZ(i) } : { x: 0, y: 1, z: 0 },
                    tangent: { x: 1, y: 0, z: 0 }, binormal: { x: 0, y: 0, z: 1 },
                    uv: geo.attributes.uv ? { u: geo.attributes.uv.getX(i), v: 1 - geo.attributes.uv.getY(i) } : { u: 0, v: 0 },
                    color: 0xFFFFFFFF, skinning_data: null,
                  });
                }
                if (geo.index) {
                  const idxArr = geo.index.array;
                  for (let i = 0; i < idxArr.length; i++) indices.push(idxArr[i]);
                } else {
                  for (let i = 0; i < posAttr.count; i++) indices.push(i);
                }
                newParts.push({ material_index: 0, vertex_mask: 0x01, vertices, indices });
              }
            }
          });
          if (newParts.length === 0) { alert("No geometry found in the OBJ file."); setIsLoading?.(false); return; }
          const updatedShapes = model.engine_shapes.map(s => s.name === selectedNode.name ? { ...s, mesh: { ...s.mesh, parts: newParts } } : s);
          onModelChange?.({ ...model, engine_shapes: updatedShapes });
          setIsLoading?.(false); alert(`Engine Shape mesh imported! Parts: ${newParts.length}`);
        } catch (e: any) { console.error(e); setIsLoading?.(false); alert(`Import failed: ${e.toString()}`); }
      }, 100);
    } catch (e: any) { console.error(e); alert(`Import dialog failed: ${e.toString()}`); }
  };
"""

content = content.replace("  const handleImportCollisionOBJ = async () => {", handlers + "\n  const handleImportCollisionOBJ = async () => {")

# 2. Add Material Assignment and Import OBJ to engine_glow
glow_target = """              <div>
                <div style={{ fontSize: "10px", color: "var(--text-muted)", marginBottom: "2px" }}>VERTICES</div>
                <div style={{ fontSize: "15px", fontWeight: "600", color: "var(--text-primary)" }}>{totalVerts}</div>
              </div>
            </div>
          </div>
        </div>
      );
    }

    if (selectedNode.type === "engine_shape") {"""

glow_ui = """              <div>
                <div style={{ fontSize: "10px", color: "var(--text-muted)", marginBottom: "2px" }}>VERTICES</div>
                <div style={{ fontSize: "15px", fontWeight: "600", color: "var(--text-primary)" }}>{totalVerts}</div>
              </div>
            </div>
          </div>
          
          <button onClick={handleImportEngineGlowOBJ} style={{ height: "32px", fontSize: "12px", display: "flex", alignItems: "center", justifyContent: "center", gap: "6px" }}>
            <Download size={14} style={{ color: "var(--accent-cyan)" }} /> Import OBJ
          </button>

          {model.materials?.length > 0 && glow.mesh?.parts?.length > 0 && (
            <div>
              <label style={{ display: "block", fontSize: "11px", color: "var(--text-secondary)", fontWeight: "500", marginBottom: "6px" }}>Material Assignment</label>
              <div style={{ display: "flex", flexDirection: "column", gap: "6px", background: "rgba(0,0,0,0.15)", padding: "8px", borderRadius: "4px" }}>
                {glow.mesh.parts.map((part, pIdx) => (
                  <div key={pIdx} style={{ display: "flex", alignItems: "center", justifyContent: "space-between", gap: "8px" }}>
                    <span style={{ fontSize: "11px", color: "var(--text-muted)" }}>Part {pIdx}</span>
                    <select value={part.material_index} onChange={(e) => {
                      const newMatIdx = parseInt(e.target.value, 10);
                      onModelChange?.({ ...model, engine_glows: model.engine_glows.map(g => g.name === glow.name ? { ...g, mesh: { ...g.mesh, parts: g.mesh.parts.map((p, i) => i === pIdx ? { ...p, material_index: newMatIdx } : p) } } : g) });
                    }} style={{ width: "160px", height: "26px", padding: "2px 6px", fontSize: "11px", background: "#050a12", border: "1px solid var(--border-color)", color: "white" }}>
                      {model.materials.map((mat, mIdx) => (<option key={mIdx} value={mIdx}>{mat.name}</option>))}
                    </select>
                  </div>
                ))}
              </div>
            </div>
          )}

        </div>
      );
    }

    if (selectedNode.type === "engine_shape") {"""
content = content.replace(glow_target, glow_ui)


# 3. Add Material Assignment and Import OBJ to engine_shape
shape_target = """              <div>
                <div style={{ fontSize: "10px", color: "var(--text-muted)", marginBottom: "2px" }}>VERTICES</div>
                <div style={{ fontSize: "15px", fontWeight: "600", color: "var(--text-primary)" }}>{totalVerts}</div>
              </div>
            </div>
          </div>
        </div>
      );
    }

    if (selectedNode.type === "dockpath") {"""

shape_ui = """              <div>
                <div style={{ fontSize: "10px", color: "var(--text-muted)", marginBottom: "2px" }}>VERTICES</div>
                <div style={{ fontSize: "15px", fontWeight: "600", color: "var(--text-primary)" }}>{totalVerts}</div>
              </div>
            </div>
          </div>
          
          <button onClick={handleImportEngineShapeOBJ} style={{ height: "32px", fontSize: "12px", display: "flex", alignItems: "center", justifyContent: "center", gap: "6px" }}>
            <Download size={14} style={{ color: "var(--accent-cyan)" }} /> Import OBJ
          </button>

          {model.materials?.length > 0 && shape.mesh?.parts?.length > 0 && (
            <div>
              <label style={{ display: "block", fontSize: "11px", color: "var(--text-secondary)", fontWeight: "500", marginBottom: "6px" }}>Material Assignment</label>
              <div style={{ display: "flex", flexDirection: "column", gap: "6px", background: "rgba(0,0,0,0.15)", padding: "8px", borderRadius: "4px" }}>
                {shape.mesh.parts.map((part, pIdx) => (
                  <div key={pIdx} style={{ display: "flex", alignItems: "center", justifyContent: "space-between", gap: "8px" }}>
                    <span style={{ fontSize: "11px", color: "var(--text-muted)" }}>Part {pIdx}</span>
                    <select value={part.material_index} onChange={(e) => {
                      const newMatIdx = parseInt(e.target.value, 10);
                      onModelChange?.({ ...model, engine_shapes: model.engine_shapes.map(s => s.name === shape.name ? { ...s, mesh: { ...s.mesh, parts: s.mesh.parts.map((p, i) => i === pIdx ? { ...p, material_index: newMatIdx } : p) } } : s) });
                    }} style={{ width: "160px", height: "26px", padding: "2px 6px", fontSize: "11px", background: "#050a12", border: "1px solid var(--border-color)", color: "white" }}>
                      {model.materials.map((mat, mIdx) => (<option key={mIdx} value={mIdx}>{mat.name}</option>))}
                    </select>
                  </div>
                ))}
              </div>
            </div>
          )}

        </div>
      );
    }

    if (selectedNode.type === "dockpath") {"""
content = content.replace(shape_target, shape_ui)

with open(file_path, "w") as f:
    f.write(content)

print("Patch applied")
