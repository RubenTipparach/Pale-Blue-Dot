// Persistent topology vertex pulling, pixel atlas materials, exposed column
// skirts and cosmetic trees. Terminator/skylight/rim treatment follows the
// previously documented Tenebris port in hex_terrain.wgsl. The sea is not
// drawn here: water cells draw their seabed and water.wgsl draws the sheet.
struct Cell {
    direction_height: vec4<f32>,
    corners: array<vec4<f32>,6>,
    metadata: vec4<u32>,
    owner_a: vec4<f32>,
    owner_b: vec4<f32>,
    floors: vec4<f32>,
    spare: vec4<f32>,
}
struct Params {
    clip_from_body: mat4x4<f32>, camera: vec4<f32>, sun: vec4<f32>, settings: vec4<f32>,
    water_absorption: vec4<f32>, water_deep: vec4<f32>, weather: vec4<f32>,
    rain: array<vec4<f32>,4>,
    lod_offsets: vec4<u32>, // x base count, y fine-region capacity
    lod_counts: vec4<u32>,  // live records per fine level, coarsest first
    lod: vec4<f32>,         // xyz player direction, w base level
    bands: vec4<f32>,       // cos(band radius / R) per fine level, coarsest first
}
fn base_level() -> u32 { return u32(params.lod.w); }
fn finest_level() -> u32 { return base_level() + 4u; }
fn band_cos(level: u32) -> f32 {
    let k = level - base_level() - 1u;
    if k == 0u { return params.bands.x; }
    if k == 1u { return params.bands.y; }
    if k == 2u { return params.bands.z; }
    return params.bands.w;
}
// Whether the level below's cell a tile belongs to is drawn at this tile's
// level: the partition rule, one dot product against the player direction.
fn owner_fine(owner: vec3<f32>, level: u32) -> bool {
    return dot(owner, params.lod.xyz) > band_cos(level);
}
// Whether a tile at this level is covered by the next finer band.
fn covered_by_finer(direction: vec3<f32>, level: u32) -> bool {
    return level < finest_level() && dot(direction, params.lod.xyz) > band_cos(level + 1u);
}
fn floor_of(cell: Cell, side: u32) -> f32 {
    if side == 0u { return cell.owner_a.w; }
    if side == 1u { return cell.owner_b.w; }
    return cell.floors[side - 2u];
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
    @location(8) @interpolate(flat) level: u32,
    @location(9) @interpolate(flat) owner_a: vec3<f32>,
    @location(10) @interpolate(flat) owner_b: vec3<f32>,
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
    let degree = cell.metadata.x & 0xffu;
    let level = cell.metadata.x >> 8u;
    let axis = cell.direction_height.xyz;
    let height = cell.direction_height.w;
    // The cap at its real height: a water cell draws its seabed here and the
    // sheet over it is the water pass's.
    let radius = params.settings.x + height;
    let tile = 1.2087*params.settings.x/f32(1u << level);
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
            uv = vec2(dot(local,tangent),dot(local,bitangent))/(1.5*tile) + 0.5;
        }
    } else if vertex < 54u {
        kind = 1u;
        let side = (vertex-18u)/6u;
        let i = (vertex-18u)%6u;
        if side < degree {
            // The wall goes down to the neighbour's cap, or, where the
            // neighbour's region is drawn by the finer band, to the fine
            // floor: the height at the edge midpoint, which is the midpoint
            // cell that meets this edge. The neighbour's direction is the
            // centre reflected through the edge midpoint.
            var lower_height = cell.corners[side].w;
            if level < finest_level() {
                let mid = normalized(cell.corners[side].xyz + cell.corners[(side+1u)%degree].xyz);
                let neighbor = normalized(2.0*mid - axis);
                if covered_by_finer(neighbor, level) {
                    lower_height = min(lower_height, floor_of(cell, side));
                }
            }
            if height > lower_height {
                let a = cell.corners[side].xyz;
                let b = cell.corners[(side+1u)%degree].xyz;
                let lower = params.settings.x + lower_height;
                let points = array<vec3<f32>,4>(a*lower,b*lower,b*radius,a*radius);
                let indices = array<u32,6>(0u,1u,2u,0u,2u,3u);
                position = points[indices[i]];
                normal = normalized(cross(points[1]-points[0],points[3]-points[0]));
                let side_uv = array<vec2<f32>,4>(vec2(0.,1.),vec2(1.,1.),vec2(1.,0.),vec2(0.,0.));
                uv = side_uv[indices[i]]*vec2(1.,max(1.,(radius-lower)/1.0));
            }
        }
    } else if vertex < 60u {
        // The cut wall: a midpoint cell with one fine owner is split along
        // its long diagonal, and where its cap stands above the coarse
        // owner's, this quad closes the step along that diagonal.
        kind = 1u;
        let i = vertex-54u;
        let a_fine = level > base_level() && owner_fine(cell.owner_a.xyz, level);
        let b_fine = level > base_level() && owner_fine(cell.owner_b.xyz, level);
        if a_fine != b_fine {
            let coarse = select(cell.owner_a.xyz, cell.owner_b.xyz, a_fine);
            // The two corners equidistant from the owners lie on the diagonal.
            var c1 = 0u; var c2 = 1u;
            var best1 = 1e9; var best2 = 1e9;
            for (var c = 0u; c < degree; c++) {
                let gap = abs(dot(cell.corners[c].xyz, cell.owner_a.xyz) - dot(cell.corners[c].xyz, cell.owner_b.xyz));
                if gap < best1 { best2 = best1; c2 = c1; best1 = gap; c1 = c; }
                else if gap < best2 { best2 = gap; c2 = c; }
            }
            // The coarse cap's height is this cell's neighbour toward it.
            var side = 0u; var best = -2.0;
            for (var s = 0u; s < degree; s++) {
                let mid = normalized(cell.corners[s].xyz + cell.corners[(s+1u)%degree].xyz);
                let along = dot(mid, coarse);
                if along > best { best = along; side = s; }
            }
            let lower_height = min(cell.corners[side].w, height);
            let lower = params.settings.x + lower_height;
            var p1 = cell.corners[c1].xyz;
            var p2 = cell.corners[c2].xyz;
            // Face the coarse side, which is where the step is seen from.
            let facing = cross(p2*lower - p1*lower, p1*radius - p1*lower);
            if dot(facing, coarse - axis) < 0.0 { let t = p1; p1 = p2; p2 = t; }
            let points = array<vec3<f32>,4>(p1*lower,p2*lower,p2*radius,p1*radius);
            let indices = array<u32,6>(0u,1u,2u,0u,2u,3u);
            position = points[indices[i]];
            normal = normalized(cross(points[1]-points[0],points[3]-points[0]));
            let side_uv = array<vec2<f32>,4>(vec2(0.,1.),vec2(1.,1.),vec2(1.,0.),vec2(0.,0.));
            uv = side_uv[indices[i]]*vec2(1.,max(1.,(radius-lower)/1.0));
        }
    } else {
        kind = 2u;
        let seed = hash(cell.metadata.w);
        // The compute pass selects nearby vegetated cells before submitting
        // this separate indirect draw; no rejected tree vertices are invoked.
        // Authored for the 2.833 m tile: a trunk under two metres across and
        // a crown about six metres up, the size of a Tenebris tree.
        let part = (vertex-60u)/36u;
        let cube = box_vertex((vertex-60u)%36u);
        let scale = (0.85+random(seed)*0.50)*0.3;
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
    var out: VertexOut;
    out.position = position;
    out.clip = params.clip_from_body*vec4(position,1.);
    out.normal = normal;
    out.uv = uv;
    out.material = material;
    out.height = height;
    out.skylight = f32(cell.metadata.z)/65535.;
    out.seed = cell.metadata.w;
    out.kind = kind;
    out.level = level;
    out.owner_a = cell.owner_a.xyz;
    out.owner_b = cell.owner_b.xyz;
    return out;
}

