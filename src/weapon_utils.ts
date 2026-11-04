import type { HODModel, HODJoint } from "./components/Viewport";

/**
 * Returns every `${baseName}Slave`, `${baseName}Slave1`, ... assembly base that
 * exists in the model. Vanilla HWRM multi-barrel weapons use these parallel
 * chains instead of extra `_MuzzleN` joints.
 */
export function getSlaveBases(model: HODModel, baseName: string): string[] {
  const prefix = baseName.toLowerCase();
  const found = new Map<string, string>();

  for (const joint of model.joints) {
    const lower = joint.name.toLowerCase();
    if (!lower.startsWith(prefix)) continue;

    const rest = joint.name.substring(baseName.length);
    const match = rest.match(/^(Slave\d*)(?:_|$)/i);
    if (!match) continue;

    const slaveBase = `${baseName}${match[1]}`;
    found.set(slaveBase.toLowerCase(), slaveBase);
  }

  return [...found.values()].sort((a, b) => a.localeCompare(b));
}

/** Next free slave base: `${base}Slave`, then `${base}Slave1`, `${base}Slave2`, ... */
export function nextSlaveBase(model: HODModel, baseName: string): string {
  const existing = new Set(getSlaveBases(model, baseName).map((b) => b.toLowerCase()));
  let index = 0;
  while (true) {
    const candidate = index === 0 ? `${baseName}Slave` : `${baseName}Slave${index}`;
    if (!existing.has(candidate.toLowerCase())) return candidate;
    index++;
  }
}

/** True for `${base}` or any descendant joint `${base}_...`. */
export function isJointInWeaponBase(name: string, baseName: string): boolean {
  const lower = name.toLowerCase();
  const base = baseName.toLowerCase();
  return lower === base || lower.startsWith(`${base}_`);
}

/** True for any base in a weapon family (main + slaves). */
export function isJointInWeaponFamily(name: string, familyBases: string[]): boolean {
  return familyBases.some((base) => isJointInWeaponBase(name, base));
}

/**
 * Clone the main weapon chain into a parallel `slaveBase` chain. For turret
 * chains (those with a `_Latitude` joint) the primary muzzle is mirrored on X
 * to match the common twin-barrel layout.
 */
export function cloneWeaponFamilyJoints(
  model: HODModel,
  baseName: string,
  slaveBase: string
): HODJoint[] {
  const familyJoints = model.joints.filter((j) => isJointInWeaponBase(j.name, baseName));
  const nameMap = new Map<string, string>();
  for (const joint of familyJoints) {
    nameMap.set(joint.name.toLowerCase(), `${slaveBase}${joint.name.substring(baseName.length)}`);
  }

  const clones: HODJoint[] = familyJoints.map((joint) => ({
    ...joint,
    name: nameMap.get(joint.name.toLowerCase()) || joint.name,
    parent_name: joint.parent_name
      ? nameMap.get(joint.parent_name.toLowerCase()) || joint.parent_name
      : joint.parent_name,
    local_transform: { m: joint.local_transform.m.map((row) => [...row]) },
    position: joint.position ? { ...joint.position } : joint.position,
    rotation: joint.rotation ? { ...joint.rotation } : joint.rotation,
    scale: joint.scale ? { ...joint.scale } : joint.scale,
  }));

  const hasLatitude = model.joints.some(
    (j) => j.name.toLowerCase() === `${baseName}_Latitude`.toLowerCase()
  );
  if (hasLatitude) {
    const mainMuzzle = familyJoints.find(
      (j) => j.name.toLowerCase() === `${baseName}_Muzzle`.toLowerCase()
    );
    const slaveMuzzle = clones.find(
      (j) => j.name.toLowerCase() === `${slaveBase}_Muzzle`.toLowerCase()
    );
    if (mainMuzzle && slaveMuzzle) {
      const x = mainMuzzle.local_transform.m[3][0];
      if (Math.abs(x) > 1e-4) {
        slaveMuzzle.local_transform.m[3][0] = -x;
      }
    }
  }

  return clones;
}

/**
 * Maps a joint name from one weapon family to another. Preserves the
 * `SlaveN` suffix and the component suffix (`_Position`, `_Muzzle`, ...).
 */
export function renameWeaponFamilyName(
  name: string,
  familyBases: string[],
  oldBaseName: string,
  newBaseName: string
): string {
  const lower = name.toLowerCase();

  for (const base of familyBases) {
    const baseLower = base.toLowerCase();
    if (lower === baseLower) {
      return `${newBaseName}${base.substring(oldBaseName.length)}`;
    }
    if (lower.startsWith(`${baseLower}_`)) {
      return `${newBaseName}${base.substring(oldBaseName.length)}${name.substring(base.length)}`;
    }
  }

  return name;
}
