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
    clutter: vec4<f32>,        // reach m, fade m, blades, base shade
    clutter_chance: vec4<f32>, // grass, flower, rock, bush
    clutter_size: vec4<f32>,   // blade height, blade half-width, rock, bush
    clutter_more: vec4<f32>,   // flower height, shrub chance, shrub size, spare
    column: vec4<f32>,         // tier reach m, cave dark floor, cave dark depth m, cos(2 x reach / R)
    ground: vec4<f32>,         // sod depth m, soil depth m, snow tileset slot, spare
    tilesets: array<vec4<u32>,2>, // atlas slot per biome, in Biome order
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
// One voxel column per finest cell inside the column tier: the solid runs to
// draw, and the slots of the neighbours whose rock decides how much of a run's
// flank is actually exposed. `planet_column.rs` builds it; a cell outside the
// tier carries slot zero and this array is never read for it.
struct ColumnRec {
    runs: vec4<u32>,
    neighbors: vec4<u32>,
    more: vec4<u32>,
}
@group(0) @binding(4) var<storage,read> columns: array<ColumnRec>;
// The sky level of every cell of every column tier slot, four layers to a
// `u32`, one byte each, `LIGHT_WORDS` words per slot. `ColumnTier::gpu_light`
// packs it; nothing here writes it.
@group(0) @binding(5) var<storage,read> light: array<u32>;
// Every layer's render code of every column tier slot, eight to a `u32`,
// `MATERIAL_WORDS` words per slot: what a column-pass face WEARS. A face is
// drawn in its own voxel's material, as Tenebris's `face_tile(block, cap)`
// draws it, and not in a rule on depth under the run's top: a placed stone is
// stone at whatever depth it sits. `ColumnTier::gpu_materials` packs it.
@group(0) @binding(6) var<storage,read> materials: array<u32>;
const MATERIAL_PER_WORD: u32 = 8u;
const MATERIAL_WORDS: u32 = 40u;
// The render code of the layer `layer` of column slot `slot` (plus one, as
// the record carries it). Off the tier, or past the span: air.
fn material_at(slot: u32, layer: u32) -> u32 {
    if slot == 0u || layer >= LIGHT_LAYERS { return 0u; }
    let word = (slot - 1u) * MATERIAL_WORDS + layer / MATERIAL_PER_WORD;
    if word >= arrayLength(&materials) { return 0u; }
    return (materials[word] >> ((layer % MATERIAL_PER_WORD) * 4u)) & 0xfu;
}

// The bottom and top of a column's span, metres against sea level.
// `column::BASE_M` and `LAYERS` in pbd-core are the one source; the visibility
// shader carries BASE_M too and a test holds all three together.
const COLUMN_BASE_M: f32 = -145.0;
const COLUMN_TOP_M: f32 = 175.0;
const COLUMN_RUNS: u32 = 4u;
// A column of `COLUMN_RUNS` runs has one more stretch of air than it has runs:
// under the lowest, between each pair, and over the highest.
const COLUMN_GAPS: u32 = COLUMN_RUNS + 1u;
// Where the column branch starts in the shared vertex shader, after the
// terrain's 60, the foliage's 198 and the clutter's 432.
const COLUMN_FIRST_VERTEX: u32 = 690u;
// Per run, a cave ceiling fan and a cave floor fan.
const COLUMN_CAP_VERTICES: u32 = COLUMN_RUNS * 36u;
// Per side, per run, per neighbouring air gap, one quad.
const COLUMN_SIDE_VERTICES: u32 = COLUMN_RUNS * COLUMN_GAPS * 6u;
// One torch: a slim four-sided post and a bright cap. A column holds at most
// one, which placement enforces, so the torch drawn and the torch lighting are
// the same torch.
const COLUMN_TORCH_VERTICES: u32 = 30u;
// Where a column record carries its torch layer, plus one. `planet_column.rs`
// packs it; `the_shader_carries_the_reference_light_constants` pins the shift.
const TORCH_SHIFT: u32 = 16u;
fn torch_layer(rec: ColumnRec) -> u32 { return rec.more[3] >> TORCH_SHIFT; }
const NO_NEIGHBOR: u32 = 0xffffffffu;

// A run is absent when its TOP field is zero, not its bottom: the run holding
// the bedrock legitimately starts at layer zero, so `from` cannot be the
// sentinel. `Run::packed` in pbd-core writes exactly this.
fn run_present(word: u32) -> bool { return ((word >> 9u) & 0x1ffu) != 0u; }
fn run_lo(word: u32) -> f32 { return COLUMN_BASE_M + f32(word & 0x1ffu); }
fn run_hi(word: u32) -> f32 { return COLUMN_BASE_M + f32((word >> 9u) & 0x1ffu); }
// A run's two materials: the one metre at its top, and what the rest of it is
// made of. One material a run is a forty metre wall of rock painted like the
// meadow standing on it, which is what the first capture from inside a cave
// came back as.
fn run_code(word: u32) -> u32 { return (word >> 18u) & 0xfu; }
fn run_body(word: u32) -> u32 { return (word >> 22u) & 0xfu; }
fn column_side(rec: ColumnRec, side: u32) -> u32 {
    if side < 4u { return rec.neighbors[side]; }
    return rec.more[side - 4u];
}
// The slot a cell record carries, plus one, so the zero a record is born with
// means "no column" and nothing has to be cleared to say so.
fn column_slot(cell: Cell) -> u32 { return cell.metadata.z >> 16u; }

// ---- Voxel light: the field, and the contact darkening over it.
//
// Tenebris's `world_light.rs` bakes the field and its `hex_mesher.rs` bakes the
// corner sample into a vertex colour. There is no CPU mesh here - every vertex
// is generated in this shader - so the field is baked on the CPU into the
// buffer above and the CORNER SAMPLE happens here, at the vertex that needs it.
// `pbd_core::light` holds the same rule in Rust, tested, and
// `the_shader_carries_the_reference_light_constants` pins these numbers
// against it.

// Layers per packed word, and words per slot. `LAYERS / LIGHT_PER_WORD`.
const LIGHT_PER_WORD: u32 = 4u;
const LIGHT_WORDS: u32 = 80u;
// The brightest a cell is: Tenebris's `voxel_sky_max`.
const LIGHT_MAX: f32 = 15.0;
// Notch's three-step ladder, which is the reference's own comment and numbers:
// how much a corner darkens for each of the two SIDE neighbours that is solid
// where the face opens onto air.
const CONTACT_1: f32 = 0.85;
const CONTACT_2: f32 = 0.70;
// The fourth rung, which only a WALL can reach: the reference's ladder counts
// the two side columns at the face's own layer, and a wall adds the two cells
// one layer past its edge (`pbd_core::light::wall_corner`).
const CONTACT_3: f32 = 0.55;
fn contact(occluders: u32) -> f32 {
    if occluders == 0u { return 1.0; }
    if occluders == 1u { return CONTACT_1; }
    if occluders == 2u { return CONTACT_2; }
    return CONTACT_3;
}
// The floor under the ambient term: what a surface the sun and sky never reach
// is still lit to. Tenebris's `hex.fs` has it as a literal `max(0.05, ...)`
// inside the ambient, and `docs/tenebris-comparison.md` has recorded its
// absence here since the port ("**no floor**").
//
// Nothing needed it while every cell was four fifths lit. With a real field
// behind the sky term a cave reads 11 of 255 without it, which is not a dark
// room, it is a black screen with a hotbar on it - and a player cannot tell a
// cave from a bug in the renderer. A const rather than a uniform because every
// other colour and threshold in this shader is one too; the change that lifts
// them all into per-body data is `per-body-rendering` and it is not this one.
const AMBIENT_FLOOR: f32 = 0.05;
// What a lamp's light looks like. Tenebris's `torch_color` is (1.00, 0.70,
// 0.30) and its shipped `torch_intensity` 1.25, which are these: a flame is
// warm and a torch at full is a little brighter than the surface it lights, or
// nobody would be able to tell it was on.
const TORCH_TINT: vec3<f32> = vec3<f32>(1.00, 0.70, 0.30);
const TORCH_GAIN: f32 = 1.25;

