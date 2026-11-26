/**
 * Engine shapes (`ETSH`) and the engine trail they size.
 *
 * Retail (hwrm-modern docs/evidence/effects.md section 15, read from the executable):
 * - An ETSH chunk is `name, parent, N, N points`. It is NOT geometry. The engine picks the nozzle from the
 *   LAST character of the name (a digit, minus 1; `EngineShape1` -> nozzle `EngineNozzle1`, a trailing 0 is
 *   ignored) and keeps from the points' (x, y) the smallest and largest distances sqrt(x^2 + y^2), the largest
 *   |x| and |y|. Only the smallest one (`minDimension`) reaches the drawing; a shape without points draws nothing.
 * - It only matters for ships whose `.ship` calls `setEngineTrail` (small craft, missiles, HW1 craft). The
 *   ribbon radius is `diameter * minDimension` (diameter is a `setEngineTrail` argument, not in the HOD).
 *
 * `EngineTrailSim` below is a port of the retail frame logic (`EngineTrail::Update` / `expireFrames`) and ribbon
 * builder (`FUN_004C03E3` + `fx_eng_trailbasic.vert`) used for the editor's optional trail simulation. The team
 * palette and the ribbon texture are approximated (retail reads `trailref.tga` / `trail_ribbon.tga`).
 */
import * as THREE from "three";
import type { HODEngineShape, HODModel, Vector3D } from "./components/Viewport";

export interface EngineTrailParams {
  /** Seconds a frame lives (ribbon length = speed * linger). */
  linger: number;
  /** Probability per update of a thickened ("bulge") frame. */
  bulge: number;
  uscroll: number;
  ufactor: number;
  /** Multiplies the shape's min dimension to give the ribbon's half width. */
  diameter: number;
}

export interface EngineTrailPreviewSettings {
  enabled: boolean;
  params: EngineTrailParams;
  /** Preview ship speed in model units per second. */
  speed: number;
  maneuver: "straight" | "weave";
}

/** Engine defaults of `setNumNozzles` (0x004C56C1). */
export const ENGINE_TRAIL_DEFAULTS: EngineTrailParams = { linger: 1, bulge: 0.95, uscroll: 0.5, ufactor: 0.1, diameter: 2 };
/** A typical small-craft `.ship` declaration: setEngineTrail(ship, 0, 4, "trail_ribbon.tga", 0.001, 0.5, 0.025, 4). */
export const TYPICAL_TRAIL_PARAMS: EngineTrailParams = { linger: 4, bulge: 0.001, uscroll: 0.5, ufactor: 0.025, diameter: 4 };

export const DEFAULT_TRAIL_PREVIEW: EngineTrailPreviewSettings = {
  enabled: false,
  params: { ...TYPICAL_TRAIL_PARAMS },
  speed: 100,
  maneuver: "straight",
};

// --- shape data -------------------------------------------------------------------------------------------------------------

/** The shape's points: the first mesh part's vertices (the parser stores every ETSH point there and the writer saves part 0). */
export function engineShapePoints(shape: HODEngineShape): Vector3D[] {
  const part = shape.mesh?.parts?.[0];
  return part ? part.vertices.map((v) => v.position) : [];
}

/** Returns `shape` with its points replaced (one mesh part, sequential indices, position-only vertices). */
export function withEngineShapePoints(shape: HODEngineShape, points: Vector3D[]): HODEngineShape {
  return {
    ...shape,
    mesh: {
      ...shape.mesh,
      parts: [
        {
          material_index: 0,
          vertex_mask: 1,
          vertices: points.map((p) => ({ position: { x: p.x, y: p.y, z: p.z } })),
          indices: points.map((_, i) => i),
        },
      ],
    },
  };
}

export interface EngineShapeStats {
  count: number;
  /** `minDimension`: the only value the engine draws with. */
  minDistance: number;
  maxDistance: number;
  maxAbsX: number;
  maxAbsY: number;
  /** Retail's `+0x20`: smallest + largest distance. */
  diameter: number;
}

