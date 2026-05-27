import React, { useEffect, useRef, useState } from "react";
import * as THREE from "three";
import { OrbitControls } from "three/examples/jsm/controls/OrbitControls.js";
import { TransformControls } from "three/examples/jsm/controls/TransformControls.js";
import { invoke } from "@tauri-apps/api/core";

// HOD Types matching Rust backend
export interface Vector3D {
  x: number;
  y: number;
  z: number;
}

export interface Matrix4D {
  m: number[][];
}

export interface HODVertex {
  position: Vector3D;
  normal?: Vector3D;
  color?: number;
  uv?: { u: number; v: number };
}

export interface HODMeshPart {
  material_index: number;
  vertex_mask: number;
  vertices: HODVertex[];
  indices: number[];
}

export interface HODMesh {
  name: string;
  parent_name: string;
  lod: number;
  parts: HODMeshPart[];
}

export interface HODJoint {
  name: string;
  parent_name?: string;
  local_transform: Matrix4D;
  position?: Vector3D;
  rotation?: Vector3D;
  scale?: Vector3D;
}

export interface HODMarker {
  name: string;
  parent_joint: string;
  position: Vector3D;
  rotation: Matrix4D;
}

export interface HODNavLight {
  name: string;
  section: number;
  size: number;
  phase: number;
  frequency: number;
  style: string;
  color: Vector3D;
  distance: number;
  sprite_visible: boolean;
  high_end_only: boolean;
}

export interface HODEngineBurn {
  name: string;
  parent_name: string;
  num_divisions: number;
  num_flames: number;
  vertices: Vector3D[];
}

export interface HODEngineGlow {
  name: string;
  parent_name: string;
  lod: number;
  mesh: HODMesh;
}

export interface HODEngineShape {
  name: string;
  parent_name: string;
  mesh: HODMesh;
}

export interface HODCollisionMesh {
  name: string;
  min_extents: Vector3D;
  max_extents: Vector3D;
  center: Vector3D;
  radius: number;
  mesh: HODMesh;
}

export interface HODDockpoint {
  position: Vector3D;
  rotation: Matrix4D;
  tolerance: number;
  max_speed: number;
}

export interface HODDockpath {
  name: string;
  parent_name: string;
  points: HODDockpoint[];
}

export interface HODTexture {
  name: string;
  width: number;
  height: number;
  format: string;
  png_preview?: string;
  png_data?: string;
}

export interface HODKeyframe {
  time: number;
  position?: Vector3D;
  rotation?: { x: number; y: number; z: number; w: number };
  rotation_euler?: Vector3D;
  scale?: Vector3D;
}

export interface HODAnimationTrack {
  joint_name: string;
  keyframes: HODKeyframe[];
}

export interface HODAnimation {
  name: string;
  duration: number; // in seconds
  tracks: HODAnimationTrack[];
}
export interface HODMaterial {
  name: string;
  shader_name: string;
  texture_maps: string[];
}

export interface HODModel {
  version: number;
  is_v2: boolean;
  name: string;
  textures: HODTexture[];
  materials: HODMaterial[];
  meshes: HODMesh[];
  joints: HODJoint[];
  markers: HODMarker[];
  nav_lights: HODNavLight[];
  engine_burns: HODEngineBurn[];
  engine_glows: HODEngineGlow[];
  engine_shapes: HODEngineShape[];
  collision_meshes: HODCollisionMesh[];
  dockpaths: HODDockpath[];
  animations?: HODAnimation[];
}

const isFiniteVector = (v: any): boolean => {
  return (
    v &&
    typeof v.x === "number" && Number.isFinite(v.x) &&
    typeof v.y === "number" && Number.isFinite(v.y) &&
    typeof v.z === "number" && Number.isFinite(v.z)
  );
};

interface ViewportProps {
  model: HODModel | null;
  selectedNode: { type: string; name: string } | null;
  setSelectedNode: (node: { type: string; name: string } | null) => void;
  transformMode: "translate" | "rotate" | "scale";
  setTransformMode: (mode: "translate" | "rotate" | "scale") => void;
  onNodeTransform: (name: string, type: string, position: Vector3D) => void;
  visibleMeshes: Record<string, boolean>;
  showNavLights: boolean;
  showCollision: boolean;
  showDockpaths: boolean;
  showEngineBurns: boolean;
  showBoneLines: boolean;
  renderMode: "untextured" | "textured" | "shaded" | "wireframe" | "shaded_team" | "textured_team";
  teamColor?: string;
  stripeColor?: string;
  onModelChange?: (updatedModel: HODModel) => void;
  selectedAnimIdx: number;
  setSelectedAnimIdx: (idx: number) => void;
  isPlaying: boolean;
  setIsPlaying: (v: boolean) => void;
  currentTime: number;
  setCurrentTime: (v: number) => void;
  playbackSpeed: number;
  loopPlayback: boolean;
  targetBoxes?: any[];
}

