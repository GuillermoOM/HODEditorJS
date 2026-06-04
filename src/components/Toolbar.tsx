import React from "react";
import { FolderOpen, Save, Hammer, FilePlus, Settings } from "lucide-react";

interface ToolbarProps {
  modelName: string;
  onOpenClick: () => void;
  onSaveClick: () => void;
  onSaveAsClick?: () => void;
  isSaving: boolean;
  onNewClick?: () => void;
  onImportDAEClick?: () => void;
  onSettingsClick?: () => void;
  transformMode?: "translate" | "rotate" | "scale";
  setTransformMode?: (mode: "translate" | "rotate" | "scale") => void;
}

export const Toolbar: React.FC<ToolbarProps> = ({
  modelName,
  onOpenClick,
  onSaveClick,
  onSaveAsClick,
  isSaving,
  onNewClick,
  onImportDAEClick,
  onSettingsClick,
  transformMode,
  setTransformMode,
}) => {
  return (
    <header className="toolbar" style={{ padding: "8px 16px", minHeight: "48px", display: "flex", justifyContent: "space-between", alignItems: "center" }}>
      {/* Brand Logo & Model Title */}
      <div style={{ display: "flex", alignItems: "center", gap: "12px" }}>
        <Hammer size={18} style={{ color: "var(--accent-cyan)", filter: "drop-shadow(var(--shadow-glow))" }} />
        <span style={{ fontSize: "14px", fontWeight: "600", textTransform: "uppercase", letterSpacing: "0.1em", color: "var(--text-primary)" }}>
          CFHodEd JS
        </span>
        <span style={{ height: "14px", width: "1px", background: "var(--border-color)" }} />
        <span style={{ fontSize: "12px", color: "var(--accent-cyan)", fontWeight: "500", fontFamily: "var(--font-mono)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap", maxWidth: "400px" }}>
          {modelName || "NO HOD FILE LOADED"}
        </span>
      </div>

      {/* Editor Controls */}
      <div style={{ display: "flex", alignItems: "center", gap: "16px" }}>
        {transformMode && setTransformMode && (
          <div style={{ display: "flex", gap: "4px", background: "var(--bg-panel)", padding: "4px", borderRadius: "4px", border: "1px solid var(--border-color)" }}>
            <button 
              className={`toolbar-btn ${transformMode === "translate" ? "primary" : "outline"}`} 
              onClick={() => setTransformMode("translate")}
              title="Move Node"
            >
              Move
            </button>
            <button 
              className={`toolbar-btn ${transformMode === "rotate" ? "primary" : "outline"}`} 
              onClick={() => setTransformMode("rotate")}
              title="Rotate Node"
            >
              Rotate
            </button>
          </div>
        )}
        {/* File Operation Triggers */}
        <div style={{ display: "flex", gap: "8px" }}>
          {onNewClick && (
            <button onClick={onNewClick} style={{ height: "30px" }}>
              <FilePlus size={14} style={{ color: "var(--accent-blue)" }} />
              <span>New HOD</span>
            </button>
          )}
          {onSettingsClick && (
            <button className="toolbar-btn outline" onClick={onSettingsClick} title="Settings">
              <Settings size={16} /> Settings
            </button>
          )}
          <button className="toolbar-btn primary" onClick={onOpenClick} title="Open HOD File">
            <FolderOpen size={16} /> Open
          </button>
          {onImportDAEClick && (
            <button className="toolbar-btn outline" onClick={onImportDAEClick} title="Import DAE File">
              <FolderOpen size={16} /> Import DAE
            </button>
          )}
           <button
            onClick={onSaveClick}
            disabled={isSaving}
            style={{ height: "30px", opacity: isSaving ? 0.6 : 1 }}
          >
            <Save size={14} style={{ color: "var(--accent-success)" }} />
            <span>{isSaving ? "Saving..." : "Save HOD"}</span>
          </button>
          {onSaveAsClick && (
            <button
              onClick={onSaveAsClick}
              disabled={isSaving}
              style={{ height: "30px", opacity: isSaving ? 0.6 : 1 }}
            >
              <Save size={14} style={{ color: "var(--accent-cyan)" }} />
              <span>Save As...</span>
            </button>
          )}
        </div>
      </div>
    </header>
  );
};
