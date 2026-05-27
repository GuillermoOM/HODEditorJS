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

## Render Modes And Overlays

Render mode, helper layers, and viewport overlays are controlled by app state and passed into Viewport. Agents should not introduce independent duplicate toggles inside Viewport when the behavior is global.

Navlight markers are expected to remain visible when collision hull helpers are enabled. Collision helpers should render as lower-order transparent overlays that do not write depth, while navlight marker/range helpers should render above them without depth testing or depth writes.

## Animation Display

Loaded animations can influence joint transforms and attached mesh deltas for playback/preview. Animation editing and compilation are owned by `AnimationDock.tsx`; viewport should consume app animation state rather than own animation CRUD.
