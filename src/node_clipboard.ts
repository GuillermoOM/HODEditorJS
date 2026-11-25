import type { HODModel } from "./components/Viewport";
import { getSlaveBases, isJointInWeaponFamily, renameWeaponFamilyName } from "./weapon_utils";

/**
 * Joint-subtree clipboard. Pure model-in / model-out helpers; the caller is
 * responsible for committing the result (so history/undo sees one step).
 */
export interface JointSubtreeClip {
  /** First root; used for status messages and single-joint selection after paste. */
  rootName: string;
  /** All subtree roots (several for a weapon assembly: main chain + Slave chains). */
  rootNames: string[];
  /** Set when the clip is a whole assembly (weapon/hardpoint/latch point family). */
  assembly?: { base: string; familyBases: string[]; type: string };
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
export function collectSubtreeNames(model: HODModel, rootNameOrNames: string | string[]): Set<string> {
  const children = new Map<string, string[]>();
  for (const j of model.joints) {
    const p = lc(j.parent_name);
    if (!children.has(p)) children.set(p, []);
    children.get(p)!.push(j.name);
  }
  const out = new Set<string>();
  const stack = Array.isArray(rootNameOrNames) ? [...rootNameOrNames] : [rootNameOrNames];
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
  return copyJoints(model, [rootName]);
}

/**
 * Copies a whole assembly (e.g. `Weapon_Gun0` plus its `Slave` chains): every joint
 * in the family, their descendants, and attached data. `isJointInWeaponFamily` decides
 * membership, so latch-point `Heading/Left/Up` joints are included.
 */
export function copyAssembly(model: HODModel, base: string, type: string): JointSubtreeClip | null {
  const familyBases = [base, ...getSlaveBases(model, base)];
  const members = model.joints.filter((j) => isJointInWeaponFamily(j.name, familyBases));
  if (members.length === 0) return null;
  // Roots = members whose parent is not itself a member.
  const memberSet = new Set(members.map((j) => lc(j.name)));
  const roots = members.filter((j) => !j.parent_name || !memberSet.has(lc(j.parent_name))).map((j) => j.name);
  const clip = copyJoints(model, roots);
  if (!clip) return null;
  clip.assembly = { base, familyBases, type };
  return clip;
}

function copyJoints(model: HODModel, rootNames: string[]): JointSubtreeClip | null {
  const names = collectSubtreeNames(model, rootNames);
  const inSet = (n: string | undefined) => names.has(lc(n));
  return clone({
    rootName: rootNames[0],
    rootNames,
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
export function removeSubtree(model: HODModel, rootNameOrNames: string | string[]): HODModel {
  const names = collectSubtreeNames(model, rootNameOrNames);
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
): { model: HODModel; newRootName: string; newAssemblyBase?: string } {
  const used = usedNames(model);

  // Assemblies get a fresh numbered base (Weapon_Gun0 -> Weapon_Gun1); members keep their suffixes.
  let newAssemblyBase: string | undefined;
  let pre = (n: string) => n;
  if (clip.assembly) {
    const { base, familyBases } = clip.assembly;
    newAssemblyBase = nextAssemblyBase(model, base);
    pre = (n) => renameWeaponFamilyName(n, familyBases, base, newAssemblyBase!);
  }

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

  // Smallest N where every colliding (post-mapping) name has a free `_N` variant.
  let n = 0;
  const mapped = [...clipNames].map(pre);
  if (mapped.some((c) => used.has(lc(c)))) {
    n = 1;
    while (mapped.some((c) => used.has(lc(c)) && used.has(lc(`${c}_${n}`)))) n++;
  }
  const rename = (name: string) => {
    const m = pre(name);
    return used.has(lc(m)) ? `${m}_${n}` : m;
  };

  const jointMap = new Map<string, string>();
  clip.joints.forEach((j) => jointMap.set(lc(j.name), rename(j.name)));
  const mapJoint = (name: string) => jointMap.get(lc(name)) ?? name;
  const newRootName = mapJoint(clip.rootName);
  const rootSet = new Set((clip.rootNames ?? [clip.rootName]).map(lc));
  const fresh = clone(clip);

  return {
    newRootName,
    newAssemblyBase,
    model: {
      ...model,
      joints: [
        ...model.joints,
        ...fresh.joints.map((j) => ({
          ...j,
          name: mapJoint(j.name),
          parent_name: rootSet.has(lc(j.name)) ? parentName : mapJoint(j.parent_name ?? parentName),
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

/** Next free numbered base for a pasted assembly: `Weapon_Gun0` -> `Weapon_Gun<max+1>`; `Foo` -> `Foo1`. */
export function nextAssemblyBase(model: HODModel, base: string): string {
  const taken = (b: string) =>
    model.joints.some((j) => lc(j.name).startsWith(lc(b)) && /^(?:$|_|slave|heading$|left$|up$)/i.test(j.name.slice(b.length)));
  if (!taken(base)) return base; // e.g. after Cut: pasting restores the original name (a move)
  const m = base.match(/^(.*?)(\d*)$/)!;
  const prefix = m[1];
  const prefixLc = lc(prefix);
  let max = m[2] ? parseInt(m[2], 10) : 0;
  for (const j of model.joints) {
    const jm = j.name.match(/^(.*?)(\d+)(?=$|_|Slave|Heading$|Left$|Up$)/i);
    if (jm && lc(jm[1]) === prefixLc) max = Math.max(max, parseInt(jm[2], 10));
  }
  let candidate = `${prefix}${max + 1}`;
  while (taken(candidate)) {
    max++;
    candidate = `${prefix}${max + 1}`;
  }
  return candidate;
}