export function engineShapeStats(points: Vector3D[]): EngineShapeStats {
  if (points.length === 0) return { count: 0, minDistance: 0, maxDistance: 0, maxAbsX: 0, maxAbsY: 0, diameter: 0 };
  let minD = Infinity;
  let maxD = 0;
  let maxX = 0;
  let maxY = 0;
  for (const p of points) {
    const d = Math.sqrt(p.x * p.x + p.y * p.y);
    minD = Math.min(minD, d);
    maxD = Math.max(maxD, d);
    maxX = Math.max(maxX, Math.abs(p.x));
    maxY = Math.max(maxY, Math.abs(p.y));
  }
  return { count: points.length, minDistance: minD, maxDistance: maxD, maxAbsX: maxX, maxAbsY: maxY, diameter: minD + maxD };
}

/** 1-based nozzle number from the shape name's last character (retail: digit - 1; 0 or a non-digit is ignored). */
export function engineShapeNozzleNumber(name: string): number | null {
  const c = name.slice(-1);
  if (!/^[0-9]$/.test(c)) return null;
  const n = parseInt(c, 10);
  return n === 0 ? null : n;
}

export function engineShapeNozzleJointName(name: string): string | null {
  const n = engineShapeNozzleNumber(name);
  return n === null ? null : `EngineNozzle${n}`;
}

/** Regular polygon in the xy plane. */
export function regularPolygonPoints(count: number, radius: number): Vector3D[] {
  const n = Math.max(3, Math.round(count));
  const pts: Vector3D[] = [];
  for (let i = 0; i < n; i++) {
    const a = (i / n) * Math.PI * 2;
    pts.push({ x: Math.cos(a) * radius, y: Math.sin(a) * radius, z: 0 });
  }
  return pts;
}

/** A free `EngineShape<digit>` name for `jointName` (prefers the nozzle's own number, else the first unused digit 1-9). */
export function suggestEngineShapeName(model: HODModel, jointName: string): string {
  const used = new Set((model.engine_shapes || []).map((s) => s.name.toLowerCase()));
  const m = /(\d)$/.exec(jointName);
  if (m && m[1] !== "0" && !used.has(`engineshape${m[1]}`)) return `EngineShape${m[1]}`;
  for (let d = 1; d <= 9; d++) if (!used.has(`engineshape${d}`)) return `EngineShape${d}`;
  return `EngineShape${m ? m[1] : "1"}`;
}

export interface EngineShapeIssue {
  level: "error" | "warn" | "info";
  text: string;
}

/** Checks a shape against what the retail ETSH handler does with it. */
export function validateEngineShape(model: HODModel, shape: HODEngineShape): EngineShapeIssue[] {
  const issues: EngineShapeIssue[] = [];
  const points = engineShapePoints(shape);
  const stats = engineShapeStats(points);
  const n = engineShapeNozzleNumber(shape.name);
  if (n === null) {
    issues.push({
      level: "error",
      text: `The name must end in a digit 1-9 (the engine takes the nozzle from the last character); "${shape.name}" is ignored in game.`,
    });
  } else {
    const jointName = `EngineNozzle${n}`;
    const joint = model.joints.find((j) => j.name.toLowerCase() === jointName.toLowerCase());
    if (!joint) {
      issues.push({ level: "error", text: `No joint named ${jointName}: the engine builds this nozzle's trail from that joint, so this shape draws nothing.` });
    } else if (shape.parent_name.toLowerCase() !== joint.name.toLowerCase()) {
      issues.push({
        level: "warn",
        text: `Parent is ${shape.parent_name || "(none)"}, but the nozzle is chosen by the name (${joint.name}). The trail system ignores the parent.`,
      });
    }
    const dup = (model.engine_shapes || []).filter((s) => s !== shape && engineShapeNozzleNumber(s.name) === n);
    if (dup.length > 0) issues.push({ level: "warn", text: `Another shape (${dup.map((d) => d.name).join(", ")}) also targets ${jointName}.` });
  }
  if (stats.count === 0) {
    issues.push({ level: "error", text: "A shape without points draws nothing (retail skips the trail)." });
  } else if (stats.minDistance < 1e-4) {
    issues.push({ level: "warn", text: "The closest point sits on the nozzle axis, so the trail has zero width." });
  }
  if (points.some((p) => Math.abs(p.z) > 1e-3)) {
    issues.push({ level: "info", text: "Only x and y are used (distance from the nozzle axis); z has no effect." });
  }
  issues.push({
    level: "info",
    text: "Only used in game when the ship's .ship file calls setEngineTrail for this nozzle (small craft, missiles, HW1 craft).",
  });
  return issues;
}

