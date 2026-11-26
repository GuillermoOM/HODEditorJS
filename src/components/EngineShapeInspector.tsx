import React, { useState } from "react";
import { Flame, Plus, Trash2, Copy, Play } from "lucide-react";
import type { HODModel, Vector3D } from "./Viewport";
import {
  ENGINE_TRAIL_DEFAULTS,
  TYPICAL_TRAIL_PARAMS,
  engineShapeNozzleJointName,
  engineShapePoints,
  engineShapeStats,
  regularPolygonPoints,
  validateEngineShape,
  withEngineShapePoints,
  type EngineTrailParams,
  type EngineTrailPreviewSettings,
} from "../engine_trail";

interface Props {
  model: HODModel;
  shapeName: string;
  onModelChange?: (m: HODModel) => void;
  trailPreview: EngineTrailPreviewSettings;
  onTrailPreviewChange?: (s: EngineTrailPreviewSettings) => void;
}

const labelStyle: React.CSSProperties = { fontSize: "11px", color: "var(--text-secondary)", fontWeight: 500, marginBottom: "6px" };
const boxStyle: React.CSSProperties = {
  background: "rgba(22, 160, 255, 0.03)",
  border: "1px solid var(--border-color)",
  borderRadius: "4px",
  padding: "10px 12px",
};
const numStyle: React.CSSProperties = {
  width: "100%",
  height: "26px",
  padding: "2px 6px",
  fontSize: "11px",
  background: "#050a12",
  border: "1px solid var(--border-color)",
  color: "white",
  boxSizing: "border-box",
};
const smallBtn: React.CSSProperties = { height: "28px", fontSize: "11px", display: "flex", alignItems: "center", justifyContent: "center", gap: "6px" };

const fmt = (n: number) => (Number.isFinite(n) ? (Math.round(n * 1000) / 1000).toString() : "-");

const issueColor = (level: "error" | "warn" | "info") =>
  level === "error" ? "#ff6b6b" : level === "warn" ? "#ffb347" : "var(--text-muted)";