// The layer holding an altitude, or a sentinel past the top.
const LIGHT_LAYERS: u32 = 320u;
fn light_layer(altitude: f32) -> u32 {
    let index = floor(altitude - COLUMN_BASE_M);
    if index < 0.0 { return LIGHT_LAYERS; }
    return u32(index);
}

// The sky level of one cell, 0..1. A slot of zero is no column and a layer
// past the top is open sky, which is what a tier edge and the air above the
// world both are.
// Sky in the high nibble, block in the low: `pbd_core::light::Light`'s own
// byte. Two channels because only ONE of them goes out at night.
fn light_at(slot: u32, layer: u32) -> vec2<f32> {
    if slot == 0u { return vec2(1.0, 0.0); }
    if layer >= LIGHT_LAYERS { return vec2(1.0, 0.0); }
    let word = (slot - 1u) * LIGHT_WORDS + layer / LIGHT_PER_WORD;
    if word >= arrayLength(&light) { return vec2(1.0, 0.0); }
    let byte = (light[word] >> ((layer % LIGHT_PER_WORD) * 8u)) & 0xffu;
    return vec2(f32(byte >> 4u), f32(byte & 0xfu)) / LIGHT_MAX;
}

fn sky_at(slot: u32, layer: u32) -> f32 {
    return light_at(slot, layer).x;
}

// Is this slot's cell solid at `layer`? Read off the run words, which already
// say exactly that: a layer inside any present run is rock.
//
// A slot of zero - no column - answers SOLID. Off the tier is the one place
// this shader and the light bake have to agree, and the bake treats off-region
// as solid so light cannot flood out of the tier's edge and back in.
fn solid_at(slot: u32, layer: u32) -> bool {
    if slot == 0u { return true; }
    if layer >= LIGHT_LAYERS { return false; }
    let rec = columns[slot - 1u];
    let altitude = COLUMN_BASE_M + f32(layer);
    for (var k = 0u; k < COLUMN_RUNS; k++) {
        let word = rec.runs[k];
        if run_present(word) && run_lo(word) <= altitude + 0.001
            && run_hi(word) >= altitude + 0.999 {
            return true;
        }
    }
    return false;
}

// What one corner of a face is lit to, in 0..1.
//
// `own`, `a` and `b` are the column slots that MEET at this corner, and
// `layer` is the AIR layer the face opens onto - above a top cap, below a
// bottom cap, beside a flank. Sampling the SOLID cell's own layer instead is
// the reference's recorded bug: a sealed room's ceiling came out pitch black.
//
// Two steps, both `pbd_core::light::corner`'s: the mean level over the cells
// there that are not solid, then the contact ladder on how many of the two
// SIDE neighbours are. The face's own cell is left out of the ladder because
// it says nothing - a cap's is always air and a flank's always rock.
// Both channels at a corner, or `-1` in x where nothing there was air. The
// contact ladder multiplies BOTH, as the reference's own does: a crease is
// dark whatever is lighting it.
// The mean of both channels over the cells at this corner that are air, in
// xy, and how many of the two SIDE cells are solid in z. A corner with no air
// at all answers -1 in x, which the caller has to be able to tell from dark.
fn corner_at(own: u32, a: u32, b: u32, layer: u32) -> vec3<f32> {
    var sum = vec2(0.0);
    var samples = 0.0;
    let slots = array<u32,3>(own, a, b);
    for (var i = 0u; i < 3u; i++) {
        if !solid_at(slots[i], layer) {
            sum += light_at(slots[i], layer);
            samples += 1.0;
        }
    }
    if samples == 0.0 { return vec3(-1.0, 0.0, 0.0); }
    return vec3(clamp(sum / samples, vec2(0.0), vec2(1.0)), f32(occluders_at(a, b, layer)));
}

fn occluders_at(a: u32, b: u32, layer: u32) -> u32 {
    var occluders = 0u;
    if solid_at(a, layer) { occluders++; }
    if solid_at(b, layer) { occluders++; }
    return occluders;
}

fn corner_light(own: u32, a: u32, b: u32, layer: u32) -> vec2<f32> {
    let here = corner_at(own, a, b, layer);
    if here.x >= 0.0 { return here.xy * contact(u32(here.z)); }
    // Every cell there is rock, so this is not a dark corner, it is the wrong
    // LAYER: the metre a face stands at is the neighbour's last solid one, and
    // the air it actually opens onto is the metre above. A one-metre terrace
    // riser is the whole of that case and it came out pitch black - a hard
    // dark line along every step in the world, which is the same symptom the
    // reference's own floor-contact fallback exists to prevent.
    let above = corner_at(own, a, b, layer + 1u);
    return max(above.xy * contact(u32(above.z)), vec2(0.0));
}

// A wall's corner: the cap rule plus the two cells one layer PAST the wall's
// edge - across and beside - which is Minecraft's third-neighbour rule on a
// lattice where three columns meet at a corner. It is what darkens a wall's
// foot where it stands on a floor and its head under a lid; without it a wall
// on flat ground measured 92 of 255 at its foot and 92 at its middle.
// `pbd_core::light::wall_corner` is the same rule in Rust, tested.
fn wall_corner_light(own: u32, a: u32, b: u32, layer: u32, beyond: u32) -> vec2<f32> {
    let extra = occluders_at(a, b, beyond);
    let here = corner_at(own, a, b, layer);
    if here.x >= 0.0 { return here.xy * contact(u32(here.z) + extra); }
    let above = corner_at(own, a, b, layer + 1u);
    return max(above.xy * contact(u32(above.z) + extra), vec2(0.0));
}

// What one vertex of a WALL is lit to: a quad on `side` running from `bottom`
// to `top` in metres, whose vertex `index` is one of the four corners.
//
// A wall's light is the air BESIDE it, so the cells sampled are this one and
// the two that share the vertical edge the vertex stands on. The bottom pair
// samples the bottom metre and the top pair the top metre, which is what makes
// a wall grade from its lit head to its shaded foot instead of being one flat
// tone - the difference `docs/tenebris-comparison.md` blames for the ground
// reading as faceted plates.
//
// **The foot has a fallback, and it is the reference's.** A wall standing on
// flat ground has every cell solid at its lowest metre, so the corner there
// samples no air at all and would come out black: a hard dark line along the
// bottom of every wall in the world. Where that happens the foot takes the
// head's value, because "a floor-level corner always has SOME adjacent air to
// derive light from, never a hard 0".
fn wall_light(cell: Cell, own: u32, degree: u32, side: u32,
              index: u32, bottom: f32, top: f32) -> vec2<f32> {
    if own == 0u { return vec2(1.0, 0.0); }
    let rec = columns[own - 1u];
    // Corners 0 and 3 stand on the edge's first ray, 1 and 2 on its second.
    let k = select(side, (side + 1u) % degree, index == 1u || index == 2u);
    let pair = corner_slots(rec, k, degree);
    // The bottom metre and the top metre of the quad. `corner_light` steps up
    // a layer where those are rock, which is what a wall standing on flat
    // ground always is at its foot.
    let low = light_layer(bottom + 0.5);
    let high = light_layer(max(top - 0.5, bottom + 0.5));
    // The head's cells past the edge are one layer up; the foot's one down.
    let head = wall_corner_light(own, pair.x, pair.y, high, high + 1u);
    if index == 2u || index == 3u { return head; }
    let foot = wall_corner_light(own, pair.x, pair.y, low, max(low, 1u) - 1u);
    return select(foot, head, foot.x == 0.0 && foot.y == 0.0);
}

