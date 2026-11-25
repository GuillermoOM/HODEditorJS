import type { HODModel } from "./components/Viewport";

/**
 * Joint-subtree clipboard. Pure model-in / model-out helpers; the caller is
 * responsible for committing the result (so history/undo sees one step).
 */
export interface JointSubtreeClip {
  rootName: string;
  joints: HODModel["joints"];
  meshes: HODModel["meshes"];
  markers: HODModel["markers"];
  nav_lights: HODModel["nav_lights"];
  engine_burns: HODModel["engine_burns"];
  engine_glows: HODModel["engine_glows"];
  engine_shapes: HODModel["engine_shapes"];
  dockpaths: HODModel["dockpaths"];
}

/** JSON deep clone: the model is plain JSON, and older WebKitGTK builds lack structuredClone. */
const clone = <T>(v: T): T => JSON.parse(JSON.stringify(v));

const lc = (s: string | undefined) => (s ?? "").toLowerCase();

/** Lower-cased names of `rootName` and every descendant joint. */
export function collectSubtreeNames(model: HODModel, rootName: string): Set<string> {
  const children = new Map<string, string[]>();
  for (const j of model.joints) {
    const p = lc(j.parent_name);
    if (!children.has(p)) children.set(p, []);
    children.get(p)!.push(j.name);
  }
  const out = new Set<string>();
  const stack = [rootName];
  while (stack.length) {
    const n = stack.pop()!;
    if (out.has(lc(n))) continue;
    out.add(lc(n));
    for (const c of children.get(lc(n)) ?? []) stack.push(c);
  }
  return out;
}

/** Deep-copies a joint, its descendants and everything attached to them. */
export function copySubtree(model: HODModel, rootName: string): JointSubtreeClip | null {
  const root = model.joints.find((j) => j.name === rootName);
  if (!root || lc(rootName) === "root") return null;
  const names = collectSubtreeNames(model, rootName);
  const inSet = (n: string | undefined) => names.has(lc(n));
  return clone({
    rootName,
    joints: model.joints.filter((j) => inSet(j.name)),
    meshes: model.meshes.filter((m) => inSet(m.parent_name)),
    markers: model.markers.filter((m) => inSet(m.parent_joint)),
    nav_lights: model.nav_lights.filter((n) => inSet(n.name)),
    engine_burns: model.engine_burns.filter((b) => inSet(b.parent_name)),
    engine_glows: model.engine_glows.filter((g) => inSet(g.parent_name)),
    engine_shapes: model.engine_shapes.filter((s) => inSet(s.parent_name)),
    dockpaths: model.dockpaths.filter((d) => inSet(d.parent_name)),
  });
}

/** Removes a joint subtree and all data attached to it (used by Cut). */
export function removeSubtree(model: HODModel, rootName: string): HODModel {
  const names = collectSubtreeNames(model, rootName);
  const inSet = (n: string | undefined) => names.has(lc(n));
  return {
    ...model,
    joints: model.joints.filter((j) => !inSet(j.name)),
    meshes: model.meshes.filter((m) => !inSet(m.parent_name)),
    markers: model.markers.filter((m) => !inSet(m.parent_joint)),
    nav_lights: model.nav_lights.filter((n) => !inSet(n.name)),
    engine_burns: model.engine_burns.filter((b) => !inSet(b.parent_name)),
    engine_glows: model.engine_glows.filter((g) => !inSet(g.parent_name)),
    engine_shapes: model.engine_shapes.filter((s) => !inSet(s.parent_name)),
    dockpaths: model.dockpaths.filter((d) => !inSet(d.parent_name)),
  };
}

function usedNames(model: HODModel): Set<string> {
  const s = new Set<string>();
  const add = (n?: string) => n && s.add(lc(n));
  model.joints.forEach((x) => add(x.name));
  model.meshes.forEach((x) => add(x.name));
  model.markers.forEach((x) => add(x.name));
  model.nav_lights.forEach((x) => add(x.name));
  model.engine_burns.forEach((x) => add(x.name));
  model.engine_glows.forEach((x) => add(x.name));
  model.engine_shapes.forEach((x) => add(x.name));
  model.dockpaths.forEach((x) => add(x.name));
  return s;
}

