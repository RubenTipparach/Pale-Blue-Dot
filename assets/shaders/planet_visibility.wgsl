// Derived GPU visibility only. CPU terrain heights remain authoritative.
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
    clutter: vec4<f32>,        // reach m, fade m, blades, base shade
    clutter_chance: vec4<f32>, // grass, flower, rock, bush
    clutter_size: vec4<f32>,   // blade height, blade half-width, rock, bush
    clutter_more: vec4<f32>,   // flower height, shrub chance, shrub size, spare
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
struct DrawArgs {
    vertex_count: u32,
    instance_count: atomic<u32>,
    first_vertex: u32,
    first_instance: u32,
}
@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<storage,read> cells: array<Cell>;
@group(0) @binding(2) var<storage,read_write> visible: array<u32>;
@group(0) @binding(3) var<storage,read_write> args: array<DrawArgs,4>;
@group(0) @binding(4) var<storage,read_write> foliage: array<u32>;
@group(0) @binding(5) var<storage,read_write> water: array<u32>;
@group(0) @binding(6) var<storage,read_write> clutter: array<u32>;

@compute @workgroup_size(1)
fn clear_indirect() {
    // Cap, walls and the cut wall; the tree draw starts after them.
    args[0].vertex_count = 60u;
    atomicStore(&args[0].instance_count, 0u);
    args[0].first_vertex = 0u;
    args[0].first_instance = 0u;
    args[1].vertex_count = 198u;
    atomicStore(&args[1].instance_count, 0u);
    args[1].first_vertex = 60u;
    args[1].first_instance = 0u;
    // The water cap: one hexagon fan per listed water cell.
    args[2].vertex_count = 18u;
    atomicStore(&args[2].instance_count, 0u);
    args[2].first_vertex = 0u;
    args[2].first_instance = 0u;
    // Ground clutter: eighteen blades of two segments, then a flower, a
    // pebble, a bush and a dead shrub. CLUTTER_VERTICES in planet.rs is the
    // same arithmetic and a test holds the two together.
    args[3].vertex_count = 432u;
    atomicStore(&args[3].instance_count, 0u);
    args[3].first_vertex = 258u;
    args[3].first_instance = 0u;
}

fn hash(x: u32) -> u32 {
    var h = x*747796405u+2891336453u;
    h = ((h >> ((h >> 28u)+4u))^h)*277803737u;
    return (h>>22u)^h;
}

// One decision about one cell, as a fraction. This is Tenebris's `hash2(tile,
// salt)` in shape rather than bit for bit: its salts index a tile of a 300 m
// world and ours index a cell record, so an identical stream would place
// nothing in the same spot anyway. What carries over is that every decision is
// a pure function of the cell, so a clutter pattern is the same on every frame
// and every run and needs nothing stored. The vertex shader repeats these
// rolls, which is the same arrangement `tile_has_rock` has with the reference's
// own mesher.
fn roll(id: u32, salt: u32) -> f32 {
    return f32(hash(id ^ (salt*2654435761u)) & 0xffffffu)/16777216.;
}

// Salts, one distinct stream per decision. Matching the reference's numbering
// so the two tables can be read side by side.
const SALT_GRASS: u32 = 0x51u;
const SALT_FLOWER: u32 = 0x52u;
const SALT_BUSH: u32 = 0x53u;
const SALT_ROCK: u32 = 0x54u;
const SALT_SHRUB: u32 = 0x55u;

fn grassy(material: u32) -> bool { return material==2u || material==3u || material==7u; }
// Dirt and beach sand share index 1, so "bare" is every dry top a pebble can
// sit on. A bush wants soil, so it asks for grass instead of this.
fn bare(material: u32) -> bool { return material==1u || material==4u || material==5u || material==6u; }

// This is the sole clutter eligibility decision, on the same terms as the
// foliage one above: the draw submits an instance only for a cell that grows
// at least one piece, and the vertex path never rejects a whole instance.
// Tenebris gates each kind on the surface BLOCK, which is this project's
// material index, and its own densities are the ones in scatter.ron.
fn has_clutter(cell: Cell, center: vec3<f32>) -> bool {
    let reach = params.clutter.x;
    if reach <= 0. { return false; }
    // The finest tier only. A coarse cell covers four or sixteen of the finest,
    // and spreading one cell's blades over that area would read as a thinning
    // sward rather than a distant one; the band it would appear in is past the
    // range a blade is a pixel wide at anyway.
    let level = cell.metadata.x >> 8u;
    if level != finest_level() { return false; }
    if !(owner_fine(cell.owner_a.xyz, level) && owner_fine(cell.owner_b.xyz, level)) { return false; }
    if distance(params.camera.xyz,center) >= reach { return false; }
    let id = cell.metadata.w;
    let material = cell.metadata.y & 0xffu;
    let green = grassy(material);
    let dry = bare(material);
    let grass = green && roll(id,SALT_GRASS) < params.clutter_chance.x;
    let flower = green && roll(id,SALT_FLOWER) < params.clutter_chance.y;
    let bush = green && roll(id,SALT_BUSH) < params.clutter_chance.w;
    // A pebble sits on bare ground, and sparsely on grass, which is the
    // reference's own third.
    let rock_chance = select(params.clutter_chance.z*0.3, params.clutter_chance.z, dry);
    let rock = (green||dry) && roll(id,SALT_ROCK) < rock_chance;
    // Dead twigs are the desert's and the tundra's only ground cover.
    let shrub = (material==4u||material==6u) && roll(id,SALT_SHRUB) < params.clutter_more.y;
    return grass || flower || bush || rock || shrub;
}

