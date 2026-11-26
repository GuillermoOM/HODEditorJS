import * as THREE from "three";

// Engine shape point handles. Each point of an ETSH shape is an invisible Object3D `shapepoint:<i>` (group-local position =
// the stored point) so it can be picked and dragged with the transform gizmo; the visible outline/markers/circle are rebuilt
// from the handles while dragging.
export const SHAPE_POINT_PREFIX = "shapepoint:";
export const shapeHandleIndex = (o: THREE.Object3D) => parseInt(o.name.slice(SHAPE_POINT_PREFIX.length), 10);

export function refreshShapeGroupGeometry(group: THREE.Object3D) {
  const handles = group.children.filter((c) => c.name.startsWith(SHAPE_POINT_PREFIX)).sort((a, b) => shapeHandleIndex(a) - shapeHandleIndex(b));
  const flat = new Float32Array(handles.length * 3);
  let minD = Infinity;
  handles.forEach((h, i) => {
    flat[i * 3] = h.position.x;
    flat[i * 3 + 1] = h.position.y;
    flat[i * 3 + 2] = h.position.z;
    minD = Math.min(minD, Math.hypot(h.position.x, h.position.y));
  });
  for (const name of ["shape-outline", "shape-points"]) {
    const obj = group.getObjectByName(name) as THREE.Line | THREE.Points | undefined;
    const attr = obj?.geometry.getAttribute("position") as THREE.BufferAttribute | undefined;
    if (obj && attr && attr.array.length === flat.length) {
      (attr.array as Float32Array).set(flat);
      attr.needsUpdate = true;
      obj.geometry.computeBoundingSphere();
    }
  }
  const ring = group.getObjectByName("shape-circle");
  if (ring) {
    ring.scale.set(minD, minD, 1);
    ring.visible = Number.isFinite(minD) && minD > 1e-4;
  }
  const marker = group.getObjectByName("shape-selected");
  const sel = group.userData.selectedPoint as number | null | undefined;
  if (marker) {
    const h = sel != null ? handles.find((c) => shapeHandleIndex(c) === sel) : undefined;
    marker.visible = !!h;
    if (h) marker.position.copy(h.position);
  }
}

/** Marks point `index` (or none) of an engine shape group as selected and returns its handle. */
export function setShapePointHighlight(group: THREE.Object3D, index: number | null): THREE.Object3D | undefined {
  group.userData.selectedPoint = index;
  refreshShapeGroupGeometry(group);
  return index === null ? undefined : group.children.find((c) => c.name === `${SHAPE_POINT_PREFIX}${index}`);
}
