# 06 Viewport Interactions

## Scope

This spec covers 3D viewport rendering, controls, selection coupling, transform gizmo behavior, overlays, and animation display.

Primary sources:

- `src/components/Viewport.tsx`
- `src/App.tsx`
- `src/components/AnimationDock.tsx` when animation state affects viewport behavior

## Controls

Viewport uses orbit controls for camera navigation and transform controls for selected editable nodes.

Transform mode supports translate and rotate modes as exposed by current UI buttons/state. Transform changes must flow back to `App.tsx` through callbacks.

Root joint transform editing is excluded where current code prevents it.

## Coordinate Space Toggle

A single app-level toggle switches the viewport transform gizmo between **Global** (world-aligned) and **Local** (node-aligned) axes, for both translate and rotate modes.

- Global mode calls `TransformControls.setSpace("world")`; Local mode calls `setSpace("local")`.
- The toggle is shared with the Inspector position/rotation fields (see `05-inspector-behavior.md`), so both surfaces always show the same coordinate space. Both the viewport toolbar and the Inspector header expose Local/Global buttons bound to the same `App.tsx` state.
- Transform commits continue to convert the gizmo result back into the node's parent-relative local transform before calling `onNodeTransform`; changing the display space must not change the stored HOD data format.
- The toggle state is owned by `App.tsx`, not by `Viewport.tsx` or `Inspector.tsx` independently.
- Animation keyframe values are excluded from the Inspector coordinate toggle: they store deltas relative to the joint rest pose and are always shown in animation space. The viewport gizmo space still applies when dragging a keyframe, but commits store the resulting keyframe delta unchanged.

## Selection Coupling

Viewport behavior follows `selectedNode` from app state. Selecting a tree row or viewport target should keep Inspector and Viewport synchronized.

The gizmo attaches only when the selected node type is supported by current `Viewport.tsx` logic.

## Rendered Helpers

Viewport renders model meshes and helper visuals for supported data collections, including:

- Joints and bones.
- Markers.
- Nav lights.
- Dockpaths.
- Engine burns.
- Engine glows.
- Engine shapes.
- Collision hulls.
- Target boxes.

Visibility layer toggles and `visibleMeshes` should hide/show matching helpers consistently with hierarchy eye toggles.

## Geometry Safety

Viewport sanitizes invalid or extreme vertex values before use. Non-finite or outlier coordinates should not break camera framing or rendering.

## Texture Rendering

Viewport-created material textures should use sRGB color space, repeat wrapping, linear filtering, and the renderer's maximum available anisotropy so full-resolution texture previews stay crisp and visibly match imported texture files.

## Render Modes And Overlays

Render mode, helper layers, and viewport overlays are controlled by app state and passed into Viewport. Agents should not introduce independent duplicate toggles inside Viewport when the behavior is global.

Navlight markers are expected to remain visible when collision hull helpers are enabled. Collision helpers should render as lower-order transparent overlays that do not write depth, while navlight marker/range helpers should render above them without depth testing or depth writes.

## Animation Display

Loaded animations can influence joint transforms and attached mesh deltas for playback/preview. Animation editing and compilation are owned by `AnimationDock.tsx`; viewport should consume app animation state rather than own animation CRUD.
