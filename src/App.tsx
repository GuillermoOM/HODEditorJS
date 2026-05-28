import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Viewport, HODModel, Vector3D } from "./components/Viewport";
import { Toolbar } from "./components/Toolbar";
import { HierarchyTree } from "./components/HierarchyTree";
import { Inspector } from "./components/Inspector";
import { AnimationDock } from "./components/AnimationDock";
import { Info, AlertTriangle, FolderOpen, FilePlus } from "lucide-react";
import "./App.css";

function App() {
  const [model, setModel] = useState<HODModel | null>(null);
  const [filePath, setFilePath] = useState("");
  const [selectedNode, setSelectedNode] = useState<{ type: string; name: string } | null>(null);
  const [transformMode, setTransformMode] = useState<"translate" | "rotate" | "scale">("translate");
  const [isSaving, setIsSaving] = useState(false);
  const [isLoading, setIsLoading] = useState(false);
  const [isDirty, setIsDirty] = useState(false);
  const [errorMsg, setErrorMsg] = useState<string | null>(null);
  const [selectedAnimIdx, setSelectedAnimIdx] = useState(0);
  const [targetBoxes, setTargetBoxes] = useState<{ index: number; min: { x: number; y: number; z: number }; max: { x: number; y: number; z: number }; visible?: boolean }[]>([]);

  // Lifted animation playback state (shared between AnimationDock and Viewport)
  const [isPlaying, setIsPlaying] = useState(false);
  const [currentTime, setCurrentTime] = useState(0);
  const [loopPlayback, setLoopPlayback] = useState(false);
  const [playbackSpeed, setPlaybackSpeed] = useState(1.0);

  // Workspace resizing widths
  const [sidebarWidth, setSidebarWidth] = useState(364);
  const [inspectorWidth, setInspectorWidth] = useState(320);

  const handleSidebarDragStart = (e: React.MouseEvent) => {
    e.preventDefault();
    const handleMouseMove = (moveEvent: MouseEvent) => {
      const newWidth = Math.max(180, Math.min(500, moveEvent.clientX));
      setSidebarWidth(newWidth);
    };
    const handleMouseUp = () => {
      window.removeEventListener("mousemove", handleMouseMove);
      window.removeEventListener("mouseup", handleMouseUp);
    };
    window.addEventListener("mousemove", handleMouseMove);
    window.addEventListener("mouseup", handleMouseUp);
  };

  const handleInspectorDragStart = (e: React.MouseEvent) => {
    e.preventDefault();
    const handleMouseMove = (moveEvent: MouseEvent) => {
      const newWidth = Math.max(200, Math.min(600, window.innerWidth - moveEvent.clientX));
      setInspectorWidth(newWidth);
    };
    const handleMouseUp = () => {
      window.removeEventListener("mousemove", handleMouseMove);
      window.removeEventListener("mouseup", handleMouseUp);
    };
    window.addEventListener("mousemove", handleMouseMove);
    window.addEventListener("mouseup", handleMouseUp);
  };

  const updateModel = (nextModel: HODModel) => {
    setModel(nextModel);
    setIsDirty(true);
  };
  const [statusMsg, setStatusMsg] = useState("System Ready");
  const [visibleMeshes, setVisibleMeshes] = useState<Record<string, boolean>>({});
  const [isMigrationModalOpen, setIsMigrationModalOpen] = useState(false);
  const [migrationMappings, setMigrationMappings] = useState<Record<string, "joint" | "marker" | "weapon" | "collision" | "navlight" | "engine_burn" | "engine_glow">>({});
  const [migrationCoordinateClones, setMigrationCoordinateClones] = useState<Record<string, string>>({});

  // Big Data Settings and Toggling states
  const [showNavLights, setShowNavLights] = useState(true);
  const [showCollision, setShowCollision] = useState(true);
  const [showDockpaths, setShowDockpaths] = useState(true);
  const [showEngineBurns, setShowEngineBurns] = useState(true);
  const [showBoneLines, setShowBoneLines] = useState(true);
  const [renderMode, setRenderMode] = useState<"untextured" | "textured" | "shaded" | "wireframe" | "shaded_team" | "textured_team">("shaded");
  const [teamColor, setTeamColor] = useState("#4278a3");
  const [stripeColor, setStripeColor] = useState("#e5d94c");
  const [keeperTxtPath, setKeeperTxtPath] = useState(() => localStorage.getItem("keeperTxtPath") || "");

  // Log application startup
  useEffect(() => {
    invoke("log_event", { level: "INFO", message: "HOD Remastered Editor React Frontend initialized." }).catch(console.error);
  }, []);

  // Log selected node changes
  useEffect(() => {
    if (selectedNode) {
      invoke("log_event", {
        level: "INFO",
        message: `User selected ${selectedNode.type} node: "${selectedNode.name}"`,
      }).catch(console.error);
    }
  }, [selectedNode]);

  // Open native OS file selector dialog via Rust host
  const selectAndLoadFile = async () => {
    if (isDirty) {
      const confirmDiscard = window.confirm("You have unsaved changes in the current HOD model. Are you sure you want to load another HOD and discard your changes?");
      if (!confirmDiscard) return;
    }
    try {
      const selectedPath = await invoke<string | null>("select_hod_file");
      if (selectedPath) {
        loadHODFile(selectedPath);
      }
    } catch (e: any) {
      const err = "Failed to open file dialog: " + e.toString();
      invoke("log_event", { level: "ERROR", message: err }).catch(console.error);
      setErrorMsg(err);
      setStatusMsg("File browser error");
    }
  };

  // Open native OS file selector dialog for DAE
  const selectAndImportDAE = async () => {
    if (isDirty) {
      const confirmDiscard = window.confirm("You have unsaved changes. Are you sure you want to load another model and discard your changes?");
      if (!confirmDiscard) return;
    }
    try {
      const selectedPath = await invoke<string | null>("select_dae_file");
      if (selectedPath) {
        setStatusMsg("Parsing DAE file...");
        setIsLoading(true);
        setErrorMsg(null);
        try {
          const parsedModel = await invoke<HODModel>("import_dae_file", { path: selectedPath });
          setModel(parsedModel);
          setFilePath(""); // No original path, it's newly imported
          setIsDirty(true);
          setSelectedNode(null);
          setStatusMsg("Successfully imported DAE as HOD 2.0");
        } catch (e: any) {
          const err = "Failed to import DAE: " + e.toString();
          invoke("log_event", { level: "ERROR", message: err }).catch(console.error);
          setErrorMsg(err);
          setStatusMsg("Import error");
        } finally {
          setIsLoading(false);
        }
      }
    } catch (e: any) {
      const err = "Failed to open DAE file dialog: " + e.toString();
      invoke("log_event", { level: "ERROR", message: err }).catch(console.error);
      setErrorMsg(err);
      setStatusMsg("File browser error");
    }
  };

  // Open native OS file selector dialog for keeper.txt
  const selectAndSaveKeeperPath = async () => {
    try {
      const selectedPath = await invoke<string | null>("select_keeper_file");
      if (selectedPath) {
        let dirPath = selectedPath;
        if (selectedPath.toLowerCase().endsWith("keeper.txt")) {
          dirPath = selectedPath.substring(0, selectedPath.toLowerCase().lastIndexOf("keeper.txt"));
          if (dirPath.endsWith("/") || dirPath.endsWith("\\")) {
            dirPath = dirPath.substring(0, dirPath.length - 1);
          }
        }
        setKeeperTxtPath(dirPath);
        localStorage.setItem("keeperTxtPath", dirPath);
        setStatusMsg("Successfully linked and persisted uncompressed directory!");
      }
    } catch (e: any) {
      const err = "Failed to select keeper.txt file: " + e.toString();
      invoke("log_event", { level: "ERROR", message: err }).catch(console.error);
      setStatusMsg("Keeper browser error");
    }
  };

  const resetUIState = () => {
    setModel(null);
    setSelectedNode(null);
    setSelectedAnimIdx(0);
    setTargetBoxes([]);
    setCurrentTime(0);
    setIsPlaying(false);
    setVisibleMeshes({});
    setMigrationMappings({});
    setErrorMsg(null);
  };

  // Load targeted HOD file from absolute filesystem path
  const loadHODFile = async (path: string) => {
    if (!path.trim()) return;
    resetUIState();
    setIsLoading(true);
    setStatusMsg("Loading HOD...");
    
    // Give the browser 100ms of breathing room to paint the gorgeous loading overlay!
    setTimeout(async () => {
      try {
        const parsedModel = await invoke<HODModel>("load_hod", { 
          filePath: path, 
          keeperPath: keeperTxtPath || null 
        });
        
        // Ensure collections are arrays to prevent undefined issues
        const processedModel: HODModel = {
          ...parsedModel,
          nav_lights: parsedModel.nav_lights || [],
          engine_burns: parsedModel.engine_burns || [],
          engine_glows: parsedModel.engine_glows || [],
          engine_shapes: parsedModel.engine_shapes || [],
          collision_meshes: parsedModel.collision_meshes || [],
          dockpaths: parsedModel.dockpaths || [],
        };

        const sanitizeNavLightChildren = (loadedModel: HODModel): HODModel => {
          let m = { ...loadedModel };
          const navlightNames = new Set(m.nav_lights.map(n => n.name));
          if (navlightNames.size === 0) return m;

          const getNavlightInfo = (navName: string) => {
            const joint = m.joints.find(j => j.name === navName);
            return {
              joint,
              parentName: joint?.parent_name || "Root",
              transform: joint?.local_transform || { m: [[1,0,0,0],[0,1,0,0],[0,0,1,0],[0,0,0,1]] }
            };
          };

          m.joints = m.joints.map(j => {
            if (j.parent_name && navlightNames.has(j.parent_name)) {
              const info = getNavlightInfo(j.parent_name);
              const parentMat = info.transform.m;
              const childMat = j.local_transform.m;
              const newMat = [
                [0,0,0,0],[0,0,0,0],[0,0,0,0],[0,0,0,0]
              ];
              for (let r = 0; r < 4; r++) {
                for (let c = 0; c < 4; c++) {
                  newMat[r][c] = parentMat[r][0]*childMat[0][c] + parentMat[r][1]*childMat[1][c] + parentMat[r][2]*childMat[2][c] + parentMat[r][3]*childMat[3][c];
                }
              }
              return {
                ...j,
                parent_name: info.parentName,
                local_transform: { m: newMat }
              };
            }
            return j;
          });

          const createProxyJoint = (childName: string, navName: string, prefix: string) => {
            const info = getNavlightInfo(navName);
            const newJointName = `${prefix}_${childName}_from_${navName}`;
            if (!m.joints.some(j => j.name === newJointName)) {
              m.joints.push({
                name: newJointName,
                parent_name: info.parentName,
                local_transform: JSON.parse(JSON.stringify(info.transform))
              });
              invoke("log_event", { level: "INFO", message: `Sanitized: Created proxy joint ${newJointName} to decouple ${childName} from navlight ${navName}` }).catch(console.error);
            }
            return newJointName;
          };

          m.engine_burns = m.engine_burns.map(b => {
            if (navlightNames.has(b.parent_name)) return { ...b, parent_name: createProxyJoint(b.name, b.parent_name, "BurnProxy") };
            return b;
          });

          m.engine_glows = m.engine_glows.map(g => {
            if (navlightNames.has(g.parent_name)) return { ...g, parent_name: createProxyJoint(g.name, g.parent_name, "GlowProxy") };
            return g;
          });

          m.engine_shapes = m.engine_shapes.map(s => {
            if (navlightNames.has(s.parent_name)) return { ...s, parent_name: createProxyJoint(s.name, s.parent_name, "ShapeProxy") };
            return s;
          });

          m.meshes = m.meshes.map(mesh => {
            if (navlightNames.has(mesh.parent_name)) return { ...mesh, parent_name: createProxyJoint(mesh.name, mesh.parent_name, "MeshProxy") };
            return mesh;
          });

          m.markers = m.markers.map(mrk => {
            if (navlightNames.has(mrk.parent_joint)) return { ...mrk, parent_joint: createProxyJoint(mrk.name, mrk.parent_joint, "MarkerProxy") };
            return mrk;
          });
          
          m.dockpaths = m.dockpaths.map(dp => {
            if (navlightNames.has(dp.parent_name)) return { ...dp, parent_name: createProxyJoint(dp.name, dp.parent_name, "DockpathProxy") };
            return dp;
          });

          m.collision_meshes = m.collision_meshes.map(col => {
            if (col.mesh && navlightNames.has(col.mesh.parent_name)) {
              return { ...col, mesh: { ...col.mesh, parent_name: createProxyJoint(col.name, col.mesh.parent_name, "CollisionProxy") } };
            }
            return col;
          });

          return m;
        };

        const sanitizedModel = sanitizeNavLightChildren(processedModel);
        setModel(sanitizedModel);
        setIsDirty(false);
        setSelectedNode(null);
        setSelectedAnimIdx(0);
        setFilePath(path);
        
        // Initialize only LOD 0 as visible by default, hiding lower-poly LOD 1, 2, 3 overlays
        const initialVisibility: Record<string, boolean> = {};
        sanitizedModel.meshes.forEach((m) => {
          initialVisibility[`${m.name}_lod_${m.lod}`] = m.lod === 0;
        });
        setVisibleMeshes(initialVisibility);

        setStatusMsg(`HOD ${sanitizedModel.name} loaded successfully | Meshes: ${sanitizedModel.meshes.length} | Joints: ${sanitizedModel.joints.length} | Markers: ${sanitizedModel.markers.length}`);
      } catch (e: any) {
        const err = `Frontend failed to load HOD from path ${path}: ${e.toString()}`;
        invoke("log_event", { level: "ERROR", message: err }).catch(console.error);
        setErrorMsg(e.toString());
        setStatusMsg("Error loading file");
      } finally {
        setIsLoading(false);
      }
    }, 100);
  };

  const handleCreateNewHOD = () => {
    if (isDirty) {
      const confirmDiscard = window.confirm("You have unsaved changes in the current HOD model. Are you sure you want to create a new HOD and discard your changes?");
      if (!confirmDiscard) return;
    }
    invoke("log_event", { level: "INFO", message: "Creating fresh, clean HOD 2.0 template..." }).catch(console.error);
    resetUIState();
    
    const newModel: HODModel = {
      version: 512, // HOD 2.0 (0x200)
      name: "New_Model",
      is_v2: true,
      joints: [
        {
          name: "Root",
          parent_name: undefined,
          local_transform: {
            m: [
              [1, 0, 0, 0],
              [0, 1, 0, 0],
              [0, 0, 1, 0],
              [0, 0, 0, 1]
            ]
          }
        }
      ],
      markers: [],
      meshes: [],
      nav_lights: [],
      materials: [],
      textures: [],
      collision_meshes: [],
      dockpaths: [],
      engine_burns: [],
      engine_glows: [],
      engine_shapes: [],
      animations: []
    };
    
    // Slight delay to ensure React unmounts previous Viewport and clears WebGL context
    setTimeout(() => {
      setModel(newModel);
      setFilePath("");
      setIsDirty(false);
      setStatusMsg("New HOD model successfully initialized!");
    }, 50);
  };

  const handleExecuteMigration = () => {
    if (!model) return;
    
    let updatedJoints = [...model.joints];
    let updatedMarkers = [...model.markers];
    let updatedCollisions = [...model.collision_meshes];
    let updatedNavlights = [...model.nav_lights];
    let updatedEngineBurns = [...model.engine_burns];
    let updatedEngineGlows = [...model.engine_glows];
    
    // Process reclassifications
    Object.entries(migrationMappings).forEach(([jointName, targetType]) => {
      // Find the original joint being mapped
      let originalJoint = model.joints.find(j => j.name === jointName);
      
      // If the user requested to clone coordinates from another joint, overwrite the transform temporarily
      const cloneTarget = migrationCoordinateClones[jointName];
      if (cloneTarget && cloneTarget !== "Self" && originalJoint) {
        const sourceJoint = model.joints.find(j => j.name === cloneTarget);
        if (sourceJoint) {
           // We clone the local_transform and position from the source joint
           originalJoint = { 
             ...originalJoint, 
             local_transform: JSON.parse(JSON.stringify(sourceJoint.local_transform)),
             position: sourceJoint.position ? { ...sourceJoint.position } : undefined,
           };
        }
      }

      if (targetType === "marker") {
        // Convert to marker:
        if (originalJoint) {
          // Decompose its transform for position:
          const m = originalJoint.local_transform.m;
          const pos = { x: m[3][0], y: m[3][1], z: m[3][2] };
          
          updatedMarkers.push({
            name: jointName,
            parent_joint: originalJoint.parent_name || "Root",
            position: pos,
            rotation: {
              m: [
                [1, 0, 0, 0],
                [0, 1, 0, 0],
                [0, 0, 1, 0],
                [0, 0, 0, 1]
              ]
            }
          });
          // Remove from joints list:
          updatedJoints = updatedJoints.filter(j => j.name !== jointName);
        }
      } else if (targetType === "weapon") {
        // Convert to standard 4-joint weapon assembly:
        if (originalJoint) {
          const m = originalJoint.local_transform.m;
          const parent = originalJoint.parent_name || "Root";
          const base = jointName.startsWith("Weapon_") || jointName.startsWith("weapon_") ? jointName : `Weapon_${jointName}`;
          
          // Add weapon joints
          updatedJoints.push({
            name: `${base}_Position`,
            parent_name: parent,
            local_transform: { m }
          }, {
            name: `${base}_Direction`,
            parent_name: `${base}_Position`,
            local_transform: {
              m: [
                [1, 0, 0, 0],
                [0, 1, 0, 0],
                [0, 0, 1, 0],
                [0, 0, 0, 1]
              ]
            }
          }, {
            name: `${base}_Muzzle`,
            parent_name: `${base}_Direction`,
            local_transform: {
              m: [
                [1, 0, 0, 0],
                [0, 1, 0, 0],
                [0, 0, 1, 0],
                [0, 0, 0, 1.5]
              ]
            }
          }, {
            name: `${base}_Rest`,
            parent_name: `${base}_Position`,
            local_transform: {
              m: [
                [1, 0, 0, 0],
                [0, 1, 0, 0],
                [0, 0, 1, 0],
                [0, 0, 0, 1.0]
              ]
            }
          });
          // Remove original joint:
          updatedJoints = updatedJoints.filter(j => j.name !== jointName);
        }
      } else if (targetType === "collision") {
        // Convert to collision:
        if (originalJoint) {
          const m = originalJoint.local_transform.m;
          const pos = { x: m[3][0], y: m[3][1], z: m[3][2] };
          
          updatedCollisions.push({
            name: jointName,
            min_extents: { x: -3, y: -3, z: -3 },
            max_extents: { x: 3, y: 3, z: 3 },
            center: pos,
            radius: 3.0,
            mesh: {
              name: jointName,
              parent_name: "Root",
              lod: 0,
              parts: []
            }
          });
        }
      } else if (targetType === "navlight") {
        if (originalJoint) {
          // If cloned coordinates were requested, we must actually modify the joint in updatedJoints
          // because Navlights derive position from the joint directly.
          if (cloneTarget && cloneTarget !== "Self") {
             const index = updatedJoints.findIndex(j => j.name === jointName);
             if (index !== -1) {
                updatedJoints[index].local_transform = JSON.parse(JSON.stringify(originalJoint.local_transform));
                if (originalJoint.position) updatedJoints[index].position = { ...originalJoint.position };
             }
          }
          updatedNavlights.push({
            name: jointName,
            section: 0,
            size: 1.0,
            phase: 0.0,
            frequency: 1.0,
            style: "default",
            color: { x: 1, y: 1, z: 1 },
            distance: 100.0,
            sprite_visible: true,
            high_end_only: false
          });
          // Do NOT remove the original joint! Navlights implicitly bind to a Joint by name to extract their position!
        }
      } else if (targetType === "engine_burn") {
        if (originalJoint) {
          updatedEngineBurns.push({
            name: jointName,
            parent_name: originalJoint.parent_name || "Root",
            num_divisions: 1,
            num_flames: 1,
            vertices: []
          });
          updatedJoints = updatedJoints.filter(j => j.name !== jointName);
        }
      } else if (targetType === "engine_glow") {
        const originalJoint = model.joints.find(j => j.name === jointName);
        if (originalJoint) {
          updatedEngineGlows.push({
            name: jointName,
            parent_name: originalJoint.parent_name || "Root",
            lod: 0,
            mesh: {
              name: jointName,
              parent_name: "Root",
              lod: 0,
              parts: []
            }
          });
          updatedJoints = updatedJoints.filter(j => j.name !== jointName);
        }
      }
    });

    // Save final migrated HOD v2.0 model state:
    const upgradedModel = {
      ...model,
      is_v2: true,
      version: 512, // HOD 2.0 (0x200)
      joints: updatedJoints,
      markers: updatedMarkers,
      collision_meshes: updatedCollisions,
      nav_lights: updatedNavlights,
      engine_burns: updatedEngineBurns,
      engine_glows: updatedEngineGlows,
    };

    setModel(upgradedModel);
    setIsDirty(true);
    setIsMigrationModalOpen(false);
    setStatusMsg("Model successfully auto-transformed to HOD 2.0 with customized schema reclassifications!");
    alert("Schema migration complete! Un-standardized joints have been promoted, and the file layout upgraded to modern HOD 2.0.");
  };

  // Update selected joint/marker/navlight/dockpoint/collision position in model state
  const handleReParentNode = (nodeName: string, nodeType: string, newParentName: string) => {
    if (!model) return;

    invoke("log_event", { 
      level: "INFO", 
      message: `Re-parenting HOD node "${nodeName}" (type: ${nodeType}) under parent joint "${newParentName}"` 
    }).catch(console.error);

    let updatedModel = { ...model };

    if (nodeType === "joint") {
      // Prevent cyclical parenting (cannot parent a joint under itself or its descendants)
      const isDescendant = (parent: string, child: string): boolean => {
        if (parent === child) return true;
        const currentJoint = model.joints.find(j => j.name === parent);
        if (!currentJoint || !currentJoint.parent_name || currentJoint.parent_name === "Root") return false;
        return isDescendant(currentJoint.parent_name, child);
      };

      if (isDescendant(newParentName, nodeName)) {
        setErrorMsg("Error: Cyclical parenting is not allowed (cannot parent a joint bone under its own descendants).");
        return;
      }

      updatedModel.joints = model.joints.map((joint) => {
        if (joint.name === nodeName) {
          return { ...joint, parent_name: newParentName };
        }
        return joint;
      });
    } else if (nodeType === "marker") {
      updatedModel.markers = model.markers.map((marker) => {
        if (marker.name === nodeName) {
          return { ...marker, parent_joint: newParentName };
        }
        return marker;
      });
    } else if (nodeType === "mesh") {
      // In HOD, meshKey contains '_lod_[number]', we extract the base mesh name
      const baseMeshName = nodeName.split("_lod_")[0] || nodeName;
      updatedModel.meshes = model.meshes.map((mesh) => {
        if (mesh.name === baseMeshName) {
          return { ...mesh, parent_name: newParentName };
        }
        return mesh;
      });
    } else if (nodeType === "engine_burn") {
      updatedModel.engine_burns = model.engine_burns.map((burn) => {
        if (burn.name === nodeName) {
          return { ...burn, parent_name: newParentName };
        }
        return burn;
      });
    } else if (nodeType.endsWith("_group")) {
      const groupJoints = model.joints.filter(j => j.name.toLowerCase().startsWith(nodeName.toLowerCase() + "_") || j.name.toLowerCase() === nodeName.toLowerCase());
      const rootGroupJoints = groupJoints.filter(j => {
        const hasParentInGroup = groupJoints.some(other => other.name === j.parent_name);
        return !hasParentInGroup;
      });

      // Prevent cyclical parenting for assemblies
      const isDescendant = (parent: string, child: string): boolean => {
        if (parent === child) return true;
        const currentJoint = model.joints.find(j => j.name === parent);
        if (!currentJoint || !currentJoint.parent_name || currentJoint.parent_name === "Root") return false;
        return isDescendant(currentJoint.parent_name, child);
      };

      for (const root of rootGroupJoints) {
         if (isDescendant(newParentName, root.name)) {
            setErrorMsg("Error: Cyclical parenting is not allowed (cannot parent an assembly under its own descendants).");
            return;
         }
      }

      updatedModel.joints = model.joints.map((joint) => {
        if (rootGroupJoints.some(rj => rj.name === joint.name)) {
          return { ...joint, parent_name: newParentName };
        }
        return joint;
      });
    } else if (nodeType === "navlight") {
      updatedModel.joints = model.joints.map((joint) => {
        if (joint.name === nodeName) {
          return { ...joint, parent_name: newParentName };
        }
        return joint;
      });
    }

    updateModel(updatedModel);
    setStatusMsg(`Successfully re-parented ${nodeName} under ${newParentName}`);
  };

  // Update selected joint/marker/navlight/dockpoint/collision position in model state
  const handleNodeTransform = (name: string, type: string, pos: Vector3D) => {
    if (!model) return;

    if (type === "joint") {
      const updatedJoints = model.joints.map((joint) => {
        if (joint.name === name) {
          const m = joint.local_transform.m.map(row => [...row]);
          m[3][0] = pos.x;
          m[3][1] = pos.y;
          m[3][2] = pos.z;
          return { ...joint, local_transform: { m } };
        }
        return joint;
      });
      updateModel({ ...model, joints: updatedJoints });
    } else if (type === "marker") {
      const updatedMarkers = model.markers.map((marker) => {
        if (marker.name === name) {
          return { ...marker, position: pos };
        }
        return marker;
      });
      updateModel({ ...model, markers: updatedMarkers });
    } else if (type === "navlight") {
      // Translating NavLight translates its matching joint bone
      const updatedJoints = model.joints.map((joint) => {
        if (joint.name === name) {
          const m = joint.local_transform.m.map(row => [...row]);
          m[3][0] = pos.x;
          m[3][1] = pos.y;
          m[3][2] = pos.z;
          return { ...joint, local_transform: { m } };
        }
        return joint;
      });
      updateModel({ ...model, joints: updatedJoints });
    } else if (type.endsWith("_group")) {
      const updatedJoints = model.joints.map((joint) => {
        if (joint.name.toLowerCase() === `${name}_Position`.toLowerCase()) {
          const m = joint.local_transform.m.map(row => [...row]);
          m[3][0] = pos.x;
          m[3][1] = pos.y;
          m[3][2] = pos.z;
          return { ...joint, local_transform: { m } };
        }
        return joint;
      });
      updateModel({ ...model, joints: updatedJoints });
    } else if (type === "dockpoint") {
      const [pathName, ptIdxStr] = name.split(":");
      const ptIdx = parseInt(ptIdxStr, 10);
      const updatedPaths = model.dockpaths.map((dp) => {
        if (dp.name === pathName) {
          const updatedPoints = dp.points.map((pt, idx) => {
            if (idx === ptIdx) {
              return { ...pt, position: pos };
            }
            return pt;
          });
          return { ...dp, points: updatedPoints };
        }
        return dp;
      });
      updateModel({ ...model, dockpaths: updatedPaths });
    } else if (type === "collision") {
      const updatedCols = model.collision_meshes.map((col) => {
        if (col.name === name) {
          return { ...col, center: pos };
        }
        return col;
      });
      updateModel({ ...model, collision_meshes: updatedCols });
    }
    
    setStatusMsg(`Moved ${type} ${name} to [${pos.x.toFixed(2)}, ${pos.y.toFixed(2)}, ${pos.z.toFixed(2)}]`);
  };

  const handleSaveHOD = async () => {
    if (!model) return;
    
    if (!filePath) {
      handleSaveHODAs();
      return;
    }

    setIsSaving(true);
    setStatusMsg("Saving HOD...");
    setErrorMsg(null);

    try {
      // Phase 4: Trigger native Rust HOD v2 compression and writer
      await invoke("save_hod", { filePath, model });
      setIsDirty(false);
      invoke("log_event", { level: "INFO", message: `Successfully saved patched HOD to path: ${filePath}` }).catch(console.error);
      setStatusMsg("HOD file saved successfully");
    } catch (e: any) {
      const err = `Frontend failed to save HOD to path ${filePath}: ${e.toString()}`;
      invoke("log_event", { level: "ERROR", message: err }).catch(console.error);
      setErrorMsg(e.toString());
      setStatusMsg("Error saving file");
    } finally {
      setIsSaving(false);
    }
  };

  const handleSaveHODAs = async () => {
    if (!model) {
      alert("Please load or create a HOD model first.");
      return;
    }
    
    try {
      const defaultName = filePath ? filePath.split(/[/\\]/).pop() : `${model.name}.hod`;
      const selectedPath = await invoke<string | null>("select_save_hod_file", { defaultName });
      if (!selectedPath) return;

      setIsSaving(true);
      setStatusMsg("Saving HOD As...");
      setErrorMsg(null);

      await invoke("save_hod_as", { sourcePath: filePath || "", targetPath: selectedPath, model });
      
      setFilePath(selectedPath);
      setIsDirty(false);
      invoke("log_event", { level: "INFO", message: `Successfully saved patched HOD as a new file at: ${selectedPath}` }).catch(console.error);
      setStatusMsg("HOD file saved as new file successfully");
      alert(`HOD file saved successfully to new path:\n${selectedPath}`);
    } catch (e: any) {
      const err = `Frontend failed to Save HOD As: ${e.toString()}`;
      invoke("log_event", { level: "ERROR", message: err }).catch(console.error);
      setErrorMsg(e.toString());
      setStatusMsg("Error saving file as new path");
      alert(`Save As failed: ${e.toString()}`);
    } finally {
      setIsSaving(false);
    }
  };

  return (
    <div className="app-container">
      {isLoading && (
        <div style={{
          position: "fixed",
          top: 0,
          left: 0,
          right: 0,
          bottom: 0,
          background: "rgba(3, 6, 10, 0.85)",
          backdropFilter: "blur(8px)",
          zIndex: 9999,
          display: "flex",
          flexDirection: "column",
          justifyContent: "center",
          alignItems: "center",
          gap: "20px"
        }}>
          <div style={{
            width: "50px",
            height: "50px",
            border: "3px solid rgba(22, 160, 255, 0.1)",
            borderTop: "3px solid var(--accent-cyan)",
            borderRadius: "50%",
            animation: "spin 1s linear infinite"
          }} />
          <div style={{
            fontSize: "14px",
            fontWeight: "600",
            color: "var(--accent-cyan)",
            letterSpacing: "0.15em",
            textTransform: "uppercase",
            animation: "pulse 1.5s ease-in-out infinite"
          }}>
            Decompressing & Loading HOD Asset
          </div>
          <div style={{
            fontSize: "11px",
            color: "var(--text-muted)",
            fontFamily: "var(--font-mono)",
            maxWidth: "400px",
            textAlign: "center",
            lineHeight: "1.4"
          }}>
            Please wait...
          </div>
        </div>
      )}
      {/* Top Header Toolbar */}
      <Toolbar
        modelName={model ? `${model.name} (${filePath ? filePath.split("/").pop() : "unsaved_file"})` : ""}
        onOpenClick={selectAndLoadFile}
        onSaveClick={handleSaveHOD}
        onSaveAsClick={handleSaveHODAs}
        isSaving={isSaving}
        onNewClick={handleCreateNewHOD}
        onImportDAEClick={selectAndImportDAE}
      />

      {/* Main Workspace Panels */}
      <div className="workspace" style={{ gridTemplateColumns: `${sidebarWidth}px 1px 1fr 1px ${inspectorWidth}px` }}>
        {/* Left Side Hierarchy Node Tree */}
        <HierarchyTree
          model={model}
          selectedNode={selectedNode}
          setSelectedNode={setSelectedNode}
          visibleMeshes={visibleMeshes}
          setVisibleMeshes={setVisibleMeshes}
          onReParentNode={handleReParentNode}
          onModelChange={setModel}
          selectedAnimIdx={selectedAnimIdx}
          setSelectedAnimIdx={setSelectedAnimIdx}
          targetBoxes={targetBoxes}
          setTargetBoxes={setTargetBoxes}
        />

        {/* Sidebar Drag Separator Divider */}
        <div 
          onMouseDown={handleSidebarDragStart}
          style={{
            width: "4px",
            marginLeft: "-2px",
            background: "transparent",
            cursor: "col-resize",
            zIndex: 100,
            height: "100%",
            transition: "background 0.2s"
          }}
          className="divider-hover"
        />

        {/* Center Viewport rendering */}
        <div style={{ position: "relative", height: "100%", overflow: "hidden", display: "flex", flexDirection: "column", flex: 1 }}>
          {model ? (
            <div style={{ position: "relative", width: "100%", height: "100%", flex: 1, minHeight: 0 }}>
              {!keeperTxtPath && (
                <div
                  onClick={selectAndSaveKeeperPath}
                  style={{
                    position: "absolute",
                    top: "12px",
                    left: "50%",
                    transform: "translateX(-50%)",
                    background: "rgba(255, 152, 0, 0.15)",
                    border: "1px solid #ff9800",
                    borderRadius: "6px",
                    padding: "6px 16px",
                    display: "flex",
                    alignItems: "center",
                    gap: "8px",
                    fontSize: "11px",
                    color: "#ffe0b2",
                    zIndex: 10,
                    backdropFilter: "blur(6px)",
                    boxShadow: "0 4px 12px rgba(0,0,0,0.5)",
                    cursor: "pointer",
                    userSelect: "none"
                  }}
                  title="Click to select keeper.txt and set your game data directory"
                >
                  <AlertTriangle size={14} style={{ color: "#ffb74d" }} />
                  <span>Uncompressed .big path is not set. Click here to configure.</span>
                </div>
              )}

              {/* Floating Viewport Overlays panel */}
              <div
                style={{
                  position: "absolute",
                  top: "12px",
                  right: "12px",
                  background: "rgba(13, 22, 37, 0.85)",
                  border: "1px solid var(--border-color)",
                  borderRadius: "6px",
                  padding: "12px 14px",
                  zIndex: 10,
                  backdropFilter: "blur(8px)",
                  boxShadow: "0 4px 16px rgba(0,0,0,0.6)",
                  display: "flex",
                  flexDirection: "column",
                  gap: "10px",
                  textAlign: "left",
                  width: "180px"
                }}
              >
                <div style={{ fontSize: "11px", fontWeight: "600", textTransform: "uppercase", letterSpacing: "0.08em", color: "var(--accent-cyan)", marginBottom: "4px" }}>
                  Viewport Layers
                </div>
                <label style={{ display: "flex", alignItems: "center", justifyContent: "flex-start", gap: "10px", fontSize: "12px", color: "var(--text-primary)", cursor: "pointer", userSelect: "none", width: "100%" }}>
                  <input
                    type="checkbox"
                    checked={showNavLights}
                    onChange={(e) => setShowNavLights(e.target.checked)}
                    style={{ width: "auto", height: "auto", margin: 0, cursor: "pointer" }}
                  />
                  <span>NavLights</span>
                </label>
                <label style={{ display: "flex", alignItems: "center", justifyContent: "flex-start", gap: "10px", fontSize: "12px", color: "var(--text-primary)", cursor: "pointer", userSelect: "none", width: "100%" }}>
                  <input
                    type="checkbox"
                    checked={showCollision}
                    onChange={(e) => setShowCollision(e.target.checked)}
                    style={{ width: "auto", height: "auto", margin: 0, cursor: "pointer" }}
                  />
                  <span>Collision Hulls</span>
                </label>
                <label style={{ display: "flex", alignItems: "center", justifyContent: "flex-start", gap: "10px", fontSize: "12px", color: "var(--text-primary)", cursor: "pointer", userSelect: "none", width: "100%" }}>
                  <input
                    type="checkbox"
                    checked={showDockpaths}
                    onChange={(e) => setShowDockpaths(e.target.checked)}
                    style={{ width: "auto", height: "auto", margin: 0, cursor: "pointer" }}
                  />
                  <span>Dockpaths</span>
                </label>
                <label style={{ display: "flex", alignItems: "center", justifyContent: "flex-start", gap: "10px", fontSize: "12px", color: "var(--text-primary)", cursor: "pointer", userSelect: "none", width: "100%" }}>
                  <input
                    type="checkbox"
                    checked={showEngineBurns}
                    onChange={(e) => setShowEngineBurns(e.target.checked)}
                    style={{ width: "auto", height: "auto", margin: 0, cursor: "pointer" }}
                  />
                  <span>Engine Burns</span>
                </label>
                <label style={{ display: "flex", alignItems: "center", justifyContent: "flex-start", gap: "10px", fontSize: "12px", color: "var(--text-primary)", cursor: "pointer", userSelect: "none", width: "100%" }}>
                  <input
                    type="checkbox"
                    checked={showBoneLines}
                    onChange={(e) => setShowBoneLines(e.target.checked)}
                    style={{ width: "auto", height: "auto", margin: 0, cursor: "pointer" }}
                  />
                  <span>Node Hierarchy Lines</span>
                </label>

                <div style={{ display: "flex", flexDirection: "column", gap: "4px", marginTop: "4px", borderTop: "1px solid var(--border-color)", paddingTop: "8px" }}>
                  <div style={{ fontSize: "10px", fontWeight: "600", textTransform: "uppercase", letterSpacing: "0.08em", color: "var(--accent-cyan)", marginBottom: "2px" }}>
                    Render Style
                  </div>
                  <select
                    value={renderMode}
                    onChange={(e) => setRenderMode(e.target.value as any)}
                    style={{ height: "26px", padding: "2px 6px", fontSize: "11px", background: "rgba(13, 22, 37, 0.9)", width: "100%", borderRadius: "4px" }}
                  >
                    <option value="shaded">Textured Shaded (Raw)</option>
                    <option value="shaded_team">Textured Shaded (Team Paint)</option>
                    <option value="textured">Textured Flat (Raw)</option>
                    <option value="textured_team">Textured Flat (Team Paint)</option>
                    <option value="untextured">Untextured Solid</option>
                    <option value="wireframe">Wireframe</option>
                  </select>
                </div>

                {(renderMode === "shaded_team" || renderMode === "textured_team") && (
                  <div style={{ display: "flex", flexDirection: "column", gap: "6px", marginTop: "4px", borderTop: "1px solid var(--border-color)", paddingTop: "8px" }}>
                    <div style={{ fontSize: "10px", fontWeight: "600", textTransform: "uppercase", letterSpacing: "0.08em", color: "var(--accent-cyan)", marginBottom: "2px" }}>
                      Team Colors
                    </div>
                    <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: "8px" }}>
                      <div style={{ display: "flex", flexDirection: "column", gap: "2px" }}>
                        <span style={{ fontSize: "9px", color: "var(--text-muted)", textTransform: "uppercase" }}>Primary</span>
                        <div style={{ display: "flex", alignItems: "center", gap: "4px", background: "rgba(0,0,0,0.3)", borderRadius: "4px", padding: "2px 4px", border: "1px solid var(--border-color)" }}>
                          <input
                            type="color"
                            value={teamColor}
                            onChange={(e) => setTeamColor(e.target.value)}
                            style={{ width: "14px", height: "14px", border: "none", padding: 0, cursor: "pointer", background: "none" }}
                          />
                          <span style={{ fontSize: "9px", fontFamily: "var(--font-mono)", color: "var(--text-primary)" }}>{teamColor.toUpperCase()}</span>
                        </div>
                      </div>
                      <div style={{ display: "flex", flexDirection: "column", gap: "2px" }}>
                        <span style={{ fontSize: "9px", color: "var(--text-muted)", textTransform: "uppercase" }}>Stripe</span>
                        <div style={{ display: "flex", alignItems: "center", gap: "4px", background: "rgba(0,0,0,0.3)", borderRadius: "4px", padding: "2px 4px", border: "1px solid var(--border-color)" }}>
                          <input
                            type="color"
                            value={stripeColor}
                            onChange={(e) => setStripeColor(e.target.value)}
                            style={{ width: "14px", height: "14px", border: "none", padding: 0, cursor: "pointer", background: "none" }}
                          />
                          <span style={{ fontSize: "9px", fontFamily: "var(--font-mono)", color: "var(--text-primary)" }}>{stripeColor.toUpperCase()}</span>
                        </div>
                      </div>
                    </div>
                  </div>
                )}
              </div>
              <Viewport
                model={model}
                selectedNode={selectedNode}
                setSelectedNode={setSelectedNode}
                transformMode={transformMode}
                setTransformMode={setTransformMode}
                onNodeTransform={handleNodeTransform}
                visibleMeshes={visibleMeshes}
                showNavLights={showNavLights}
                showCollision={showCollision}
                showDockpaths={showDockpaths}
                showEngineBurns={showEngineBurns}
                showBoneLines={showBoneLines}
                renderMode={renderMode}
                teamColor={teamColor}
                stripeColor={stripeColor}
                onModelChange={updateModel}
                selectedAnimIdx={selectedAnimIdx}
                setSelectedAnimIdx={setSelectedAnimIdx}
                isPlaying={isPlaying}
                setIsPlaying={setIsPlaying}
                currentTime={currentTime}
                setCurrentTime={setCurrentTime}
                playbackSpeed={playbackSpeed}
                loopPlayback={loopPlayback}
                targetBoxes={targetBoxes}
              />
            </div>
          ) : (
            <div
              style={{
                width: "100%",
                height: "100%",
                display: "flex",
                flexDirection: "column",
                justifyContent: "center",
                alignItems: "center",
                background: "#03060a",
                gap: "16px",
                padding: "20px",
                textAlign: "center",
              }}
            >
              <Info size={40} style={{ color: "var(--border-color)", filter: "drop-shadow(var(--shadow-glow))" }} />
              <div style={{ fontSize: "18px", color: "var(--accent-cyan)", fontWeight: "600", letterSpacing: "0.05em" }}>
                Welcome to HOD Remastered Editor
              </div>
              <div style={{ color: "var(--text-muted)", fontSize: "13px", maxWidth: "450px", lineHeight: "1.6" }}>
                Browse or enter the absolute path of a Homeworld Remastered HOD file to load and edit its joint bones, markers, meshes, and textures natively.
              </div>
              
              {!keeperTxtPath && (
                <div
                  style={{
                    background: "rgba(255, 171, 0, 0.06)",
                    border: "1px solid #ffab00",
                    borderRadius: "8px",
                    padding: "16px 20px",
                    maxWidth: "500px",
                    textAlign: "left",
                    color: "#ffe0b2",
                    display: "flex",
                    flexDirection: "column",
                    gap: "10px",
                    marginTop: "8px"
                  }}
                >
                  <div style={{ display: "flex", alignItems: "center", gap: "10px", fontWeight: "600", fontSize: "14px", color: "#ffb74d" }}>
                    <AlertTriangle size={18} />
                    <span>HWRM Uncompressed .big Path Required</span>
                  </div>
                  <div style={{ fontSize: "12px", lineHeight: "1.5", color: "rgba(255, 255, 255, 0.85)" }}>
                    Configure your uncompressed game data directory (containing 'keeper.txt') to automatically render .TGA textures and high-fidelity shader materials.
                  </div>
                  <div style={{ display: "flex", gap: "8px", width: "100%" }}>
                    <input
                      placeholder="e.g. C:\Homeworld2\GBXTools\WorkshopTool\mod-tools\uncompressed"
                      value={keeperTxtPath}
                      onChange={(e) => {
                        const val = e.target.value;
                        setKeeperTxtPath(val);
                        localStorage.setItem("keeperTxtPath", val);
                      }}
                      style={{ height: "32px", fontSize: "12px", flex: 1, padding: "0 8px", borderRadius: "4px", background: "rgba(13, 22, 37, 0.6)", border: "1px solid var(--border-color)", color: "var(--text-primary)" }}
                    />
                    <button 
                      onClick={selectAndSaveKeeperPath}
                      style={{
                        height: "32px",
                        fontSize: "12px",
                        padding: "0 12px",
                        background: "#ff9800",
                        color: "#000",
                        border: "none",
                        borderRadius: "4px",
                        cursor: "pointer",
                        fontWeight: "600",
                        display: "flex",
                        alignItems: "center",
                        gap: "6px"
                      }}
                    >
                      <FolderOpen size={14} />
                      Browse keeper.txt...
                    </button>
                  </div>
                </div>
              )}

              <div style={{ display: "flex", flexDirection: "column", alignItems: "center", gap: "16px", marginTop: "12px", width: "100%", maxWidth: "500px" }}>
                <button 
                  className="secondary" 
                  onClick={handleCreateNewHOD} 
                  style={{ 
                    height: "42px", 
                    padding: "0 24px", 
                    display: "flex", 
                    alignItems: "center", 
                    gap: "10px", 
                    fontSize: "14px", 
                    fontWeight: "600", 
                    width: "100%", 
                    justifyContent: "center",
                    border: "1px dashed var(--accent-cyan)",
                    background: "rgba(22, 160, 255, 0.05)",
                    color: "var(--accent-cyan)",
                    cursor: "pointer"
                  }}
                >
                  <FilePlus size={18} />
                  <span>Start a New HOD Model</span>
                </button>

                <button className="primary" onClick={selectAndLoadFile} style={{ height: "42px", padding: "0 24px", display: "flex", alignItems: "center", gap: "10px", fontSize: "14px", fontWeight: "600", width: "100%", justifyContent: "center" }}>
                  <FolderOpen size={18} />
                  <span>Browse HOD File...</span>
                </button>
                
                <div style={{ display: "flex", alignItems: "center", gap: "8px", width: "100%" }}>
                  <div style={{ flex: 1, height: "1px", background: "rgba(255,255,255,0.08)" }} />
                  <span style={{ fontSize: "11px", color: "var(--text-muted)", textTransform: "uppercase", letterSpacing: "0.1em" }}>or enter path manually</span>
                  <div style={{ flex: 1, height: "1px", background: "rgba(255,255,255,0.08)" }} />
                </div>

                <div style={{ width: "100%", display: "flex", gap: "8px" }}>
                  <input
                    placeholder="e.g. /path/to/my_ship.hod"
                    value={filePath}
                    onChange={(e) => setFilePath(e.target.value)}
                    style={{ height: "36px", fontSize: "13px" }}
                  />
                  <button className="secondary" onClick={() => loadHODFile(filePath)} style={{ height: "36px" }}>
                    Load File
                  </button>
                </div>
              </div>

              {/* Error Message banner */}
              {errorMsg && (
                <div
                  style={{
                    marginTop: "20px",
                    background: "rgba(255, 23, 68, 0.1)",
                    border: "1px solid var(--accent-danger)",
                    borderRadius: "6px",
                    padding: "12px 16px",
                    display: "flex",
                    alignItems: "center",
                    gap: "12px",
                    color: "#ffcdd2",
                    fontSize: "13px",
                    maxWidth: "500px",
                    textAlign: "left",
                  }}
                >
                  <AlertTriangle size={20} style={{ color: "var(--accent-danger)", flexShrink: 0 }} />
                  <span>{errorMsg}</span>
                </div>
              )}
            </div>
          )}

          {/* Animation Dock — always visible below the model area */}
          {model && (
            <AnimationDock
              model={model}
              selectedAnimIdx={selectedAnimIdx}
              setSelectedAnimIdx={setSelectedAnimIdx}
              isPlaying={isPlaying}
              setIsPlaying={setIsPlaying}
              currentTime={currentTime}
              setCurrentTime={setCurrentTime}
              loopPlayback={loopPlayback}
              setLoopPlayback={setLoopPlayback}
              playbackSpeed={playbackSpeed}
              setPlaybackSpeed={setPlaybackSpeed}
              onModelChange={updateModel}
              selectedNode={selectedNode}
              onSelectedNodeChange={setSelectedNode}
            />
          )}
        </div>

        {/* Inspector Drag Separator Divider */}
        <div 
          onMouseDown={handleInspectorDragStart}
          style={{
            width: "4px",
            marginRight: "-2px",
            background: "transparent",
            cursor: "col-resize",
            zIndex: 100,
            height: "100%",
            transition: "background 0.2s"
          }}
          className="divider-hover"
        />

        {/* Right Side Detail Inspector */}
        <Inspector
          key={selectedNode ? `${selectedNode.type}:${selectedNode.name}` : "empty"}
          model={model}
          selectedNode={selectedNode}
          onPositionChange={handleNodeTransform}
          onModelChange={setModel}
          onSelectedNodeChange={setSelectedNode}
          filePath={filePath}
          selectedAnimIdx={selectedAnimIdx}
          visibleMeshes={visibleMeshes}
          onToggleVisibility={(meshKey) => setVisibleMeshes(prev => ({ ...prev, [meshKey]: prev[meshKey] === false ? true : false }))}
        />
      </div>

      {/* Schema Migration Assistant Modal */}
      {isMigrationModalOpen && model && (
        <div style={{
          position: "fixed",
          top: 0,
          left: 0,
          right: 0,
          bottom: 0,
          background: "rgba(3, 8, 16, 0.8)",
          backdropFilter: "blur(8px)",
          display: "flex",
          justifyContent: "center",
          alignItems: "center",
          zIndex: 2000,
          padding: "20px"
        }}>
          <div style={{
            background: "rgba(10, 20, 35, 0.98)",
            border: "1px solid rgba(22, 160, 255, 0.45)",
            borderRadius: "14px",
            width: "100%",
            maxWidth: "600px",
            boxShadow: "0 12px 48px rgba(0,0,0,0.7)",
            display: "flex",
            flexDirection: "column",
            overflow: "hidden",
            maxHeight: "90vh"
          }}>
            {/* Header */}
            <div style={{
              background: "linear-gradient(135deg, rgba(22, 160, 255, 0.2), transparent)",
              padding: "18px 24px",
              borderBottom: "1px solid var(--border-color)",
              display: "flex",
              justifyContent: "space-between",
              alignItems: "center"
            }}>
              <div style={{ display: "flex", alignItems: "center", gap: "10px" }}>
                <span>🚀</span>
                <span style={{ fontWeight: "700", fontSize: "16px", color: "var(--accent-cyan)", letterSpacing: "0.04em" }}>
                  HOD Schema Migration Assistant
                </span>
              </div>
              <button 
                onClick={() => setIsMigrationModalOpen(false)}
                style={{ background: "transparent", border: "none", color: "var(--text-muted)", fontSize: "18px", cursor: "pointer" }}
              >
                ✕
              </button>
            </div>

            {/* Content Area */}
            <div style={{ padding: "24px", overflowY: "auto", display: "flex", flexDirection: "column", gap: "16px", flex: 1 }}>
              <div style={{ fontSize: "13px", lineHeight: "1.6", color: "rgba(255, 255, 255, 0.85)" }}>
                The following joints in this legacy model do not align with modern Homeworld Remastered rigid joint schemas. To optimize layout performance, reduce rendering overhead, and prevent crashes, you can re-classify them:
              </div>

              {Object.keys(migrationMappings).length > 0 ? (
                <div style={{ display: "flex", flexDirection: "column", gap: "10px", maxHeight: "350px", overflowY: "auto", border: "1px solid var(--border-color)", borderRadius: "8px", background: "rgba(0,0,0,0.2)", padding: "10px" }}>
                  {Object.keys(migrationMappings).map((jointName) => {
                    const mappedType = migrationMappings[jointName];
                    const cloneTarget = migrationCoordinateClones[jointName] || "Self";
                    return (
                      <div key={jointName} style={{ display: "flex", justifyContent: "space-between", alignItems: "center", padding: "10px 14px", background: "rgba(22, 160, 255, 0.03)", border: "1px solid rgba(22, 160, 255, 0.1)", borderRadius: "6px", gap: "10px" }}>
                        <span style={{ fontSize: "13px", fontWeight: "600", color: "#fff", fontFamily: "var(--font-mono)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap", flex: 1 }}>
                          {jointName}
                        </span>
                        
                        <div style={{ display: "flex", flexDirection: "column", gap: "4px" }}>
                          <span style={{ fontSize: "10px", color: "var(--text-muted)" }}>Target Type:</span>
                          <select
                            value={mappedType}
                            onChange={(e: any) => {
                              const val = e.target.value;
                              setMigrationMappings(prev => ({ ...prev, [jointName]: val }));
                            }}
                            style={{ width: "160px", height: "30px", fontSize: "12px", background: "#050a12", border: "1px solid var(--border-color)", color: "#fff", borderRadius: "4px" }}
                          >
                            <option value="joint">Standard Joint Bone</option>
                            <option value="marker">Marker / Attachment Point</option>
                            <option value="weapon">Weapon Assembly (HWRM template)</option>
                            <option value="collision">Collision Hull Mesh</option>
                            <option value="navlight">Navlight (Standalone)</option>
                            <option value="engine_burn">Engine Burn</option>
                            <option value="engine_glow">Engine Glow</option>
                          </select>
                        </div>

                        <div style={{ display: "flex", flexDirection: "column", gap: "4px" }}>
                          <span style={{ fontSize: "10px", color: "var(--text-muted)" }}>Clone Transform From:</span>
                          <select
                            value={cloneTarget}
                            onChange={(e: any) => {
                              const val = e.target.value;
                              setMigrationCoordinateClones(prev => ({ ...prev, [jointName]: val }));
                            }}
                            style={{ width: "160px", height: "30px", fontSize: "12px", background: "#050a12", border: "1px solid var(--border-color)", color: "#fff", borderRadius: "4px" }}
                          >
                            <option value="Self">Self (Preserve)</option>
                            {model?.joints.map(j => (
                               <option key={j.name} value={j.name}>{j.name}</option>
                            ))}
                          </select>
                        </div>
                      </div>
                    );
                  })}
                </div>
              ) : (
                <div style={{ padding: "20px", background: "rgba(0, 230, 118, 0.1)", border: "1px solid #00e676", borderRadius: "8px", fontSize: "13px", color: "#b9f6ca", textAlign: "center" }}>
                  ✓ All joints conform cleanly to Homeworld Remastered standards! No ambiguous nodes detected.
                </div>
              )}

              <div style={{ padding: "12px 16px", background: "rgba(22, 160, 255, 0.05)", borderLeft: "4px solid var(--accent-cyan)", fontSize: "12px", color: "var(--text-secondary)", lineHeight: "1.5" }}>
                💡 <b>Mapping rules:</b> Selecting 'Marker' promotes joint coordinates into a clean HOD attachment marker. Selecting 'Weapon' auto-groups joints and repairs standard turret paths (Position, Direction, Muzzle, Rest) automatically.
              </div>
            </div>

            {/* Footer */}
            <div style={{
              padding: "16px 24px",
              background: "rgba(3, 8, 16, 0.5)",
              borderTop: "1px solid var(--border-color)",
              display: "flex",
              justifyContent: "flex-end",
              gap: "12px"
            }}>
              <button
                className="secondary"
                onClick={() => setIsMigrationModalOpen(false)}
                style={{ padding: "8px 18px", fontSize: "12px", borderRadius: "4px" }}
              >
                Cancel
              </button>
              <button
                className="primary"
                onClick={handleExecuteMigration}
                style={{
                  padding: "8px 18px",
                  fontSize: "12px",
                  borderRadius: "4px",
                  background: "linear-gradient(135deg, var(--accent-cyan), #00e676)",
                  color: "#000",
                  fontWeight: "700",
                  border: "none",
                  cursor: "pointer"
                }}
              >
                Confirm and Convert to HOD 2.0
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Bottom Status bar */}
      <footer className="statusbar">
        <span>STATUS: {statusMsg}</span>
        <span>ENGINE: TAURI + RUST + WEBGL</span>
      </footer>
    </div>
  );
}

export default App;