/**
 * Pastes a clip under `parentName`. Names that already exist in the model get
 * a `_N` suffix (same N for the whole paste); unused names are kept as-is, so
 * cut + paste is a pure move. Local transforms are preserved.
 */
export function pasteSubtree(
  model: HODModel,
  clip: JointSubtreeClip,
  parentName: string,
): { model: HODModel; newRootName: string } {
  const used = usedNames(model);
  const clipNames = new Set<string>();
  const addC = (n?: string) => n && clipNames.add(n);
  clip.joints.forEach((x) => addC(x.name));
  clip.meshes.forEach((x) => addC(x.name));
  clip.markers.forEach((x) => addC(x.name));
  clip.nav_lights.forEach((x) => addC(x.name));
  clip.engine_burns.forEach((x) => addC(x.name));
  clip.engine_glows.forEach((x) => addC(x.name));
  clip.engine_shapes.forEach((x) => addC(x.name));
  clip.dockpaths.forEach((x) => addC(x.name));

  // Smallest N where every colliding name has a free `_N` variant.
  let n = 0;
  const conflicts = [...clipNames].some((c) => used.has(lc(c)));
  if (conflicts) {
    n = 1;
    while ([...clipNames].some((c) => used.has(lc(c)) && used.has(lc(`${c}_${n}`)))) n++;
  }
  const rename = (name: string) => (used.has(lc(name)) ? `${name}_${n}` : name);

  const jointMap = new Map<string, string>();
  clip.joints.forEach((j) => jointMap.set(lc(j.name), rename(j.name)));
  const mapJoint = (name: string) => jointMap.get(lc(name)) ?? name;
  const newRootName = mapJoint(clip.rootName);
  const fresh = clone(clip);

  return {
    newRootName,
    model: {
      ...model,
      joints: [
        ...model.joints,
        ...fresh.joints.map((j) => ({
          ...j,
          name: mapJoint(j.name),
          parent_name: lc(j.name) === lc(clip.rootName) ? parentName : mapJoint(j.parent_name ?? parentName),
        })),
      ],
      meshes: [...model.meshes, ...fresh.meshes.map((m) => ({ ...m, name: rename(m.name), parent_name: mapJoint(m.parent_name) }))],
      markers: [...model.markers, ...fresh.markers.map((m) => ({ ...m, name: rename(m.name), parent_joint: mapJoint(m.parent_joint) }))],
      nav_lights: [...model.nav_lights, ...fresh.nav_lights.map((l) => ({ ...l, name: mapJoint(l.name) }))],
      engine_burns: [...model.engine_burns, ...fresh.engine_burns.map((b) => ({ ...b, name: rename(b.name), parent_name: mapJoint(b.parent_name) }))],
      engine_glows: [
        ...model.engine_glows,
        ...fresh.engine_glows.map((g) => ({
          ...g,
          name: rename(g.name),
          parent_name: mapJoint(g.parent_name),
          mesh: { ...g.mesh, name: rename(g.mesh.name), parent_name: mapJoint(g.mesh.parent_name) },
        })),
      ],
      engine_shapes: [
        ...model.engine_shapes,
        ...fresh.engine_shapes.map((s) => ({
          ...s,
          name: rename(s.name),
          parent_name: mapJoint(s.parent_name),
          mesh: { ...s.mesh, name: rename(s.mesh.name), parent_name: mapJoint(s.mesh.parent_name) },
        })),
      ],
      dockpaths: [...model.dockpaths, ...fresh.dockpaths.map((d) => ({ ...d, name: rename(d.name), parent_name: mapJoint(d.parent_name) }))],
    },
  };
}
