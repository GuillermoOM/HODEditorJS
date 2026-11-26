import React, { useState } from "react";
import { Flame, Plus, Trash2, Copy, Play } from "lucide-react";
import type { HODModel, Vector3D } from "./Viewport";
import { NumericInput } from "./Inspector";
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
  /** Index of the point selected in the viewport / table (null = none). */
  selectedPoint?: number | null;
  onSelectPoint?: (index: number | null) => void;
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

const round4 = (n: number) => Math.round(n * 10000) / 10000;

/** Mouse-wheel stepping for NumericInput: wheel up/down = +/- step, Shift = x10, Alt = x0.1. */
const wheelStep =
  (apply: (v: number) => void, step: number, min?: number) =>
  (e: React.WheelEvent<HTMLInputElement>) => {
    const mult = e.shiftKey ? 10 : e.altKey ? 0.1 : 1;
    const dir = e.deltaY < 0 ? 1 : -1;
    const cur = parseFloat(e.currentTarget.value) || 0;
    let v = round4(cur + dir * step * mult);
    if (min !== undefined) v = Math.max(min, v);
    apply(v);
  };

const fmt = (n: number) => (Number.isFinite(n) ? (Math.round(n * 1000) / 1000).toString() : "-");

const issueColor = (level: "error" | "warn" | "info") =>
  level === "error" ? "#ff6b6b" : level === "warn" ? "#ffb347" : "var(--text-muted)";

export const EngineShapeInspector: React.FC<Props> = ({
  model,
  shapeName,
  onModelChange,
  trailPreview,
  onTrailPreviewChange,
  selectedPoint = null,
  onSelectPoint,
}) => {
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
      <NumericInput
        value={round4(params[key])}
        step={String(step)}
        onChange={(v) => {
          const n = parseFloat(v);
          if (Number.isFinite(n)) setParams({ [key]: n } as Partial<EngineTrailParams>);
        }}
        onWheel={wheelStep((n) => setParams({ [key]: n } as Partial<EngineTrailParams>), step, 0)}
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
            const isSel = selectedPoint === i;
            return (
              <div
                key={i}
                onFocusCapture={() => onSelectPoint?.(i)}
                style={{
                  display: "grid",
                  gridTemplateColumns: "18px 1fr 1fr 1fr 26px",
                  gap: "4px",
                  alignItems: "center",
                  padding: "2px",
                  borderRadius: "3px",
                  background: isSel ? "rgba(77, 210, 255, 0.15)" : "transparent",
                  outline: isSel ? "1px solid rgba(77, 210, 255, 0.5)" : "none",
                }}
              >
                <span
                  style={{ fontSize: "10px", cursor: "pointer", color: isMin ? "#4dd2ff" : "var(--text-muted)" }}
                  title={isMin ? "Closest point (min dimension). Click to select it in the viewport." : "Click to select it in the viewport."}
                  onClick={() => onSelectPoint?.(i)}
                >
                  {i + 1}
                </span>
                {(["x", "y", "z"] as const).map((axis) => (
                  <NumericInput
                    key={axis}
                    value={round4(p[axis])}
                    step="0.05"
                    onChange={(v) => setPoint(i, axis, v)}
                    onWheel={wheelStep((n) => commitPoints(points.map((q, idx) => (idx === i ? { ...q, [axis]: n } : q))), 0.05)}
                    style={{ ...numStyle, opacity: axis === "z" ? 0.6 : 1 }}
                  />
                ))}
                <button
                  style={{ ...smallBtn, height: "26px", padding: 0 }}
                  title="Remove point"
                  onClick={() => {
                    if (selectedPoint !== null) onSelectPoint?.(null);
                    commitPoints(points.filter((_, idx) => idx !== i));
                  }}
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
              <NumericInput
                value={polyCount}
                step="1"
                min="3"
                max="32"
                onChange={(v) => setPolyCount(Math.min(32, Math.max(3, parseInt(v, 10) || 3)))}
                onWheel={wheelStep((n) => setPolyCount(Math.min(32, Math.max(3, Math.round(n)))), 1, 3)}
                style={numStyle}
              />
            </label>
            <label style={{ fontSize: "10px", color: "var(--text-muted)" }}>
              Radius
              <NumericInput
                value={round4(polyRadius)}
                step="0.05"
                min="0.01"
                onChange={(v) => setPolyRadius(Math.max(0.01, parseFloat(v) || 1))}
                onWheel={wheelStep((n) => setPolyRadius(Math.max(0.01, n)), 0.05, 0.01)}
                style={numStyle}
              />
            </label>
            <button style={smallBtn} onClick={() => commitPoints(regularPolygonPoints(polyCount, polyRadius))}>
              Generate
            </button>
          </div>
          <div style={{ display: "grid", gridTemplateColumns: "1fr auto", gap: "6px", alignItems: "end" }}>
            <label style={{ fontSize: "10px", color: "var(--text-muted)" }}>
              Scale points so the min dimension is
              <NumericInput
                value={targetMin === "" ? round4(stats.minDistance) : round4(parseFloat(targetMin) || 0)}
                step="0.05"
                min="0.01"
                onChange={(v) => setTargetMin(v)}
                onWheel={wheelStep((n) => setTargetMin(String(Math.max(0.01, n))), 0.05, 0.01)}
                style={numStyle}
              />
            </label>
            <button style={smallBtn} onClick={scaleToMin} disabled={!targetMin || Math.abs((parseFloat(targetMin) || 0) - stats.minDistance) < 1e-6}>
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
              <NumericInput
                value={round4(trailPreview.speed)}
                step="10"
                min="0"
                onChange={(v) => {
                  const n = parseFloat(v);
                  if (Number.isFinite(n)) onTrailPreviewChange?.({ ...trailPreview, speed: Math.max(0, n) });
                }}
                onWheel={wheelStep((n) => onTrailPreviewChange?.({ ...trailPreview, speed: Math.max(0, n) }), 10, 0)}
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
