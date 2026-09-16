// Vertex pulling + Tenebris terminator, baked light, underwater absorption,
// screen-door transparency and orbital limb math. See docs/shader-port.md.
// Same Cell/Column/Face ABI as hex_faces.wgsl; checked by the validator.
struct Cell {
    column: u32, material: u32, light: u32, flags: u32,
    radii: vec2<f32>, padding: vec2<u32>,
}
struct Column { corners: array<vec4<f32>, 6>, center_degree: vec4<f32> }
struct Face { cell: u32, face: u32, material: u32, light: u32 }
struct TerrainView {
    clip_from_local: mat4x4<f32>,
    planet_center: vec4<f32>, // xyz body center relative to rebased local origin; w sea radius
    camera_local: vec4<f32>, // SAME local origin as planet_center
    sun: vec4<f32>,         // xyz direction toward sun; w direct intensity
    ambient: vec4<f32>,     // rgb linear ambient; w block-light intensity
    terminator: vec4<f32>,  // x low, y high, z night ambient, w fog-height metres
    absorption: vec4<f32>, // rgb water absorption / m; w unused
    water_color: vec4<f32>,
    rim: vec4<f32>,         // rgb; w intensity
    settings: vec4<f32>,    // x rim power; y minimum ambient; zw reserved
}
@group(0) @binding(0) var<uniform> view: TerrainView;
@group(0) @binding(1) var<storage, read> faces: array<Face>;
@group(0) @binding(2) var<storage, read> cells: array<Cell>;
@group(0) @binding(3) var<storage, read> columns: array<Column>;
@group(1) @binding(0) var tiles: texture_2d_array<f32>;
@group(1) @binding(1) var nearest_sampler: sampler;

struct VertexOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) local_position: vec3<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) @interpolate(flat) layer: u32,
    @location(4) @interpolate(flat) light: u32,
}
fn safe_normal(v: vec3<f32>) -> vec3<f32> { return v * inverseSqrt(max(dot(v,v), 1e-12)); }

@vertex
fn vertex(@builtin(vertex_index) vertex_id: u32, @builtin(instance_index) instance: u32) -> VertexOut {
    let face = faces[instance];
    let cell = cells[face.cell];
    let column = columns[cell.column];
    let degree = u32(clamp(column.center_degree.w, 5.0, 6.0));
    let axis = safe_normal(column.center_degree.xyz);
    let corner = vertex_id % 3u;
    let triangle = vertex_id / 3u;
    var position = axis * cell.radii.y;
    var normal = axis;
    var uv = vec2<f32>(0.5);
    var tile_variant = 0u; // top; side=1; bottom=2
    if (face.face < 2u) {
        let is_bottom = face.face == 1u;
        let radius = select(cell.radii.y, cell.radii.x, is_bottom);
        normal = select(axis, -axis, is_bottom);
        position = axis * radius;
        // Corner rays must be CCW as seen from outside. Reverse bottom winding.
        let reversed = select(corner, (3u - corner) % 3u, is_bottom);
        if (triangle < degree && reversed != 0u) {
            let c = (triangle + reversed - 1u) % degree;
            position = column.corners[c].xyz * radius;
            let angle = 6.28318530718 * f32(c) / f32(degree);
            uv = vec2<f32>(0.5) + 0.5 * vec2<f32>(cos(angle), sin(angle));
        }
        tile_variant = select(0u, 2u, is_bottom);
    } else {
        let side = min(face.face - 2u, degree - 1u);
        let a = column.corners[side].xyz;
        let b = column.corners[(side + 1u) % degree].xyz;
        let points = array<vec3<f32>, 4>(a * cell.radii.x, b * cell.radii.x, b * cell.radii.y, a * cell.radii.y);
        // These indices face out for the documented CCW column convention.
        let indices = array<u32, 6>(0u, 1u, 2u, 0u, 2u, 3u);
        let uvs = array<vec2<f32>, 4>(vec2<f32>(0.,1.), vec2<f32>(1.,1.), vec2<f32>(1.,0.), vec2<f32>(0.,0.));
        let i = select(0u, indices[min(vertex_id, 5u)], vertex_id < 6u);
        position = points[i];
        uv = uvs[i];
        normal = safe_normal(cross(points[1]-points[0], points[3]-points[0]));
        tile_variant = 1u;
    }
    var out: VertexOut;
    out.local_position = view.planet_center.xyz + position;
    out.clip = view.clip_from_local * vec4<f32>(out.local_position, 1.0);
    out.normal = normal;
    out.uv = uv;
    out.layer = (max(face.material, 1u) - 1u) * 3u + tile_variant;
    out.light = face.light;
    return out;
}

fn bayer4(pixel: vec2<u32>) -> f32 {
    let bayer = array<f32,16>(0.,8.,2.,10.,12.,4.,14.,6.,3.,11.,1.,9.,15.,7.,13.,5.);
    return (bayer[(pixel.y & 3u) * 4u + (pixel.x & 3u)] + 0.5) / 16.0;
}

@fragment
fn fragment(in: VertexOut) -> @location(0) vec4<f32> {
    let texel = textureSample(tiles, nearest_sampler, in.uv, i32(in.layer));
    if (texel.a < bayer4(vec2<u32>(in.clip.xy))) { discard; }
    let radial = safe_normal(in.local_position - view.planet_center.xyz);
    let sun = safe_normal(view.sun.xyz);
    let brightness = smoothstep(view.terminator.x, max(view.terminator.x + 1e-4, view.terminator.y), dot(radial, sun));
    let sky = f32((in.light >> 12u) & 15u) / 15.0;
    let rgb = vec3<f32>(f32(in.light & 15u), f32((in.light >> 4u) & 15u), f32((in.light >> 8u) & 15u)) / 15.0;
    let ambient = view.ambient.rgb * max(view.settings.y, mix(view.terminator.z, 1.0, brightness) * sky);
    let direct = max(dot(safe_normal(in.normal), sun), 0.0) * brightness * sky * view.sun.w;
    var lit = texel.rgb * (ambient + vec3<f32>(direct) + rgb * view.ambient.w);
    let water_depth = max(view.planet_center.w - length(in.local_position - view.planet_center.xyz), 0.0);
    let attenuation = exp(-max(view.absorption.rgb, vec3<f32>(0.0)) * water_depth);
    lit = mix(lit * attenuation, view.water_color.rgb * attenuation, (1.0-dot(attenuation,vec3<f32>(1.0/3.0))) * 0.5);
    let camera_body = view.camera_local.xyz - view.planet_center.xyz;
    let altitude = length(camera_body) - view.planet_center.w;
    let fog_gate = clamp(1.0-altitude/max(view.terminator.w,1.0),0.0,1.0);
    let toward_camera = safe_normal(view.camera_local.xyz-in.local_position);
    let fresnel = pow(1.0-clamp(dot(radial,toward_camera),0.0,1.0),max(view.settings.x,0.01));
    lit += view.rim.rgb * (fresnel * (0.25+0.75*max(dot(radial,sun),0.0)) * view.rim.w * (1.0-fog_gate*fog_gate) * sky);
    return vec4<f32>(lit, 1.0);
}
