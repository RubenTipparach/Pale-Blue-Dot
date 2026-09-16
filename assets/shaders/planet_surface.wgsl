// Persistent topology vertex pulling, pixel atlas materials, exposed column
// skirts and cosmetic trees. Terminator/skylight/rim treatment follows the
// previously documented Tenebris port in hex_terrain.wgsl.
struct Cell {
    direction_height: vec4<f32>,
    corners: array<vec4<f32>,6>,
    metadata: vec4<u32>,
}
struct Params {
    clip_from_world: mat4x4<f32>, camera: vec4<f32>, sun: vec4<f32>, settings: vec4<f32>,
}
@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<storage,read> cells: array<Cell>;
@group(0) @binding(2) var<storage,read> visible: array<u32>;
@group(0) @binding(3) var atlas: texture_2d<f32>;

struct VertexOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) @interpolate(flat) material: u32,
    @location(4) @interpolate(flat) height: f32,
    @location(5) @interpolate(flat) skylight: f32,
    @location(6) @interpolate(flat) seed: u32,
    @location(7) @interpolate(flat) kind: u32,
}
fn hash(x: u32) -> u32 {
    var h = x*747796405u+2891336453u;
    h = ((h >> ((h >> 28u)+4u))^h)*277803737u;
    return (h>>22u)^h;
}
fn random(x: u32) -> f32 { return f32(hash(x)&65535u)/65535.0; }
fn normalized(v: vec3<f32>) -> vec3<f32> { return v*inverseSqrt(max(dot(v,v),1e-12)); }

struct BoxVertex { position: vec3<f32>, normal: vec3<f32>, uv: vec2<f32> }
fn box_vertex(index: u32) -> BoxVertex {
    let face = index/6u;
    let triangle = array<u32,6>(0u,1u,2u,0u,2u,3u);
    let quad = array<vec2<f32>,4>(vec2(-1.,-1.),vec2(1.,-1.),vec2(1.,1.),vec2(-1.,1.));
    let uv = quad[triangle[index%6u]];
    let normals = array<vec3<f32>,6>(vec3(1.,0.,0.),vec3(-1.,0.,0.),vec3(0.,1.,0.),vec3(0.,-1.,0.),vec3(0.,0.,1.),vec3(0.,0.,-1.));
    let right = array<vec3<f32>,6>(vec3(0.,0.,-1.),vec3(0.,0.,1.),vec3(1.,0.,0.),vec3(1.,0.,0.),vec3(1.,0.,0.),vec3(-1.,0.,0.));
    let up = cross(normals[face],right[face]);
    return BoxVertex(normals[face]+right[face]*uv.x+up*uv.y,normals[face],uv*0.5+0.5);
}

@vertex
fn vertex(@builtin(vertex_index) vertex: u32, @builtin(instance_index) instance: u32) -> VertexOut {
    let cell = cells[visible[instance]];
    let degree = cell.metadata.x;
    let axis = cell.direction_height.xyz;
    let height = cell.direction_height.w;
    let radius = params.settings.x + max(height,0.0);
    let reference = select(vec3(0.,1.,0.),vec3(1.,0.,0.),abs(axis.y)>0.95);
    let tangent = normalized(cross(reference,axis));
    let bitangent = cross(axis,tangent);
    var position = axis*radius;
    var normal = axis;
    var uv = vec2(0.5);
    var kind = 0u;
    var material = cell.metadata.y;
    if vertex < 18u {
        let triangle = vertex/3u;
        let corner = vertex%3u;
        if triangle < degree && corner != 0u {
            let ray = cell.corners[(triangle+corner-1u)%degree].xyz;
            position = ray*radius;
            let local = (ray-axis)*params.settings.x;
            uv = vec2(dot(local,tangent),dot(local,bitangent))/28.0 + 0.5;
        }
    } else if vertex < 54u {
        kind = 1u;
        let side = (vertex-18u)/6u;
        let i = (vertex-18u)%6u;
        if side < degree && max(height,0.0)>cell.corners[side].w {
            let a = cell.corners[side].xyz;
            let b = cell.corners[(side+1u)%degree].xyz;
            let lower = params.settings.x + cell.corners[side].w;
            let points = array<vec3<f32>,4>(a*lower,b*lower,b*radius,a*radius);
            let indices = array<u32,6>(0u,1u,2u,0u,2u,3u);
            position = points[indices[i]];
            normal = normalized(cross(points[1]-points[0],points[3]-points[0]));
            let side_uv = array<vec2<f32>,4>(vec2(0.,1.),vec2(1.,1.),vec2(1.,0.),vec2(0.,0.));
            uv = side_uv[indices[i]]*vec2(1.,max(1.,(radius-lower)/18.));
        }
    } else {
        kind = 2u;
        let seed = hash(cell.metadata.w);
        let tree = (material==3u && seed%4u!=0u) || (material==2u && seed%9u==0u) || (material==7u && seed%7u==0u);
        // Cosmetic geometry is omitted at orbital distances. The draw remains
        // bounded and terrain authority does not depend on a visual tree.
        if tree && distance(params.camera.xyz,axis*radius)<2300.0 {
            let part = (vertex-54u)/36u;
            let cube = box_vertex((vertex-54u)%36u);
            let scale = 0.85+random(seed)*0.50;
            var halfsize = vec3(1.7,9.,1.7)*scale;
            var elevation = 9.*scale;
            material = 8u;
            if part==1u { halfsize=vec3(9.,6.,9.)*scale; elevation=20.*scale; material=9u; }
            if part==2u { halfsize=vec3(6.,5.,6.)*scale; elevation=28.*scale; material=9u; }
            let local = cube.position*halfsize + vec3(0.,elevation,0.);
            position = axis*radius+tangent*local.x+axis*local.y-bitangent*local.z;
            normal = tangent*cube.normal.x+axis*cube.normal.y-bitangent*cube.normal.z;
            uv = cube.uv;
        }
    }
    var out: VertexOut;
    out.position = position;
    out.clip = params.clip_from_world*vec4(position,1.);
    out.normal = normal;
    out.uv = uv;
    out.material = material;
    out.height = height;
    out.skylight = f32(cell.metadata.z)/65535.;
    out.seed = cell.metadata.w;
    out.kind = kind;
    return out;
}

