// The vertex stage of a school's mesh (`fish-behaviour` design 8.4): Bevy's
// mesh vertex for a mesh with positions, normals, a UV and colours, with the
// body wave of `swim.wgsl` applied before the mesh is placed. The field-lit
// fragment (`field_lit.wgsl`) shades it as before. `FieldLit::specialize`
// gives this to a mesh that carries the swim attributes; every other
// field-lit mesh keeps Bevy's own vertex stage.

#import bevy_pbr::{
    mesh_functions,
    forward_io::VertexOutput,
    view_transformations::position_world_to_clip,
}
#import pbd::swim::{SwimRow, swim_body}

@group(#{MATERIAL_BIND_GROUP}) @binding(106) var<storage, read> swim_rows: array<SwimRow>;

struct Vertex {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(5) color: vec4<f32>,
    // The creature's side (metres a body length) and `u`; its wave's phase,
    // amplitude and bend and the vertex's `d`; its up and its table row.
    @location(10) swim_axes: vec4<f32>,
    @location(11) swim_wave: vec4<f32>,
    @location(12) swim_up: vec4<f32>,
}

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;
    var position = vertex.position;
    var normal = vertex.normal;
    // Zero amplitude and bend (a crab, a bird, a body with no wave) is the
    // body as laid.
    if (vertex.swim_wave.y != 0.0 || vertex.swim_wave.z != 0.0) {
        let count = arrayLength(&swim_rows);
        let index = min(u32(max(vertex.swim_up.w, 0.0)), count - 1u);
        let swum = swim_body(
            vertex.position,
            vertex.normal,
            vertex.swim_axes.xyz,
            vertex.swim_up.xyz,
            vertex.swim_axes.w,
            vertex.swim_wave.w,
            vertex.swim_wave.x,
            vertex.swim_wave.y,
            vertex.swim_wave.z,
            swim_rows[index],
        );
        position = swum.position;
        normal = swum.normal;
    }
    let world_from_local = mesh_functions::get_world_from_local(vertex.instance_index);
    out.world_normal = mesh_functions::mesh_normal_local_to_world(normal, vertex.instance_index);
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
