import re

with open("src/components/Inspector.tsx", "r", encoding="utf-8") as f:
    content = f.read()

# Remove the old `handleImportEngineGlowOBJ` at the top level
old_import_glow_match = re.search(r'  const handleImportEngineGlowOBJ = async \(\) => \{.*?\} catch \(e: any\) \{ console\.error\(e\); alert\(`Import dialog failed: \$\{e\.toString\(\)\}`\); \}\s*\};\s*', content, re.DOTALL)
if old_import_glow_match:
    content = content.replace(old_import_glow_match.group(0), "")
else:
    print("WARNING: Could not find old handleImportEngineGlowOBJ")

# 1. Add Export to GlowLODInspector
glow_export_fn = """
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
"""

content = content.replace("  const handleImportEngineGlowOBJ = async () => {", glow_export_fn + "\n  const handleImportEngineGlowOBJ = async () => {")

glow_buttons = """      <div style={{ display: "flex", gap: "8px" }}>
        <button onClick={handleImportEngineGlowOBJ} style={{ height: "32px", fontSize: "12px", display: "flex", alignItems: "center", justifyContent: "center", gap: "6px", flex: 1 }}>
          <Download size={14} /> Import OBJ
        </button>
        <button onClick={handleExportEngineGlowOBJ} style={{ height: "32px", fontSize: "12px", display: "flex", alignItems: "center", justifyContent: "center", gap: "6px", flex: 1 }}>
          <Upload size={14} /> Export OBJ
        </button>
      </div>"""

content = re.sub(r'<div style=\{\{ display: "flex", gap: "8px" \}\}>\s*<button onClick=\{handleImportEngineGlowOBJ\}.*?</div>', glow_buttons, content, flags=re.DOTALL)


# 2. Add Export to engine_shape
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

# Replace in the `engine_shape` block
# The `engine_shape` block renders a button for import.
# I will find that block and inject the export function right before the `return` statement of that block,
# or right inside the `if (selectedNode.type === "engine_shape") {`

shape_block_match = re.search(r'if \(selectedNode\.type === "engine_shape"\) \{.*?const totalVerts = shape\.mesh\.parts\.reduce.*?;', content, re.DOTALL)
if shape_block_match:
    content = content.replace(shape_block_match.group(0), shape_block_match.group(0) + "\n" + shape_export_fn)

shape_buttons = """        <div style={{ display: "flex", gap: "8px" }}>
          <button onClick={handleImportEngineShapeOBJ} style={{ height: "32px", fontSize: "12px", display: "flex", alignItems: "center", justifyContent: "center", gap: "6px", flex: 1 }}>
            <Download size={14} /> Import OBJ
          </button>
          <button onClick={handleExportEngineShapeOBJ} style={{ height: "32px", fontSize: "12px", display: "flex", alignItems: "center", justifyContent: "center", gap: "6px", flex: 1 }}>
            <Upload size={14} /> Export OBJ
          </button>
        </div>"""

content = re.sub(r'<button onClick=\{handleImportEngineShapeOBJ\}.*?</button>', shape_buttons, content, flags=re.DOTALL)


with open("src/components/Inspector.tsx", "w", encoding="utf-8") as f:
    f.write(content)
print("fix_exports applied")