// The first layer at or above `altitude` in this column that is not rock.
//
// A heightfield cap sits at a FLOAT height - 76.3 m - which lies inside the
// layer the ground fills, so the layer of the cap's own altitude is solid and
// holds no light. What the cap opens onto is the metre above that. Three steps
// is more than enough: the cap is by construction within a metre of the top.
fn air_above(slot: u32, altitude: f32) -> u32 {
    var layer = light_layer(altitude);
    for (var i = 0u; i < 3u; i++) {
        if !solid_at(slot, layer) { return layer; }
        layer++;
    }
    return layer;
}

// The two columns that share corner `k` of this cell, as slots.
//
// Side `s` spans corners `s` and `s+1`, so corner `k` is shared by sides
// `k-1` and `k` - the same relation the reference's `edge_neighbours` keeps.
fn corner_slots(rec: ColumnRec, k: u32, degree: u32) -> vec2<u32> {
    let before = column_side(rec, (k + degree - 1u) % degree);
    let after = column_side(rec, k % degree);
    // A neighbour off the tier has no column: the slot word is the sentinel
    // and `solid_at` answers solid for it, which is what the bake assumes too.
    return vec2<u32>(
        select(before + 1u, 0u, before == NO_NEIGHBOR),
        select(after + 1u, 0u, after == NO_NEIGHBOR),
    );
}

/// The `g`th stretch of AIR in a neighbouring column, bottom up.
///
/// This is why a column record names its neighbours. A flank drawn down its
/// run's full height is right wherever the neighbour is rock - buried,
/// invisible - and SEALS THE PASSAGE wherever the neighbour is air, which is
/// what a cave is made of: a tunnel is one run of air crossing many cells, and
/// a wall at every cell boundary turns it into sealed rooms.
///
/// Every gap, not the largest one. The first cut drew a single quad per side,
/// over whichever stretch of the neighbour's air was widest, on the reasoning
/// that the rest is a sliver. It is not: a neighbour with two gaps had the
/// second one drawn as solid rock with nothing in it, and from inside a cave
/// that is a window. Four runs against five gaps is twenty quads a side, which
/// is 864 vertices a column against the terrain pass's 60 - affordable on a
/// tier of a few thousand cells, and the exact answer rather than most of one.
fn column_gap(neighbor: u32, g: u32) -> vec2<f32> {
    // Off the tier: solid below its cap, which is what the heightfield assumes
    // everywhere, and what `planet_column.rs` makes TRUE by generating the
    // tier's outermost ring with no carve in it.
    if neighbor == NO_NEIGHBOR { return vec2(0.); }
    let rec = columns[neighbor];
    var count = 0u;
    for (var k = 0u; k < COLUMN_RUNS; k++) {
        if run_present(rec.runs[k]) { count = count + 1u; }
    }
    if g > count { return vec2(0.); }
    var lo = COLUMN_BASE_M;
    if g > 0u { lo = run_hi(rec.runs[g - 1u]); }
    var hi = COLUMN_TOP_M;
    if g < count { hi = run_lo(rec.runs[g]); }
    return vec2(lo, hi);
}

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
    // How brightly this vertex takes its own albedo. One everywhere except
    // down a grass blade, where the root is darker than the tip.
    @location(11) shade: f32,
    // The voxel field at this vertex, INTERPOLATED - which is the point of it.
    @location(12) voxel: vec2<f32>,
    // The column tier slot plus one, or zero off the tier: what a column-pass
    // face asks for the material of the layer it stands on.
    @location(13) @interpolate(flat) slot: u32,
}
fn hash(x: u32) -> u32 {
    var h = x*747796405u+2891336453u;
    h = ((h >> ((h >> 28u)+4u))^h)*277803737u;
    return (h>>22u)^h;
}
fn random(x: u32) -> f32 { return f32(hash(x)&65535u)/65535.0; }

// The reference's `block_hex_width` for a leaf: how far a layer's hex is
// shrunk toward the tile centre, its own roll per layer, so no two layers of
// one crown are the same width.
fn leaf_width(id: u32, layer: u32) -> f32 {
    var h = id*0xDEADBEEFu+layer*0x0BADF00Du+0x12345678u;
    h = h^(h>>13u);
    return 0.65+f32(h&0xffu)/255.0*0.35;
}
fn normalized(v: vec3<f32>) -> vec3<f32> { return v*inverseSqrt(max(dot(v,v),1e-12)); }

// One decision about one cell. The visibility pass decides eligibility with
// exactly this function and these salts; here the same rolls are repeated to
// build what it listed. Keep the two in step - a cell listed for a pebble that
// then rolls no pebble draws nothing at all.
fn roll(id: u32, salt: u32) -> f32 {
    return f32(hash(id ^ (salt*2654435761u)) & 0xffffffu)/16777216.;
}
const SALT_GRASS: u32 = 0x51u;
const SALT_FLOWER: u32 = 0x52u;
const SALT_BUSH: u32 = 0x53u;
const SALT_ROCK: u32 = 0x54u;
const SALT_SHRUB: u32 = 0x55u;
const TAU: f32 = 6.28318530718;

struct Piece { position: vec3<f32>, normal: vec3<f32>, uv: vec2<f32> }

// A small hexagonal prism standing on the cap: 36 vertices of sides, then an
// 18-vertex top fan. The same construction the trunk above uses, but centred on
// a hashed point INSIDE the cell rather than on the cell's own corners, because
// a pebble is a pebble rather than a shrunk copy of the tile.
fn prism_vertex(part: u32, base: vec3<f32>, up: vec3<f32>, tangent: vec3<f32>,
                bitangent: vec3<f32>, rad: f32, h: f32, phase: f32) -> Piece {
    var out: Piece;
    out.position = base;
    out.normal = up;
    out.uv = vec2(0.5);
    if part < 36u {
        let side = part/6u;
        let i = part%6u;
        let a0 = phase+f32(side)*TAU/6.;
        let a1 = phase+f32(side+1u)*TAU/6.;
        let c0 = base+(tangent*cos(a0)+bitangent*sin(a0))*rad;
        let c1 = base+(tangent*cos(a1)+bitangent*sin(a1))*rad;
        let points = array<vec3<f32>,4>(c0,c1,c1+up*h,c0+up*h);
        let indices = array<u32,6>(0u,1u,2u,0u,2u,3u);
        out.position = points[indices[i]];
        out.normal = normalized(cross(points[1]-points[0],points[3]-points[0]));
        let side_uv = array<vec2<f32>,4>(vec2(0.,1.),vec2(1.,1.),vec2(1.,0.),vec2(0.,0.));
        out.uv = side_uv[indices[i]];
    } else {
        let t = (part-36u)/3u;
        let c = (part-36u)%3u;
        let top = base+up*h;
        let a0 = phase+f32(t)*TAU/6.;
        let a1 = phase+f32(t+1u)*TAU/6.;
        let p0 = top+(tangent*cos(a0)+bitangent*sin(a0))*rad;
        let p1 = top+(tangent*cos(a1)+bitangent*sin(a1))*rad;
        out.position = select(select(p1,p0,c==1u),top,c==0u);
        out.uv = select(select(vec2(0.9,0.9),vec2(0.1,0.9),c==1u),vec2(0.5,0.5),c==0u);
    }
    return out;
}

