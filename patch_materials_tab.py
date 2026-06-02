import re

def main():
    file_path = "/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld 347380/GBXTools/WorkshopTool/mod-tools/HODEditorJS/src/components/HierarchyTree.tsx"
    with open(file_path, "r", encoding="utf-8") as f:
        content = f.read()

    # Add imports
    if "FlipVertical" not in content:
        content = re.sub(r'from "lucide-react";', r', Image, FlipVertical } from "lucide-react";', content)
        content = content.replace("} ,", ",")

    # Replace activeTab === "materials" block
    old_materials_tab = """          ) : activeTab === "materials" ? (
            model.materials && model.materials.length > 0 ? (
              model.materials
                .filter(m => !searchTerm || m.name.toLowerCase().includes(searchTerm.toLowerCase()) || m.shader_name.toLowerCase().includes(searchTerm.toLowerCase()))
                .map((material, idx) => {
                  const isMatSelected = selectedNode?.type === "material" && selectedNode.name === material.name;
                  return (
                    <div
                      key={material.name}
                      className={`list-item ${isMatSelected ? "active" : ""}`}
                      onClick={() => setSelectedNode({ type: "material", name: material.name })}
                      style={{ padding: "10px 12px", display: "flex", flexDirection: "column", gap: "2px", alignItems: "flex-start", marginBottom: "4px" }}
                    >
                      <div style={{ display: "flex", alignItems: "center", gap: "8px", width: "100%" }}>
                        <Palette size={14} style={{ color: "var(--accent-cyan)", flexShrink: 0 }} />
                        <span style={{ fontWeight: "600", color: isMatSelected ? "var(--accent-cyan)" : "var(--text-primary)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                          {material.name}
                        </span>
                        {renderDeleteButton(material.name, "material")}
                        <span style={{ fontSize: "9px", padding: "1px 4px", background: "rgba(255,255,255,0.05)", borderRadius: "3px", color: "var(--text-muted)", marginLeft: "auto", flexShrink: 0 }}>
                          ID {idx}
                        </span>
                      </div>
                      <div style={{ fontSize: "11px", color: "var(--text-secondary)", fontFamily: "var(--font-mono)", paddingLeft: "22px" }}>
                        Shader: {material.shader_name}
                      </div>
                    </div>
                  );
                })
            ) : (
              <div style={{ padding: "20px", color: "var(--text-muted)", fontSize: "13px", textAlign: "center" }}>
                No materials defined.
              </div>
            )
          ) : activeTab === "animations" ? ("""

    new_materials_tab = """          ) : activeTab === "materials" ? (
            <div style={{ display: "flex", flexDirection: "column", gap: "16px", padding: "4px" }}>
              <div>
                <div style={{ fontSize: "11px", fontWeight: "600", color: "var(--text-muted)", textTransform: "uppercase", marginBottom: "8px", paddingLeft: "8px" }}>Materials</div>
                {model.materials && model.materials.length > 0 ? (
                  model.materials
                    .filter(m => !searchTerm || m.name.toLowerCase().includes(searchTerm.toLowerCase()) || m.shader_name.toLowerCase().includes(searchTerm.toLowerCase()))
                    .map((material, idx) => {
                      const isMatSelected = selectedNode?.type === "material" && selectedNode.name === material.name;
                      return (
                        <div
                          key={material.name}
                          className={`list-item ${isMatSelected ? "active" : ""}`}
                          onClick={() => setSelectedNode({ type: "material", name: material.name })}
                          style={{ padding: "10px 12px", display: "flex", flexDirection: "column", gap: "2px", alignItems: "flex-start", marginBottom: "4px" }}
                        >
                          <div style={{ display: "flex", alignItems: "center", gap: "8px", width: "100%" }}>
                            <Palette size={14} style={{ color: "var(--accent-cyan)", flexShrink: 0 }} />
                            <span style={{ fontWeight: "600", color: isMatSelected ? "var(--accent-cyan)" : "var(--text-primary)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                              {material.name}
                            </span>
                            {renderDeleteButton(material.name, "material")}
                            <span style={{ fontSize: "9px", padding: "1px 4px", background: "rgba(255,255,255,0.05)", borderRadius: "3px", color: "var(--text-muted)", marginLeft: "auto", flexShrink: 0 }}>
                              ID {idx}
                            </span>
                          </div>
                          <div style={{ fontSize: "11px", color: "var(--text-secondary)", fontFamily: "var(--font-mono)", paddingLeft: "22px" }}>
                            Shader: {material.shader_name}
                          </div>
                        </div>
                      );
                    })
                ) : (
                  <div style={{ padding: "10px", color: "var(--text-muted)", fontSize: "12px", textAlign: "center" }}>
                    No materials defined.
                  </div>
                )}
              </div>

              <hr style={{ border: "none", borderTop: "1px solid rgba(255,255,255,0.05)", margin: "0 8px" }} />

              <div>
                <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", paddingLeft: "8px", paddingRight: "8px", marginBottom: "8px" }}>
                  <div style={{ fontSize: "11px", fontWeight: "600", color: "var(--text-muted)", textTransform: "uppercase" }}>Textures</div>
                  <span style={{ fontSize: "9px", background: "rgba(255,255,255,0.05)", padding: "2px 6px", borderRadius: "4px", color: "var(--text-muted)" }}>
                    {model.textures?.length || 0}
                  </span>
                </div>
                {model.textures && model.textures.length > 0 ? (
                  <div style={{ display: "grid", gridTemplateColumns: "1fr", gap: "4px" }}>
                    {model.textures
                      .filter(t => !searchTerm || t.name.toLowerCase().includes(searchTerm.toLowerCase()))
                      .map((texture, idx) => (
                        <div
                          key={texture.name + "_" + idx}
                          onContextMenu={(e) => {
                            e.preventDefault();
                            e.stopPropagation();
                            setContextMenu({ x: e.clientX, y: e.clientY, name: texture.name, type: "texture" });
                          }}
                          className="list-item"
                          style={{
                            padding: "6px 8px",
                            display: "flex",
                            alignItems: "center",
                            gap: "8px",
                            background: "rgba(0,0,0,0.15)",
                            borderRadius: "4px",
                            border: "1px solid rgba(255,255,255,0.03)",
                            cursor: "context-menu"
                          }}
                        >
                          {texture.png_preview ? (
                            <img
                              src={texture.png_preview.startsWith("data:") ? texture.png_preview : `data:image/png;base64,${texture.png_preview}`}
                              alt={texture.name}
                              style={{ width: "24px", height: "24px", objectFit: "cover", borderRadius: "3px", border: "1px solid var(--border-color)", background: "#000", flexShrink: 0 }}
                            />
                          ) : (
                            <div style={{ width: "24px", height: "24px", borderRadius: "3px", background: "rgba(255,255,255,0.1)", display: "flex", alignItems: "center", justifyContent: "center", flexShrink: 0 }}>
                              <Image size={12} style={{ color: "var(--text-muted)" }} />
                            </div>
                          )}
                          <div style={{ display: "flex", flexDirection: "column", overflow: "hidden" }}>
                            <span style={{ fontSize: "11px", fontWeight: "600", color: "var(--text-primary)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                              {texture.name}
                            </span>
                            <span style={{ fontSize: "9px", color: "var(--text-muted)", fontFamily: "var(--font-mono)" }}>
                              {texture.width}x{texture.height} [{texture.format}] {texture.legacy_storage_y_flipped ? "(Y-Flipped)" : ""}
                            </span>
                          </div>
                        </div>
                      ))}
                  </div>
                ) : (
                  <div style={{ padding: "10px", color: "var(--text-muted)", fontSize: "12px", textAlign: "center" }}>
                    No textures loaded.
                  </div>
                )}
              </div>
            </div>
          ) : activeTab === "animations" ? ("""

    if old_materials_tab in content:
        content = content.replace(old_materials_tab, new_materials_tab)
    else:
        print("COULD NOT FIND MATERIALS TAB BLOCK!")

    # Replace contextMenu block
    old_context_menu = """            {contextMenu.type !== "collision" && (
              <div 
                className="list-item"
                style={{ padding: '8px 12px', cursor: 'pointer', fontSize: '12px', color: 'var(--text-primary)' }}
                onClick={() => {
                   handleRenameNode(contextMenu.name, contextMenu.type);
                   setContextMenu(null);
                }}
              >
                ✏️ Rename
              </div>
            )}
            {isNodeDeletable(contextMenu.name, contextMenu.type) && (
              <div 
                className="list-item"
                style={{ padding: '8px 12px', cursor: 'pointer', fontSize: '12px', color: '#ff1744' }}
                onClick={() => {
                   const confirmMsg = contextMenu.type === "weapon_group" 
                     ? `Are you sure you want to delete the entire weapon/turret family "${contextMenu.name}"? This will remove all of its component joints safely.` 
                     : `Are you sure you want to delete "${contextMenu.name}"?`;
                   if (window.confirm(confirmMsg)) {
                     handleDeleteNode(contextMenu.name, contextMenu.type);
                   }
                   setContextMenu(null);
                }}
              >
                ✕ Delete
              </div>
            )}"""

    new_context_menu = """            {contextMenu.type === "texture" ? (
              <>
                <div 
                  className="list-item"
                  style={{ padding: '8px 12px', cursor: 'pointer', fontSize: '12px', color: 'var(--text-primary)', display: 'flex', alignItems: 'center' }}
                  onClick={(e) => {
                    e.stopPropagation();
                    if (!model) return;
                    const updatedTextures = model.textures.map(t => {
                      if (t.name === contextMenu.name) {
                        return { ...t, legacy_storage_y_flipped: !t.legacy_storage_y_flipped };
                      }
                      return t;
                    });
                    onModelChange?.({ ...model, textures: updatedTextures });
                    setContextMenu(null);
                  }}
                >
                  <FlipVertical size={12} style={{ marginRight: 6 }} />
                  Toggle Y-Flip
                </div>
                <div 
                  className="list-item"
                  style={{ padding: '8px 12px', cursor: 'pointer', fontSize: '12px', color: '#ff1744', display: 'flex', alignItems: 'center' }}
                  onClick={(e) => {
                    e.stopPropagation();
                    if (!model) return;
                    const updatedTextures = model.textures.filter(t => t.name !== contextMenu.name);
                    onModelChange?.({ ...model, textures: updatedTextures });
                    setContextMenu(null);
                  }}
                >
                  <Trash2 size={12} style={{ marginRight: 6 }} />
                  Remove Texture
                </div>
              </>
            ) : (
              <>
                {contextMenu.type !== "collision" && (
                  <div 
                    className="list-item"
                    style={{ padding: '8px 12px', cursor: 'pointer', fontSize: '12px', color: 'var(--text-primary)' }}
                    onClick={() => {
                       handleRenameNode(contextMenu.name, contextMenu.type);
                       setContextMenu(null);
                    }}
                  >
                    ✏️ Rename
                  </div>
                )}
                {isNodeDeletable(contextMenu.name, contextMenu.type) && (
                  <div 
                    className="list-item"
                    style={{ padding: '8px 12px', cursor: 'pointer', fontSize: '12px', color: '#ff1744' }}
                    onClick={() => {
                       const confirmMsg = contextMenu.type === "weapon_group" 
                         ? `Are you sure you want to delete the entire weapon/turret family "${contextMenu.name}"? This will remove all of its component joints safely.` 
                         : `Are you sure you want to delete "${contextMenu.name}"?`;
                       if (window.confirm(confirmMsg)) {
                         handleDeleteNode(contextMenu.name, contextMenu.type);
                       }
                       setContextMenu(null);
                    }}
                  >
                    ✕ Delete
                  </div>
                )}
              </>
            )}"""

    if old_context_menu in content:
        content = content.replace(old_context_menu, new_context_menu)
    else:
        print("COULD NOT FIND CONTEXT MENU BLOCK!")

    with open(file_path, "w", encoding="utf-8") as f:
        f.write(content)
        
    print("Done")

if __name__ == "__main__":
    main()