export const EngineShapeInspector: React.FC<Props> = ({ model, shapeName, onModelChange, trailPreview, onTrailPreviewChange }) => {
  const [polyCount, setPolyCount] = useState(6);
  const [polyRadius, setPolyRadius] = useState(1);
  const [targetMin, setTargetMin] = useState<string>("");

  const shape = model.engine_shapes?.find((s) => s.name === shapeName);
  if (!shape) return <div style={{ color: "var(--text-muted)", textAlign: "center" }}>Engine Shape not found</div>;

  const points = engineShapePoints(shape);
  const stats = engineShapeStats(points);
  const issues = validateEngineShape(model, shape);
  const nozzleJoint = engineShapeNozzleJointName(shape.name);
  const joint = nozzleJoint ? model.joints.find((j) => j.name.toLowerCase() === nozzleJoint.toLowerCase()) : undefined;
  const params = trailPreview.params;
  const halfWidth = params.diameter * stats.minDistance;

  const commitPoints = (pts: Vector3D[]) => {
    onModelChange?.({
      ...model,
      engine_shapes: model.engine_shapes.map((s) => (s.name === shape.name ? withEngineShapePoints(s, pts) : s)),
    });
  };
  const setPoint = (i: number, axis: "x" | "y" | "z", valStr: string) => {
    const v = parseFloat(valStr);
    if (!Number.isFinite(v)) return;
    commitPoints(points.map((p, idx) => (idx === i ? { ...p, [axis]: v } : p)));
  };
  const addPoint = () => {
    const last = points[points.length - 1];
    const base = last ? Math.hypot(last.x, last.y) || 1 : 1;
    const a = last ? Math.atan2(last.y, last.x) + Math.PI / 4 : 0;
    commitPoints([...points, { x: Math.cos(a) * base, y: Math.sin(a) * base, z: 0 }]);
  };
  const scaleToMin = () => {
    const target = parseFloat(targetMin);
    if (!Number.isFinite(target) || target <= 0 || stats.minDistance <= 1e-6) return;
    const k = target / stats.minDistance;
    commitPoints(points.map((p) => ({ x: p.x * k, y: p.y * k, z: p.z })));
  };
  const copyToOthers = () => {
    onModelChange?.({
      ...model,
      engine_shapes: model.engine_shapes.map((s) => (s.name === shape.name ? s : withEngineShapePoints(s, points))),
    });
  };
  const setParams = (patch: Partial<EngineTrailParams>) =>
    onTrailPreviewChange?.({ ...trailPreview, params: { ...params, ...patch } });
  const paramInput = (key: keyof EngineTrailParams, label: string, step: number, title: string) => (
    <label key={key} title={title} style={{ display: "flex", flexDirection: "column", gap: "2px", fontSize: "10px", color: "var(--text-muted)" }}>
      {label}
      <input
        type="number"
        step={step}
        value={params[key]}
        onChange={(e) => {
          const v = parseFloat(e.target.value);
          if (Number.isFinite(v)) setParams({ [key]: v } as Partial<EngineTrailParams>);
        }}
        style={numStyle}
      />
    </label>
  );

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: "16px" }}>
      <div>
        <div style={{ display: "flex", alignItems: "center", gap: "8px", fontSize: "10px", textTransform: "uppercase", letterSpacing: "0.15em", color: "var(--text-muted)", marginBottom: "4px" }}>
          <Flame size={12} style={{ color: "var(--accent-cyan)" }} />
          <span>Selected Engine Shape</span>
        </div>
        <div style={{ fontSize: "16px", fontWeight: 600, color: "var(--accent-cyan)", wordBreak: "break-all" }}>{shape.name}</div>
      </div>
      <hr style={{ border: "none", borderTop: "1px solid var(--border-color)", margin: 0 }} />

      <div style={{ fontSize: "11px", color: "var(--text-muted)", lineHeight: 1.5 }}>
        An engine shape is the nozzle's cross-section, not a mesh. The engine keeps only the point closest to the nozzle axis and uses it to
        size the engine trail ribbon of small craft (<code>setEngineTrail</code> in the <code>.ship</code>).
      </div>

      <div>
        <div style={labelStyle}>Nozzle</div>
        <div style={{ ...boxStyle, fontSize: "12px", display: "flex", flexDirection: "column", gap: "6px" }}>
          <div>
            Picked by the name's last digit: <b>{nozzleJoint ?? "(none)"}</b>
            {nozzleJoint && !joint && <span style={{ color: "#ff6b6b" }}> (joint missing)</span>}
          </div>
          <div style={{ color: "var(--text-secondary)" }}>Parent joint: {shape.parent_name || "(none)"}</div>
          {joint && shape.parent_name.toLowerCase() !== joint.name.toLowerCase() && (
            <button
              style={smallBtn}
              onClick={() =>
                onModelChange?.({
                  ...model,
                  engine_shapes: model.engine_shapes.map((s) =>
                    s.name === shape.name ? { ...s, parent_name: joint.name, mesh: { ...s.mesh, parent_name: joint.name } } : s
                  ),
                })
              }
            >
              Set parent to {joint.name}
            </button>
          )}
        </div>
      </div>

      <div>
        <div style={labelStyle}>Trail Size</div>
        <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: "8px", ...boxStyle }}>
          <div>
            <div style={{ fontSize: "10px", color: "var(--text-muted)" }}>POINTS</div>
            <div style={{ fontSize: "15px", fontWeight: 600 }}>{stats.count}</div>
          </div>
          <div title="Smallest sqrt(x² + y²): the only value the engine draws with">
            <div style={{ fontSize: "10px", color: "var(--text-muted)" }}>MIN DIMENSION</div>
            <div style={{ fontSize: "15px", fontWeight: 600, color: "#4dd2ff" }}>{fmt(stats.minDistance)}</div>
          </div>
          <div>
            <div style={{ fontSize: "10px", color: "var(--text-muted)" }}>LARGEST DISTANCE</div>
            <div style={{ fontSize: "15px", fontWeight: 600 }}>{fmt(stats.maxDistance)}</div>
          </div>
          <div title="diameter (setEngineTrail) × min dimension; the ribbon extends this far to each side">
            <div style={{ fontSize: "10px", color: "var(--text-muted)" }}>RIBBON HALF WIDTH</div>
            <div style={{ fontSize: "15px", fontWeight: 600 }}>{fmt(halfWidth)}</div>
          </div>
        </div>
        <div style={{ fontSize: "10px", color: "var(--text-muted)", marginTop: "4px" }}>
          Half width uses the trail simulation's diameter ({fmt(params.diameter)}); the real value comes from the .ship file.
        </div>
      </div>

      {issues.length > 0 && (
        <div style={{ display: "flex", flexDirection: "column", gap: "4px" }}>
          {issues.map((iss, i) => (
            <div key={i} style={{ fontSize: "11px", color: issueColor(iss.level), lineHeight: 1.4 }}>
              {iss.level === "error" ? "✖ " : iss.level === "warn" ? "⚠ " : "ℹ "}
              {iss.text}
            </div>
          ))}
        </div>
      )}

      <div>
        <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", marginBottom: "6px" }}>
          <div style={labelStyle}>Points (x, y, z)</div>
          <button style={{ ...smallBtn, padding: "0 8px" }} onClick={addPoint}>
            <Plus size={12} style={{ color: "var(--accent-cyan)" }} /> Add
          </button>
        </div>
        <div style={{ display: "flex", flexDirection: "column", gap: "4px", maxHeight: "260px", overflowY: "auto" }}>
          {points.length === 0 && <div style={{ fontSize: "11px", color: "var(--text-muted)" }}>No points: the shape draws nothing.</div>}
          {points.map((p, i) => {
            const d = Math.hypot(p.x, p.y);
            const isMin = stats.count > 0 && Math.abs(d - stats.minDistance) < 1e-6;
            return (
              <div key={i} style={{ display: "grid", gridTemplateColumns: "18px 1fr 1fr 1fr 26px", gap: "4px", alignItems: "center" }}>
                <span style={{ fontSize: "10px", color: isMin ? "#4dd2ff" : "var(--text-muted)" }} title={isMin ? "Closest point (min dimension)" : ""}>
                  {i + 1}
                </span>
                {(["x", "y", "z"] as const).map((axis) => (
                  <input
                    key={axis}
                    type="number"
                    step={0.05}
                    value={Math.round(p[axis] * 10000) / 10000}
                    onChange={(e) => setPoint(i, axis, e.target.value)}
                    style={{ ...numStyle, opacity: axis === "z" ? 0.6 : 1 }}
                    title={axis === "z" ? "z is not used by the engine" : undefined}
                  />
                ))}
                <button
                  style={{ ...smallBtn, height: "26px", padding: 0 }}
                  title="Remove point"
                  onClick={() => commitPoints(points.filter((_, idx) => idx !== i))}
                >
                  <Trash2 size={12} />
                </button>
              </div>
            );
          })}
        </div>
      </div>

      <div>
        <div style={labelStyle}>Tools</div>
        <div style={{ ...boxStyle, display: "flex", flexDirection: "column", gap: "8px" }}>
          <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr auto", gap: "6px", alignItems: "end" }}>
            <label style={{ fontSize: "10px", color: "var(--text-muted)" }}>
              Polygon points
              <input type="number" min={3} max={32} value={polyCount} onChange={(e) => setPolyCount(Math.max(3, parseInt(e.target.value, 10) || 3))} style={numStyle} />
            </label>
            <label style={{ fontSize: "10px", color: "var(--text-muted)" }}>
              Radius
              <input type="number" step={0.05} min={0.01} value={polyRadius} onChange={(e) => setPolyRadius(parseFloat(e.target.value) || 1)} style={numStyle} />
            </label>
            <button style={smallBtn} onClick={() => commitPoints(regularPolygonPoints(polyCount, polyRadius))}>
              Generate
            </button>
          </div>
          <div style={{ display: "grid", gridTemplateColumns: "1fr auto", gap: "6px", alignItems: "end" }}>
            <label style={{ fontSize: "10px", color: "var(--text-muted)" }}>
              Scale points so the min dimension is
              <input type="number" step={0.05} min={0.01} placeholder={fmt(stats.minDistance)} value={targetMin} onChange={(e) => setTargetMin(e.target.value)} style={numStyle} />
            </label>
            <button style={smallBtn} onClick={scaleToMin} disabled={!targetMin}>
              Apply
            </button>
          </div>
          <button style={smallBtn} onClick={copyToOthers} disabled={(model.engine_shapes?.length ?? 0) < 2}>
            <Copy size={12} style={{ color: "var(--accent-cyan)" }} /> Copy these points to all other engine shapes
          </button>
        </div>
      </div>

      <div>
        <div style={labelStyle}>Trail Simulation (preview only)</div>
        <div style={{ ...boxStyle, display: "flex", flexDirection: "column", gap: "8px" }}>
          <label style={{ display: "flex", alignItems: "center", gap: "8px", fontSize: "12px", cursor: "pointer" }}>
            <input
              type="checkbox"
              checked={trailPreview.enabled}
              onChange={(e) => onTrailPreviewChange?.({ ...trailPreview, enabled: e.target.checked })}
              style={{ width: "auto", height: "auto", margin: 0 }}
            />
            <Play size={12} style={{ color: "var(--accent-cyan)" }} />
            Fly the ship forward and draw the engine trails
          </label>
          <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: "8px" }}>
            <label style={{ display: "flex", flexDirection: "column", gap: "2px", fontSize: "10px", color: "var(--text-muted)" }}>
              Speed (units/s)
              <input
                type="number"
                step={10}
                min={0}
                value={trailPreview.speed}
                onChange={(e) => {
                  const v = parseFloat(e.target.value);
                  if (Number.isFinite(v)) onTrailPreviewChange?.({ ...trailPreview, speed: Math.max(0, v) });
                }}
                style={numStyle}
              />
            </label>
            <label style={{ display: "flex", flexDirection: "column", gap: "2px", fontSize: "10px", color: "var(--text-muted)" }}>
              Maneuver
              <select
                value={trailPreview.maneuver}
                onChange={(e) => onTrailPreviewChange?.({ ...trailPreview, maneuver: e.target.value as "straight" | "weave" })}
                style={{ ...numStyle, padding: "2px 4px" }}
              >
                <option value="straight">Straight</option>
                <option value="weave">Weave (turns)</option>
              </select>
            </label>
            {paramInput("linger", "Linger (s)", 0.5, "Seconds a trail point lives; ribbon length = speed × linger")}
            {paramInput("diameter", "Diameter", 0.5, "Ribbon half width = diameter × the shape's min dimension")}
            {paramInput("bulge", "Bulge", 0.001, "Chance per update of a thickened segment")}
            {paramInput("uscroll", "U scroll", 0.1, "Texture scroll with speed")}
            {paramInput("ufactor", "U factor", 0.005, "Texture stretch along the ribbon")}
          </div>
          <div style={{ display: "flex", gap: "6px" }}>
            <button style={{ ...smallBtn, flex: 1 }} onClick={() => setParams({ ...TYPICAL_TRAIL_PARAMS })}>
              Typical small craft
            </button>
            <button style={{ ...smallBtn, flex: 1 }} onClick={() => setParams({ ...ENGINE_TRAIL_DEFAULTS })}>
              Engine defaults
            </button>
          </div>
          <div style={{ fontSize: "10px", color: "var(--text-muted)", lineHeight: 1.4 }}>
            These values live in the ship's <code>.ship</code> file (<code>setEngineTrail(ship, nozzle, linger, texture, bulge, uscroll, ufactor, diameter)</code>),
            not in the HOD. Frame logic follows the retail engine; the ribbon colour and texture are approximations. Nozzles that animate are drawn at their rest position.
          </div>
        </div>
      </div>
    </div>
  );
};