fn grassy(material: u32) -> bool { return material==2u || material==3u || material==7u; }
fn bare(material: u32) -> bool { return material==1u || material==4u || material==5u || material==6u; }


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
    var material = cell.metadata.y & 0xffu;
    // One everywhere but down a grass blade, whose root is darker than its tip.
    var out_shade = 1.;
    // The height a face measures its depth from: the cell's surface, except on
    // a cave run's flank, where it is that run's own top.
    var out_height = height;
    // The voxel field's answer at THIS vertex, one outside the column tier
    // where there is no field to ask. Per vertex rather than per cell, which
    // is the whole of what "baked vertex colours" means here: a face grades
    // across itself because its corners were sampled apart.
    var out_voxel = vec2(1., 0.);
    let lit_slot = column_slot(cell);
    if vertex < 18u {
        let triangle = vertex/3u;
        let corner = vertex%3u;
        if triangle < degree && corner != 0u {
            let ray = cell.corners[(triangle+corner-1u)%degree].xyz;
            position = ray*radius;
            let local = (ray-axis)*params.settings.x;
            uv = vec2(dot(local,tangent),dot(local,bitangent))/(1.5*tile) + 0.5;
        }
        // The ground the player walks on is drawn HERE rather than by the
        // column pass, so this is where a block standing on it darkens the
        // crease beside it. The air the cap opens onto is the layer above it.
        if lit_slot != 0u {
            let air = air_above(lit_slot, height);
            if triangle < degree && corner != 0u {
                let k = (triangle+corner-1u)%degree;
                let pair = corner_slots(columns[lit_slot-1u], k, degree);
                out_voxel = corner_light(lit_slot, pair.x, pair.y, air);
            } else {
                // The CENTRE takes the field alone: no smoothing, no contact.
                // The reference does the same and the reason is the picture -
                // a bright centre with darkened corners is what makes the
                // rasteriser's interpolation read as a shadow gathering in the
                // corner rather than as a tile that is uniformly dimmer.
                out_voxel = light_at(lit_slot, air);
            }
        }
    } else if vertex < 54u {
        kind = 1u;
        let side = (vertex-18u)/6u;
        let i = (vertex-18u)%6u;
        // Between two cells that BOTH have columns, this wall is not drawn at
        // all: the column pass owns the whole side and draws it run by run
        // against the neighbour's air, from the bedrock to this cell's own cap.
        // ONE drawer per side. The two rules this replaces - a wall here that
        // yielded only where the rock did not fill the step, and a flank that
        // stopped at the neighbour's cap - each answered half of the side, and
        // where a dig broke a column's side under an unchanged cap neither
        // half was anybody's: a hole to the sky from inside every pit.
        var columns_side = false;
        let slot = column_slot(cell);
        if slot != 0u && side < degree {
            columns_side = column_side(columns[slot-1u], side) != NO_NEIGHBOR;
        }
        if side < degree && !columns_side {
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
                out_voxel = wall_light(cell, lit_slot, degree, side,
                    indices[i], lower_height, height);
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
    } else if vertex >= COLUMN_FIRST_VERTEX {
        // ---- The inside of the world.
        //
        // What the terrain pass draws is a cap at the surface and a wall from it
        // down to the neighbour's cap. This pass draws only what is BELOW that,
        // so nothing here is coincident with it and the LOD partition, the fine
        // floors and the cut wall are all untouched: a cave ceiling, a cave
        // floor, and a run's flank wherever the neighbour leaves air against it.
        kind = 4u;
        let slot = column_slot(cell);
        position = axis*radius;
        normal = axis;
        if slot != 0u {
            let rec = columns[slot-1u];
            let v = vertex-COLUMN_FIRST_VERTEX;
            if v < COLUMN_CAP_VERTICES {
                // A cave ceiling, then a cave floor: the same fan the cap
                // draws, at the run's own altitude, facing the air it bounds.
                let r = v/36u;
                let part = v%36u;
                let word = rec.runs[r];
                if run_present(word) {
                    let lo = run_lo(word);
                    let hi = run_hi(word);
                    let up = part >= 18u;
                    let i = select(part,part-18u,up);
                    // A cave FLOOR is the run's own top, so it wears the top
                    // material; a cave CEILING is the underside of the run
                    // above and is made of whatever that run is made of.
                    material = select(run_body(word),run_code(word),up);
                    // The lowest run's bottom is the bedrock floor of the span,
                    // which nothing can ever be under, and the highest run's top
                    // is the surface the terrain pass already capped.
                    let skip = select((word & 0x1ffu) == 0u, hi >= height-0.001, up);
                    let level = params.settings.x + select(lo,hi,up);
                    normal = select(-axis,axis,up);
                    position = axis*level;
                    let t = i/3u;
                    let c = i%3u;
                    // The air this cap opens onto: the layer above a top cap
                    // and below a bottom one. Sampling the cap's OWN layer is
                    // the reference's recorded bug - it made a sealed room's
                    // ceiling read pitch black, because the ceiling is rock
                    // and rock holds no light.
                    let air = select(light_layer(lo) - 1u, light_layer(hi), up);
                    if !skip && t < degree && c != 0u {
                        let k = select((t+2u-c)%degree,(t+c-1u)%degree,up);
                        let ray = cell.corners[k].xyz;
                        position = ray*level;
                        let local = (ray-axis)*params.settings.x;
                        uv = vec2(dot(local,tangent),dot(local,bitangent))/(1.5*tile)+0.5;
                        let pair = corner_slots(rec, k, degree);
                        out_voxel = corner_light(lit_slot, pair.x, pair.y, air);
                    } else {
                        out_voxel = light_at(lit_slot, air);
                    }
                }
            } else if v >= COLUMN_CAP_VERTICES + 6u*COLUMN_SIDE_VERTICES {
                // ---- A torch: a slim post standing on the floor with a lit
                // head on it.
                //
                // Drawn HERE rather than in the clutter, because a torch is at
                // a layer rather than on the surface: one can stand on a cave
                // floor forty metres down, which is the whole reason to carry
                // one.
                let t = v-(COLUMN_CAP_VERTICES+6u*COLUMN_SIDE_VERTICES);
                let layer = torch_layer(rec);
                if layer != 0u {
                    let foot = params.settings.x+COLUMN_BASE_M+f32(layer-1u);
                    let up = axis;
                    let post = 0.55;
                    let rad = 0.06;
                    if t < 24u {
                        // The post: four flat sides, so it reads as a stick
                        // rather than a cylinder at the pixel size it is drawn.
                        let face = t/6u;
                        let i = t%6u;
                        let a0 = f32(face)*TAU/4.;
                        let a1 = f32(face+1u)*TAU/4.;
                        let base = axis*foot;
                        let c0 = base+(tangent*cos(a0)+bitangent*sin(a0))*rad;
                        let c1 = base+(tangent*cos(a1)+bitangent*sin(a1))*rad;
                        let points = array<vec3<f32>,4>(c0,c1,c1+up*post,c0+up*post);
                        let indices = array<u32,6>(0u,1u,2u,0u,2u,3u);
                        position = points[indices[i]];
                        normal = normalized(cross(points[1]-points[0],points[3]-points[0]));
                        let post_uv = array<vec2<f32>,4>(vec2(0.,1.),vec2(1.,1.),vec2(1.,0.),vec2(0.,0.));
                        uv = post_uv[indices[i]];
                        material = 7u;
                        out_voxel = vec2(sky_at(lit_slot,layer-1u),1.0);
                    } else {
                        // The head, at full lamp: it IS the light, so it is
                        // drawn at the brightest the block channel goes rather
                        // than at whatever reached the cell it stands in.
                        let i = t-24u;
                        let head = axis*(foot+post);
                        let a0 = f32(i/3u)*TAU/2.;
                        let a1 = f32(i/3u+1u)*TAU/2.;
                        let c = i%3u;
                        let p0 = head+(tangent*cos(a0)+bitangent*sin(a0))*rad*2.2;
                        let p1 = head+(tangent*cos(a1)+bitangent*sin(a1))*rad*2.2;
                        let tip = head+up*0.16;
                        position = select(select(tip,p1,c==1u),p0,c==0u);
                        uv = select(select(vec2(0.5,0.1),vec2(0.9,0.9),c==1u),vec2(0.1,0.9),c==0u);
                        normal = up;
                        material = 7u;
                        out_voxel = vec2(0.0,1.0);
                    }
                }
            } else {
                // A flank: this run's rock against one stretch of the
                // neighbour's air. This is Tenebris's side-face rule, a face
                // wherever solid meets not-solid at the same layer, and it
                // needs nothing else because a cap and the column top under
                // it are one number (`column::surface_m`).
                let w = v-COLUMN_CAP_VERTICES;
                let side = w/COLUMN_SIDE_VERTICES;
                let rest = w%COLUMN_SIDE_VERTICES;
                let r = rest/(COLUMN_GAPS*6u);
                let g = (rest%(COLUMN_GAPS*6u))/6u;
                let i = rest%6u;
                let word = rec.runs[r];
                if side < degree && run_present(word) {
                    let lo = run_lo(word);
                    let hi = run_hi(word);
                    let gap = column_gap(column_side(rec,side),g);
                    let bottom = max(lo, gap.x);
                    let top = min(hi, gap.y);
                    if top-bottom > 0.001 {
                        // The run's TOP material, and the depth under it
                        // decides the face - sod, earth, then stone - which is
                        // the terrain wall's own rule (`face_code`) and the
                        // reference's, where a voxel's side wears the voxel.
                        // A flank painted in the run's bottom material was a
                        // stone wall standing in the first metre under the
                        // grass, beside a terrain wall drawn in earth.
                        material = run_code(word);
                        out_height = hi;
                        let a = cell.corners[side].xyz;
                        let b = cell.corners[(side+1u)%degree].xyz;
                        let lower = params.settings.x + bottom;
                        let upper = params.settings.x + top;
                        let points = array<vec3<f32>,4>(a*lower,b*lower,b*upper,a*upper);
                        let indices = array<u32,6>(0u,1u,2u,0u,2u,3u);
                        position = points[indices[i]];
                        normal = normalized(cross(points[1]-points[0],points[3]-points[0]));
                        let side_uv = array<vec2<f32>,4>(vec2(0.,1.),vec2(1.,1.),vec2(1.,0.),vec2(0.,0.));
                        uv = side_uv[indices[i]]*vec2(1.,max(1.,top-bottom));
                        out_voxel = wall_light(cell, lit_slot, degree, side,
                            indices[i], bottom, top);
                    }
                }
            }
        }
    } else if vertex < 258u {
        kind = 2u;
        // Tenebris's tree, which is not a mesh but voxels in ONE column drawn
        // as ordinary hex prisms shrunk toward the tile centre: wood at 0.20
        // of the cell and each leaf layer at 0.65 + 0.35 * its own hash, so a
        // canopy is irregular rather than stamped. The record's corner rays
        // are what a prism needs, and `shrink_corner` in the reference is the
        // same lerp toward the centre. The compute pass selects the cells; no
        // tree vertex is invoked for a cell without one.
        let id = cell.metadata.w;
        let roll = hash(id);
        let biome = (cell.metadata.y >> 8u) & 0xffu;
        // Trunk metres: the reference's 3 + a hash bit + the biome's extra,
        // so a jungle closes its canopy overhead and a swamp grove stands
        // over a long bare bole.
        var extra = 0u;
        if biome == 4u { extra = 2u; }
        if biome == 5u { extra = 3u; }
        if biome == 7u { extra = 3u; }
        let trunk = f32(3u + ((roll >> 8u) & 1u) + extra);
        // The tundra pine is the reference's one per-shape override: a single
        // wood block under a leaf run that tapers from a wide skirt to a
        // point. Its run is eight or nine one-metre layers there; here it is
        // the same span in the two the vertex budget carries, so the taper is
        // two segments rather than nine.
        let pine = biome == 7u;
        var trunk_top = trunk;
        var lo0 = trunk; var hi0 = trunk+1.; var w0a = 0.; var w0b = 0.;
        var lo1 = trunk+1.; var hi1 = trunk+2.; var w1a = 0.; var w1b = 0.;
        if pine {
            trunk_top = 1.;
            let crown = trunk+3.;
            let mid = (1.+crown)*0.5;
            lo0 = 1.; hi0 = mid; w0a = 0.95; w0b = 0.55;
            lo1 = mid; hi1 = crown; w1a = 0.55; w1b = 0.15;
        } else {
            w0a = leaf_width(id,0u); w0b = w0a;
            w1a = leaf_width(id,1u); w1b = w1a;
        }
        // Bottom and top metre, and bottom and top width, of this vertex's
        // part: the trunk, then a leaf layer's sides, floor and ceiling.
        let v = vertex-60u;
        var lo = 0.; var hi = trunk_top; var wa = 0.20; var wb = 0.20;
        material = 8u;
        if v >= 54u && v < 126u { lo = lo0; hi = hi0; wa = w0a; wb = w0b; material = 9u; }
        if v >= 126u { lo = lo1; hi = hi1; wa = w1a; wb = w1b; material = 9u; }
        let lower = radius+lo;
        let upper = radius+hi;
        let part = select(v-54u, v, v < 54u) % 72u;
        if part < 36u {
            // A side of the prism, built like the terrain wall so the two
            // agree about which way a face points.
            let side = part/6u;
            let i = part%6u;
            if side < degree {
                let ca = cell.corners[side].xyz;
                let cb = cell.corners[(side+1u)%degree].xyz;
                let points = array<vec3<f32>,4>(
                    (axis+(ca-axis)*wa)*lower,(axis+(cb-axis)*wa)*lower,
                    (axis+(cb-axis)*wb)*upper,(axis+(ca-axis)*wb)*upper);
                let indices = array<u32,6>(0u,1u,2u,0u,2u,3u);
                position = points[indices[i]];
                normal = normalized(cross(points[1]-points[0],points[3]-points[0]));
                let side_uv = array<vec2<f32>,4>(vec2(0.,1.),vec2(1.,1.),vec2(1.,0.),vec2(0.,0.));
                uv = side_uv[indices[i]]*vec2(1.,max(1.,hi-lo));
            }
        } else {
            // A cap: the fan the terrain cap already draws, at the shrunk
            // corners. The trunk carries a top cap only, because it stands on
            // the terrain and a narrow canopy does not cover it; a leaf layer
            // carries both, because the layer it meets rolled its own width.
            let up = v < 54u || part >= 54u;
            let base = select(36u,54u,v >= 54u && up);
            let t = (part-base)/3u;
            let c = (part-base)%3u;
            let r = select(lower,upper,up);
            let w = select(wa,wb,up);
            normal = select(-axis,axis,up);
            position = axis*r;
            if t < degree && c != 0u {
                let k = select((t+2u-c)%degree,(t+c-1u)%degree,up);
                let shrunk = axis+(cell.corners[k].xyz-axis)*w;
                position = shrunk*r;
                let local = (shrunk-axis)*params.settings.x;
                uv = vec2(dot(local,tangent),dot(local,bitangent))/(1.5*tile)+0.5;
            }
        }
    } else {
        kind = 3u;
        // Ground clutter: Tenebris's surface scatter, which it bakes into a CPU
        // chunk mesh and this project has no chunk mesh to bake into. So the
        // RULES are ported - the per-cell hash, the densities, the sizes, the
        // gates - and the geometry is built here from the record's own corner
        // rays, exactly as the tree above took `block_hex_width` and left the
        // mesher behind. The compute pass listed this cell; every piece below
        // repeats the roll that listed it.
        //
        // The instance's 432 vertices, in order: 18 blades of two segments,
        // then a flower, a pebble, a bush of two lumps, and a dead shrub. A
        // piece this cell did not roll collapses to a degenerate triangle at
        // the cell centre, which is how a tree part a cell does not carry is
        // already handled.
        let v = vertex-258u;
        let id = cell.metadata.w;
        let green = grassy(material);
        let dry = bare(material);
        // Everything shrinks into the ground over the last stretch of the
        // reach rather than popping out of existence: at 60 m a cell is under
        // three metres wide and its blades are two pixels, so a hard edge
        // would be a line of shimmer travelling with the player.
        let reach = params.clutter.x;
        let fade = clamp((reach-distance(params.camera.xyz,axis*radius))
            /max(params.clutter.y,0.001),0.,1.);
        let base_shade = params.clutter.w;
        // A hashed point on the cap: the reference's own lerp from the centre
        // toward a hashed corner, so a piece sits flush on the plane the cap
        // is drawn in.
        let seat = axis*radius;
        var grown = false;
        // Which piece this vertex belongs to, and the six-vertex quad or
        // three-vertex triangle within it.
        var quad = array<vec3<f32>,4>(seat,seat,seat,seat);
        var quad_uv = array<vec2<f32>,4>(vec2(0.),vec2(0.),vec2(0.),vec2(0.));
        var is_quad = false;
        var shade_lo = 1.;
        var shade_hi = 1.;
        position = seat;
        normal = axis;

        if v < 216u {
            // ---- grass: a blade of two stacked quads that curve and taper.
            let j = v/12u;
            let i = v%12u;
            let blades = u32((0.6+0.4*roll(id,SALT_GRASS+1u))*params.clutter.z);
            if green && roll(id,SALT_GRASS) < params.clutter_chance.x && j < blades {
                let sj = SALT_GRASS+16u+j*8u;
                let k = u32(roll(id,sj)*f32(degree));
                let f0 = 0.15+0.60*roll(id,sj+1u);
                let seat_dir = axis*(1.-f0)+cell.corners[min(k,degree-1u)].xyz*f0;
                let bottom = seat_dir*radius;
                let ang = roll(id,sj+2u)*TAU;
                let side = tangent*cos(ang)+bitangent*sin(ang);
                let across = tangent*cos(ang+0.9)+bitangent*sin(ang+0.9);
                let h = params.clutter_size.x*(0.8+0.4*roll(id,sj+3u))*fade;
                let hw = params.clutter_size.y*(0.7+0.6*roll(id,sj+4u));
                // Each blade crops its own vertical strip of the ground tile,
                // so neighbouring blades pick up different pixels of art the
                // project already ships. That is the reference's
                // `tile_uv_slice` and it is why clutter needs no new texture.
                let strip = roll(id,sj+5u)*0.78;
                let seg = i/6u;
                let a = f32(seg)*0.5;
                let b = f32(seg+1u)*0.5;
                // One cross-section: the centre walks up and leans on the
                // SQUARE of the height fraction, so the blade curves rather
                // than shearing, and the half-width tapers toward the tip.
                let ca = bottom+axis*(h*a)+across*(h*0.15*a*a);
                let cb = bottom+axis*(h*b)+across*(h*0.15*b*b);
                let wa = hw*(1.-0.55*a);
                let wb = hw*(1.-0.55*b);
                quad = array<vec3<f32>,4>(ca-side*wa,ca+side*wa,cb+side*wb,cb-side*wb);
                quad_uv = array<vec2<f32>,4>(vec2(strip,1.-a),vec2(strip+0.18,1.-a),
                    vec2(strip+0.18,1.-b),vec2(strip,1.-b));
                shade_lo = mix(base_shade,1.,a);
                shade_hi = mix(base_shade,1.,b);
                is_quad = true;
                grown = h > 0.0001;
            }
        } else if v < 234u {
            // ---- flower: a slim stem and a bright four-triangle head.
            let i = v-216u;
            if green && roll(id,SALT_FLOWER) < params.clutter_chance.y {
                let k = u32(roll(id,SALT_FLOWER+1u)*f32(degree));
                let f0 = 0.15+0.60*roll(id,SALT_FLOWER+2u);
                let bottom = (axis*(1.-f0)+cell.corners[min(k,degree-1u)].xyz*f0)*radius;
                let ang = roll(id,SALT_FLOWER+3u)*TAU;
                let side = tangent*cos(ang)+bitangent*sin(ang);
                let h = params.clutter_more.x*fade;
                if i < 6u {
                    let ca = bottom;
                    let cb = bottom+axis*h;
                    quad = array<vec3<f32>,4>(ca-side*0.03,ca+side*0.03,cb+side*0.03,cb-side*0.03);
                    quad_uv = array<vec2<f32>,4>(vec2(0.1,1.),vec2(0.28,1.),vec2(0.28,0.),vec2(0.1,0.));
                    shade_lo = base_shade;
                    is_quad = true;
                } else {
                    // The head takes its own colour: a white bloom or a warm
                    // one, which is the reference's snow/crag pair.
                    material = select(4u,6u,roll(id,SALT_FLOWER+4u) < 0.5);
                    let t = (i-6u)/3u;
                    let c = (i-6u)%3u;
                    let tip = bottom+axis*h;
                    let a0 = ang+f32(t)*TAU*0.25;
                    let a1 = ang+f32(t+1u)*TAU*0.25;
                    let p0 = tip+(tangent*cos(a0)+bitangent*sin(a0))*0.07;
                    let p1 = tip+(tangent*cos(a1)+bitangent*sin(a1))*0.07;
                    let top = tip+axis*0.03;
                    position = select(select(top,p1,c==1u),p0,c==0u);
                    uv = select(select(vec2(0.5,0.5),vec2(0.9,0.9),c==1u),vec2(0.1,0.9),c==0u);
                }
                grown = h > 0.0001;
            }
        } else if v < 288u {
            // ---- pebble: one squat hexagonal prism on bare ground, and
            // sparsely on grass at the reference's own third.
            let rock_chance = select(params.clutter_chance.z*0.3,params.clutter_chance.z,dry);
            if (green||dry) && roll(id,SALT_ROCK) < rock_chance {
                let k = u32(roll(id,SALT_ROCK+1u)*f32(degree));
                let f0 = 0.15+0.60*roll(id,SALT_ROCK+2u);
                let bottom = (axis*(1.-f0)+cell.corners[min(k,degree-1u)].xyz*f0)*radius;
                let rad = params.clutter_size.z*(0.6+0.8*roll(id,SALT_ROCK+3u));
                let h = rad*(0.4+0.5*roll(id,SALT_ROCK+4u))*fade;
                let phase = roll(id,SALT_ROCK+5u)*TAU;
                material = 5u;
                let piece = prism_vertex(v-234u,bottom,axis,tangent,bitangent,rad,h,phase);
                position = piece.position;
                normal = piece.normal;
                uv = piece.uv;
                grown = h > 0.0001;
            }
        } else if v < 396u {
            // ---- bush: two leafy lumps, the reference's pair.
            if green && roll(id,SALT_BUSH) < params.clutter_chance.w {
                let k = u32(roll(id,SALT_BUSH+1u)*f32(degree));
                let f0 = 0.15+0.60*roll(id,SALT_BUSH+2u);
                let bottom = (axis*(1.-f0)+cell.corners[min(k,degree-1u)].xyz*f0)*radius;
                let size = params.clutter_size.w*(0.8+0.4*roll(id,SALT_BUSH+3u));
                let phase = roll(id,SALT_BUSH+4u)*TAU;
                material = 9u;
                let second = v >= 342u;
                let lump = select(bottom,
                    bottom+(tangent*cos(phase)+bitangent*sin(phase))*(size*1.1),second);
                let rad = select(size,size*0.6,second);
                let h = select(size*0.9,size*0.55,second)*fade;
                let turn = select(phase,phase+0.7,second);
                let part = select(v-288u,v-342u,second);
                let piece = prism_vertex(part,lump,axis,tangent,bitangent,rad,h,turn);
                position = piece.position;
                normal = piece.normal;
                uv = piece.uv;
                grown = h > 0.0001;
            }
        } else {
            // ---- dead shrub: a splay of dry twigs, which on desert sand and
            // tundra snow is the only ground cover there is.
            let i = v-396u;
            if (material==4u||material==6u) && roll(id,SALT_SHRUB) < params.clutter_more.y {
                let twig = i/6u;
                let k = u32(roll(id,SALT_SHRUB+1u)*f32(degree));
                let f0 = 0.15+0.60*roll(id,SALT_SHRUB+2u);
                let bottom = (axis*(1.-f0)+cell.corners[min(k,degree-1u)].xyz*f0)*radius;
                let phase = roll(id,SALT_SHRUB+3u)*TAU;
                let ang = phase+f32(twig)*TAU/6.+roll(id,SALT_SHRUB+8u+twig)*0.5;
                let h = params.clutter_more.z*(0.6+0.7*roll(id,SALT_SHRUB+16u+twig))*fade;
                let out_dir = tangent*cos(ang)+bitangent*sin(ang);
                let side = tangent*cos(ang+1.57)+bitangent*sin(ang+1.57);
                let top = bottom+axis*h+out_dir*(h*0.6);
                material = 8u;
                quad = array<vec3<f32>,4>(bottom-side*0.02,bottom+side*0.02,
                    top+side*0.02,top-side*0.02);
                quad_uv = array<vec2<f32>,4>(vec2(0.1,1.),vec2(0.3,1.),vec2(0.3,0.),vec2(0.1,0.));
                is_quad = true;
                grown = h > 0.0001;
            }
        }

        if is_quad {
            // A blade has to be visible from both sides, and the reference pays
            // for that by emitting both windings. The vertex shader knows where
            // the camera is, so instead the quad is WOUND toward it: one
            // pipeline, half the vertices, the same result.
            let i = v%6u;
            let facing = dot(cross(quad[1]-quad[0],quad[3]-quad[0]),params.camera.xyz-quad[0]);
            let forward = array<u32,6>(0u,1u,2u,0u,2u,3u);
            let reversed = array<u32,6>(0u,2u,1u,0u,3u,2u);
            var c = forward[i];
            if facing < 0. { c = reversed[i]; }
            position = quad[c];
            uv = quad_uv[c];
            // The normal is the surface UP, as the reference has it, so a piece
            // shades exactly like the cap it stands on instead of popping
            // against it.
            normal = axis;
            out_shade = select(shade_hi,shade_lo,c==0u||c==1u);
        }
        if !grown { position = axis*radius; }
    }
    var out: VertexOut;
    out.position = position;
    out.clip = params.clip_from_body*vec4(position,1.);
    out.normal = normal;
    out.uv = uv;
    // The material in the low byte and the cell's BIOME above it, exactly as
    // the record packs them: the fragment needs the biome to pick its sheet,
    // and carrying it in a word it already has costs no interpolator.
    out.material = (material & 0xffu) | (cell.metadata.y & 0xff00u);
    out.height = out_height;
    // The low half only: the high half carries the column tier slot.
    out.skylight = f32(cell.metadata.z & 0xffffu)/65535.;
    out.seed = cell.metadata.w;
    out.kind = kind;
    out.level = level;
    out.owner_a = cell.owner_a.xyz;
    out.owner_b = cell.owner_b.xyz;
    out.shade = out_shade;
    out.voxel = out_voxel;
    out.slot = lit_slot;
    return out;
}

