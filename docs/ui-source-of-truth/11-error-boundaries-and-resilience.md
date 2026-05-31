# 11 Error Boundaries And Resilience

## Scope

This document specifies how the application should handle malformed or unexpected data without crashing the entire React tree.

## Resilience Philosophy

A single corrupted node (e.g., a missing mesh file or a non-finite transform matrix) should not result in a white-screen-of-death for the entire editor. 

## Implementation Rules

1. **Viewport Error Boundary**:
   - The 3D viewport (`Viewport.tsx` and underlying Three.js canvas) must be wrapped in a React Error Boundary. If the rendering context crashes due to bad WebGL data, the user must still be able to access the Hierarchy Tree to delete or fix the offending node.

2. **Inspector Resilience**:
   - The `Inspector.tsx` must safely handle cases where a selected node has missing properties or undefined references. It should render a fallback UI or clear warning rather than throwing an unhandled exception.

3. **Data Coercion**:
   - When ingesting values from user input (e.g., manual XYZ coordinate entry), validate and sanitize inputs. Prevent `NaN` or `Infinity` from propagating into the `model` state, as these will corrupt the HOD binary upon saving.

4. **Silent Failures**:
   - Avoid silent failures. If an action fails (e.g., reparenting a node creates a cycle), the UI must provide clear feedback via a toast, alert, or console warning indicating why the action was rejected.
