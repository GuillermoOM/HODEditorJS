## What's Changed
* **Engine shapes are now edited as what they are**: an `ETSH` shape is the nozzle's cross-section point list that sizes the small-craft engine trail ribbon, not a mesh. The viewport draws the outline, the points and the min-dimension circle; the inspector has a point table with validation, polygon generator, scale-to-min-dimension and copy-to-all tools. Points can be picked and dragged in the viewport, and every number box scrolls with the mouse wheel (Shift x10, Alt x0.1).
* **Engine Trail Simulation (preview)**: optional toggle that flies the ship and draws the trail ribbons the engine shapes size, using the retail frame logic. The `.ship` trail values (linger, diameter, ...) are editable preview inputs.
* **Add Engine Shape** now creates a valid `EngineShape<n>` with a starting cross-section.
* **Tighter generated collision**: "Generate collision" builds a simplified copy of the visible hull (about 50-360 triangles, like vanilla) instead of an inflated convex hull, so shots hit the visible ship.
* **Collision and battle damage fixes**: the real collision mesh is loaded and saved correctly (resaved ships no longer lose hits against the hull), and battle-scar decals follow the hull after a resave (#8).
* **Engine glows** preview like the game (faint additive shell) and point out of the stern.
* **Joint HIER "bounds" are a rotation, not a scale**: weapon pods, nozzles and fins no longer appear stretched or turned in the viewer (e.g. `hgn_attackbomber`, `vgr_probe_prox`), including while animations play.
* **Animation (MAD) saving**: saved `.mad` files keep the original channel set, so animated parts (e.g. the `vgr_probe_prox` fins) behave in game as they did before the save.
* Nav light range spheres only show for the selected nav light.

## Reverse Engineering Milestones
* HIER joint record = two Euler triples; the second one is a rotation (`M(B) * M(A)`).
* `ETSH` semantics: nozzle picked by the name's last digit, only the smallest (x, y) distance is used, only for ships whose `.ship` calls `setEngineTrail`.
* Retail collision (`COLD`) is a tight, concave, ~LOD0-volume surface of 60-400 triangles; `COLD` vertices are 12-byte positions in the pool.
* Retail `AnimJointMad` overrides only the channels a track has curves for.