// `atlas.png` is every biome's tileset baked into one texture: four sheets
// across and four down, each a 4x4 grid of 32-texel tiles. `slot` picks the
// sheet, `tile` the picture within it. Texture-load is intentionally nearest,
// and the bake took the shader's own sample points, so what a tile draws is
// what the single-sheet build drew, texel for texel.
fn pixel_tile(uv: vec2<f32>, tile: vec2<f32>, slot: u32) -> vec3<f32> {
    let pixel = (floor(fract(uv)*32.)+0.5)/32.;
    let sheet = vec2(f32(slot%4u),f32(slot/4u))*0.25;
    let sheet_uv = sheet+(tile+pixel)*(1./16.);
    return textureLoad(atlas,vec2<i32>(sheet_uv*vec2<f32>(textureDimensions(atlas))),0).rgb;
}

// Which sheet a biome draws from, as the app packed it.
fn tileset_slot(biome: u32) -> u32 {
    let row = params.tilesets[biome/4u];
    if biome%4u == 0u { return row.x; }
    if biome%4u == 1u { return row.y; }
    if biome%4u == 2u { return row.z; }
    return row.w;
}

// The material codes that are FACES rather than materials: the picture the
// ground shows on its side. Nothing in a column is ever made of these, so no
// cell carries them - they are derived here, from the cap and the depth.
const DIRT_CODE = 10u;
const GRASS_SIDE_CODE = 11u;
const SNOW_SIDE_CODE = 12u;
// What the side tile averages to. A face seen from far enough away fades to
// it, exactly as a cap fades to its flat base, because a point-sampled tile
// at range is shimmer rather than detail. The earth is two thirds of the tile
// so its own average is the honest one.
const GROUND_MEAN = vec3(0.466,0.342,0.255);