fn pixel_tile(uv: vec2<f32>, tile: vec2<f32>) -> vec3<f32> {
    // Texture-load is intentionally nearest, with an explicit 32x32 source
    // grid per tile. Interior sampling avoids the source sheet's soft seams.
    let pixel = (floor(fract(uv)*32.)+0.5)/32.;
    let sheet_uv = (tile+0.025+pixel*0.95)*0.25;
    return textureLoad(atlas,vec2<i32>(sheet_uv*vec2<f32>(textureDimensions(atlas))),0).rgb;
}

// ---- Rain wetness, from Tenebris's hex.fs: raindrop rings and a rippled wet
// sheet on up-faces, rivulets trickling down side faces and trunks, a wet
// darkening, a sky sheen and a sun glint. The kernels' inner constants are the
// effect's identity and stay inline; the designer knobs are `params.rain`.
fn rr_hash12(p: vec2<f32>) -> f32 {
    var p3 = fract(vec3(p.x,p.y,p.x)*0.1031);
    p3 += dot(p3, p3.yzx+19.19);
    return fract((p3.x+p3.y)*p3.z);
}
fn rr_hash22(p: vec2<f32>) -> vec2<f32> {
    var p3 = fract(vec3(p.x,p.y,p.x)*vec3(0.1031,0.1030,0.0973));
    p3 += dot(p3, p3.yzx+19.19);
    return fract((p3.xx+p3.yz)*p3.zy);
}
// Zavie / Ctrl-Alt-Test "H - Immersion" raindrop rings as a surface gradient.
// The finite-difference step must exceed planet-scale f32 quantisation or the
// derivative degrades to noise.
fn rain_ripple_grad(uv: vec2<f32>, t: f32) -> vec2<f32> {
    let p0 = floor(uv);
    var circles = vec2(0.);
    for (var j=-1; j<=1; j++) {
        for (var i=-1; i<=1; i++) {
            let pi = p0+vec2(f32(i),f32(j));
            let p = pi+rr_hash22(pi);
            let tt = fract(0.3*t+rr_hash12(pi));
            let v = p-uv;
            let len = length(v)+1e-6;
            let d = len-2.0*tt;
            let h = 0.012;
            let d1 = d-h;
            let d2 = d+h;
            let q1 = sin(31.0*d1)*smoothstep(-0.6,-0.3,d1)*smoothstep(0.0,-0.3,d1);
            let q2 = sin(31.0*d2)*smoothstep(-0.6,-0.3,d2)*smoothstep(0.0,-0.3,d2);
            circles += 0.5*(v/len)*((q2-q1)/(2.0*h)*(1.0-tt)*(1.0-tt));
        }
    }
    return circles/9.0;
}
fn rd_n(t: f32) -> f32 { return fract(sin(t*12345.564)*7658.76); }
// Rivulets: a few vertical channels per face at hashed offsets, carrying
// brightness ripples that scroll down, so side water reads as streams.
fn rivulets(uv: vec2<f32>, t: f32) -> f32 {
    let lane = floor(uv.x);
    let fx = fract(uv.x);
    let h1 = rd_n(lane);
    let h2 = rd_n(lane+41.0);
    let h3 = rd_n(lane+91.0);
    let lane_on = step(0.32,h3);
    let center = 0.5+(h1-0.5)*0.3+0.06*sin(uv.y*1.7+h2*6.2831);
    let halfw = 0.07+0.09*h2;
    var across = smoothstep(halfw,0.0,abs(fx-center));
    across *= smoothstep(0.0,0.06,fx)*smoothstep(1.0,0.94,fx);
    let y = uv.y+t;
    let flow = 0.62+0.26*sin(y*6.2831+h1*6.2831)+0.12*sin(y*17.0-h2*9.0);
    return lane_on*across*clamp(flow,0.0,1.0);
}