export const Viewport: React.FC<ViewportProps> = ({
  model,
  selectedNode,
  setSelectedNode: _setSelectedNode,
  transformMode,
  setTransformMode,
  onNodeTransform,
  visibleMeshes,
  showNavLights,
  showCollision,
  showDockpaths,
  showEngineBurns,
  showBoneLines,
  renderMode,
  teamColor = "#4278a3",
  stripeColor = "#e5d94c",
  onModelChange,
  selectedAnimIdx,
  setSelectedAnimIdx: _setSelectedAnimIdx,
  isPlaying,
  setIsPlaying: _setIsPlaying,
  currentTime,
  setCurrentTime,
  playbackSpeed,
  loopPlayback,
  targetBoxes,
}) => {
  const mountRef = useRef<HTMLDivElement>(null);
  const transformControlsRef = useRef<TransformControls | null>(null);
  const orbitControlsRef = useRef<OrbitControls | null>(null);
  const sceneRef = useRef<THREE.Scene | null>(null);
  const cameraRef = useRef<THREE.PerspectiveCamera | null>(null);
  const markersGroupRef = useRef<THREE.Group | null>(null);
  const jointsGroupRef = useRef<THREE.Group | null>(null);
  const meshesGroupRef = useRef<THREE.Group | null>(null);
  
  const navLightsGroupRef = useRef<THREE.Group | null>(null);
  const collisionGroupRef = useRef<THREE.Group | null>(null);
  const dockpathsGroupRef = useRef<THREE.Group | null>(null);
  const engineBurnsGroupRef = useRef<THREE.Group | null>(null);
  const engineGlowsGroupRef = useRef<THREE.Group | null>(null);
  const engineShapesGroupRef = useRef<THREE.Group | null>(null);
  const targetBoxesGroupRef = useRef<THREE.Group | null>(null);

  const clockRef = useRef(new THREE.Clock());
  const navLightsAnimateRef = useRef<{ mesh: THREE.Mesh; material: THREE.MeshBasicMaterial; frequency: number; phase: number }[]>([]);
  
  const [isInitialized, setIsInitialized] = useState(false);

  // Animation timeline player state — now driven by props from App.tsx
  // Local state only for the create-anim modal inside Viewport (legacy, now removed)

  const isPlayingRef = useRef(isPlaying);
  const currentTimeRef = useRef(currentTime);
  const selectedAnimIdxRef = useRef(selectedAnimIdx);
  const playbackSpeedRef = useRef(playbackSpeed);
  const loopPlaybackRef = useRef(loopPlayback);

  useEffect(() => { isPlayingRef.current = isPlaying; }, [isPlaying]);
  useEffect(() => { currentTimeRef.current = currentTime; }, [currentTime]);
  useEffect(() => { selectedAnimIdxRef.current = selectedAnimIdx; }, [selectedAnimIdx]);
  useEffect(() => { playbackSpeedRef.current = playbackSpeed; }, [playbackSpeed]);
  useEffect(() => { loopPlaybackRef.current = loopPlayback; }, [loopPlayback]);

  const scaleFactorRef = useRef<number>(1.0);
  const selectionAxesHelperRef = useRef<THREE.AxesHelper | null>(null);
  const lastFocusedModelRef = useRef<string | null>(null);
  const textureCacheRef = useRef<Map<string, THREE.Texture>>(new Map());

  const modelRef = useRef(model);
  const onModelChangeRef = useRef(onModelChange);
  useEffect(() => {
    modelRef.current = model;
    onModelChangeRef.current = onModelChange;
  }, [model, onModelChange]);

  const selectedNodeRef = useRef(selectedNode);
  useEffect(() => {
    selectedNodeRef.current = selectedNode;
  }, [selectedNode]);

  const onNodeTransformRef = useRef(onNodeTransform);
  useEffect(() => {
    onNodeTransformRef.current = onNodeTransform;
  }, [onNodeTransform]);

  const getAnimatedJointGlobalMatrix = (jointName: string, time: number, visited = new Set<string>()): THREE.Matrix4 => {
    const currentModel = modelRef.current;
    if (!currentModel || visited.has(jointName.toLowerCase())) {
      return new THREE.Matrix4();
    }
    const joint = currentModel.joints.find(j => j.name.toLowerCase() === jointName.toLowerCase());
    if (!joint) return new THREE.Matrix4();

    const nextVisited = new Set(visited);
    nextVisited.add(jointName.toLowerCase());

    let parentMatrix = new THREE.Matrix4();
    if (joint.parent_name && joint.parent_name.toLowerCase() !== jointName.toLowerCase()) {
      parentMatrix = getAnimatedJointGlobalMatrix(joint.parent_name, time, nextVisited);
    }

    let localMatrix = toThreeMatrix(joint.local_transform);

    const anims = currentModel?.animations || [];
    const activeAnim = anims[selectedAnimIdxRef.current];
    const isPlay = isPlayingRef.current;
    const selNode = selectedNodeRef.current;
    const isKeyframeActive = selNode?.type === "keyframe";

    // Only apply animation if playing or actively inspecting a keyframe.
    // This allows joints to reset to their natural coordinates when not animating/editing keyframes.
    if (activeAnim && (isPlay || isKeyframeActive)) {
      const track = activeAnim.tracks.find(t => t.joint_name.toLowerCase() === joint.name.toLowerCase());
      if (track && track.keyframes.length > 0) {
        const keys = track.keyframes;
        let t1 = keys[0];
        let t2 = keys[keys.length - 1];

        for (let i = 0; i < keys.length - 1; i++) {
          if (time >= keys[i].time && time <= keys[i+1].time) {
            t1 = keys[i];
            t2 = keys[i+1];
            break;
          }
        }

        let alpha = 0;
        if (t2.time !== t1.time) {
          alpha = (time - t1.time) / (t2.time - t1.time);
        }
        alpha = Math.max(0, Math.min(1, alpha));

        const basePos = new THREE.Vector3();
        const baseQuat = new THREE.Quaternion();
        const baseScale = new THREE.Vector3();
        localMatrix.decompose(basePos, baseQuat, baseScale);

        const pos = basePos.clone();
        if (t1.position && t2.position) {
          pos.set(
            THREE.MathUtils.lerp(t1.position.x, t2.position.x, alpha),
            THREE.MathUtils.lerp(t1.position.y, t2.position.y, alpha),
            THREE.MathUtils.lerp(t1.position.z, t2.position.z, alpha)
          );
        }

        const quat = baseQuat.clone();
        if (t1.rotation_euler && t2.rotation_euler) {
          const rx = THREE.MathUtils.lerp(t1.rotation_euler.x, t2.rotation_euler.x, alpha);
          const ry = THREE.MathUtils.lerp(t1.rotation_euler.y, t2.rotation_euler.y, alpha);
          const rz = THREE.MathUtils.lerp(t1.rotation_euler.z, t2.rotation_euler.z, alpha);
          const euler = new THREE.Euler(rx, ry, rz, "YXZ");
          quat.setFromEuler(euler);
        } else if (t1.rotation && t2.rotation) {
          const q1 = new THREE.Quaternion(t1.rotation.x, t1.rotation.y, t1.rotation.z, t1.rotation.w).normalize();
          const q2 = new THREE.Quaternion(t2.rotation.x, t2.rotation.y, t2.rotation.z, t2.rotation.w).normalize();
          q1.slerp(q2, alpha).normalize();
          quat.copy(q1);
        }

        const scale = baseScale.clone();
        if (t1.scale && t2.scale) {
          scale.set(
            THREE.MathUtils.lerp(t1.scale.x, t2.scale.x, alpha),
            THREE.MathUtils.lerp(t1.scale.y, t2.scale.y, alpha),
            THREE.MathUtils.lerp(t1.scale.z, t2.scale.z, alpha)
          );
        }

        localMatrix = new THREE.Matrix4().compose(pos, quat, scale);
      }
    }

    return parentMatrix.clone().multiply(localMatrix);
  };

  const evaluateAnimation = (mModel: HODModel, animIdx: number, time: number) => {
    const anims = mModel.animations || [];
    let activeAnim = anims[animIdx];

    if (!activeAnim) {
      return 4.0;
    }

    if (jointsGroupRef.current) {
      mModel.joints.forEach(joint => {
        const jointMesh = jointsGroupRef.current?.children.find(
          c => c.name.toLowerCase() === `joint:${joint.name}`.toLowerCase()
        ) as THREE.Mesh | undefined;
        if (jointMesh) {
          const animatedWorldMatrix = getAnimatedJointGlobalMatrix(joint.name, time);
          const pos = new THREE.Vector3();
          const quat = new THREE.Quaternion();
          const scale = new THREE.Vector3();
          animatedWorldMatrix.decompose(pos, quat, scale);
          jointMesh.position.copy(pos);
          jointMesh.quaternion.copy(quat);

          const activeNode = selectedNodeRef.current;
          const isSelected = activeNode && activeNode.type === "joint" && activeNode.name.toLowerCase() === joint.name.toLowerCase();
          if (isSelected) {
            jointMesh.scale.setScalar(1.8);
            (jointMesh.material as THREE.MeshBasicMaterial).color.set("#ffd600");
          } else {
            jointMesh.scale.setScalar(1.0);
            (jointMesh.material as THREE.MeshBasicMaterial).color.set("#00e676");
          }
        }
      });

      jointsGroupRef.current.children.forEach(child => {
        if (child.name.startsWith("bone:")) {
          const boneLine = child as THREE.Line;
          const parentName = boneLine.userData.parentJointName as string | undefined;
          const childName = boneLine.userData.childJointName as string | undefined;
          if (parentName && childName) {
            const parentMesh = jointsGroupRef.current?.children.find(
              c => c.name.toLowerCase() === `joint:${parentName}`.toLowerCase()
            ) as THREE.Mesh | undefined;
            const childMesh = jointsGroupRef.current?.children.find(
              c => c.name.toLowerCase() === `joint:${childName}`.toLowerCase()
            ) as THREE.Mesh | undefined;
            if (parentMesh && childMesh) {
              const positions = boneLine.geometry.attributes.position.array as Float32Array;
              positions[0] = childMesh.position.x;
              positions[1] = childMesh.position.y;
              positions[2] = childMesh.position.z;
              positions[3] = parentMesh.position.x;
              positions[4] = parentMesh.position.y;
              positions[5] = parentMesh.position.z;
              boneLine.geometry.attributes.position.needsUpdate = true;
            }
          }
        }
      });
    }

    if (meshesGroupRef.current) {
      meshesGroupRef.current.children.forEach(child => {
        const parentJointName = child.userData.parentJointName as string | undefined;
        if (!parentJointName) return;

        const animatedWorldMatrix = getAnimatedJointGlobalMatrix(parentJointName, time);
        child.matrix.copy(animatedWorldMatrix);
        child.matrix.decompose(child.position, child.quaternion, child.scale);
        child.matrixWorldNeedsUpdate = true;
      });
    }

    const updateGroupChildren = (group: THREE.Group | null) => {
      if (!group) return;
      group.children.forEach(child => {
        const parentJointName = child.userData.parentJointName as string | undefined;
        const baseMatrix = child.userData.baseMatrix as THREE.Matrix4 | undefined;
        if (!parentJointName || !baseMatrix) return;

        const animatedWorldMatrix = getAnimatedJointGlobalMatrix(parentJointName, time);
        const newMatrix = animatedWorldMatrix.clone().multiply(baseMatrix);
        child.matrix.copy(newMatrix);
        child.matrix.decompose(child.position, child.quaternion, child.scale);
        child.matrixWorldNeedsUpdate = true;
      });
    };

    updateGroupChildren(markersGroupRef.current);
    updateGroupChildren(navLightsGroupRef.current);
    updateGroupChildren(dockpathsGroupRef.current);
    updateGroupChildren(engineBurnsGroupRef.current);
    updateGroupChildren(engineGlowsGroupRef.current);
    updateGroupChildren(engineShapesGroupRef.current);
    updateGroupChildren(collisionGroupRef.current);

    if (dockpathsGroupRef.current) {
      dockpathsGroupRef.current.children.forEach(child => {
        if (child.name.startsWith("dockpath_line:")) {
          const pathLine = child as THREE.Line;
          const pathName = pathLine.userData.pathName as string | undefined;
          const pointCount = pathLine.userData.pointCount as number | undefined;
          if (pathName && pointCount) {
            const pointsInWorld: THREE.Vector3[] = [];
            for (let i = 0; i < pointCount; i++) {
              const ptMesh = dockpathsGroupRef.current?.children.find(
                c => c.name === `dockpoint:${pathName}:${i}`
              ) as THREE.Mesh | undefined;
              if (ptMesh) {
                pointsInWorld.push(ptMesh.position);
              }
            }
            if (pointsInWorld.length > 1) {
              const positions = pathLine.geometry.attributes.position.array as Float32Array;
              pointsInWorld.forEach((pt, idx) => {
                positions[idx * 3] = pt.x;
                positions[idx * 3 + 1] = pt.y;
                positions[idx * 3 + 2] = pt.z;
              });
              pathLine.geometry.attributes.position.needsUpdate = true;
            }
          }
        }
      });
    }

    return activeAnim.duration;
  };

  // Helper to convert Matrix4D to THREE.Matrix4 with singular scale sanitization
  const toThreeMatrix = (matrix: Matrix4D): THREE.Matrix4 => {
    const m = matrix?.m;
    if (!m || m.length < 4 || m.some(row => !row || row.length < 4)) {
      invoke("log_event", { level: "ERROR", message: `toThreeMatrix check FAILED. matrix: ${JSON.stringify(matrix)}` }).catch(console.error);
      return new THREE.Matrix4();
    }

    // Check if the scale/basis vectors are zero (singular matrix), and restore them to identity if so
    let s0 = Math.sqrt(m[0][0]*m[0][0] + m[0][1]*m[0][1] + m[0][2]*m[0][2]);
    let s1 = Math.sqrt(m[1][0]*m[1][0] + m[1][1]*m[1][1] + m[1][2]*m[1][2]);
    let s2 = Math.sqrt(m[2][0]*m[2][0] + m[2][1]*m[2][1] + m[2][2]*m[2][2]);

    let r00 = m[0][0], r01 = m[0][1], r02 = m[0][2];
    let r10 = m[1][0], r11 = m[1][1], r12 = m[1][2];
    let r20 = m[2][0], r21 = m[2][1], r22 = m[2][2];

    if (s0 < 0.0001) { r00 = 1.0; r01 = 0.0; r02 = 0.0; }
    if (s1 < 0.0001) { r10 = 0.0; r11 = 1.0; r12 = 0.0; }
    if (s2 < 0.0001) { r20 = 0.0; r21 = 0.0; r22 = 1.0; }

    return new THREE.Matrix4().set(
      r00, r10, r20, m[3][0],
      r01, r11, r21, m[3][1],
      r02, r12, r22, m[3][2],
      m[0][3], m[1][3], m[2][3], m[3][3]
    );
  };

  // Helper to decompose matrices safely with NaN/Infinity recovery
  const safeDecompose = (matrix: THREE.Matrix4, pos: THREE.Vector3, q: THREE.Quaternion, s: THREE.Vector3) => {
    matrix.decompose(pos, q, s);
    if (!Number.isFinite(pos.x) || !Number.isFinite(pos.y) || !Number.isFinite(pos.z)) {
      const te = matrix.elements;
      pos.set(te[12], te[13], te[14]);
    }
    if (!Number.isFinite(q.x) || !Number.isFinite(q.y) || !Number.isFinite(q.z) || !Number.isFinite(q.w)) {
      q.set(0, 0, 0, 1);
    }
    if (!Number.isFinite(s.x) || !Number.isFinite(s.y) || !Number.isFinite(s.z)) {
      s.set(1, 1, 1);
    }
  };

  // Helper to compute joint global transform matrix recursively (with stack-overflow protection!)
  const getJointGlobalMatrix = (jointName: string, visited = new Set<string>()): THREE.Matrix4 => {
    const currentModel = modelRef.current;
    if (!currentModel || visited.has(jointName.toLowerCase())) {
      return new THREE.Matrix4();
    }

    const joint = currentModel.joints.find(j => j.name.toLowerCase() === jointName.toLowerCase());
    if (!joint) return new THREE.Matrix4();

    const localMatrix = toThreeMatrix(joint.local_transform);

    if (joint.name.toLowerCase().includes("nozzle") || joint.name.toLowerCase().includes("engine")) {
      invoke("log_event", {
        level: "INFO",
        message: `Joint ${joint.name} local translation: [${joint.local_transform.m[3][0].toFixed(3)}, ${joint.local_transform.m[3][1].toFixed(3)}, ${joint.local_transform.m[3][2].toFixed(3)}], parent_name: ${joint.parent_name}`
      }).catch(console.error);
    }

    if (joint.parent_name && joint.parent_name.toLowerCase() !== jointName.toLowerCase()) {
      const nextVisited = new Set(visited);
      nextVisited.add(jointName.toLowerCase());
      const parentGlobalMatrix = getJointGlobalMatrix(joint.parent_name, nextVisited);
      return parentGlobalMatrix.clone().multiply(localMatrix);
    } else {
      return localMatrix;
    }
  };

  // Helper to resolve helper world position to local position of parent joint
  const getLocalPosition = (obj: THREE.Object3D, selectedType: string, selectedName: string): Vector3D => {
    const currentModel = modelRef.current;
    let parentName = "";
    if (selectedType === "marker") {
      const marker = currentModel?.markers.find(m => m.name.toLowerCase() === selectedName.toLowerCase());
      parentName = marker?.parent_joint || "";
    } else if (selectedType === "joint") {
      const joint = currentModel?.joints.find(j => j.name.toLowerCase() === selectedName.toLowerCase());
      parentName = joint?.parent_name || "";
    } else if (selectedType === "navlight") {
      const joint = currentModel?.joints.find(j => j.name.toLowerCase() === selectedName.toLowerCase());
      parentName = joint?.parent_name || "";
    } else if (selectedType === "weapon_group") {
      const joint = currentModel?.joints.find(j => j.name.toLowerCase() === `${selectedName}_Position`.toLowerCase());
      parentName = joint?.parent_name || "";
    } else if (selectedType === "dockpoint") {
      const [pathName] = selectedName.split(":");
      const path = currentModel?.dockpaths.find(p => p.name.toLowerCase() === pathName.toLowerCase());
      parentName = path?.parent_name || "";
    } else if (selectedType === "collision") {
      parentName = selectedName;
    } else if (selectedType === "engine_burn") {
      const burn = currentModel?.engine_burns.find(b => b.name.toLowerCase() === selectedName.toLowerCase());
      parentName = burn?.parent_name || "";
    } else if (selectedType === "engine_glow") {
      const glow = currentModel?.engine_glows.find(g => g.name.toLowerCase() === selectedName.toLowerCase());
      parentName = glow?.parent_name || "";
    } else if (selectedType === "engine_shape") {
      const shape = currentModel?.engine_shapes.find(s => s.name.toLowerCase() === selectedName.toLowerCase());
      parentName = shape?.parent_name || "";
    }

    if (parentName && parentName.toLowerCase() !== "root") {
      const parentMatrix = getJointGlobalMatrix(parentName);
      const invParent = parentMatrix.clone().invert();
      const localPos = obj.position.clone().applyMatrix4(invParent);
      return { x: localPos.x, y: localPos.y, z: localPos.z };
    }

    return { x: obj.position.x, y: obj.position.y, z: obj.position.z };
  };

  useEffect(() => {
    if (!mountRef.current) return;

    try {
      // 1. Initialize Scene, Camera, WebGLRenderer
      const width = mountRef.current.clientWidth || 800;
      const height = mountRef.current.clientHeight || 600;

      // Clean out any stale, duplicated, or frozen canvas elements
      mountRef.current.innerHTML = "";

      const scene = new THREE.Scene();
      scene.background = new THREE.Color("#03060a");
      sceneRef.current = scene;

      const camera = new THREE.PerspectiveCamera(50, width / height, 0.05, 100000);
      camera.position.set(20, 15, 30);
      cameraRef.current = camera;

      const renderer = new THREE.WebGLRenderer({ antialias: true });
      renderer.setSize(width, height);
      renderer.setPixelRatio(window.devicePixelRatio);
      mountRef.current.appendChild(renderer.domElement);

      const handlePrevent = (e: Event) => e.preventDefault();
      const container = mountRef.current;
      if (container) {
        container.addEventListener("dragstart", handlePrevent);
        container.addEventListener("selectstart", handlePrevent);
      }
      renderer.domElement.addEventListener("dragstart", handlePrevent);
      renderer.domElement.addEventListener("selectstart", handlePrevent);

      // 2. Add Cinematic Stellar Space Lighting (Distant Sun + Nebula Bounce/Fill)
      // Deep Space Ambient Fill
      const ambientLight = new THREE.AmbientLight("#080c14", 0.18);
      scene.add(ambientLight);

      // The Distant Sun (Bright, crisp neutral white stellar light)
      const sunLight = new THREE.DirectionalLight("#ffffff", 1.6);
      sunLight.position.set(150, 80, 100);
      scene.add(sunLight);

      // Cool Nebula Glow (Opposite fill light to softly illuminate shadowed areas)
      const nebulaFill = new THREE.DirectionalLight("#355c7d", 0.7);
      nebulaFill.position.set(-150, -40, -100);
      scene.add(nebulaFill);

      // Warm Dust Reflection (Bottom-up bounce light simulating stellar dust reflections)
      const dustBounce = new THREE.DirectionalLight("#ffd1a9", 0.3);
      dustBounce.position.set(0, -100, 0);
      scene.add(dustBounce);

      // 3. Add Grid and Axis Helpers (Adaptive Infinite Grids with Gray Slate lines)
      const gridFine = new THREE.GridHelper(200, 200, "#16a0ff", "#384e6e");
      // Safely duplicate the material properties without breaking the internal shader
      const fineMat = new THREE.LineBasicMaterial({
        color: 0xffffff,
        vertexColors: true,
        transparent: true,
        opacity: 0.45,
        depthTest: true
      });
      gridFine.material = fineMat;
      gridFine.renderOrder = 0;
      gridFine.position.y = 0;
      scene.add(gridFine);

      const gridCoarse = new THREE.GridHelper(2000, 200, "#16a0ff", "#25344a");
      const coarseMat = new THREE.LineBasicMaterial({
        vertexColors: true,
        transparent: true,
        opacity: 0.5,
        depthTest: true
      });
      gridCoarse.material = coarseMat;
      gridCoarse.renderOrder = 0;
      gridCoarse.position.y = 0;
      scene.add(gridCoarse);

      const gridMega = new THREE.GridHelper(20000, 200, "#16a0ff", "#182333");
      const megaMat = new THREE.LineBasicMaterial({
        vertexColors: true,
        transparent: true,
        opacity: 0.0,
        depthTest: true
      });
      gridMega.material = megaMat;
      gridMega.renderOrder = 0;
      gridMega.position.y = 0;
      scene.add(gridMega);

      const axes = new THREE.AxesHelper(5);
      scene.add(axes);

      // 4. Set up Camera Orbit Controls with Right-Click Rotation!
      const orbitControls = new OrbitControls(camera, renderer.domElement);
      orbitControls.enableDamping = false;
      orbitControls.mouseButtons = {
        LEFT: THREE.MOUSE.ROTATE,
        MIDDLE: THREE.MOUSE.DOLLY,
        RIGHT: THREE.MOUSE.ROTATE, // Allow right click to rotate camera around the origin!
      };
      orbitControlsRef.current = orbitControls;

      // 5. Set up Transform Controls for Visually Moving Joints/Markers
      const transformControls = new TransformControls(camera, renderer.domElement);
      transformControlsRef.current = transformControls;
      scene.add(transformControls as unknown as THREE.Object3D);

      // Prevent orbit rotation while transforming nodes
      transformControls.addEventListener("dragging-changed", (event) => {
        orbitControls.enabled = !event.value;
      });

      // Notify parent on node movement
      transformControls.addEventListener("objectChange", () => {
        const activeNode = selectedNodeRef.current;
        if (transformControls.object && activeNode) {
          const obj = transformControls.object;
          const localPos = getLocalPosition(obj, activeNode.type, activeNode.name);
          if (onNodeTransformRef.current) {
            onNodeTransformRef.current(activeNode.name, activeNode.type, localPos);
          }
        }
      });

      const meshesGroup = new THREE.Group();
      const jointsGroup = new THREE.Group();
      const markersGroup = new THREE.Group();
      const navLightsGroup = new THREE.Group();
      const collisionGroup = new THREE.Group();
      const dockpathsGroup = new THREE.Group();
      const engineBurnsGroup = new THREE.Group();
      const engineGlowsGroup = new THREE.Group();
      const engineShapesGroup = new THREE.Group();
      const targetBoxesGroup = new THREE.Group();
      
      scene.add(meshesGroup);
      scene.add(jointsGroup);
      scene.add(markersGroup);
      scene.add(navLightsGroup);
      scene.add(collisionGroup);
      scene.add(dockpathsGroup);
      scene.add(engineBurnsGroup);
      scene.add(engineGlowsGroup);
      scene.add(engineShapesGroup);
      scene.add(targetBoxesGroup);

      meshesGroupRef.current = meshesGroup;
      jointsGroupRef.current = jointsGroup;
      markersGroupRef.current = markersGroup;
      navLightsGroupRef.current = navLightsGroup;
      collisionGroupRef.current = collisionGroup;
      dockpathsGroupRef.current = dockpathsGroup;
      engineBurnsGroupRef.current = engineBurnsGroup;
      engineGlowsGroupRef.current = engineGlowsGroup;
      engineShapesGroupRef.current = engineShapesGroup;
      targetBoxesGroupRef.current = targetBoxesGroup;

      // 7. Render Loop with NavLight Pulsing Animate Update
      const clock = clockRef.current;
      const playbackClock = new THREE.Clock();
      let animationId: number;
      const animate = () => {
        animationId = requestAnimationFrame(animate);
        orbitControls.update();

        // Adaptive grid fading based on camera distance (zoom)
        if (camera && fineMat && coarseMat && megaMat) {
          const dist = camera.position.distanceTo(orbitControls.target);
          
          // 1. Fine Grid Logic
          if (dist < 40) {
            fineMat.opacity = 0.45;
          } else if (dist < 150) {
            // Smoothly fades from 0.45 down to 0
            fineMat.opacity = 0.45 * (1 - (dist - 40) / 110);
          } else {
            fineMat.opacity = 0;
          }

          // 2. Coarse Grid Logic
          if (dist < 5) {
            coarseMat.opacity = 0; // Fully invisible when microscopic
          } else if (dist < 40) {
            // Fade in from 0 to 0.5 as we pull away from microscopic view
            coarseMat.opacity = 0.5 * ((dist - 5) / 35);
          } else if (dist < 400) {
            coarseMat.opacity = 0.5; // Solid holding zone
          } else if (dist < 1500) {
            // Smoothly fades out to the mega grid
            coarseMat.opacity = 0.5 * (1 - (dist - 400) / 1100);
          } else {
            coarseMat.opacity = 0;
          }

          // 3. Mega Grid Logic
          if (dist < 300) {
            megaMat.opacity = 0;
          } else if (dist < 1200) {
            megaMat.opacity = 0.6 * ((dist - 300) / 900);
          } else {
            megaMat.opacity = 0.6;
          }
        }

        // Pulsing NavLights
        const time = clock.getElapsedTime();
        navLightsAnimateRef.current.forEach(({ mesh, material, frequency, phase }) => {
          if (frequency > 0) {
            const pulse = Math.sin(time * frequency * Math.PI * 2 + phase) * 0.4 + 0.6;
            material.opacity = pulse * 0.8;
          } else {
            material.opacity = 0.8;
          }
          mesh.scale.setScalar(1.0);
        });

        // Evaluate LERP/SLERP keyframe tracks
        if (modelRef.current) {
          const isPlay = isPlayingRef.current;
          const speed = playbackSpeedRef.current;
          const loop = loopPlaybackRef.current;
          let currTime = currentTimeRef.current;

          if (isPlay) {
            currTime += playbackClock.getDelta() * speed;
          } else {
            playbackClock.getDelta(); // keep delta clean
          }

          const duration = evaluateAnimation(modelRef.current, selectedAnimIdxRef.current, currTime);

          if (isPlay) {
            if (currTime >= duration) {
              currTime = loop ? 0 : duration;
            }
            currentTimeRef.current = currTime;
            setCurrentTime(currTime);
          }
        }

        renderer.render(scene, camera);
      };
      animate();

      // 8. Window Resize Handler
      const handleResize = () => {
        if (!mountRef.current) return;
        const w = mountRef.current.clientWidth;
        const h = mountRef.current.clientHeight;
        camera.aspect = w / h;
        camera.updateProjectionMatrix();
        renderer.setSize(w, h);
      };
      window.addEventListener("resize", handleResize);

      setIsInitialized(true);

      const handleExportGLTFEvent = async () => {
        const curModel = modelRef.current;
        if (!meshesGroupRef.current || !curModel) return;
        
        invoke("log_event", { level: "INFO", message: "Starting GLTF model export..." }).catch(console.error);
        
        try {
          const { GLTFExporter } = await import("three/examples/jsm/exporters/GLTFExporter.js");
          const exporter = new GLTFExporter();
          
          exporter.parse(
            meshesGroupRef.current,
            async (gltf) => {
              try {
                const jsonStr = JSON.stringify(gltf, null, 2);
                const defaultName = `${curModel.name}.gltf`;
                
                const savedPath = await invoke<string | null>("save_text_file", {
                  defaultName,
                  filters: ["gltf"],
                  contents: jsonStr
                });
                
                if (savedPath) {
                  invoke("log_event", { level: "INFO", message: `HOD 3D meshes exported successfully to GLTF: ${savedPath}` }).catch(console.error);
                }
              } catch (e: any) {
                console.error(e);
                invoke("log_event", { level: "ERROR", message: `Failed to write exported GLTF file: ${e.toString()}` }).catch(console.error);
              }
            },
            (error) => {
              console.error("GLTF export error:", error);
              invoke("log_event", { level: "ERROR", message: `GLTF compilation failed: ${error.toString()}` }).catch(console.error);
            },
            {
              binary: false,
              animations: [],
              includeCustomExtensions: false
            }
          );
        } catch (e: any) {
          console.error(e);
          invoke("log_event", { level: "ERROR", message: `GLTFExporter dynamic import failed: ${e.toString()}` }).catch(console.error);
        }
      };

      const handleImportGLTFEvent = async () => {
        const curModel = modelRef.current;
        if (!curModel) return;
        try {
          const fileContent = await invoke<string | null>("load_text_file", {
            filters: ["gltf"]
          });
          if (!fileContent) return;

          invoke("log_event", { level: "INFO", message: "Parsing imported GLTF file..." }).catch(console.error);

          const { GLTFLoader } = await import("three/examples/jsm/loaders/GLTFLoader.js");
          const loader = new GLTFLoader();

          const blob = new Blob([fileContent], { type: "application/json" });
          const url = URL.createObjectURL(blob);

          loader.load(url, (gltf) => {
            URL.revokeObjectURL(url);
            
            const newMeshes: HODMesh[] = [];
            gltf.scene.traverse((child) => {
              if ((child as any).isMesh) {
                const mesh = child as THREE.Mesh;
                const geo = mesh.geometry;
                
                if (geo) {
                  const posAttr = geo.attributes.position;
                  if (posAttr) {
                    const vertices: any[] = [];
                    const indices: number[] = [];
                    
                    for (let i = 0; i < posAttr.count; i++) {
                      vertices.push({
                        pos: { x: posAttr.getX(i), y: posAttr.getY(i), z: posAttr.getZ(i) },
                        normal: geo.attributes.normal ? { x: geo.attributes.normal.getX(i), y: geo.attributes.normal.getY(i), z: geo.attributes.normal.getZ(i) } : { x: 0, y: 1, z: 0 },
                        uv: geo.attributes.uv ? { u: geo.attributes.uv.getX(i), v: geo.attributes.uv.getY(i) } : { u: 0, v: 0 }
                      });
                    }
                    
                    const indexAttr = geo.index;
                    if (indexAttr) {
                      for (let i = 0; i < indexAttr.count; i++) {
                        indices.push(indexAttr.getX(i));
                      }
                    } else {
                      for (let i = 0; i < posAttr.count; i++) {
                        indices.push(i);
                      }
                    }

                    newMeshes.push({
                      name: mesh.name || `Imported_Mesh_${Date.now()}`,
                      lod: 0,
                      parent_name: "Root",
                      parts: [{
                        material_index: 0,
                        vertex_mask: 19,
                        vertices,
                        indices
                      }]
                    });
                  }
                }
              }
            });

            if (newMeshes.length > 0) {
              onModelChangeRef.current?.({
                ...curModel,
                meshes: [...curModel.meshes, ...newMeshes]
              });
              invoke("log_event", { level: "INFO", message: `Successfully imported ${newMeshes.length} meshes from GLTF.` }).catch(console.error);
            } else {
              alert("No valid meshes found in the GLTF file.");
            }
          });
        } catch (e: any) {
          console.error(e);
          invoke("log_event", { level: "ERROR", message: `Failed to import GLTF: ${e.toString()}` }).catch(console.error);
          alert(`Import Failed: ${e.toString()}`);
        }
      };

      window.addEventListener("export-gltf-model", handleExportGLTFEvent);
      window.addEventListener("import-gltf-mesh", handleImportGLTFEvent);

      setIsInitialized(true);

      return () => {
        if (container) {
          container.removeEventListener("dragstart", handlePrevent);
          container.removeEventListener("selectstart", handlePrevent);
        }
        renderer.domElement.removeEventListener("dragstart", handlePrevent);
        renderer.domElement.removeEventListener("selectstart", handlePrevent);
        window.removeEventListener("resize", handleResize);
        window.removeEventListener("export-gltf-model", handleExportGLTFEvent);
        window.removeEventListener("import-gltf-mesh", handleImportGLTFEvent);
        cancelAnimationFrame(animationId);
        orbitControls.dispose();
        transformControls.dispose();
        renderer.dispose();
        if (mountRef.current && renderer.domElement) {
          mountRef.current.removeChild(renderer.domElement);
        }
        setIsInitialized(false);
      };
    } catch (e: any) {
      invoke("log_event", {
        level: "ERROR",
        message: `Exception thrown inside Viewport mount useEffect: ${e.stack || e.toString()}`,
      }).catch(console.error);
    }
  }, []); // Mounts only once! Persistent WebGL context prevents reset on node selection change.

  // Update transform mode (translate/rotate/scale)
  useEffect(() => {
    if (transformControlsRef.current) {
      transformControlsRef.current.setMode(transformMode);
    }
  }, [transformMode]);

  // Update Scene when new model is loaded or visibility settings change
  useEffect(() => {
    try {
      const scene = sceneRef.current;
      const meshesGroup = meshesGroupRef.current;
      const jointsGroup = jointsGroupRef.current;
      const markersGroup = markersGroupRef.current;
      const navLightsGroup = navLightsGroupRef.current;
      const collisionGroup = collisionGroupRef.current;
      const dockpathsGroup = dockpathsGroupRef.current;
      const engineBurnsGroup = engineBurnsGroupRef.current;
      const engineGlowsGroup = engineGlowsGroupRef.current;
      const engineShapesGroup = engineShapesGroupRef.current;
      const targetBoxesGroup = targetBoxesGroupRef.current;
      const camera = cameraRef.current;
      const orbitControls = orbitControlsRef.current;

      invoke("log_event", {
        level: "INFO",
        message: `Scene useEffect triggered. scene: ${!!scene}, model: ${!!model}, meshesGroup: ${!!meshesGroup}, joints: ${model?.joints?.length || 0}`,
      }).catch(console.error);

      if (!scene || !model || !meshesGroup || !jointsGroup || !markersGroup || !navLightsGroup || !collisionGroup || !dockpathsGroup || !engineBurnsGroup || !engineGlowsGroup || !engineShapesGroup || !targetBoxesGroup) {
        return;
      }

      // Clear existing meshes, joints, markers, and helpers with full recursive GPU resource disposal!
      const disposeNode = (node: THREE.Object3D) => {
        if ((node as any).geometry) {
          (node as any).geometry.dispose();
        }
        if ((node as any).material) {
          const mats = Array.isArray((node as any).material) ? (node as any).material : [(node as any).material];
          mats.forEach((mat: any) => {
            if (mat.map) mat.map.dispose();
            if (mat.normalMap) mat.normalMap.dispose();
            if (mat.emissiveMap) mat.emissiveMap.dispose();
            if (mat.roughnessMap) mat.roughnessMap.dispose();
            if (mat.metalnessMap) mat.metalnessMap.dispose();
            mat.dispose();
          });
        }
      };

      const disposeHierarchy = (group: THREE.Group) => {
        group.traverse(disposeNode);
      };

      disposeHierarchy(meshesGroup);
      disposeHierarchy(jointsGroup);
      disposeHierarchy(markersGroup);
      disposeHierarchy(navLightsGroup);
      disposeHierarchy(collisionGroup);
      disposeHierarchy(dockpathsGroup);
      disposeHierarchy(engineBurnsGroup);
      disposeHierarchy(engineGlowsGroup);
      disposeHierarchy(engineShapesGroup);
      disposeHierarchy(targetBoxesGroup);

      while (meshesGroup.children.length > 0) meshesGroup.remove(meshesGroup.children[0]);
      while (jointsGroup.children.length > 0) jointsGroup.remove(jointsGroup.children[0]);
      while (markersGroup.children.length > 0) markersGroup.remove(markersGroup.children[0]);
      while (navLightsGroup.children.length > 0) navLightsGroup.remove(navLightsGroup.children[0]);
      while (collisionGroup.children.length > 0) collisionGroup.remove(collisionGroup.children[0]);
      while (dockpathsGroup.children.length > 0) dockpathsGroup.remove(dockpathsGroup.children[0]);
      while (engineBurnsGroup.children.length > 0) engineBurnsGroup.remove(engineBurnsGroup.children[0]);
      while (engineGlowsGroup.children.length > 0) engineGlowsGroup.remove(engineGlowsGroup.children[0]);
      while (engineShapesGroup.children.length > 0) engineShapesGroup.remove(engineShapesGroup.children[0]);
      while (targetBoxesGroup.children.length > 0) targetBoxesGroup.remove(targetBoxesGroup.children[0]);
      
      navLightsAnimateRef.current = [];

      invoke("log_event", { level: "INFO", message: "Viewport: Cleared old 3D groups. Starting mesh diagnostics..." }).catch(console.error);

      // --- DETAILED MESH DIAGNOSTICS ---
      model.meshes.forEach((mesh) => {
        mesh.parts.forEach((part, pi) => {
          const verts = part.vertices;
          const idxCount = part.indices.length;
          let maxIdx = -1;
          for (let i = 0; i < part.indices.length; i++) {
            if (part.indices[i] > maxIdx) {
              maxIdx = part.indices[i];
            }
          }
          let minX = Infinity, maxX = -Infinity;
          let minY = Infinity, maxY = -Infinity;
          let minZ = Infinity, maxZ = -Infinity;
          let nanCount = 0;
          verts.forEach(v => {
            if (!v.position || !Number.isFinite(v.position.x)) { nanCount++; return; }
            if (v.position.x < minX) minX = v.position.x;
            if (v.position.x > maxX) maxX = v.position.x;
            if (v.position.y < minY) minY = v.position.y;
            if (v.position.y > maxY) maxY = v.position.y;
            if (v.position.z < minZ) minZ = v.position.z;
            if (v.position.z > maxZ) maxZ = v.position.z;
          });
          invoke("log_event", {
            level: "DEBUG",
            message: `MESH[${mesh.name}] part[${pi}] mask=0x${part.vertex_mask.toString(16)} verts=${verts.length} idxCount=${idxCount} maxIdx=${maxIdx} nan=${nanCount} X=[${minX.toFixed(1)},${maxX.toFixed(1)}] Y=[${minY.toFixed(1)},${maxY.toFixed(1)}] Z=[${minZ.toFixed(1)},${maxZ.toFixed(1)}]`,
          }).catch(console.error);
        });
      });
      // ---------------------------------

      invoke("log_event", { level: "INFO", message: `Viewport: Mesh diagnostics finished. Starting mesh rendering (count: ${model.meshes.length})...` }).catch(console.error);

      const bbox = new THREE.Box3();
      let hasValidBounds = false;

      // 1. Render ship meshes
      model.meshes.forEach((mesh) => {
        const isVisible = visibleMeshes[`${mesh.name}_lod_${mesh.lod}`] !== false;

        mesh.parts.forEach((part, index) => {
          const geometry = new THREE.BufferGeometry();
          
          // Compile position array
          const positions = new Float32Array(part.vertices.length * 3);
          const normals = new Float32Array(part.vertices.length * 3);
          const uvs = new Float32Array(part.vertices.length * 2);

          // Identify valid vs uninitialized/sentinel/skeletal outlier vertices
          const isValid = part.vertices.map((v) => {
            return (
              v.position &&
              isFiniteVector(v.position) &&
              Math.abs(v.position.x) < 10000.0 &&
              Math.abs(v.position.y) < 10000.0 &&
              Math.abs(v.position.z) < 10000.0
            );
          });

          part.vertices.forEach((v, i) => {
            if (isValid[i]) {
              positions[i * 3] = v.position.x;
              positions[i * 3 + 1] = v.position.y;
              positions[i * 3 + 2] = v.position.z;
            } else {
              positions[i * 3] = 0;
              positions[i * 3 + 1] = 0;
              positions[i * 3 + 2] = 0;
            }

            if (v.normal && isFiniteVector(v.normal)) {
              normals[i * 3] = v.normal.x;
              normals[i * 3 + 1] = v.normal.y;
              normals[i * 3 + 2] = v.normal.z;
            } else {
              normals[i * 3] = 0;
              normals[i * 3 + 1] = 1;
              normals[i * 3 + 2] = 0;
            }

            if (v.uv) {
              uvs[i * 2] = Number.isFinite(v.uv.u) ? v.uv.u : 0;
              uvs[i * 2 + 1] = Number.isFinite(v.uv.v) ? v.uv.v : 0;
            }
          });

          // Gracefully collapse triangles containing outlier/invalid vertices to zero area
          // so WebGL skips rendering them entirely, preventing glitchy spikes to infinity.
          for (let t = 0; t < part.indices.length; t += 3) {
            const iA = part.indices[t];
            const iB = part.indices[t + 1];
            const iC = part.indices[t + 2];

            const valA = isValid[iA];
            const valB = isValid[iB];
            const valC = isValid[iC];

            const validCount = (valA ? 1 : 0) + (valB ? 1 : 0) + (valC ? 1 : 0);

            if (validCount === 1 || validCount === 2) {
              // Mix of valid and invalid vertices in the same triangle.
              // Copy the position of a valid vertex to the invalid ones.
              const targetValidIdx = valA ? iA : (valB ? iB : iC);
              const targetX = positions[targetValidIdx * 3];
              const targetY = positions[targetValidIdx * 3 + 1];
              const targetZ = positions[targetValidIdx * 3 + 2];

              if (!valA) {
                positions[iA * 3] = targetX;
                positions[iA * 3 + 1] = targetY;
                positions[iA * 3 + 2] = targetZ;
              }
              if (!valB) {
                positions[iB * 3] = targetX;
                positions[iB * 3 + 1] = targetY;
                positions[iB * 3 + 2] = targetZ;
              }
              if (!valC) {
                positions[iC * 3] = targetX;
                positions[iC * 3 + 1] = targetY;
                positions[iC * 3 + 2] = targetZ;
              }
            }
          }

          geometry.setAttribute("position", new THREE.BufferAttribute(positions, 3));
          if (part.vertex_mask & 2) { // Normal is bit 2 (0x2)
            geometry.setAttribute("normal", new THREE.BufferAttribute(normals, 3));
          } else {
            geometry.computeVertexNormals();
          }

          if (part.vertex_mask & 8) { // Texture0 is bit 8 (0x8)
            geometry.setAttribute("uv", new THREE.BufferAttribute(uvs, 2));
          }

          // Use Uint32Array to support large meshes with > 65535 vertices
          geometry.setIndex(new THREE.BufferAttribute(new Uint32Array(part.indices), 1));

          // Add to bounding box using only active referenced vertices to avoid uninitialized padding garbage!
          let meshJointMatrix = new THREE.Matrix4();
          if (mesh.parent_name && mesh.parent_name !== "Root") {
            meshJointMatrix = getJointGlobalMatrix(mesh.parent_name);
          }

          part.indices.forEach((idx) => {
            if (isValid[idx]) {
              const v = part.vertices[idx];
              const pt = new THREE.Vector3(v.position.x, v.position.y, v.position.z);
              pt.applyMatrix4(meshJointMatrix); // Bring to world space
              bbox.expandByPoint(pt);
              hasValidBounds = true;
            }
          });

          // Helper to get or create a texture from base64Png
          const getCachedTexture = (base64Png: string, name: string): THREE.Texture => {
            const cache = textureCacheRef.current;
            if (cache.has(name)) {
              return cache.get(name)!;
            }
            const img = new Image();
            img.src = base64Png.startsWith("data:") ? base64Png : `data:image/png;base64,${base64Png}`;
            const tex = new THREE.Texture(img);
            tex.wrapS = THREE.RepeatWrapping;
            tex.wrapT = THREE.RepeatWrapping;
            tex.flipY = true;
            img.onload = () => {
              tex.needsUpdate = true;
            };
            cache.set(name, tex);
            return tex;
          };

          // Find texture mapping
          let textureMap: THREE.Texture | null = null;
          let normalMap: THREE.Texture | null = null;
          let glowMap: THREE.Texture | null = null;
          let teamMap: THREE.Texture | null = null;

          if (model.materials && part.material_index < model.materials.length) {
            const hMaterial = model.materials[part.material_index];
            if (hMaterial && hMaterial.texture_maps) {
              hMaterial.texture_maps.forEach((texName, tIdx) => {
                const cleanTexName = (name: string) => name.toLowerCase().replace(/\.(tga|png|dds|bmp|jpg|jpeg)$/, "").trim();
                const hTexture = model.textures?.find(t => {
                  const tName = cleanTexName(t.name);
                  const mName = cleanTexName(texName);
                  return tName === mName || tName.includes(mName) || mName.includes(tName);
                });
                if (hTexture) {
                  const b64 = hTexture.png_data || hTexture.png_preview;
                  if (b64) {
                    const tex = getCachedTexture(b64, hTexture.name);
                    const lowerName = texName.toLowerCase();
                    if (lowerName.includes("_team") || lowerName.includes("_temx")) {
                      teamMap = tex;
                    } else if (lowerName.includes("_diff") || lowerName.includes("_difx") || (tIdx === 0 && !lowerName.includes("_norm") && !lowerName.includes("_glow") && !lowerName.includes("_team"))) {
                      textureMap = tex;
                    } else if (lowerName.includes("_norm") || lowerName.includes("_normal")) {
                      normalMap = tex;
                    } else if (lowerName.includes("_glow") || lowerName.includes("_glox")) {
                      glowMap = tex;
                    }
                  }
                }
              });
            }
          }

          // Harmonized tech color palette (cobalt, indigo, cyan, violet) to color-code different parts beautifully!
          const partColors = ["#4361ee", "#3f37c9", "#4cc9f0", "#7209b7", "#f72585"];
          const colorHex = partColors[part.material_index % partColors.length] || "#4361ee";

          let material: THREE.Material;

          const hMaterial = model.materials && part.material_index < model.materials.length ? model.materials[part.material_index] : null;
          const isBadgeShader = !!(hMaterial && hMaterial.shader_name.toLowerCase().includes("badge"));

          if (renderMode === "wireframe") {
            material = new THREE.MeshBasicMaterial({
              color: new THREE.Color(colorHex),
              wireframe: true,
              side: THREE.DoubleSide,
            });
          } else if (renderMode === "untextured") {
            material = new THREE.MeshStandardMaterial({
              color: new THREE.Color("#8da6c4"),
              roughness: 0.6,
              metalness: 0.1,
              side: THREE.DoubleSide,
              flatShading: true,
            });
          } else if (renderMode === "textured" || renderMode === "textured_team") {
            material = new THREE.MeshBasicMaterial({
              color: textureMap ? new THREE.Color("#ffffff") : new THREE.Color(colorHex),
              map: textureMap,
              transparent: isBadgeShader,
              side: THREE.DoubleSide,
            });
          } else { // "shaded" or "shaded_team"
            material = new THREE.MeshStandardMaterial({
              color: textureMap ? new THREE.Color("#ffffff") : new THREE.Color(colorHex),
              map: textureMap,
              normalMap: normalMap,
              normalScale: new THREE.Vector2(1.0, 1.0),
              emissiveMap: glowMap,
              emissive: glowMap ? new THREE.Color("#ffffff") : new THREE.Color("#000000"),
              roughness: 0.5,
              metalness: 0.15,
              side: THREE.DoubleSide,
            });
          }

          if (teamMap && (renderMode === "textured_team" || renderMode === "shaded_team")) {
            const uTeam = new THREE.Color(teamColor);
            const uStripe = new THREE.Color(stripeColor);

            material.onBeforeCompile = (shader, _renderer) => {
              shader.uniforms.teamMap = { value: teamMap };
              shader.uniforms.uColTeam = { value: uTeam };
              shader.uniforms.uColStripe = { value: uStripe };

              shader.fragmentShader = `
                uniform sampler2D teamMap;
                uniform vec3 uColTeam;
                uniform vec3 uColStripe;
              ` + shader.fragmentShader;

              shader.fragmentShader = shader.fragmentShader.replace(
                '#include <map_fragment>',
                `
                #include <map_fragment>
                #ifdef USE_MAP
                  vec4 texTeam = texture2D( teamMap, vMapUv );
                  vec3 paint = mix(uColTeam, diffuseColor.rgb, texTeam.r);
                  paint = mix(uColStripe, paint, texTeam.g);
                  diffuseColor.rgb = paint;
                #else
                  diffuseColor.rgb *= uColTeam;
                #endif
                `
              );
            };
          }

          const threeMesh = new THREE.Mesh(geometry, material);
          threeMesh.name = `${mesh.name}_part_${index}`;
          threeMesh.visible = isVisible;

          const parentName = mesh.parent_name || "Root";
          const jointMatrix = getJointGlobalMatrix(parentName);
          threeMesh.applyMatrix4(jointMatrix);
          threeMesh.userData.parentJointName = parentName;
          threeMesh.userData.baseJointMatrix = jointMatrix.clone();

          meshesGroup.add(threeMesh);

          // Stunning wireframe overlay overlay for an advanced sci-tech engineering blueprint aesthetic!
          const wireframeGeo = new THREE.WireframeGeometry(geometry);
          const wireframeMat = new THREE.LineBasicMaterial({
            color: new THREE.Color("#16a0ff"),
            transparent: true,
            opacity: 0.15,
          });
          const wireframe = new THREE.LineSegments(wireframeGeo, wireframeMat);
          threeMesh.add(wireframe);
        });
      });

      // 2. Pre-calculate joint positions to build bounding box and connections safely
      invoke("log_event", { level: "INFO", message: `Viewport: Mesh rendering done. Pre-calculating ${model.joints.length} joint positions...` }).catch(console.error);
      const jointPositions = new Map<string, THREE.Vector3>();

      model.joints.forEach((joint) => {
        // Find global position from transform matrix recursively
        const matrix = getJointGlobalMatrix(joint.name);
        
        const pos = new THREE.Vector3();
        const q = new THREE.Quaternion();
        const s = new THREE.Vector3();
        safeDecompose(matrix, pos, q, s);

        if (isFiniteVector(pos) && Math.abs(pos.x) < 100000 && Math.abs(pos.y) < 100000 && Math.abs(pos.z) < 100000) {
          jointPositions.set(joint.name.toLowerCase(), pos);
          bbox.expandByPoint(pos);
          hasValidBounds = true;
        } else {
          jointPositions.set(joint.name.toLowerCase(), new THREE.Vector3(0, 0, 0));
        }
      });

      // Process markers to expand bounds:
      model.markers.forEach((marker) => {
        const pos = new THREE.Vector3(marker.position.x, marker.position.y, marker.position.z);
        bbox.expandByPoint(pos);
        hasValidBounds = true;
      });

      // Calculate max dimension and dynamic scale factor
      const size = new THREE.Vector3();
      bbox.getSize(size);
      const maxDim = Math.max(size.x, size.y, size.z);
      // Scale factor: scale of joints/markers relative to the ship's longest dimension
      const scaleFactor = hasValidBounds && maxDim > 0 ? Math.max(0.15, maxDim * 0.025) : 1.0;
      scaleFactorRef.current = scaleFactor;

      invoke("log_event", { level: "INFO", message: `Viewport: Joint pre-calculation complete. scaleFactor: ${scaleFactor}. Rendering joints and connections...` }).catch(console.error);

      // 3. Render joint skeletons using scaleFactor
      const sphereGeo = new THREE.SphereGeometry(Math.max(0.12, Math.min(2.5, 0.08 * scaleFactor)), 16, 16);
      const jointMat = new THREE.MeshBasicMaterial({ 
        color: "#00e676", 
        depthTest: false, 
        transparent: true, 
        opacity: 0.9 
      });

      model.joints.forEach((joint) => {
        const isJointVisible = visibleMeshes[`joint:${joint.name}`] !== false;
        if (!isJointVisible) return;

        const pos = jointPositions.get(joint.name.toLowerCase());
        if (!pos) return;

        invoke("log_event", {
          level: "INFO",
          message: `Joint ${joint.name}: parent=${joint.parent_name}, localCoords=[${joint.local_transform.m[3][0].toFixed(3)}, ${joint.local_transform.m[3][1].toFixed(3)}, ${joint.local_transform.m[3][2].toFixed(3)}], worldPos=[${pos.x.toFixed(3)}, ${pos.y.toFixed(3)}, ${pos.z.toFixed(3)}]`
        }).catch(console.error);

        // Decompose global orientation matrix
        const matrix = getJointGlobalMatrix(joint.name);
        const q = new THREE.Quaternion();
        const dummyPos = new THREE.Vector3();
        const dummyScale = new THREE.Vector3();
        safeDecompose(matrix, dummyPos, q, dummyScale);

        const jointMesh = new THREE.Mesh(sphereGeo, jointMat);
        jointMesh.position.copy(pos);
        jointMesh.quaternion.copy(q);
        jointMesh.name = `joint:${joint.name}`;
        jointsGroup.add(jointMesh);

        // Draw bone line if there is a parent joint, parent is visible, and showBoneLines is enabled
        if (showBoneLines && joint.parent_name) {
          const isParentVisible = visibleMeshes[`joint:${joint.parent_name}`] !== false;
          if (isParentVisible) {
            const parentPos = jointPositions.get(joint.parent_name.toLowerCase());
            if (parentPos) {
              const boneGeo = new THREE.BufferGeometry().setFromPoints([pos, parentPos]);
              const boneMat = new THREE.LineBasicMaterial({ 
                color: "#00b0ff", 
                depthTest: false, 
                transparent: true, 
                opacity: 0.6 
              });
              const boneLine = new THREE.Line(boneGeo, boneMat);
              boneLine.name = `bone:${joint.parent_name}->${joint.name}`;
              boneLine.userData = {
                parentJointName: joint.parent_name,
                childJointName: joint.name
              };
              jointsGroup.add(boneLine);
            }
          }
        }
      });

      invoke("log_event", { level: "INFO", message: `Viewport: Joint rendering complete. Rendering ${model.markers.length} markers...` }).catch(console.error);

      // 4. Render markers using scaleFactor
      const markerGeo = new THREE.ConeGeometry(0.15 * scaleFactor, 0.6 * scaleFactor, 8);
      markerGeo.rotateX(Math.PI / 2); // Point forward along Z
      const markerMat = new THREE.MeshBasicMaterial({ 
        color: "#00d2ff", 
        depthTest: false, 
        transparent: true, 
        opacity: 0.9 
      });

      model.markers.forEach((marker) => {
        const isMarkerVisible = visibleMeshes[`marker:${marker.name}`] !== false;
        if (!isMarkerVisible) return;

        const markerMesh = new THREE.Mesh(markerGeo, markerMat);
        
        // Calculate global transform recursively from parent joint
        const parentMatrix = getJointGlobalMatrix(marker.parent_joint);
        const localPos = new THREE.Vector3(marker.position.x, marker.position.y, marker.position.z);
        const localRotMatrix = toThreeMatrix(marker.rotation);

        const localMatrix = localRotMatrix.clone();
        localMatrix.setPosition(localPos);

        const globalMatrix = parentMatrix.clone().multiply(localMatrix);

        const globalPos = new THREE.Vector3();
        const globalQuat = new THREE.Quaternion();
        const globalScale = new THREE.Vector3();
        safeDecompose(globalMatrix, globalPos, globalQuat, globalScale);

        markerMesh.position.copy(globalPos);
        markerMesh.quaternion.copy(globalQuat);
        
        markerMesh.name = `marker:${marker.name}`;
        markerMesh.userData = {
          parentJointName: marker.parent_joint,
          baseMatrix: localMatrix.clone()
        };
        markersGroup.add(markerMesh);
        
        const markerAxes = new THREE.AxesHelper(0.8 * scaleFactor);
        markerMesh.add(markerAxes);
      });

      // 5. Render NavLights (Pulsing Previews)
      if (showNavLights && model.nav_lights) {
        model.nav_lights.forEach((nav) => {
          const isNavVisible = visibleMeshes[`navlight:${nav.name}`] !== false;
          if (!isNavVisible) return;

          const color = new THREE.Color(nav.color.x, nav.color.y, nav.color.z);
          
          // Render the actual sprite size as a small solid pulsing sphere
          const spriteRadius = (nav.size && nav.size > 0) ? (nav.size * 0.25) : 0.15;
          const navGeo = new THREE.SphereGeometry(spriteRadius, 16, 16);
          const navMat = new THREE.MeshBasicMaterial({
            color: color,
            transparent: true,
            opacity: 0.8,
            depthTest: false,
            depthWrite: false,
          });
           const navMesh = new THREE.Mesh(navGeo, navMat);
          navMesh.name = `navlight:${nav.name}`;
          navMesh.renderOrder = 20;
          navMesh.userData = {
            parentJointName: nav.name,
            baseMatrix: new THREE.Matrix4()
          };

          const jointPos = jointPositions.get(nav.name.toLowerCase());
          if (jointPos) {
            navMesh.position.copy(jointPos);
          } else {
            navMesh.position.set(0, 0, 0);
          }

          navLightsGroup.add(navMesh);

          if (nav.distance && nav.distance > 0) {
            const rangeGeo = new THREE.SphereGeometry(nav.distance, 16, 16);
            const rangeMat = new THREE.MeshBasicMaterial({
              color: color,
              wireframe: true,
              transparent: true,
              opacity: 0.12,
              depthTest: false,
              depthWrite: false,
            });
            const rangeMesh = new THREE.Mesh(rangeGeo, rangeMat);
            rangeMesh.renderOrder = 19;
            navMesh.add(rangeMesh);
          }

          navLightsAnimateRef.current.push({
            mesh: navMesh,
            material: navMat,
            frequency: typeof nav.frequency === "number" ? nav.frequency : 0.0,
            phase: typeof nav.phase === "number" ? nav.phase : 0.0,
          });
        });
      }

      // 6. Render Dockpaths (Pyramid helpers and line connection)
      if (showDockpaths && model.dockpaths) {
        model.dockpaths.forEach((path) => {
          const isPathVisible = visibleMeshes[`dockpath:${path.name}`] !== false;
          if (!isPathVisible) return;

          const parentMatrix = getJointGlobalMatrix(path.parent_name);
          const pointsInWorld: THREE.Vector3[] = [];

          path.points.forEach((pt, ptIndex) => {
            const r = pt.rotation.m;
            const localMatrix = new THREE.Matrix4().set(
              r[0][0], r[1][0], r[2][0], r[3][0],
              r[0][1], r[1][1], r[2][1], r[3][1],
              r[0][2], r[1][2], r[2][2], r[3][2],
              r[0][3], r[1][3], r[2][3], r[3][3]
            );
            localMatrix.setPosition(pt.position.x, pt.position.y, pt.position.z);

            const globalMatrix = parentMatrix.clone().multiply(localMatrix);

            const ptPos = new THREE.Vector3();
            const ptQuat = new THREE.Quaternion();
            const ptScale = new THREE.Vector3();
            safeDecompose(globalMatrix, ptPos, ptQuat, ptScale);

            pointsInWorld.push(ptPos);

            const pyramidGeo = new THREE.ConeGeometry(0.15 * scaleFactor, 0.4 * scaleFactor, 4);
            pyramidGeo.rotateX(Math.PI / 2);

            const pyramidMat = new THREE.MeshBasicMaterial({
              color: "#ffaa00",
              depthTest: false,
              transparent: true,
              opacity: 0.9,
            });
            const pyramidMesh = new THREE.Mesh(pyramidGeo, pyramidMat);
            pyramidMesh.name = `dockpoint:${path.name}:${ptIndex}`;
            pyramidMesh.position.copy(ptPos);
            pyramidMesh.quaternion.copy(ptQuat);
            pyramidMesh.userData = {
              parentJointName: path.parent_name,
              baseMatrix: localMatrix.clone()
            };

            dockpathsGroup.add(pyramidMesh);
          });

          if (pointsInWorld.length > 1) {
            const lineGeo = new THREE.BufferGeometry().setFromPoints(pointsInWorld);
            const lineMat = new THREE.LineBasicMaterial({
              color: "#ffd54f",
              linewidth: 2,
              transparent: true,
              opacity: 0.8,
            });
            const pathLine = new THREE.Line(lineGeo, lineMat);
            pathLine.name = `dockpath_line:${path.name}`;
            pathLine.userData = {
              pathName: path.name,
              pointCount: path.points.length
            };
            dockpathsGroup.add(pathLine);
          }
        });
      }

      const createThreeMeshGroup = (mesh: HODMesh, defaultColor: string, opacity: number): THREE.Group => {
        const group = new THREE.Group();
        mesh.parts.forEach((part) => {
          const geometry = new THREE.BufferGeometry();
          const positions = new Float32Array(part.vertices.length * 3);
          const normals = new Float32Array(part.vertices.length * 3);
          const uvs = new Float32Array(part.vertices.length * 2);

          const isValid = part.vertices.map((v) => {
            return (
              v.position &&
              isFiniteVector(v.position) &&
              Math.abs(v.position.x) < 10000.0 &&
              Math.abs(v.position.y) < 10000.0 &&
              Math.abs(v.position.z) < 10000.0
            );
          });

          part.vertices.forEach((v, i) => {
            if (isValid[i]) {
              positions[i * 3] = v.position.x;
              positions[i * 3 + 1] = v.position.y;
              positions[i * 3 + 2] = v.position.z;
            } else {
              positions[i * 3] = 0;
              positions[i * 3 + 1] = 0;
              positions[i * 3 + 2] = 0;
            }

            if (v.normal && isFiniteVector(v.normal)) {
              normals[i * 3] = v.normal.x;
              normals[i * 3 + 1] = v.normal.y;
              normals[i * 3 + 2] = v.normal.z;
            } else {
              normals[i * 3] = 0;
              normals[i * 3 + 1] = 1;
              normals[i * 3 + 2] = 0;
            }

            if (v.uv) {
              uvs[i * 2] = Number.isFinite(v.uv.u) ? v.uv.u : 0;
              uvs[i * 2 + 1] = Number.isFinite(v.uv.v) ? v.uv.v : 0;
            }
          });

          for (let t = 0; t < part.indices.length; t += 3) {
            const iA = part.indices[t];
            const iB = part.indices[t + 1];
            const iC = part.indices[t + 2];

            const valA = isValid[iA];
            const valB = isValid[iB];
            const valC = isValid[iC];

            const validCount = (valA ? 1 : 0) + (valB ? 1 : 0) + (valC ? 1 : 0);

            if (validCount === 1 || validCount === 2) {
              const targetValidIdx = valA ? iA : (valB ? iB : iC);
              const targetX = positions[targetValidIdx * 3];
              const targetY = positions[targetValidIdx * 3 + 1];
              const targetZ = positions[targetValidIdx * 3 + 2];

              if (!valA) {
                positions[iA * 3] = targetX;
                positions[iA * 3 + 1] = targetY;
                positions[iA * 3 + 2] = targetZ;
              }
              if (!valB) {
                positions[iB * 3] = targetX;
                positions[iB * 3 + 1] = targetY;
                positions[iB * 3 + 2] = targetZ;
              }
              if (!valC) {
                positions[iC * 3] = targetX;
                positions[iC * 3 + 1] = targetY;
                positions[iC * 3 + 2] = targetZ;
              }
            }
          }

          geometry.setAttribute("position", new THREE.BufferAttribute(positions, 3));
          if (part.vertex_mask & 2) {
            geometry.setAttribute("normal", new THREE.BufferAttribute(normals, 3));
          } else {
            geometry.computeVertexNormals();
          }

          if (part.vertex_mask & 8) {
            geometry.setAttribute("uv", new THREE.BufferAttribute(uvs, 2));
          }

          geometry.setIndex(new THREE.BufferAttribute(new Uint32Array(part.indices), 1));

          const material = new THREE.MeshBasicMaterial({
            color: new THREE.Color(defaultColor),
            transparent: opacity < 1.0,
            opacity: opacity,
            side: THREE.DoubleSide,
            depthWrite: false, // Prevents transparent parts from clipping out each other
            blending: THREE.AdditiveBlending,
          });

          const partMesh = new THREE.Mesh(geometry, material);
          group.add(partMesh);
        });
        return group;
      };

      // 7. Render Engine Burns (Sequential line loops and particle spawn dots)
      if (showEngineBurns && model.engine_burns) {
        model.engine_burns.forEach((burn) => {
          const isBurnVisible = visibleMeshes[`engine_burn:${burn.name}`] !== false;
          if (!isBurnVisible) return;

          const parentMatrix = getJointGlobalMatrix(burn.parent_name);
          const parentPos = new THREE.Vector3();
          const parentQuat = new THREE.Quaternion();
          const parentScale = new THREE.Vector3();
          safeDecompose(parentMatrix, parentPos, parentQuat, parentScale);

          const burnPoints: THREE.Vector3[] = [];
          burn.vertices.forEach((v) => {
            const pt = new THREE.Vector3(v.x, v.y, v.z);
            burnPoints.push(pt);
          });

          if (burnPoints.length > 0) {
            const burnGroup = new THREE.Group();
            burnGroup.name = `engine_burn:${burn.name}`;
            burnGroup.position.copy(parentPos);
            burnGroup.quaternion.copy(parentQuat);
            burnGroup.userData = {
              parentJointName: burn.parent_name,
              baseMatrix: new THREE.Matrix4()
            };

            const burnGeo = new THREE.BufferGeometry().setFromPoints(burnPoints);
            const burnMat = new THREE.LineBasicMaterial({
              color: "#ff3d00",
              transparent: true,
              opacity: 0.8,
            });
            const burnLine = new THREE.Line(burnGeo, burnMat);
            burnGroup.add(burnLine);

            // Add glowing particle spawn indicator spheres at each burn vertex coordinate
            const sphereGeo = new THREE.SphereGeometry(0.12 * scaleFactor, 8, 8);
            const dotMat = new THREE.MeshBasicMaterial({
              color: "#ff9100",
              depthTest: false,
              transparent: true,
              opacity: 0.95,
            });

            burnPoints.forEach((pt) => {
              const dotMesh = new THREE.Mesh(sphereGeo, dotMat);
              dotMesh.position.copy(pt);
              burnGroup.add(dotMesh);
            });

            engineBurnsGroup.add(burnGroup);
          }
        });
      }

      // Render Engine Glows
      if (showEngineBurns && model.engine_glows) {
        model.engine_glows.forEach((glow) => {
          const isGlowVisible = visibleMeshes[`engine_glow:${glow.name}`] !== false;
          if (!isGlowVisible) return;

          const parentMatrix = getJointGlobalMatrix(glow.parent_name);
          const parentPos = new THREE.Vector3();
          const parentQuat = new THREE.Quaternion();
          const parentScale = new THREE.Vector3();
          safeDecompose(parentMatrix, parentPos, parentQuat, parentScale);

          const glowGroup = createThreeMeshGroup(glow.mesh, "#ffd600", 0.5);
          glowGroup.name = `engine_glow:${glow.name}`;
          glowGroup.position.copy(parentPos);
          glowGroup.quaternion.copy(parentQuat);
          glowGroup.userData = {
            parentJointName: glow.parent_name,
            baseMatrix: new THREE.Matrix4()
          };

          engineGlowsGroup.add(glowGroup);
        });
      }

      // Render Engine Shapes
      if (showEngineBurns && model.engine_shapes) {
        model.engine_shapes.forEach((shape) => {
          const isShapeVisible = visibleMeshes[`engine_shape:${shape.name}`] !== false;
          if (!isShapeVisible) return;

          const parentMatrix = getJointGlobalMatrix(shape.parent_name);
          const parentPos = new THREE.Vector3();
          const parentQuat = new THREE.Quaternion();
          const parentScale = new THREE.Vector3();
          safeDecompose(parentMatrix, parentPos, parentQuat, parentScale);

          const shapeGroup = createThreeMeshGroup(shape.mesh, "#7209b7", 0.4);
          shapeGroup.name = `engine_shape:${shape.name}`;
          shapeGroup.position.copy(parentPos);
          shapeGroup.quaternion.copy(parentQuat);
          shapeGroup.userData = {
            parentJointName: shape.parent_name,
            baseMatrix: new THREE.Matrix4()
          };

          engineShapesGroup.add(shapeGroup);
        });
      }

      // 8. Render Collision Hulls (Boxes and wireframe sphere center + radius)
      if (showCollision && model.collision_meshes) {
        model.collision_meshes.forEach((col) => {
          const isColVisible = visibleMeshes[`collision:${col.name}`] !== false;
          if (!isColVisible) return;

          const jointMatrix = getJointGlobalMatrix(col.name);

          const colGroup = new THREE.Group();
          colGroup.name = `collision:${col.name}`;
          colGroup.renderOrder = 1;

          // Semi-transparent red box
          const sizeX = col.max_extents.x - col.min_extents.x;
          const sizeY = col.max_extents.y - col.min_extents.y;
          const sizeZ = col.max_extents.z - col.min_extents.z;
          const boxGeo = new THREE.BoxGeometry(sizeX || 0.1, sizeY || 0.1, sizeZ || 0.1);
          const boxMat = new THREE.MeshBasicMaterial({
            color: "#ff1744",
            transparent: true,
            opacity: 0.15,
            wireframe: false,
            depthWrite: false,
          });
          const boxMesh = new THREE.Mesh(boxGeo, boxMat);
          boxMesh.renderOrder = 1;

          const boxWire = new THREE.LineSegments(
            new THREE.EdgesGeometry(boxGeo),
            new THREE.LineBasicMaterial({ color: "#ff1744", transparent: true, opacity: 0.6, depthWrite: false })
          );
          boxWire.renderOrder = 2;
          boxMesh.add(boxWire);

          const boxCenter = new THREE.Vector3(
            (col.min_extents.x + col.max_extents.x) / 2,
            (col.min_extents.y + col.max_extents.y) / 2,
            (col.min_extents.z + col.max_extents.z) / 2
          );
          boxMesh.position.copy(boxCenter);
          colGroup.add(boxMesh);

          // Wireframe sphere representing center and radius
          const sphereGeo = new THREE.SphereGeometry(col.radius || 0.1, 16, 16);
          const sphereMat = new THREE.MeshBasicMaterial({
            color: "#ff1744",
            wireframe: true,
            transparent: true,
            opacity: 0.25,
            depthWrite: false,
          });
          const sphereMesh = new THREE.Mesh(sphereGeo, sphereMat);
          sphereMesh.renderOrder = 1;
          sphereMesh.position.set(col.center.x, col.center.y, col.center.z);
          colGroup.add(sphereMesh);

          colGroup.applyMatrix4(jointMatrix);
          colGroup.userData = {
            parentJointName: col.name,
            baseMatrix: new THREE.Matrix4()
          };
          collisionGroup.add(colGroup);
        });
      }

      if (targetBoxes) {
        targetBoxes.forEach((box) => {
          if (box.visible === false) return;

          const center = bbox.getCenter(new THREE.Vector3());
          const size = bbox.getSize(new THREE.Vector3());
          const halfSize = size.clone().multiplyScalar(0.5);

          const minX = center.x + halfSize.x * box.min.x;
          const minY = center.y + halfSize.y * box.min.y;
          const minZ = center.z + halfSize.z * box.min.z;

          const maxX = center.x + halfSize.x * box.max.x;
          const maxY = center.y + halfSize.y * box.max.y;
          const maxZ = center.z + halfSize.z * box.max.z;

          const width = Math.abs(maxX - minX);
          const height = Math.abs(maxY - minY);
          const length = Math.abs(maxZ - minZ);

          const boxGeo = new THREE.BoxGeometry(width || 0.1, height || 0.1, length || 0.1);
          const boxMat = new THREE.MeshBasicMaterial({
            color: "#ffab00",
            transparent: true,
            opacity: 0.15,
            depthTest: false,
          });
          const boxMesh = new THREE.Mesh(boxGeo, boxMat);

          const boxWire = new THREE.LineSegments(
            new THREE.EdgesGeometry(boxGeo),
            new THREE.LineBasicMaterial({ color: "#ffab00", transparent: true, opacity: 0.6, depthTest: false })
          );
          boxMesh.add(boxWire);

          const boxCenter = new THREE.Vector3(
            (minX + maxX) / 2,
            (minY + maxY) / 2,
            (minZ + maxZ) / 2
          );
          boxMesh.position.copy(boxCenter);
          
          if (targetBoxesGroupRef.current) {
            targetBoxesGroupRef.current.add(boxMesh);
          }
        });
      }

      // 9. Auto-Focus and Camera Fitting (Only run once when a new HOD model is loaded!)
      if (camera && orbitControls && hasValidBounds && maxDim > 0 && lastFocusedModelRef.current !== model.name) {
        const center = new THREE.Vector3();
        bbox.getCenter(center);

        const fovRad = (camera.fov * Math.PI) / 180;
        let cameraDistance = maxDim / (2 * Math.tan(fovRad / 2));
        cameraDistance *= 1.5; // Padding

         camera.near = Math.max(0.05, cameraDistance * 0.001);
        camera.far = Math.max(50000, cameraDistance * 20);
        camera.updateProjectionMatrix();

        const camX = cameraDistance * 0.5;
        const camY = cameraDistance * 0.4;
        const camZ = cameraDistance * 1.0;
        camera.position.set(camX, camY, camZ);
        orbitControls.target.set(0, 0, 0);
        orbitControls.update();

        // Save this model name so we don't refocus on sub-component toggles or parameter edits!
        lastFocusedModelRef.current = model.name;

        invoke("log_event", {
          level: "INFO",
          message: `Camera auto-focused for loaded HOD model "${model.name}": far=${camera.far.toFixed(0)} pos=[${camX.toFixed(1)},${camY.toFixed(1)},${camZ.toFixed(1)}]`,
        }).catch(console.error);
      }
    } catch (e: any) {
      invoke("log_event", {
        level: "ERROR",
        message: `Exception thrown inside Viewport scene update useEffect: ${e.stack || e.toString()}`,
      }).catch(console.error);
    }
  }, [model, visibleMeshes, isInitialized, showNavLights, showCollision, showDockpaths, showEngineBurns, showBoneLines, renderMode, teamColor, stripeColor, targetBoxes]);

  // Hook visual gizmo to the selected node
  useEffect(() => {
    const transformControls = transformControlsRef.current;
    const scene = sceneRef.current;
    const jointsGroup = jointsGroupRef.current;
    const markersGroup = markersGroupRef.current;

    if (!transformControls || !scene || !jointsGroup || !markersGroup) return;

    // Force Three.js to immediately compute absolute world positions/matrices
    scene.updateMatrixWorld(true);

    // Remove old axes helper if attached to anything
    if (selectionAxesHelperRef.current && selectionAxesHelperRef.current.parent) {
      selectionAxesHelperRef.current.parent.remove(selectionAxesHelperRef.current);
    }

    transformControls.detach();

    if (selectedNode) {
      if (selectedNode.type === "joint" && selectedNode.name.toLowerCase() === "root") {
        transformControls.detach();
        return;
      }

      let targetObj: THREE.Object3D | undefined;

      invoke("log_event", {
        level: "INFO",
        message: `jointsGroup children count: ${jointsGroup.children.length}, names: ${jointsGroup.children.map(c => c.name).join(", ")}`
      }).catch(console.error);

      if (selectedNode.type === "joint") {
        targetObj = jointsGroup.children.find(c => c.name.toLowerCase() === `joint:${selectedNode.name}`.toLowerCase());
      } else if (selectedNode.type === "weapon_group") {
        targetObj = jointsGroup.children.find(c => c.name.toLowerCase() === `joint:${selectedNode.name}_position`.toLowerCase());
      } else if (selectedNode.type === "marker") {
        targetObj = markersGroup.children.find(c => c.name.toLowerCase() === `marker:${selectedNode.name}`.toLowerCase());
      } else if (selectedNode.type === "navlight") {
        targetObj = navLightsGroupRef.current?.children.find(c => c.name.toLowerCase() === `navlight:${selectedNode.name}`.toLowerCase());
      } else if (selectedNode.type === "dockpoint") {
        targetObj = dockpathsGroupRef.current?.children.find(c => c.name.toLowerCase() === `dockpoint:${selectedNode.name}`.toLowerCase());
      } else if (selectedNode.type === "collision") {
        targetObj = collisionGroupRef.current?.children.find(c => c.name.toLowerCase() === `collision:${selectedNode.name}`.toLowerCase());
      } else if (selectedNode.type === "engine_burn") {
        targetObj = engineBurnsGroupRef.current?.children.find(c => c.name.toLowerCase() === `engine_burn:${selectedNode.name}`.toLowerCase());
      } else if (selectedNode.type === "engine_glow") {
        targetObj = engineGlowsGroupRef.current?.children.find(c => c.name.toLowerCase() === `engine_glow:${selectedNode.name}`.toLowerCase());
      } else if (selectedNode.type === "engine_shape") {
        targetObj = engineShapesGroupRef.current?.children.find(c => c.name.toLowerCase() === `engine_shape:${selectedNode.name}`.toLowerCase());
      }

      invoke("log_event", {
        level: "INFO",
        message: `Selection changed: type=${selectedNode.type}, name=${selectedNode.name}, foundTarget=${!!targetObj}, targetPos=${targetObj ? JSON.stringify(targetObj.position) : "none"}`
      }).catch(console.error);

      if (targetObj) {
        transformControls.attach(targetObj);

        // Highlight selected node by showing a smaller axes helper on its local orientation!
        const targetScale = 2.2 * (scaleFactorRef.current || 1.0);
        if (!selectionAxesHelperRef.current) {
          selectionAxesHelperRef.current = new THREE.AxesHelper(1.0);
        }
        selectionAxesHelperRef.current.position.set(0, 0, 0);
        selectionAxesHelperRef.current.rotation.set(0, 0, 0);
        selectionAxesHelperRef.current.scale.setScalar(targetScale);

        // Apply depth test bypass so it draws on top clearly
        const helperMaterial = selectionAxesHelperRef.current.material;
        if (Array.isArray(helperMaterial)) {
          helperMaterial.forEach(mat => {
            mat.depthTest = false;
            mat.transparent = true;
            mat.opacity = 0.95;
          });
        } else if (helperMaterial) {
          helperMaterial.depthTest = false;
          helperMaterial.transparent = true;
          helperMaterial.opacity = 0.95;
        }

        targetObj.add(selectionAxesHelperRef.current);
      }
    }
  }, [selectedNode, model]);

  return (
    <div className="viewport-container" ref={mountRef}>
      <div
        style={{
          position: "absolute",
          top: "12px",
          left: "12px",
          display: "flex",
          gap: "8px",
          zIndex: 10,
        }}
      >
        <button
          className={transformMode === "translate" ? "primary" : ""}
          onClick={() => setTransformMode("translate")}
          title="Translate Gizmo"
        >
          Move
        </button>
        <button
          className={transformMode === "rotate" ? "primary" : ""}
          onClick={() => setTransformMode("rotate")}
          title="Rotate Gizmo"
        >
          Rotate
        </button>
      </div>

      {model && model.animations && model.animations.length > 0 && (
        <div
          style={{
            position: "absolute",
            top: "12px",
            left: "50%",
            transform: "translateX(-50%)",
            background: "rgba(0, 230, 118, 0.10)",
            border: "1px solid rgba(0, 230, 118, 0.35)",
            borderRadius: "20px",
            padding: "3px 12px",
            display: "flex",
            alignItems: "center",
            gap: "6px",
            zIndex: 10,
            fontSize: "9px",
            fontWeight: "700",
            color: "#00e676",
            textTransform: "uppercase",
            letterSpacing: "0.1em",
            pointerEvents: "none",
          }}
        >
          <span style={{ display: "inline-block", width: "5px", height: "5px", borderRadius: "50%", background: "#00e676" }} />
          {model.animations.length} Anim{model.animations.length !== 1 ? "s" : ""} — See Dock Below
        </div>
      )}
    </div>
  );
};
