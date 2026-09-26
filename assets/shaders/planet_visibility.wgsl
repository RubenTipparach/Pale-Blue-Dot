// Derived GPU visibility only. CPU terrain heights remain authoritative.
struct Cell {
    direction_height: vec4<f32>,
    corners: array<vec4<f32>,6>,
    metadata: vec4<u32>,
    owner_a: vec4<f32>,
    owner_b: vec4<f32>,
    floors: vec4<f32>,
    spare: vec4<u32>,  // x the tree it stands (`distance-lod-fade`)
}
struct Params {
    clip_from_body: mat4x4<f32>, camera: vec4<f32>, sun: vec4<f32>, settings: vec4<f32>,
    water_absorption: vec4<f32>, water_deep: vec4<f32>, weather: vec4<f32>,
    rain: array<vec4<f32>,6>, // wet knobs; overcast sun, fill, fog per cover, fog in rain; sky blue cut, sky dim
    lod_offsets: vec4<u32>, // x base count, y fine-region capacity
    lod_counts: vec4<u32>,  // live records per fine level, coarsest first
    lod: vec4<f32>,         // xyz player direction, w base level
    bands: vec4<f32>,       // cos(band radius / R) per fine level, coarsest first
    clutter: vec4<f32>,        // reach m, fade m, blades, base shade
    clutter_chance: vec4<f32>, // grass, flower, rock, bush
    clutter_size: vec4<f32>,   // blade height, blade half-width, rock, bush
    clutter_more: vec4<f32>,   // flower height, shrub chance, shrub size, spare
    column: vec4<f32>,         // tier reach m, cave dark floor, cave dark depth m, cos(2 x reach / R)
    ground: vec4<f32>,         // sod depth m, soil depth m, snow tileset slot, spare
    tilesets: array<vec4<u32>,2>, // atlas slot per biome, in Biome order
    fade: vec4<f32>,           // `detail-fade`: tree fade m, cross-fade progress 0..1, one while it runs, spare
    lod_prev: vec4<f32>,       // the partition the cross-fade leaves: xyz its anchor
    bands_prev: vec4<f32>,     // and its band cosines
    records_in: vec4<f32>,     // the records' ring per fine level round `anchor`, cosines, a tile inside:
    records_out: vec4<f32>,    // inner and outer (`detail-fade` section 4)
    anchor: vec4<f32>,         // `distance-lod-fade`: the fine set's anchor (`lod` is the fade centre),
    bands_in: vec4<f32>,       // the bands' cross-fade rings' inner edges, cosines,
    bands_prev_in: vec4<f32>,  // and those of the partition a landing dissolves from
}
fn base_level() -> u32 { return u32(params.lod.w); }
fn finest_level() -> u32 { return base_level() + 4u; }
// A partition of the ground into detail levels: the anchor the bands are
// measured from and the cosine of each fine level's band. The current one is
// `lod`/`bands`; while a landing cross-fades (`detail-fade`) the one it
// replaced is `lod_prev`/`bands_prev`, and an instance drawn for it carries
// PART_OLD in the top bits of its list entry.
struct Partition { anchor: vec3<f32>, bands: vec4<f32>, bands_in: vec4<f32> }
const PART_BOTH: u32 = 0u;
const PART_NEW: u32 = 1u;
const PART_OLD: u32 = 2u;
const PART_SHIFT: u32 = 30u;
const PART_MASK: u32 = 0x3fffffffu;
fn partition_of(mark: u32) -> Partition {
    if mark == PART_OLD { return Partition(params.lod_prev.xyz, params.bands_prev, params.bands_prev_in); }
    return Partition(params.lod.xyz, params.bands, params.bands_in);
}
// How far a direction is into fine level `level`'s band, 0..1, across its
// cross-fade ring (`distance-lod-fade`), as the surface pass measures it.
fn band_t(direction: vec3<f32>, level: u32, part: Partition) -> f32 {
    if level <= base_level() { return 1.0; }
    if level > finest_level() { return 0.0; }
    let out_cos = band_cos_in(part.bands, level);
    if out_cos > 1.0 { return 0.0; }
    let in_cos = band_cos_in(part.bands_in, level);
    let d = dot(direction, part.anchor);
    let span = in_cos - out_cos;
    if span <= 1e-9 { return select(0.0, 1.0, d > out_cos); }
    return clamp((d - out_cos)/span, 0.0, 1.0);
}
fn band_cos_in(bands: vec4<f32>, level: u32) -> f32 {
    let k = level - base_level() - 1u;
    if k == 0u { return bands.x; }
    if k == 1u { return bands.y; }
    if k == 2u { return bands.z; }
    return bands.w;
}
fn band_cos(level: u32) -> f32 { return band_cos_in(params.bands, level); }
// Whether the level below's cell a tile belongs to is drawn at this tile's
// level: the partition rule, one dot product against the partition's anchor.
fn owner_fine_in(owner: vec3<f32>, level: u32, part: Partition) -> bool {
    return dot(owner, part.anchor) > band_cos_in(part.bands, level);
}
fn owner_fine(owner: vec3<f32>, level: u32) -> bool {
    return owner_fine_in(owner, level, partition_of(PART_NEW));
}
// Whether a tile at this level is covered by the next finer band.
fn covered_by_finer_in(direction: vec3<f32>, level: u32, part: Partition) -> bool {
    return level < finest_level() && dot(direction, part.anchor) > band_cos_in(part.bands, level + 1u);
}
fn covered_by_finer(direction: vec3<f32>, level: u32) -> bool {
    return covered_by_finer_in(direction, level, partition_of(PART_NEW));
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
@group(0) @binding(3) var<storage,read_write> args: array<DrawArgs,5>;
@group(0) @binding(4) var<storage,read_write> foliage: array<u32>;
@group(0) @binding(5) var<storage,read_write> water: array<u32>;
@group(0) @binding(6) var<storage,read_write> clutter: array<u32>;
@group(0) @binding(7) var<storage,read_write> column: array<u32>;

// The bottom of a column's span, metres against sea level. `column::BASE_M` in
// pbd-core is the one source; planet_surface.wgsl carries the same constant and
// a test holds all three together.
const COLUMN_BASE_M: f32 = -145.0;

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
    // The inside of the world: a ceiling and a floor per run, then a flank per
    // side per run per stretch of the neighbour's air, then a torch.
    // COLUMN_VERTICES and COLUMN_FIRST_VERTEX in planet.rs are the same
    // arithmetic and a test holds them together.
    args[4].vertex_count = 894u;
    atomicStore(&args[4].instance_count, 0u);
    args[4].first_vertex = 690u;
    args[4].first_instance = 0u;
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
fn has_nearby_foliage(cell: Cell, center: vec3<f32>, part: Partition) -> bool {
    let level = cell.metadata.x >> 8u;
    // Trees stand on every fine level, the coarsest fading them out across
    // its ring (`distance-lod-fade`).
    if level + 3u < finest_level() || level > finest_level() { return false; }
    if !(owner_fine_in(cell.owner_a.xyz, level, part) && owner_fine_in(cell.owner_b.xyz, level, part)) { return false; }
    // The Tenebris scatter rule at its own rates. Eligibility is the top
    // block, as it is there: a grass of any kind, or the one tree that grows
    // on a non-grass top, the tundra pine standing in snow. Density is per
    // BIOME out of 256 - jungles pack a closed canopy, swamps grow scattered
    // groves, fields keep the classic 5%, tundra scatters lone pines, and
    // desert and mountain rock grow nothing. A coarse cell stands the tree
    // of the finest cell at its centre (`distance-lod-fade`): the same roll
    // on the same id, so the tree a coarse level keeps is the fine tree that
    // stood there, never one of its own at another place. A quarter of the
    // trees per level out, each wider (the surface pass), so the cover holds.
    let roll = hash(cell.spare.x) & 0xffu;
    let material = cell.metadata.y & 0xffu;
    let biome = (cell.metadata.y >> 8u) & 0xffu;
    let grass = material==2u || material==3u || material==7u;
    let pine = biome==7u && material==6u;
    var density = 0u;
    if biome==4u { density = 115u; }
    if biome==5u { density = 34u; }
    if biome==2u || biome==1u { density = 13u; }
    if biome==7u { density = 2u; }
    let tree = (grass||pine) && roll < density;
    return tree && distance(params.camera.xyz,center)<params.settings.w;
}

// Whether this cell has a voxel column to draw the inside of. The CPU decides
// membership when it builds the tier and stamps the slot into the record, so
// this is a read rather than a second copy of the radius rule. A reach of zero
// is the config's off switch and turns the whole pass off here.
fn has_column(cell: Cell, center: vec3<f32>) -> bool {
    if params.column.x <= 0. { return false; }
    if (cell.metadata.x >> 8u) != finest_level() { return false; }
    // The slot plus one, in the high half of the skylight word. An integer
    // field rather than a spare f32: a slot written as float bits is a denormal
    // and a driver may flush it to zero, which would read as no column at all.
    if (cell.metadata.z >> 16u) == 0u { return false; }
    // ANGULAR reach, not a straight-line distance to the cell's cap.
    //
    // The first cut measured `distance(camera, center)` against twice the
    // tier's reach, and `center` is the cell's SURFACE point. A camera deep
    // underground is far from every surface point around it, so at 217 m down
    // the gate dropped every column in the tier and the capture came back with
    // no cave in it at all - the world seen from below, exactly as it looks
    // with the tier switched off. How far down the camera is has nothing to do
    // with whether a column near it is worth drawing.
    let camera_dir = params.camera.xyz*inverseSqrt(max(dot(params.camera.xyz,params.camera.xyz),1.));
    return dot(cell.direction_height.xyz,camera_dir) > params.column.w;
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
// midpoint cell with one fine owner draws and is split per fragment. Across a
// band's cross-fade ring a tile is drawn in some pixels and not others
// (`distance-lod-fade`): it is listed when any pixel draws it, which is when
// its owner is inside the band's edge and its centre is not inside the finer
// band's ring, and the surface pass keeps its share of the pixels.
fn drawn(cell: Cell, part: Partition) -> bool {
    let level = cell.metadata.x >> 8u;
    let direction = cell.direction_height.xyz;
    if level < finest_level() && band_t(direction, level + 1u, part) >= 1.0 { return false; }
    if level > base_level() {
        return owner_fine_in(cell.owner_a.xyz, level, part) || owner_fine_in(cell.owner_b.xyz, level, part);
    }
    return true;
}

// Whether a cell both partitions draw is drawn the SAME by both: its owners'
// test (the split midpoint cell and its cut wall) and every side's test of
// the finer band (the wall down to a fine floor) agree. Such a cell is listed
// once for both; any other is listed once per partition that draws it
// (`detail-fade`). The neighbour is found as the vertex shader finds it.
fn same_in_both(cell: Cell, a: Partition, b: Partition) -> bool {
    let level = cell.metadata.x >> 8u;
    // Every pixel draws it the same under both: the owners' fades, the finer
    // band's fade at its centre (`distance-lod-fade`), and every side's wall.
    if level > base_level() {
        if abs(band_t(cell.owner_a.xyz, level, a) - band_t(cell.owner_a.xyz, level, b)) > 1e-6 { return false; }
        if abs(band_t(cell.owner_b.xyz, level, a) - band_t(cell.owner_b.xyz, level, b)) > 1e-6 { return false; }
    }
    if level < finest_level() {
        let axis = cell.direction_height.xyz;
        if abs(band_t(axis, level + 1u, a) - band_t(axis, level + 1u, b)) > 1e-6 { return false; }
        let degree = cell.metadata.x & 0xffu;
        for (var side = 0u; side < degree; side++) {
            let mid = normalize(cell.corners[side].xyz + cell.corners[(side+1u)%degree].xyz);
            let neighbor = normalize(2.0*mid - axis);
            if covered_by_finer_in(neighbor, level, a) != covered_by_finer_in(neighbor, level, b) { return false; }
        }
    }
    return true;
}

// Whether the records hold the old partition's cells at `direction` while a
// fade runs (`detail-fade` design section 4): the old partition's level there
// (the finest of its bands that covers the point) lies inside that level's
// ring of the records, a tile inside. The base is always held. Where it is
// not held, the old half of the dither would be a hole.
fn old_held(direction: vec3<f32>) -> bool {
    let old = partition_of(PART_OLD);
    let along = dot(direction, params.anchor.xyz);
    for (var level = finest_level(); level > base_level(); level--) {
        if dot(direction, old.anchor) > band_cos_in(old.bands, level) {
            let k = level - base_level() - 1u;
            return along <= params.records_in[k] && along >= params.records_out[k];
        }
    }
    return true;
}

// The list entry for a cell: its index, and which partition it is drawn for
// in the top bits. Outside a cross-fade every entry is PART_BOTH.
fn entry(index: u32, now: bool, before: bool, same: bool) -> u32 {
    if now && before && same { return index; }
    return index | (select(PART_OLD, PART_NEW, now) << PART_SHIFT);
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
    // The terrain and foliage lists take a cell twice while a landing
    // cross-fades, once per partition (`detail-fade`), so they hold two.
    if arrayLength(&cells)<count || arrayLength(&visible)<2u*count || arrayLength(&foliage)<2u*count || arrayLength(&water)<count || arrayLength(&clutter)<count || arrayLength(&column)<count { return; }
    if id.x >= count { return; }
    if !slot_live(id.x) { return; }
    let cell = cells[id.x];
    let degree = cell.metadata.x & 0xffu;
    if degree<5u || degree>6u { return; }
    let now_part = partition_of(PART_NEW);
    let before_part = partition_of(PART_OLD);
    let fading = params.fade.z > 0.5;
    let now = drawn(cell, now_part);
    let before = fading && drawn(cell, before_part);
    if !now && !before { return; }
    // A cell only the new partition draws, where the old partition's cells
    // are not among the records, is drawn whole: that ring switches at the
    // landing and the rest of it fades (`detail-fade` design section 4).
    let whole = fading && now && !before && !old_held(cell.direction_height.xyz);
    let same = !fading || whole || (now && before && same_in_both(cell, now_part, before_part));
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
        if same {
            let slot = atomicAdd(&args[0].instance_count,1u);
            visible[slot] = id.x;
        } else {
            if now {
                let slot = atomicAdd(&args[0].instance_count,1u);
                visible[slot] = entry(id.x, true, false, false);
            }
            if before {
                let slot = atomicAdd(&args[0].instance_count,1u);
                visible[slot] = entry(id.x, false, true, false);
            }
        }
    }
    // The sea sheet, the clutter and the columns are drawn for the current
    // partition only; the foliage below is drawn for both.
    if now && cell.direction_height.w < 0.0 {
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
    let tree_now = has_nearby_foliage(cell,center,now_part);
    let tree_before = fading && before && has_nearby_foliage(cell,center,before_part);
    if (tree_now || tree_before) && in_frustum(center,15.) {
        if tree_now && (tree_before || !fading || whole) {
            let slot = atomicAdd(&args[1].instance_count,1u);
            foliage[slot] = id.x;
        } else {
            let slot = atomicAdd(&args[1].instance_count,1u);
            foliage[slot] = entry(id.x, tree_now, tree_before, false);
        }
    }
    // A clutter bound is the cell's own hexagon and the tallest piece standing
    // on it, which is a couple of metres rather than a tree's fifteen.
    if now && has_clutter(cell,center) && in_frustum(center,4.) {
        let slot = atomicAdd(&args[3].instance_count,1u);
        clutter[slot] = id.x;
    }
    // A column's faces all lie between the bedrock floor of the span and the
    // cap, so the bound is the cap's own hexagon plus that depth. Loose on
    // purpose: the tier is a few thousand cells inside ninety metres, so the
    // tight bound - which would mean binding the runs to this pass as well -
    // buys nothing a profile could see.
    if now && has_column(cell,center) {
        let span = cell.direction_height.w - COLUMN_BASE_M;
        if in_frustum(center,terrain_bound(cell,center,surface_radius)+span) {
            let slot = atomicAdd(&args[4].instance_count,1u);
            column[slot] = id.x;
        }
    }
}