fn pixel_tile(uv: vec2<f32>, tile: vec2<f32>) -> vec3<f32> {
    // Texture-load is intentionally nearest, with an explicit 32x32 source
    // grid per tile. Interior sampling avoids the source sheet's soft seams.
    let pixel = (floor(fract(uv)*32.)+0.5)/32.;
    let sheet_uv = (tile+0.025+pixel*0.95)*0.25;
    return textureLoad(atlas,vec2<i32>(sheet_uv*vec2<f32>(textureDimensions(atlas))),0).rgb;
}

@fragment
fn fragment(input: VertexOut) -> @location(0) vec4<f32> {
    let radial = normalized(input.position);
    let n = normalized(input.normal);
    let sun = params.sun.xyz;
    let toward_camera = normalized(params.camera.xyz-input.position);
    let distance_to_camera = distance(input.position,params.camera.xyz);
    let sun_elevation = dot(radial,sun);
    let daylight = smoothstep(-0.13,0.20,sun_elevation);
    let direct = max(dot(n,sun),0.0)*daylight;
    let skylight = input.skylight;
    let cell_variation = 0.94+random(input.seed)*0.12;
    var base = vec3(0.12,0.32,0.075);
    var tile = vec2(0.,0.);
    if input.material==1u { base=vec3(0.61,0.48,0.25); tile=vec2(3.,2.); }
    if input.material==3u { base=vec3(0.07,0.25,0.105); }
    if input.material==4u { base=vec3(0.64,0.36,0.13); tile=vec2(3.,2.); }
    if input.material==5u { base=vec3(0.31,0.34,0.33); tile=vec2(3.,0.); }
    if input.material==6u { base=vec3(0.80,0.90,0.91); tile=vec2(3.,0.); }
    if input.material==7u { base=vec3(0.32,0.36,0.22); }
    if input.kind==1u {
        base=mix(vec3(0.30,0.21,0.13),vec3(0.34,0.36,0.37),smoothstep(60.,180.,input.height));
        tile=select(vec2(2.,0.),vec2(3.,0.),input.height>170.);
    }
    if input.material==8u { base=vec3(0.16,0.105,0.055); tile=vec2(2.,1.); }
    if input.material==9u { base=vec3(0.085,0.24,0.060); tile=vec2(0.,2.); }
    let texel = pixel_tile(input.uv,tile);
    let luminance = dot(texel,vec3(0.2126,0.7152,0.0722));
    let detail = mix(clamp(luminance*3.2,0.55,1.65),1.0,smoothstep(180.,1400.,distance_to_camera));
    var albedo = base*detail*cell_variation;
    // A tiny cap-edge darkening makes the actual hex-column silhouette legible
    // while the atlas supplies the committed source pixel art at close range.
    var color = albedo*(vec3(0.16,0.21,0.27)*mix(0.12,1.,daylight)*skylight
        + vec3(1.12,1.03,0.87)*direct*skylight);
    if input.kind==1u {
        // Broad sky fill keeps the pixel-art dirt and stone legible on the
        // shaded sides of terraces, instead of turning every step into black.
        color=albedo*(vec3(0.30,0.32,0.34)*mix(0.20,1.,daylight)*skylight
            + vec3(1.12,1.03,0.87)*direct*skylight)*0.95;
    }
    if input.material==0u {
        let depth = max(-input.height,0.);
        let shallows = exp(-depth*0.028);
        let water = mix(vec3(0.012,0.072,0.16),vec3(0.055,0.39,0.38),shallows);
        let fresnel = pow(1.-clamp(dot(radial,toward_camera),0.,1.),4.);
        let pixel_world = floor(input.position/3.0)*3.0;
        let wave = sin(dot(pixel_world,vec3(0.13,0.18,0.17))+params.settings.z*0.75)
            *sin(dot(pixel_world,vec3(-0.19,0.11,0.08))-params.settings.z*0.55);
        let halfvector = normalized(sun+toward_camera);
        let specular = pow(max(dot(radial,halfvector)+wave*0.008,0.),160.);
        color = water*(0.25+direct*0.85)*mix(0.18,1.,daylight);
        color+=vec3(0.09,0.22,0.34)*fresnel*daylight;
        color+=vec3(1.3,1.18,0.85)*specular*daylight;
        color+=vec3(0.12,0.26,0.26)*max(wave-0.65,0.)*shallows*daylight;
        if depth<14. { color=mix(color,vec3(0.38,0.58,0.49)*daylight,0.18); }
    }
    // Tenebris-style limb and distance haze, all in the same body-local frame.
    let altitude = max(length(params.camera.xyz)-params.settings.x,0.);
    let air = exp(-altitude/1050.);
    let fog = (1.-exp(-distance_to_camera*0.00036))*air*daylight;
    let sky = mix(vec3(0.10,0.20,0.29),vec3(0.32,0.49,0.57),max(sun_elevation,0.));
    color=mix(color,sky,fog*0.55);
    let rim = pow(1.-clamp(dot(radial,toward_camera),0.,1.),4.);
    color+=vec3(0.07,0.16,0.25)*rim*daylight*(1.-air)*0.55;
    return vec4(color,1.);
}