// --- trail simulation (port of EngineTrail::Update / expireFrames / FUN_004C03E3) --------------------------------------------

const MIN_SPEED = 5;
const INTENSITY_FALL = 2;
const INTENSITY_GAIN = 0.01;
const MIN_GAP_SQ = 4;
const FAR_GAP_SQ = 250000;
const AWAY_LIMIT = 0.95;
const AWAY_DAMPING = 3;
const BULGE_SCALE = 3;
const BULGE_BUMP = 0.85;
const SHORTEST_SEGMENT = 2;
const SHORTEST_WALK = 0.5;
const UPDATE_PERIOD = 0.1;

interface TrailFrame {
  time: number;
  intensity: number;
  scale: number;
  length: number;
  position: THREE.Vector3;
  direction: THREE.Vector3;
}

class Trail {
  frames: TrailFrame[] = []; // newest first
  intensity = 0;
  constructor(public linger: number, public bulge: number) {}

  private add(direction: THREE.Vector3, length: number, position: THREE.Vector3, time: number, scale: number) {
    this.frames.unshift({ time, intensity: this.intensity, scale, length, position: position.clone(), direction: direction.clone() });
  }

  update(time: number, dt: number, forward: THREE.Vector3, velocity: THREE.Vector3, place: THREE.Vector3, diameterScale: number) {
    const speed = velocity.length();
    const unitVelocity = speed > 0 ? velocity.clone().multiplyScalar(1 / speed) : velocity.clone();
    const facing = forward.dot(unitVelocity);
    const occlusion = facing <= 0 ? 1 : facing < 1 ? 1 - facing : 0;
    let addWanted = true;
    if (speed * (1 - occlusion * occlusion) < MIN_SPEED) {
      addWanted = false;
      this.intensity = Math.max(0, this.intensity - INTENSITY_FALL * dt);
    }
    if (this.frames.length === 0) {
      if (addWanted) this.add(new THREE.Vector3(), 0, place, time, 1 * diameterScale);
    } else {
      const gap = this.frames[0].position.clone().sub(place);
      const gapSq = gap.lengthSq();
      if (gapSq > MIN_GAP_SQ) {
        const distance = Math.sqrt(gapSq);
        const direction = gap.multiplyScalar(1 / distance);
        this.intensity = Math.min(1, distance * INTENSITY_GAIN + this.intensity);
        let add = false;
        let scale = 1 * diameterScale;
        if (this.frames.length < 2) {
          add = addWanted;
        } else if (this.bulge >= Math.random()) {
          add = true;
          scale = BULGE_SCALE * diameterScale;
        } else {
          const away = -forward.dot(direction);
          if (away < AWAY_LIMIT) {
            add = true;
            this.intensity *= Math.min(1, Math.max(0, 1 - (AWAY_LIMIT - away) * AWAY_DAMPING));
          } else {
            add = addWanted;
          }
          if (gapSq > FAR_GAP_SQ) add = true;
        }
        if (add) this.add(direction, distance, place, time, scale);
      }
    }
    this.expire(time);
  }

  expire(time: number) {
    const cutoff = time - this.linger;
    while (this.frames.length > 0) {
      const oldest = this.frames[this.frames.length - 1];
      if (cutoff <= oldest.time) break;
      if (this.frames.length < 2) {
        this.frames.pop();
        continue;
      }
      const newer = this.frames[this.frames.length - 2];
      let length = 0;
      if (newer.time >= cutoff) length = ((newer.time - cutoff) / (newer.time - oldest.time)) * newer.length;
      if (newer.time < cutoff || length <= SHORTEST_SEGMENT) {
        this.frames.pop();
        continue;
      }
      oldest.time = cutoff;
      oldest.position = newer.position.clone().addScaledVector(newer.direction, length);
      newer.length = length;
    }
  }
}

export interface TrailNozzle {
  /** Nozzle position in the ship's frame (model space). */
  offset: THREE.Vector3;
  /** Smallest (x, y) distance of the nozzle's ETSH points. */
  minDimension: number;
}

