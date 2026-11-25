// The vertex stage of a reef's fish cloud (`memory-leak` design, "The
// reef's fish on the GPU"): every fish of a cloud is laid once, in its own
// body frame (nose +x, up +y), into one mesh in the cloud's frame (x and z
// the cloud's plane, y its up). This moves each one round its orbit, bobs
// it, turns it to face the way it swims and waves its tail, as the CPU's
// `ambient` did each frame; the field-lit fragment (`field_lit.wgsl`)
// shades it. `FieldLit::specialize` gives this to a mesh carrying the fish
// attributes.
//
// Per fish (the same on each of its vertices):
//   fish0: the vertex in the body frame (m), and the orbit's radius (m)
//   fish1: the orbit's start angle, its speed (rad/s, signed), the bob's
//          middle (m) and the orbit's squash across (z)
//   fish2: the bob's reach (m), rate (rad/s) and phase, and the wave's phase
//   fish3: the vertex's place along the body, 0 at the nose to 1 at the
//          tail, and the body's length (m)

#import bevy_pbr::{
    mesh_functions,
    forward_io::VertexOutput,
    view_transformations::position_world_to_clip,
}

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
    @location(14) fish0: vec4<f32>,
    @location(15) fish1: vec4<f32>,
    @location(16) fish2: vec4<f32>,
    @location(17) fish3: vec4<f32>,
}

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;
    let t = scene.sway.x;
    let radius = vertex.fish0.w;
    let a = vertex.fish1.x + t * vertex.fish1.y;
    let squash = vertex.fish1.w;
    let h = vertex.fish1.z + vertex.fish2.x * sin(t * vertex.fish2.y + vertex.fish2.z);
    // Round the cloud's up: x along its first tangent, z along minus its
    // second (the frame is right-handed).
    let centre = vec3<f32>(cos(a) * radius, h, -sin(a) * radius * squash);
    let along = vec3<f32>(-sin(a), 0.0, -cos(a) * squash) * sign(vertex.fish1.y);
    let f = normalize(along + vec3<f32>(1e-5, 0.0, 0.0));
    let side = vec3<f32>(-f.z, 0.0, f.x);
    // The tail's wave: more toward the tail.
    var p = vertex.fish0.xyz;
    let u = vertex.fish3.x;
    p.z = p.z + sin(t * 9.0 + vertex.fish2.w - u * 5.0) * 0.12 * vertex.fish3.y * u * u;
    let local = f * p.x + vec3<f32>(0.0, p.y, 0.0) + side * p.z + centre;
    let n = vertex.normal;
    let local_n = f * n.x + vec3<f32>(0.0, n.y, 0.0) + side * n.z;

    let world_from_local = mesh_functions::get_world_from_local(vertex.instance_index);
    out.world_normal = mesh_functions::mesh_normal_local_to_world(local_n, vertex.instance_index);
    out.world_position = mesh_functions::mesh_position_local_to_world(
        world_from_local,
        vec4<f32>(local, 1.0),
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
