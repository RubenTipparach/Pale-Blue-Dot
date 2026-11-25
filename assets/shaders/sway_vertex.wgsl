// The vertex stage of a reef tile's swaying pieces and its kelp
// (`memory-leak` design, "The reef sways on the GPU"): Bevy's mesh vertex
// for a mesh with positions, normals, a UV and colours, with the surge's
// sway applied before the mesh is placed. It is the CPU's old `fill_bent`
// to the letter: the vertex moves along its axis by
// `sin(clock * 1.1 + phase) * reach * bend`, where the per-vertex attribute
// carries the axis already scaled by the reach (`min(0.07 h, 0.7)`) and the
// phase with the height's share (`phase + 0.6 h`), and `bend` eases from 1
// to 0 over the last `SWAY_EASE_M` before `NEAR_M` from the eye. The
// field-lit fragment (`field_lit.wgsl`) shades it as before.
// `FieldLit::specialize` gives this to a mesh that carries the sway
// attribute.

#import bevy_pbr::{
    mesh_functions,
    forward_io::VertexOutput,
    mesh_view_bindings::view,
    view_transformations::position_world_to_clip,
}

// Must match `places::NEAR_M` and `places::SWAY_EASE_M`.
const NEAR_M: f32 = 30.0;
const SWAY_EASE_M: f32 = 4.0;

// The field's view buffer, as `field_lit.wgsl`'s `Scene`; only `sway` is
// read here (x: the sun clock's seconds, wrapped at an hour).
struct Scene {
    centre: vec4<f32>,
    sun: vec4<f32>,
    model: vec4<f32>,
    sky_sh: array<vec4<f32>, 18>,
    carried: array<vec4<f32>, 8>,
    cut: array<vec4<f32>, 2>,
    sway: vec4<f32>,
}

@group(#{MATERIAL_BIND_GROUP}) @binding(100) var<storage, read> scene: Scene;

struct Vertex {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
#ifdef VERTEX_UVS_A
    @location(2) uv: vec2<f32>,
#endif
    @location(5) color: vec4<f32>,
    // xyz: the vertex's axis times its reach; w: its phase.
    @location(13) sway: vec4<f32>,
}

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;
    let world_from_local = mesh_functions::get_world_from_local(vertex.instance_index);
    // The bend by the tile's distance from the eye: the tile's origin is its
    // anchor, as the CPU measured it.
    let anchor = world_from_local[3].xyz;
    let m = distance(anchor, view.world_position);
    let bend = clamp((NEAR_M - m) / SWAY_EASE_M, 0.0, 1.0);
    var position = vertex.position;
    if (bend > 0.0) {
        let wave = sin(scene.sway.x * 1.1 + vertex.sway.w);
        position = position + vertex.sway.xyz * (wave * bend);
    }
    out.world_normal = mesh_functions::mesh_normal_local_to_world(vertex.normal, vertex.instance_index);
    out.world_position = mesh_functions::mesh_position_local_to_world(
        world_from_local,
        vec4<f32>(position, 1.0),
    );
    out.position = position_world_to_clip(out.world_position.xyz);
#ifdef VERTEX_UVS_A
    out.uv = vertex.uv;
#endif
#ifdef VERTEX_COLORS
    out.color = vertex.color;
#endif
#ifdef VERTEX_OUTPUT_INSTANCE_INDEX
    out.instance_index = vertex.instance_index;
#endif
#ifdef VISIBILITY_RANGE_DITHER
    out.visibility_range_dither = mesh_functions::get_visibility_range_dither_level(
        vertex.instance_index,
        world_from_local[3],
    );
#endif
    return out;
}