// What a wall shows `depth` metres under the ground: the sod's own SIDE for
// the first metre, the earth under it to the bottom of the soil, then stone.
// `pbd_core::column::material_at_depth` is the same three bands for the cells
// themselves, and `params.ground` carries its two depths, so the rule is
// written twice and its numbers have one source.
fn face_code(cap: u32, depth: f32) -> u32 {
    if depth <= params.ground.x {
        // Sward, jungle and marsh all fade into earth; snow fades into it too,
        // which is Tenebris's `dirt_snow`. Sand, stone and earth show their own
        // picture on every face, as the reference has them.
        if cap == 2u || cap == 3u || cap == 7u { return GRASS_SIDE_CODE; }
        if cap == 6u { return SNOW_SIDE_CODE; }
        return cap;
    }
    if depth <= params.ground.y {
        // Sand runs deep under a beach or a desert; rock and snow sit on rock
        // with no soil between; everything else has earth under it.
        if cap == 0u || cap == 1u || cap == 4u { return cap; }
        if cap == 5u || cap == 6u { return 5u; }
        return DIRT_CODE;
    }
    return 5u;
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
    // The heightfield's per-cell occlusion, times the voxel field's answer at
    // this vertex. Outside the column tier the second is one and this is what
    // it always was; inside it, it is what carries the cave and the crease.
    let skylight = input.skylight * input.voxel.x;
    // What a lamp put here. NOT multiplied by daylight - that is the whole of
    // why the field carries two channels, and a torch that went out at dusk
    // would be a torch nobody would place.
    let lamp = input.voxel.y;
    let cell_variation = 0.94+random(input.seed)*0.12;
    // A cap shows its own material. A WALL - a terrace step or the flank of a
    // cave run - shows what stands at ITS depth under the ground, which is the
    // sod's side, then earth, then stone. The wall used to blend earth into
    // stone over ABSOLUTE PLANET HEIGHT instead, so the same one-metre step
    // drew stone at two hundred metres and earth at forty, and no face on the
    // body ever showed the sod fading into the ground under it.
    let cap = input.material & 0xffu;
    // Which sheet this cell draws from. Every biome authors its own ground,
    // its own side, its own earth and its own stone at the same four tile
    // coordinates, so a biome is a slot and nothing else about the drawing
    // changes. Snow is the one material that is not a biome - it caps a field
    // above the snow line and a pole at any height - so it takes the tundra
    // sheet, whose side is snow over earth, or a snowy meadow would fade into
    // summer grass.
    var slot = tileset_slot((input.material >> 8u) & 0xffu);
    var code = cap;
    let altitude = length(input.position)-params.settings.x;
    if input.kind==1u {
        // The heightfield's own wall, which stands only where a cell has no
        // column: no layer to ask, so the depth rule, which is the
        // generator's own stack.
        code = face_code(cap,max(input.height-altitude,0.));
    }
    if input.kind==4u {
        // A column-pass face wears the LAYER it bounds: the block under an
        // upward face, over a downward one, and at the fragment's altitude on
        // a flank, clamped inside the run so its top edge never reads the air
        // over it. The side rule is Tenebris's `face_tile`: the sod's side is
        // the transition, snow's its own, a sod's underside earth, and
        // everything else its own tile.
        let radial = normalized(input.position);
        let facing = dot(input.normal, radial);
        var layer = light_layer(altitude);
        if facing > 0.5 { layer = light_layer(altitude-0.5); }
        else if facing < -0.5 { layer = light_layer(altitude+0.5); }
        else { layer = min(layer, max(light_layer(input.height-0.5), 0u)); }
        let own = material_at(input.slot, layer);
        code = own;
        if abs(facing) < 0.5 {
            if grassy(own) { code = GRASS_SIDE_CODE; }
            if own == 6u { code = SNOW_SIDE_CODE; }
        } else if facing < -0.5 && grassy(own) {
            code = DIRT_CODE;
        }
    }
    // Which tile each material draws, per sheet. Every sheet keeps the same
    // layout - #0 (0,0) its own ground, #1 that ground over the earth, #2 the
    // earth, #3 the stone - so sand is each sheet's own ground and snow is the
    // tundra sheet's wind packed snow at #10 (2,2). `tools/block_audit.py`
    // reads this table off the shipped file and lays every pick beside the
    // sheet's name for it, and a test holds the names; the snow cap drew
    // "cold granite" and the sand caps "packed path" until it did.
    var base = vec3(0.12,0.32,0.075);
    var tile = vec2(0.,0.);
    // Material 0 is the seabed: sand, darkened by the water column below.
    if code==0u { base=vec3(0.52,0.45,0.30); tile=vec2(0.,0.); }
    if code==1u { base=vec3(0.61,0.48,0.25); tile=vec2(0.,0.); }
    if code==3u { base=vec3(0.07,0.25,0.105); }
    if code==4u { base=vec3(0.64,0.36,0.13); tile=vec2(0.,0.); }
    if code==5u { base=vec3(0.31,0.34,0.33); tile=vec2(3.,0.); }
    if code==6u { base=vec3(0.80,0.90,0.91); tile=vec2(2.,2.); }
    if code==7u { base=vec3(0.32,0.36,0.22); }
    if code==8u { base=vec3(0.16,0.105,0.055); tile=vec2(2.,1.); }
    if code==9u { base=vec3(0.085,0.24,0.060); tile=vec2(0.,2.); }
    if code==6u || code==SNOW_SIDE_CODE { slot = u32(params.ground.z); }
    // A face is drawn in its tile's COLOURS - the grass top is the grass art,
    // a wall's first metre is the sod-into-earth transition - and fades at
    // range to a flat mean, because a point-sampled tile at range is shimmer
    // rather than detail. A cap used to take only its tile's BRIGHTNESS over a
    // flat base, so a meadow was green paper with the sward stamped on it and
    // the grass art showed nowhere but the top metre of a wall. Tenebris draws
    // `grass.png` on the top, `dirt_grass.png` on the side and `dirt.png`
    // underneath: three pictures, none of them a tint.
    var far = base;
    if code>=DIRT_CODE {
        tile = vec2(2.,0.);
        if code>=GRASS_SIDE_CODE { tile = vec2(1.,0.); }
        far = GROUND_MEAN;
    }
    let art = pixel_tile(input.uv,tile,slot);
    // `shade` is one everywhere but down a grass blade, where the root sits at
    // the configured fraction of full light and eases to the tip. That gradient
    // is what makes a sward read as lush rather than as flat green paper.
    var albedo = mix(art,far,smoothstep(180.,1400.,distance_to_camera))
        *cell_variation*input.shade;
    // A tiny cap-edge darkening makes the actual hex-column silhouette legible
    // while the atlas supplies the committed source pixel art at close range.
    // What sky a face takes. A cap takes the cool overhead tone; a WALL - a
    // terrace step or a cave flank - takes a broader, paler one, which is what
    // keeps pixel-art dirt and stone legible on a shaded side instead of
    // turning every step into black.
    //
    // ONE expression with the fill chosen, rather than three that each rebuild
    // it: the floor under the ambient has to apply to every face, and while
    // these were three assignments the later two silently dropped it. A cave
    // is made almost entirely of the third kind, so the term that stops a cave
    // being a black screen was missing from exactly the faces that needed it.
    var fill = vec3(0.16,0.21,0.27);
    var night = 0.12;
    var gain = 1.;
    if input.kind==1u || input.kind==4u {
        fill = vec3(0.30,0.32,0.34);
        night = 0.20;
        gain = 0.95;
    }
    // The BURIAL stand-in is gone with this. It faded a cave face toward
    // `cave_dark` over `cave_dark_depth_m` of depth, and its own comment named
    // it as a placeholder for the baked voxel light that is now in `skylight`:
    // "The real answer is the baked voxel light this change defers." Depth is
    // not darkness - a cave mouth is deep and bright - and keeping both would
    // be two rules for one fact, with the wrong one winning at every mouth.
    var color = albedo*(fill*max(AMBIENT_FLOOR,mix(night,1.,daylight)*skylight)
        + vec3(1.12,1.03,0.87)*direct*skylight)*gain;
    // A lamp, ADDED. Warm, because everything that burns is, and over the
    // albedo so a torch lights the ground it stands on rather than painting a
    // flat orange patch over it. This is the term that makes a night worth
    // carrying a light through.
    color += albedo*TORCH_TINT*(lamp*TORCH_GAIN);
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
    // Both take the FIELD: a face the sky does not reach has no atmosphere
    // between it and the eye, and without this a cave wall forty metres
    // underground fogged toward the sky colour exactly as a ridge at the same
    // distance did. Tenebris gates its rim by `v_sky_light` for the same
    // reason; the haze here is the same term one step earlier.
    let fog = (1.-exp(-distance_to_camera*0.00036))*air*daylight*skylight;
    let sky = mix(vec3(0.10,0.20,0.29),vec3(0.32,0.49,0.57),max(sun_elevation,0.));
    color=mix(color,sky,fog*0.55);
    let rim = pow(1.-clamp(dot(radial,toward_camera),0.,1.),4.);
    color+=vec3(0.07,0.16,0.25)*rim*(0.25+0.75*daylight)*(1.-air)*0.55*skylight;
    return vec4(color,1.);
}