/** Ribbon vertex streams ready for a BufferGeometry. */
export interface RibbonBuffers {
  positions: Float32Array;
  uv0: Float32Array; // age (1 at the nozzle, 0 at the tail), edge
  uv1: Float32Array; // slideU, bump
  fade: Float32Array;
  indices: Uint32Array;
  vertexCount: number;
  indexCount: number;
}

const tmpA = new THREE.Vector3();
const tmpB = new THREE.Vector3();
const tmpC = new THREE.Vector3();

export class EngineTrailSim {
  readonly nozzles: TrailNozzle[];
  private trails: Trail[];
  private time = 0;
  private lastUpdate = -1;
  private heading = 0;
  private shipPos = new THREE.Vector3();
  private velocity = new THREE.Vector3();
  params: EngineTrailParams;
  speed: number;
  maneuver: "straight" | "weave";
  private capacity = 0;
  buffers: RibbonBuffers = { positions: new Float32Array(0), uv0: new Float32Array(0), uv1: new Float32Array(0), fade: new Float32Array(0), indices: new Uint32Array(0), vertexCount: 0, indexCount: 0 };

  constructor(nozzles: TrailNozzle[], params: EngineTrailParams, speed: number, maneuver: "straight" | "weave") {
    this.nozzles = nozzles;
    this.params = params;
    this.speed = speed;
    this.maneuver = maneuver;
    this.trails = nozzles.map(() => new Trail(params.linger, params.bulge));
    this.ensureCapacity(2048);
  }

  private ensureCapacity(verts: number) {
    if (verts <= this.capacity) return;
    this.capacity = verts;
    this.buffers = {
      positions: new Float32Array(verts * 3),
      uv0: new Float32Array(verts * 2),
      uv1: new Float32Array(verts * 2),
      fade: new Float32Array(verts),
      indices: new Uint32Array(Math.ceil((verts / 4) * 6)),
      vertexCount: 0,
      indexCount: 0,
    };
  }

  /** Ship forward (orientation row 2) for the current heading. */
  private forward(): THREE.Vector3 {
    return new THREE.Vector3(Math.sin(this.heading), 0, Math.cos(this.heading));
  }

  /** The ship's world matrix (translation + heading about Y); the ribbon is built in world space. */
  shipMatrix(): THREE.Matrix4 {
    const m = new THREE.Matrix4().makeRotationY(this.heading);
    m.setPosition(this.shipPos);
    return m;
  }

  step(dt: number) {
    dt = Math.min(Math.max(dt, 0), 0.1);
    if (dt <= 0) return;
    this.time += dt;
    for (const t of this.trails) {
      t.linger = Math.max(0.05, this.params.linger);
      t.bulge = this.params.bulge;
    }
    this.heading = this.maneuver === "weave" ? (35 * Math.PI) / 180 * Math.sin((this.time / 6) * Math.PI * 2) : 0;
    const fwd = this.forward();
    this.velocity.copy(fwd).multiplyScalar(this.speed);
    this.shipPos.addScaledVector(this.velocity, dt);
    if (this.time - this.lastUpdate > UPDATE_PERIOD) {
      this.lastUpdate = this.time;
      const right = new THREE.Vector3(Math.cos(this.heading), 0, -Math.sin(this.heading));
      const up = new THREE.Vector3(0, 1, 0);
      this.trails.forEach((trail, i) => {
        const o = this.nozzles[i].offset;
        const place = this.shipPos.clone().addScaledVector(right, o.x).addScaledVector(up, o.y).addScaledVector(fwd, o.z);
        trail.update(this.time, dt, fwd, this.velocity, place, 1);
      });
    }
    // Keep world coordinates small so float32 ribbon positions stay precise.
    if (this.shipPos.length() > 4000) {
      const shift = this.shipPos.clone();
      for (const t of this.trails) for (const f of t.frames) f.position.sub(shift);
      this.shipPos.set(0, 0, 0);
    }
  }

