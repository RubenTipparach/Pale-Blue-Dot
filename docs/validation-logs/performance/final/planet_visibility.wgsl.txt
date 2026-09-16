// Derived GPU visibility only. CPU terrain heights remain authoritative.
struct Cell {
    direction_height: vec4<f32>,
    corners: array<vec4<f32>,6>,
    metadata: vec4<u32>,
}
struct Params {
    clip_from_world: mat4x4<f32>, camera: vec4<f32>, sun: vec4<f32>, settings: vec4<f32>,
}
struct DrawArgs {
    vertex_count: u32,
    instance_count: atomic<u32>,
    first_vertex: u32,
    first_instance: u32,
}
@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<storage,read> cells: array<Cell>;
@group(0) @binding(2) var<storage,read_write> visible: array<u32>;
@group(0) @binding(3) var<storage,read_write> args: DrawArgs;

@compute @workgroup_size(1)
fn clear_indirect() {
    args.vertex_count = u32(params.settings.w);
    atomicStore(&args.instance_count, 0u);
    args.first_vertex = 0u;
    args.first_instance = 0u;
}

@compute @workgroup_size(128)
fn compact_visible(@builtin(global_invocation_id) id: vec3<u32>) {
    if id.x >= u32(params.settings.y) || id.x >= arrayLength(&cells) { return; }
    let cell = cells[id.x];
    let radius = params.settings.x;
    let camera_radius = length(params.camera.xyz);
    // The angular horizons of camera and raised terrain overlap. Include a
    // 55m column/foliage margin and base the occluder below the sea surface.
    let occluder = radius - 8.0;
    let top = radius + max(cell.direction_height.w,0.0) + 55.0;
    // cos(acos(a)+acos(b)+0.018), expanded by angle addition. Inputs are
    // in [0,1], so both angle sines are their nonnegative square roots.
    // (1-c)*(1+c) retains precision near c=1. The tiny downward allowance
    // makes rounding at the horizon conservative rather than hiding a cell.
    let ca = clamp(occluder/max(camera_radius,occluder),0.0,1.0);
    let cb = clamp(occluder/top,0.0,1.0);
    let sa = sqrt(max((1.0-ca)*(1.0+ca),0.0));
    let sb = sqrt(max((1.0-cb)*(1.0+cb),0.0));
    let horizon_cosine = (ca*cb-sa*sb)*0.9998380043739528
        - (sa*cb+ca*sb)*0.017999028015746276 - 0.000002;
    let facing = dot(cell.direction_height.xyz, params.camera.xyz/max(camera_radius,1.0));
    if facing < horizon_cosine { return; }
    // Every invocation emits at most once, capacity equals authoritative cell
    // count. No partial geometry generation can be published on overflow.
    let slot = atomicAdd(&args.instance_count,1u);
    if slot < arrayLength(&visible) { visible[slot] = id.x; }
}
