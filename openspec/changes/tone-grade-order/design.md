# Design: the tone grade runs after tonemapping, every frame

## The repro

`target/fast/pbd-app --world <new> --walk --at 20 40 --time 2 --pitch 40
--turn 45 --frames 240 --capture <png>` on the Mac (Apple M4 Max, Metal,
1440 x 900 at 2x). Where: frame latitude 20, longitude 40 (`--at`; the map
reads 20 S 40 E), on foot, the eye 1.6 m above the ground, 02:00 on the planet
clock, `Weather here: cover 0.00, rain 0.00`. Looking up 40 degrees and turning
45 degrees a second, so 0.75 degrees a frame at the capture's fixed step; the
shot is frame 240 after `READY`.

A capture steps time by a fixed amount a frame (`loading-screen` decision
5), so the turn per frame is the same on every run; the smear is a
frame-to-frame effect and a capture shows it as play does.

## Bisect (all `--profile fast`, the same capture)

| Commit | What | Trails |
| --- | --- | --- |
| `db25fe2` | main before PR #27 | none |
| `588ded9` | the lighting model, every part | stars in dotted arcs, trees in stepped copies |
| `f15a647` | main after PR #27 | as `588ded9` |
| `b76c3a9` | `claude/perf-tuning` head | as `588ded9` |

## Which part (switches on `b76c3a9`, no rebuild)

| Switch | Trails |
| --- | --- |
| `PBD_NO_CLOUDS=1` | yes |
| `PBD_REFLECT=0` | yes |
| `PBD_NO_BLOOM=1` | yes |
| `PBD_SOFT_SHADOWS=0`, `PBD_BOUNCE=0`, `PBD_LAMP_BAKE=0` | yes |
| `PBD_LIGHTING=today` | none |
| `PBD_NO_TONE=1` (the new shading, without the display transform) | none |

`PBD_NO_TONE`'s frame is darker, so it was brightened 12 times to check that
the trails were not merely too faint to see: none. A local split (not kept)
then separated the display transform's two parts: the grade pass with Bevy's
tonemapper back still smeared, and `Tonemapping::None` without the grade was
clean. The grade pass is the trigger, yet `tone_grade.wgsl` is per pixel and
keeps no history.

## Cause

`ToneGrade` uses Bevy's `FullscreenMaterial` with no `sub_graph()`. On that
path (`bevy_core_pipeline-0.18.1/src/fullscreen_material.rs`,
`extract_on_add`) Bevy adds the node to the 3D graph each time a camera
**gains** the component. `dress_cameras` gives every `Camera3d` the grade:
the walking, flight and vehicle cameras, active or not. On the second add,
`RenderGraph::add_node` replaces the node's state and its edges, and
`try_add_node_edge(Tonemapping, grade)` finds the edge still listed on the
tonemapping side, reports it as existing and skips it. The grade is left with
no input edge, so the graph may run it before the main pass.

Run there, its `post_process_write` marks the next main texture as cleared.
The main pass then **loads** its multisampled colour, which still holds the
last frame, instead of clearing it. Opaque geometry writes over it, but the
sky is drawn premultiplied over whatever is there (`sky.rs`, mostly
transparent at night), so each frame's sky is laid over the frame before:
every star and every silhouette leaves a trail that fades frame by frame,
only where sky is drawn. That is the owner's description, and the captures.

## Fix

`fn sub_graph() -> Option<InternedRenderSubGraph> { Some(Core3d.intern()) }`
on `ToneGrade`. Bevy then adds the node at plugin build, once, with
`Tonemapping -> grade -> EndMainPassPostProcessing`, whatever cameras gain
the component later.

## Verified

The same capture after the fix: point stars, clean tree edges, the grade
applied. Before-and-after shots, and a daytime pair with cloud, in
`docs/screenshots/tone-grade-order/`, each named with where it was taken.