  /** Builds the camera-facing ribbons for `eyeWorld` (world space, same as the frames). */
  build(eyeWorld: THREE.Vector3) {
    let total = 0;
    for (const t of this.trails) total += (t.frames.length + 2) * 4;
    this.ensureCapacity(Math.max(this.capacity, total + 16));
    const b = this.buffers;
    let v = 0;
    let ix = 0;
    const fwd = this.forward();
    const right = new THREE.Vector3(Math.cos(this.heading), 0, -Math.sin(this.heading));
    const speed = this.velocity.length();
    const now = this.time;

    const emit = (center: THREE.Vector3, dir: THREE.Vector3, radius: number, slide: number, age: number, fade: number, bump: number) => {
      const edge = bump < 0 ? 0 : 1;
      const b2 = bump < 0 ? bump + 1 : bump;
      tmpA.copy(center).sub(eyeWorld);
      if (tmpA.lengthSq() < 1e-12) tmpA.set(0, 0, 1);
      tmpA.normalize();
      tmpB.copy(dir);
      if (tmpB.lengthSq() < 1e-12) tmpB.copy(fwd);
      tmpB.normalize();
      tmpC.crossVectors(tmpA, tmpB);
      if (tmpC.lengthSq() < 1e-12) tmpC.copy(right);
      tmpC.normalize();
      const facingFade = Math.min(1, Math.max(0, (Math.abs(tmpA.dot(tmpB)) - 0.8) * 5.25));
      const f = fade * (1 - Math.pow(facingFade, 1.5));
      const side = (edge * 2 - 1) * radius;
      b.positions[v * 3] = center.x + tmpC.x * side;
      b.positions[v * 3 + 1] = center.y + tmpC.y * side;
      b.positions[v * 3 + 2] = center.z + tmpC.z * side;
      b.uv0[v * 2] = Math.min(1, Math.max(0, age));
      b.uv0[v * 2 + 1] = edge;
      b.uv1[v * 2] = slide;
      b.uv1[v * 2 + 1] = b2;
      b.fade[v] = f;
      v++;
    };

    this.trails.forEach((trail, ti) => {
      const frames = trail.frames;
      const count = frames.length;
      if (count === 0) return;
      const nz = this.nozzles[ti];
      const o = nz.offset;
      const place = this.shipPos.clone().addScaledVector(right, o.x).addScaledVector(new THREE.Vector3(0, 1, 0), o.y).addScaledVector(fwd, o.z);
      const radius = this.params.diameter * nz.minDimension;
      const slideRaw = -(now * speed * this.params.uscroll);
      const scroll = slideRaw - Math.trunc(slideRaw);
      const inverseLinger = 1 / Math.max(0.05, trail.linger);
      const dir = new THREE.Vector3();
      let distance = 0;
      let first = 0;
      const aim = (i: number) => {
        dir.copy(frames[i].position).sub(place);
        distance = dir.length();
        if (distance > 0) dir.multiplyScalar(1 / distance);
        else dir.set(0, 0, 0);
      };
      aim(0);
      if (count > 1 && distance < 1) {
        if (1 >= count) return;
        first = 1;
        aim(first);
      }
      let prevCenter = place.clone();
      let prevDir = dir.clone();
      let prevSlide = scroll;
      let prevAge = 1 - (now - now) * inverseLinger;
      let prevFade = trail.intensity;
      let prevBump = 1;
      let prevDirForNext = dir.clone();
      let walked = 0;
      let step = distance;
      for (let i = first; i < count; i++) {
        const frame = frames[i];
        walked += step;
        step = frame.length;
        if (!(walked > SHORTEST_WALK)) continue;
        const curDir = frame.direction.clone().add(prevDirForNext);
        const curSlide = this.params.ufactor * walked + scroll;
        const curAge = 1 - (now - frame.time) * inverseLinger;
        const curFade = frame.intensity;
        const curBump = frame.scale > 1 ? BULGE_BUMP : 1;
        prevDirForNext = frame.direction.clone();
        const base = v;
        emit(prevCenter, prevDir, radius, prevSlide, prevAge, prevFade, -prevBump);
        emit(prevCenter, prevDir, radius, prevSlide, prevAge, prevFade, prevBump);
        emit(frame.position, curDir, radius, curSlide, curAge, curFade, curBump);
        emit(frame.position, curDir, radius, curSlide, curAge, curFade, -curBump);
        b.indices[ix++] = base;
        b.indices[ix++] = base + 1;
        b.indices[ix++] = base + 2;
        b.indices[ix++] = base;
        b.indices[ix++] = base + 2;
        b.indices[ix++] = base + 3;
        prevCenter = frame.position.clone();
        prevDir = curDir;
        prevSlide = curSlide;
        prevAge = curAge;
        prevFade = curFade;
        prevBump = curBump;
        if (!(curAge > 0)) break;
      }
    });
    b.vertexCount = v;
    b.indexCount = ix;
  }
}