// This is the sole foliage eligibility decision. The foliage indirect draw
// submits geometry only for these cells; its vertex path never rejects trees.
// Trees stand on the three finest levels, never on a midpoint cell that is
// split between a fine and a coarse owner, since its centre is the cut, and
// a coarser cell carries the chance of the finest cells it covers so the
// cover per area is the same at every distance they are drawn at.
fn has_nearby_foliage(cell: Cell, center: vec3<f32>) -> bool {
    let level = cell.metadata.x >> 8u;
    if level + 2u < finest_level() || level > finest_level() { return false; }
    if !(owner_fine(cell.owner_a.xyz, level) && owner_fine(cell.owner_b.xyz, level)) { return false; }
    // The Tenebris scatter rule at its own rates. Eligibility is the top
    // block, as it is there: a grass of any kind, or the one tree that grows
    // on a non-grass top, the tundra pine standing in snow. Density is per
    // BIOME out of 256 - jungles pack a closed canopy, swamps grow scattered
    // groves, fields keep the classic 5%, tundra scatters lone pines, and
    // desert and mountain rock grow nothing. Times four per level above the
    // finest, so the cover per area is the same at every distance.
    let cover = 1u << (2u*(finest_level()-level));
    let roll = hash(cell.metadata.w) & 0xffu;
    let material = cell.metadata.y & 0xffu;
    let biome = (cell.metadata.y >> 8u) & 0xffu;
    let grass = material==2u || material==3u || material==7u;
    let pine = biome==7u && material==6u;
    var density = 0u;
    if biome==4u { density = 115u; }
    if biome==5u { density = 34u; }
    if biome==2u || biome==1u { density = 13u; }
    if biome==7u { density = 2u; }
    let tree = (grass||pine) && roll < density*cover;
    return tree && distance(params.camera.xyz,center)<params.settings.w;
}

// A record slot is live when it is in the base or below its fine level's
// live count; the rest of a fine region is stale from an earlier set.
fn slot_live(slot: u32) -> bool {
    let base = params.lod_offsets.x;
    if slot < base { return true; }
    let k = (slot - base) / params.lod_offsets.y;
    let within = (slot - base) % params.lod_offsets.y;
    if k == 0u { return within < params.lod_counts.x; }
    if k == 1u { return within < params.lod_counts.y; }
    if k == 2u { return within < params.lod_counts.z; }
    return within < params.lod_counts.w;
}

// The partition rule: a tile draws at its level when the next finer band does
// not cover it and its owner at the level below is drawn at this level. A
// midpoint cell with one fine owner draws and is split per fragment.
fn drawn(cell: Cell) -> bool {
    let level = cell.metadata.x >> 8u;
    let direction = cell.direction_height.xyz;
    if covered_by_finer(direction, level) { return false; }
    if level > base_level() {
        return owner_fine(cell.owner_a.xyz, level) || owner_fine(cell.owner_b.xyz, level);
    }
    return true;
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
    let degree = cell.metadata.x & 0xffu;
    for (var i=0u; i<degree; i++) {
        let a = cell.corners[i].xyz;
        let b = cell.corners[(i+1u)%degree].xyz;
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
    if arrayLength(&cells)<count || arrayLength(&visible)<count || arrayLength(&foliage)<count || arrayLength(&water)<count || arrayLength(&clutter)<count { return; }
    if id.x >= count { return; }
    if !slot_live(id.x) { return; }
    let cell = cells[id.x];
    let degree = cell.metadata.x & 0xffu;
    if degree<5u || degree>6u { return; }
    if !drawn(cell) { return; }
    let radius = params.settings.x;
    let camera_radius = length(params.camera.xyz);
    // The angular horizons of camera and raised terrain overlap. Include a
    // column/foliage margin and base the occluder below the sea surface.
    let occluder = radius - 8.0;
    let top = radius + max(cell.direction_height.w,0.0) + 20.0;
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
    // The terrain cap sits at its real height, the seabed included; the water
    // sheet over a submerged cell is listed separately at the sheet radius.
    let surface_radius = radius + cell.direction_height.w;
    let center = cell.direction_height.xyz*surface_radius;
    if in_frustum(center,terrain_bound(cell,center,surface_radius)) {
        let slot = atomicAdd(&args[0].instance_count,1u);
        visible[slot] = id.x;
    }
    if cell.direction_height.w < 0.0 {
        let sea = params.water_absorption.w;
        let sheet = cell.direction_height.xyz*sea;
        var reach = 0.0;
        for (var i=0u; i<degree; i++) {
            reach = max(reach, distance(cell.corners[i].xyz*sea, sheet));
        }
        // The swell lifts a vertex by at most a few metres; 4 m covers it.
        if in_frustum(sheet, reach + 4.0) {
            let slot = atomicAdd(&args[2].instance_count,1u);
            water[slot] = id.x;
        }
    }
    // The largest authored tree fits inside 15 m of its base. Keep a tree
    // whose crown enters the frustum even when its terrain cap is outside.
    if has_nearby_foliage(cell,center) && in_frustum(center,15.) {
        let slot = atomicAdd(&args[1].instance_count,1u);
        foliage[slot] = id.x;
    }
    // A clutter bound is the cell's own hexagon and the tallest piece standing
    // on it, which is a couple of metres rather than a tree's fifteen.
    if has_clutter(cell,center) && in_frustum(center,4.) {
        let slot = atomicAdd(&args[3].instance_count,1u);
        clutter[slot] = id.x;
    }
}