@fragment
fn fragment(input: VertexOut) -> @location(0) vec4<f32> {
    let radial = normalized(input.position);
    // The partition: a midpoint cell split between a fine and a coarse owner
    // draws only the half nearer the fine one; the coarse cap draws the rest.
    if input.level > base_level() && input.kind != 2u {
        let a_fine = owner_fine(input.owner_a, input.level);
        let b_fine = owner_fine(input.owner_b, input.level);
        if a_fine != b_fine {
            let nearer_a = dot(radial, input.owner_a) >= dot(radial, input.owner_b);
            if (nearer_a && !a_fine) || (!nearer_a && !b_fine) { discard; }
        }
    }
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
    // Material 0 is the seabed: sand, darkened by the water column below.
    if input.material==0u { base=vec3(0.52,0.45,0.30); tile=vec2(3.,2.); }
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
    // Submerged terrain: Tenebris's hex.fs absorption, the sheet's own
    // absorption and deep colour so the seabed tints the way its sea does.
    let water_depth = max(params.water_absorption.w - length(input.position), 0.);
    if water_depth > 0. {
        let attenuation = exp(-params.water_absorption.rgb*water_depth);
        color = mix(color*attenuation, params.water_deep.rgb*attenuation,
            (1.-dot(attenuation,vec3(1./3.)))*0.5);
    }
    // Rain wetness, gated by sky light (caves stay dry) and by being above
    // the waterline (no rings on the seabed).
    let above_water = 1.-step(0.001,water_depth);
    let wet_amt = clamp(params.weather.x,0.,1.)*skylight*above_water;
    if wet_amt > 0.001 {
        let k_ripple_scale = params.rain[0].x;
        let k_ripple_strength = params.rain[0].y;
        let k_flow_across = params.rain[0].z;
        let k_flow_down = params.rain[0].w;
        let k_flow_speed = params.rain[1].x;
        let k_flow_strength = params.rain[1].y;
        let k_wave_scale = params.rain[1].z;
        let k_wave_strength = params.rain[1].w;
        let k_wave_speed = params.rain[2].x;
        let k_wet_darken = params.rain[2].y;
        let k_sky_sheen = params.rain[2].z;
        let k_glint_power = params.rain[2].w;
        let k_glint_strength = params.rain[3].x;
        let wet_up = radial;
        let face = dot(n,wet_up);
        let top_w = smoothstep(0.35,0.85,face);
        let side_w = 1.-smoothstep(0.1,0.5,face);
        let wet_t = params.settings.z;
        // Triplanar UV on fixed body axes: a dot against the radial tangent is
        // roundoff at planet scale and reads as per-pixel noise.
        let an = abs(n);
        var uv = input.position.xz;
        var ax_a = vec3(1.,0.,0.);
        var ax_b = vec3(0.,0.,1.);
        if an.x >= an.y && an.x >= an.z { uv = input.position.zy; ax_a = vec3(0.,0.,1.); ax_b = vec3(0.,1.,0.); }
        else if an.z > an.y { uv = input.position.xy; ax_a = vec3(1.,0.,0.); ax_b = vec3(0.,1.,0.); }
        ax_a = normalized(ax_a-n*dot(ax_a,n));
        ax_b = normalized(ax_b-n*dot(ax_b,n));
        var pg = vec2(0.);
        if top_w > 0.001 {
            // A continuous rippled sheet so the whole wet face reads as a
            // normal map, then raindrop impact rings on top of it.
            pg += vec2(
                cos(uv.x*k_wave_scale+wet_t*k_wave_speed)
                    +0.7*cos((uv.x+uv.y)*k_wave_scale*0.6-wet_t*k_wave_speed*0.8),
                cos(uv.y*k_wave_scale-wet_t*k_wave_speed*0.9)
                    +0.7*cos((uv.x-uv.y)*k_wave_scale*0.6+wet_t*k_wave_speed*0.7)
            )*(k_wave_strength*top_w);
            pg += rain_ripple_grad(uv*k_ripple_scale,wet_t)*(k_ripple_strength*top_w);
        }
        var pert = ax_a*pg.x+ax_b*pg.y;
        if side_w > 0.001 {
            // Down is radially inward within the face; the rivulets live in the
            // face's own texture UV so they line up with it.
            var down_dir = -wet_up-n*dot(-wet_up,n);
            let ddl = length(down_dir);
            down_dir = select(ax_a, down_dir/ddl, ddl > 1e-4);
            let across_dir = normalized(cross(n,down_dir));
            let duv = vec2(input.uv.x*k_flow_across,input.uv.y*k_flow_down);
            let dt = wet_t*k_flow_speed;
            let dm = rivulets(duv,dt);
            let de = vec2(0.01,0.);
            let dn = vec2(rivulets(duv+de,dt)-dm, rivulets(duv+de.yx,dt)-dm);
            pert += (across_dir*dn.x+down_dir*dn.y)*(k_flow_strength*side_w);
        }
        let wet = wet_amt*max(top_w,side_w);
        let wet_n = normalized(n+pert);
        color *= mix(1.,k_wet_darken,wet);
        // Sky sheen off the ambient so the ripples show under the overcast,
        // and a direct-sun glint that only adds when the sun is out.
        let ambient = vec3(0.16,0.21,0.27)*mix(0.12,1.,daylight);
        let sky_tilt = clamp(dot(wet_n,wet_up),0.,1.)-clamp(dot(n,wet_up),0.,1.);
        color += ambient*(sky_tilt*k_sky_sheen*wet*skylight);
        let glint = pow(max(dot(wet_n,sun),0.),max(k_glint_power,1.));
        color += vec3(k_glint_strength)*(glint*wet*daylight);
    }
    // Tenebris-style limb and distance haze, all in the same body-local frame.
    let altitude = max(length(params.camera.xyz)-params.settings.x,0.);
    let air = exp(-altitude/1050.);
    let fog = (1.-exp(-distance_to_camera*0.00036))*air*daylight;
    let sky = mix(vec3(0.10,0.20,0.29),vec3(0.32,0.49,0.57),max(sun_elevation,0.));
    color=mix(color,sky,fog*0.55);
    let rim = pow(1.-clamp(dot(radial,toward_camera),0.,1.),4.);
    color+=vec3(0.07,0.16,0.25)*rim*(0.25+0.75*daylight)*(1.-air)*0.55;
    return vec4(color,1.);
}
