// Derived GPU visibility only. CPU terrain heights remain authoritative.
struct Cell {
    direction_height: vec4<f32>,
    corners: array<vec4<f32>,6>,
    metadata: vec4<u32>,
}
struct Params {
    clip_from_body: mat4x4<f32>, camera: vec4<f32>, sun: vec4<f32>, settings: vec4<f32>,
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
@group(0) @binding(3) var<storage,read_write> args: array<DrawArgs,2>;
@group(0) @binding(4) var<storage,read_write> foliage: array<u32>;

@compute @workgroup_size(1)
fn clear_indirect() {
    args[0].vertex_count = 54u;
    atomicStore(&args[0].instance_count, 0u);
    args[0].first_vertex = 0u;
    args[0].first_instance = 0u;
    args[1].vertex_count = 108u;
    atomicStore(&args[1].instance_count, 0u);
    args[1].first_vertex = 54u;
    args[1].first_instance = 0u;
}

fn hash(x: u32) -> u32 {
    var h = x*747796405u+2891336453u;
    h = ((h >> ((h >> 28u)+4u))^h)*277803737u;
    return (h>>22u)^h;
}

// This is the sole foliage eligibility decision. The foliage indirect draw
// submits geometry only for these cells; its vertex path never rejects trees.
fn has_nearby_foliage(cell: Cell, center: vec3<f32>) -> bool {
    let seed = hash(cell.metadata.w);
    let material = cell.metadata.y;
    let tree = (material==3u && seed%4u!=0u) || (material==2u && seed%9u==0u) || (material==7u && seed%7u==0u);
    return tree && distance(params.camera.xyz,center)<params.settings.w;
}

fn in_frustum(center: vec3<f32>, radius: f32) -> bool {
    // WebGPU clip volume is -w <= x,y <= w and 0 <= z <= w. Using
    // homogeneous planes also covers reverse-Z and an infinite far plane:
    // a degenerate far plane has zero normal and cannot reject a sphere.
    let rows = transpose(params.clip_from_body);
    let planes = array<vec4<f32>,6>(rows[3]+rows[0], rows[3]-rows[0],
        rows[3]+rows[1], rows[3]-rows[1], rows[2], rows[3]-rows[2]);
    for (var i=0u; i<6u; i++) {
        let plane = planes[i];
        if dot(plane,vec4(center,1.)) < -(radius+0.01)*length(plane.xyz) { return false; }
    }
    return true;
}

fn terrain_bound(cell: Cell, center: vec3<f32>, top: f32) -> f32 {
    var radius_squared = 0.0;
    // Bound every cap and exposed wall endpoint, including the bottom of a
    // cliff. Testing only the centre would pop cells along the viewport edge.
    for (var i=0u; i<cell.metadata.x; i++) {
        let a = cell.corners[i].xyz;
        let b = cell.corners[(i+1u)%cell.metadata.x].xyz;
        let lower = min(top,params.settings.x+cell.corners[i].w);
        let cap = a*top-center;
        let wall_a = a*lower-center;
        let wall_b = b*lower-center;
        radius_squared = max(radius_squared,max(dot(cap,cap),max(dot(wall_a,wall_a),dot(wall_b,wall_b))));
    }
    return sqrt(radius_squared);
}

@compute @workgroup_size(128)
fn compact_visible(@builtin(global_invocation_id) id: vec3<u32>) {
    // Capacities cover the whole dispatch before any counts can be published;
    // malformed bindings must not create partial generations or invalid IDs.
    let count = u32(params.settings.y);
    if arrayLength(&cells)<count || arrayLength(&visible)<count || arrayLength(&foliage)<count { return; }
    if id.x >= count { return; }
    let cell = cells[id.x];
    if cell.metadata.x<5u || cell.metadata.x>6u { return; }
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
    let surface_radius = radius + max(cell.direction_height.w,0.0);
    let center = cell.direction_height.xyz*surface_radius;
    if in_frustum(center,terrain_bound(cell,center,surface_radius)) {
        let slot = atomicAdd(&args[0].instance_count,1u);
        visible[slot] = id.x;
    }
    // The largest authored tree fits inside 55 m of its base. Keep a tree
    // whose crown enters the frustum even when its terrain cap is outside.
    if has_nearby_foliage(cell,center) && in_frustum(center,55.) {
        let slot = atomicAdd(&args[1].instance_count,1u);
        foliage[slot] = id.x;
    }
}