// --- ribbon material -------------------------------------------------------------------------------------------------------

const RIBBON_VERT = `
attribute vec2 aUV0;
attribute vec2 aUV1;
attribute float aFade;
varying vec2 vUV0;
varying vec2 vUV1;
varying float vFade;
void main() {
  vUV0 = aUV0;
  vUV1 = aUV1;
  vFade = aFade;
  gl_Position = projectionMatrix * modelViewMatrix * vec4(position, 1.0);
}
`;

// Approximates fx_eng_trailbasic.frag: team palette along the age (dark ramp, black at both ends) times the
// ribbon profile across v (white in the middle, black at the edges), times 1.5; blend srcAlpha/one.
const RIBBON_FRAG = `
uniform vec3 uColor;
varying vec2 vUV0;
varying vec2 vUV1;
varying float vFade;
void main() {
  float u = clamp(vUV0.x, 0.0, 1.0);
  float team = smoothstep(0.0, 0.3, u) * (1.0 - smoothstep(0.9, 1.0, u));
  float s = 1.0 - abs(vUV1.y * 2.0 - 1.0);
  float shape = smoothstep(0.0, 0.6, s);
  vec3 rgb = uColor * team * shape * 1.5;
  float a = clamp(vFade, 0.0, 1.0);
  gl_FragColor = vec4(rgb, a);
}
`;

export function createRibbonMaterial(color: THREE.Color): THREE.ShaderMaterial {
  return new THREE.ShaderMaterial({
    uniforms: { uColor: { value: color.clone() } },
    vertexShader: RIBBON_VERT,
    fragmentShader: RIBBON_FRAG,
    transparent: true,
    depthWrite: false,
    depthTest: true,
    side: THREE.DoubleSide,
    blending: THREE.CustomBlending,
    blendSrc: THREE.SrcAlphaFactor,
    blendDst: THREE.OneFactor,
    blendEquation: THREE.AddEquation,
  });
}

export function createRibbonGeometry(): THREE.BufferGeometry {
  const g = new THREE.BufferGeometry();
  g.setAttribute("position", new THREE.BufferAttribute(new Float32Array(3), 3));
  g.setAttribute("aUV0", new THREE.BufferAttribute(new Float32Array(2), 2));
  g.setAttribute("aUV1", new THREE.BufferAttribute(new Float32Array(2), 2));
  g.setAttribute("aFade", new THREE.BufferAttribute(new Float32Array(1), 1));
  g.setIndex(new THREE.BufferAttribute(new Uint32Array(3), 1));
  return g;
}

/** Copies the simulation's current ribbon streams into `geometry` (reallocating when the capacity grew). */
export function uploadRibbon(geometry: THREE.BufferGeometry, sim: EngineTrailSim) {
  const b = sim.buffers;
  const pos = geometry.getAttribute("position") as THREE.BufferAttribute;
  if (pos.array.length !== b.positions.length) {
    geometry.setAttribute("position", new THREE.BufferAttribute(b.positions, 3).setUsage(THREE.DynamicDrawUsage));
    geometry.setAttribute("aUV0", new THREE.BufferAttribute(b.uv0, 2).setUsage(THREE.DynamicDrawUsage));
    geometry.setAttribute("aUV1", new THREE.BufferAttribute(b.uv1, 2).setUsage(THREE.DynamicDrawUsage));
    geometry.setAttribute("aFade", new THREE.BufferAttribute(b.fade, 1).setUsage(THREE.DynamicDrawUsage));
    geometry.setIndex(new THREE.BufferAttribute(b.indices, 1).setUsage(THREE.DynamicDrawUsage));
  } else {
    (geometry.getAttribute("position") as THREE.BufferAttribute).needsUpdate = true;
    (geometry.getAttribute("aUV0") as THREE.BufferAttribute).needsUpdate = true;
    (geometry.getAttribute("aUV1") as THREE.BufferAttribute).needsUpdate = true;
    (geometry.getAttribute("aFade") as THREE.BufferAttribute).needsUpdate = true;
    if (geometry.index) geometry.index.needsUpdate = true;
  }
  geometry.setDrawRange(0, b.indexCount);
}
