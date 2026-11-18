// Persistent topology vertex pulling, pixel atlas materials, exposed column
// skirts and cosmetic trees. Terminator/skylight/rim treatment follows the
// previously documented Tenebris port in hex_terrain.wgsl. The sea is not
// drawn here: water cells draw their seabed and water.wgsl draws the sheet.
#import pbd::clouds::cloud_map_smooth
#import pbd::sun_shadow::{SunCascades, sun_shadow_soft}

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
    // `water_absorption` rgb is the DOWNWELLING per metre (`ocean-environments`
    // B2), the daylight arriving at a depth; w the sheet's radius.
    water_absorption: vec4<f32>, water_deep: vec4<f32>, weather: vec4<f32>,
    rain: array<vec4<f32>,6>, // wet knobs; overcast sun, fill, fog per cover, fog in rain; sky blue cut, sky dim
    lod_offsets: vec4<u32>, // x base count, y fine-region capacity, z the current set's half (`two-set-dissolve`)
    lod_counts: vec4<u32>,  // live records per fine level of the current set, coarsest first
    lod: vec4<f32>,         // xyz player direction, w base level
    bands: vec4<f32>,       // cos(band radius / R) per fine level, coarsest first
    clutter: vec4<f32>,        // reach m, fade m, blades, base shade
    clutter_chance: vec4<f32>, // grass, flower, rock, bush
    clutter_size: vec4<f32>,   // blade height, blade half-width, rock, bush
    clutter_more: vec4<f32>,   // flower height, shrub chance, shrub size, spare
    column: vec4<f32>,         // tier reach m, cave dark floor, cave dark depth m, cos(2 x reach / R)
    ground: vec4<f32>,         // sod depth m, soil depth m, snow tileset slot, spare
    tilesets: array<vec4<u32>,2>, // atlas slot per biome, in Biome order
    fade: vec4<f32>,           // `detail-fade`: tree fade m, cross-fade progress 0..1, one while it runs; w one for a sun cascade (`sun-shadows`)
    lod_prev: vec4<f32>,       // the partition the cross-fade leaves: xyz its anchor
    bands_prev: vec4<f32>,     // and its band cosines
    lod_counts_prev: vec4<u32>, // the dissolve's old set's counts (`two-set-dissolve`);
    lod_spare: vec4<f32>,       // caustics (`ocean-environments` B5): strength, the web's cell m, gone by m; the visibility pass reads none of it
    anchor: vec4<f32>,         // `distance-lod-fade`: the fine set's anchor (`lod` is the fade centre),
    bands_in: vec4<f32>,       // the bands' cross-fade rings' inner edges, cosines,
    bands_prev_in: vec4<f32>,  // and those of the partition a landing dissolves from
    light: vec4<f32>,          // `lighting-model`: x one for the new model, y a lamp's glow (HDR), z the sun's radius as a tangent (L3), w its widening under a full deck
    sky_sh: array<vec4<f32>,18>, // L2: the sky's SH, clear then a full deck, local frame
    wet: vec4<f32>,            // the new wet film: roughness wet and in a puddle, lamp highlights, one when the rings follow the rain
    lamps: array<vec4<f32>,8>, // the lights nearest the eye, planet frame; w the level 0..1 (zero: none)
    reflect: vec4<f32>,        // L8: x one while the reflected ray walks the columns, y its reach m; L4: z the bounce's strength now; w spare
    lamp_bake: vec4<f32>,      // L7: x the newest bake's fade-in 0..1, y the older half holds this set, z the newest does, w the newest half
    ssr_clip: mat4x4<f32>,     // L8 (10.2): the screen trace history's clip_from_body
    ssr_eye: vec4<f32>,        // and its eye; w one once there is a history
    rain_near: vec4<f32>,      // `wet-near-detail`: x the reach of rain's fine detail, m; y the fade band before it, m; z a plant's darkening at full wetness (`flora-v2` 13.2)
    light_ease: vec4<f32>,     // `baked-light-ease` 7.3: x the light field's newest half blended in 0..1, y that half, z one while the other is blended from; w the far trees' baked relief 0..1 (`tree-impostor-depth`)
    flora: vec4<f32>,          // `flora-v2`: x tufts per cell (zero: blades), y tuft width m, z tuft height m, w the year 0..1
    flora_more: vec4<f32>,     // x mushroom chance, y fern chance, zw spare
    flora_biome: array<vec4<f32>,8>, // per biome: fern weight, mushroom weight, flower bits, deciduous
    tuft_ramps: array<vec4<u32>,8>,  // per biome: the tuft's three tones 0xRRGGBB, w red caps
    plant_means: array<vec4<u32>,8>, // `flora-v2` 9.2, 9.6: per biome the leaves, bark, stone tiles' means 0xRRGGBB, w fruit bits (low 16), tree table (high 16)
    flora_fall: vec4<f32>,     // 9.4, 9.6: x leaf fall, y gathered under trees, z leaf size, w fruit chance
    tree_fade: array<vec4<f32>,2>, // `flora-v2` 13.3: the point the trees' cross-fade runs from (w its progress 0..1), the point it runs to
    night: vec4<f32>,          // `night-lights-from-space`: x how many far lights `night_lights` holds
    night_lights: array<vec4<f32>,128>, // two a light: direction, cos(reach); radius m, strength now, seed, points' spacing m
    felled: array<vec4<u32>,16>,  // `flora-v2` 12.5: felled tree ids near the camera, four a lane, zero empty
    picked: array<vec4<u32>,8>,   // and the trees picked bare
    clutter_taken: array<vec4<u32>,16>, // `gather-by-hand`: cells with a piece taken, two a lane (id, a sink byte a piece), zero empty
}
// The weather maps (`planet_weather.rs`): cover, cloud top, rain and optical
// depth per place, the wind aloft, the overlay; one sampler.
@group(1) @binding(0) var weather_cloud: texture_cube<f32>;
@group(1) @binding(1) var weather_wind: texture_cube<f32>;
@group(1) @binding(2) var weather_overlay: texture_cube<f32>;
@group(1) @binding(3) var weather_sampler: sampler;
// The sun's cascades (`sun-shadows`): their depth maps, the comparison sampler
// that reads them, and where each stands. `planet_shadow.rs` draws and binds
// them; `sun_shadow.wgsl` reads them, for this pass and `field_lit.wgsl` alike.
@group(2) @binding(0) var sun_map: texture_depth_2d_array;
@group(2) @binding(1) var sun_cmp: sampler_comparison;
@group(2) @binding(2) var<storage, read> sun_cascades: SunCascades;

// How much of the sun reaches a point past the clouds: the cloud where the
// ray toward the sun crosses the cloud layer (`clutter_more.w`, a radius),
// thinned by that cloud's own optical depth. A cloud's shadow lies where the
// sun puts it and moves with it; `ground.w` is how dark a full one is.
fn cloud_sun(p: vec3<f32>, sun: vec3<f32>) -> f32 {
    let strength = params.ground.w;
    let layer = params.clutter_more.w;
    if strength <= 0.0 || layer <= 0.0 { return 1.0; }
    let up = normalized(p);
    let rise = max(dot(up, sun), 0.08);
    let t = max(layer - length(p), 0.0) / rise;
    let at = normalized(p + sun * t);
    let w = cloud_map_smooth(weather_cloud, weather_sampler, at);
    let blocked = clamp(w.x, 0.0, 1.0) * (1.0 - exp(-max(w.w, 0.0) * 0.25));
    return 1.0 - strength * blocked;
}

// Over a cell's half-width: a tree has faded out before its cell's centre
// reaches the foliage range (`detail-fade`).
const TREE_FADE_MARGIN_M: f32 = 4.0;

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
// How far a direction is into fine level `level`'s band, 0..1, across the
// band's cross-fade ring (`distance-lod-fade`): 0 outside its edge, 1 inside
// its ring. A pixel whose dither value is `m` draws the level where this
// exceeds `m`, so across the ring the two levels share the pixels between
// them in proportion, as Unity's LOD cross-fade does. A band not laid is 0;
// a ring of no width is the band's hard edge.
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
// Every column tier slot's unit direction (`ColumnTier::gpu_centres`): the
// border between two columns is the plane midway between their directions,
// square to the step from one to the other, which is what the reflected
// ray's walk crosses (`lighting-model` L8).
@group(0) @binding(7) var<storage,read> centres: array<vec4<f32>>;
// `lighting-model` L4: the sun's bounce over the tier (`pbd_core::bounce`):
// `BOUNCE_BASES` words of each slot's window base layer, then per slot
// `BOUNCE_WORDS` words of sixteen-bit 5:6:5 cells, two to a word.
@group(0) @binding(8) var<storage,read> bounce: array<u32>;
const BOUNCE_BASES: u32 = 16384u;
const BOUNCE_WORDS: u32 = 16u;
const BOUNCE_SPAN: u32 = 32u;
const BOUNCE_MAX: f32 = 0.5;
// `lighting-model` L7: the lamp bake (`pbd_core::lamp_bake`), two halves of
// `LAMP_HALF_WORDS`: the lamps (four words each: where, as f32 bits, and the
// level), each slot's window base, then per cell the lamps it sees (four
// indices, 255 none) and its bounce (5:6:5, the in-reach flag over it).
@group(0) @binding(9) var<storage,read> lamp_bake: array<u32>;
const LAMP_LIST_WORDS: u32 = 1024u;
const LAMP_HALF_WORDS: u32 = 1065984u;
const LAMP_CELL_WORDS: u32 = 2u;
const LAMP_BOUNCE_MAX: f32 = 1.0;
const LAMP_REACH_M: f32 = 16.0;
const LAMP_BAKE_GAIN: f32 = 2.4;
// L8 (10.2): the screen trace's history (`planet_ssr.rs`): last frame's
// colour, and in w each pixel's distance from that frame's eye.
@group(0) @binding(10) var ssr_history: texture_2d<f32>;
// `flora-v2` design 10.2: the detailed trees' kit (`pbd_core::tree_kit`):
// per species a header, per variant a header, the kit's own tiles, the
// vertices, the anchors and the baked impostors.
@group(0) @binding(11) var<storage,read> kit: array<u32>;
const MATERIAL_PER_WORD: u32 = 8u;
const MATERIAL_WORDS: u32 = 63u;
// The render code of the layer `layer` of column slot `slot` (plus one, as
// the record carries it). Off the tier, or past the span: air.
fn material_at(slot: u32, layer: u32) -> u32 {
    if slot == 0u || layer >= LIGHT_LAYERS { return 0u; }
    let word = (slot - 1u) * MATERIAL_WORDS + layer / MATERIAL_PER_WORD;
    if word >= arrayLength(&materials) { return 0u; }
    return (materials[word] >> ((layer % MATERIAL_PER_WORD) * 4u)) & 0xfu;
}
// `placed-blocks-keep-material`: after every slot's render codes, every
// slot's PLACED sheets, laid out as the codes are: nought for generated
// ground, which takes its biome's sheet, and for a block somebody placed its
// own sheet plus one, so it is drawn as the block it is whatever biome it
// stands in. `ColumnTier::gpu_placed_sheets` packs it.
const PLACED_SHEETS_BASE: u32 = 1032192u;
fn placed_sheet_at(slot: u32, layer: u32) -> u32 {
    if slot == 0u || layer >= LIGHT_LAYERS { return 0u; }
    let word = PLACED_SHEETS_BASE + (slot - 1u) * MATERIAL_WORDS + layer / MATERIAL_PER_WORD;
    if word >= arrayLength(&materials) { return 0u; }
    return (materials[word] >> ((layer % MATERIAL_PER_WORD) * 4u)) & 0xfu;
}
// And the record's own: the sheet its PLACED top is drawn from, plus one,
// in four bits of its material word (`planet_terrain::PLACED_SHEET_SHIFT`),
// which the cap and its heightfield wall carry to the fragment.
const PLACED_SHEET_SHIFT: u32 = 26u;
const PLACED_SHEET_MASK: u32 = 0x3c000000u;

// The bottom and top of a column's span, metres against sea level.
// `column::BASE_M` and `LAYERS` in pbd-core are the one source; the visibility
// shader carries BASE_M too and a test holds all three together.
const COLUMN_BASE_M: f32 = -145.0;
const COLUMN_TOP_M: f32 = 359.0;
const COLUMN_RUNS: u32 = 4u;
// A column of `COLUMN_RUNS` runs has one more stretch of air than it has runs:
// under the lowest, between each pair, and over the highest.
const COLUMN_GAPS: u32 = COLUMN_RUNS + 1u;
// Where the column branch starts in the shared vertex shader, after the
// terrain's 60, the trees' 30 (four crossed cards and the crown cap,
// `crossed-tree-cards`) and the clutter's 564 (432, then `flora-v2`'s mushroom and
// fern).
const COLUMN_FIRST_VERTEX: u32 = 654u;
// Where the clutter branch starts: after the terrain and the trees.
const CLUTTER_FIRST_VERTEX: u32 = 90u;
// Per run, a cave ceiling fan and a cave floor fan.
const COLUMN_CAP_VERTICES: u32 = COLUMN_RUNS * 36u;
// Per side, per run, per neighbouring air gap, one quad.
const COLUMN_SIDE_VERTICES: u32 = COLUMN_RUNS * COLUMN_GAPS * 6u;
// One lamp: a torch's slim post and bright cap, or two boxes for every other
// light (`lamps-and-lanterns` task 5.2). A column holds at most one, which
// placement enforces, so the lamp drawn and the lamp lighting are the same one.
const COLUMN_LAMP_VERTICES: u32 = 72u;
// Where a column record carries its lamp's layer, plus one, and which lamp it
// is, as its index in `Material::LAMPS`. `planet_column.rs` packs both;
// `the_shader_carries_the_reference_light_constants` pins the shifts.
const TORCH_SHIFT: u32 = 16u;
const LAMP_LAYER_MASK: u32 = 0x1ffu;
const LAMP_KIND_SHIFT: u32 = 25u;
fn lamp_layer(rec: ColumnRec) -> u32 { return (rec.more[3] >> TORCH_SHIFT) & LAMP_LAYER_MASK; }
fn lamp_kind(rec: ColumnRec) -> u32 { return (rec.more[3] >> LAMP_KIND_SHIFT) & 7u; }
// Whether the lamp burns now: a street lantern's glass is dark by day.
const LAMP_LIT_BIT: u32 = 0x10000000u;
fn lamp_lit(rec: ColumnRec) -> bool { return (rec.more[3] & LAMP_LIT_BIT) != 0u; }
// `Material::LAMPS` in order.
const LAMP_TORCH: u32 = 0u;
const LAMP_POST: u32 = 1u;
const LAMP_WALL: u32 = 2u;
const LAMP_HANGING: u32 = 3u;
const LAMP_BRAZIER: u32 = 4u;
const LAMP_CANDLE: u32 = 5u;
const LAMP_CAMPFIRE: u32 = 6u;
// The column pass's lamps are their own draw kind, so the fragment colours
// them from the part (iron, wax, glass, flame) rather than from the layer's
// material, which for a lamp's cell is nothing that looks like a lamp.
const KIND_LAMP: u32 = 5u;
// A lamp part's code, in the material byte, above every terrain code.
const LAMP_IRON_CODE: u32 = 240u;
const LAMP_WAX_CODE: u32 = 241u;
const LAMP_WOOD_CODE: u32 = 243u;
// A campfire's ring of stones (`cooking` design S2).
const LAMP_STONE_CODE: u32 = 245u;
// A lantern's glass while it is out: dark panes in the iron frame.
const LAMP_GLASS_CODE: u32 = 242u;
// What a lit part carries in the block channel instead of a level, so the
// fragment draws it as the light it is: a lantern's lit panes, or a bare
// flame.
const LAMP_PANES: f32 = 2.0;
const LAMP_FLAME: f32 = 3.0;
// A glowing flower's head while it burns: drawn as its own cool light
// (`lamps-and-lanterns` decision 8).
const LAMP_BLOOM: f32 = 4.0;
// The record bit that says a cell grows a glowing flower (`spare.z`), and the
// column bit that says it burns (`more[3]`). `planet_column.rs` sets both;
// `the_shader_carries_the_reference_light_constants` pins them.
const GLOW_FLOWER_BIT: u32 = 1u;
const GLOW_LIT_BIT: u32 = 0x20000000u;
// A glowing flower's head by day: pale, and cooler than the white and warm
// blooms, so the species is known before dark.
const BLOOM_CODE: u32 = 244u;
// How bright a glowing flower's head burns: a quarter of what it was, the
// owner's "1/4 brightness on the flower lights" (`night-lights-from-space`).
const GLOW_HEAD_GAIN: f32 = 0.25;
// A flame also carries how far up it is, 0 at its root and 1 at its tip, as
// `LAMP_FLAME + FLAME_RISE * rise`, so it shades from root to tip without a
// varying of its own.
const FLAME_RISE: f32 = 0.5;

// Whether a lantern's pixel is its iron frame: the rim of each side, and all
// of its top and bottom.
fn lantern_frame(uv: vec2<f32>, up: f32) -> bool {
    let edge = min(min(uv.x, 1.0 - uv.x), min(uv.y, 1.0 - uv.y));
    return edge < 0.16 || abs(up) > 0.5;
}

// How much of a lamp's colour a level `f` (0..1 of the brightest) adds: the
// towns mockup's `(1 - (d/R)^2)^2`, written in the level, which falls about
// a metre a step (`lamps-and-lanterns` decision 7). Brighter than linear
// near the light, and down smoothly to nothing at the edge of its reach.
fn lamp_strength(level: f32) -> f32 {
    let f = clamp(level, 0.0, 1.0);
    let g = f * (2.0 - f);
    return g * g;
}

// How much of a lamp's light shows where the sun reaches (`lamps-and-
// lanterns` decision 15, survey L4): none in the open at noon, all of it in
// shade, indoors and at night. `light::lamp_share` in the core, word for word.
fn lamp_share(sky: f32, daylight: f32) -> f32 {
    return 1.0 - clamp(daylight, 0.0, 1.0) * clamp(sky, 0.0, 1.0);
}

// One vertex of a lamp.
struct LampVertex {
    position: vec3<f32>,
    normal: vec3<f32>,
    uv: vec2<f32>,
    // How far up the box, 0 at its foot and 1 at its top: what a flame
    // shades by, from root to tip, on its sides and its top alike.
    rise: f32,
}

// Vertex `i` (0..36) of a box: centre `c`, a right-handed frame `x`, `y`, `z`,
// half extents `e` along them. Faces wind counter-clockwise from outside,
// which is what back-face culling keeps. On each of the four sides `uv.y`
// runs down `z`, from 0 at the top to 1 at the foot, so a flame shades from
// root to tip and a lantern's frame is the same on every side.
fn lamp_box(i: u32, c: vec3<f32>, x: vec3<f32>, y: vec3<f32>, z: vec3<f32>, e: vec3<f32>) -> LampVertex {
    let face = i / 6u;
    var n = x; var u = y; var w = z; var hn = e.x; var hu = e.y; var hw = e.z;
    if face == 2u || face == 3u { n = y; u = -x; w = z; hn = e.y; hu = e.x; hw = e.z; }
    if face == 4u || face == 5u { n = z; u = x; w = y; hn = e.z; hu = e.x; hw = e.y; }
    let s = select(-1.0, 1.0, face % 2u == 0u);
    let corners = array<vec2<f32>,4>(vec2(-1.,-1.),vec2(1.,-1.),vec2(1.,1.),vec2(-1.,1.));
    let order = array<u32,6>(0u,1u,2u,0u,2u,3u);
    let q = corners[order[i % 6u]];
    var out: LampVertex;
    out.position = c + n*(hn*s) + u*(q.x*hu*s) + w*(q.y*hw);
    out.normal = n*s;
    out.uv = vec2(q.x*0.5+0.5, 0.5-q.y*0.5);
    out.rise = q.y*0.5+0.5;
    if face == 4u { out.rise = 1.0; }
    if face == 5u { out.rise = 0.0; }
    return out;
}
// Where the record carries the top of this column's water, as a layer plus
// one; `planet_column.rs` packs it and a test holds the two shifts together.
const COLUMN_WATER_SHIFT: u32 = 4u;
const COLUMN_WATER_MASK: u32 = 0x1ffu;
// How far the top water layer's surface stands under its top, in eighths of
// a metre: nought for a source, more for a flowing cell's lower level.
const COLUMN_WATER_SHORT_SHIFT: u32 = 13u;
const COLUMN_WATER_SHORT_MASK: u32 = 0x7u;
// The altitude of this column's water SURFACE, or a depth no world reaches
// when it holds none. Below the world rather than a flag, so the reader is a
// `min` against the sheet and never a branch: a column with no water has its
// surface infinitely far down and nothing in it is ever wet.
fn column_water_top_m(rec: ColumnRec) -> f32 {
    let packed = (rec.more[3] >> COLUMN_WATER_SHIFT) & COLUMN_WATER_MASK;
    if packed == 0u { return -1.0e9; }
    let short = (rec.more[3] >> COLUMN_WATER_SHORT_SHIFT) & COLUMN_WATER_SHORT_MASK;
    return COLUMN_BASE_M + f32(packed) - f32(short)*0.125;
}
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
// `more[2]` is the cell's degree in its low byte and, in the records the
// GPU is given, the column's reach top above it (`lighting-perf` 2.1,
// `ColumnTier::gpu_records_with_reach`): the highest solid layer, plus one,
// of every column a reflected ray in this one could still enter.
const DEGREE_MASK: u32 = 255u;
const REACH_TOP_SHIFT: u32 = 8u;
fn column_degree(rec: ColumnRec) -> u32 { return min(rec.more[2] & DEGREE_MASK, 6u); }
fn reach_top(rec: ColumnRec) -> u32 { return (rec.more[2] >> REACH_TOP_SHIFT) & 0x1ffu; }
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
const LIGHT_WORDS: u32 = 126u;
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
const LIGHT_LAYERS: u32 = 504u;
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
//
// The buffer holds two fields of `LIGHT_HALF_WORDS` (`baked-light-ease` 7.3):
// the one drawn, `params.light_ease.y`, and across a dusk or dawn relight the
// one before it, blended from by `light_ease.x` while `light_ease.z` says it
// is there, so the street lamps come on over a second, not in a frame.
const LIGHT_HALF_WORDS: u32 = 2064384u;
fn light_cell(half: u32, slot: u32, layer: u32) -> vec2<f32> {
    let word = half * LIGHT_HALF_WORDS + (slot - 1u) * LIGHT_WORDS + layer / LIGHT_PER_WORD;
    if word >= arrayLength(&light) { return vec2(1.0, 0.0); }
    let byte = (light[word] >> ((layer % LIGHT_PER_WORD) * 8u)) & 0xffu;
    return vec2(f32(byte >> 4u), f32(byte & 0xfu)) / LIGHT_MAX;
}
fn light_at(slot: u32, layer: u32) -> vec2<f32> {
#ifdef SHADOW_CASTER
    // `gpu-cuts` 2: a depth map takes no light; the cascades skip its reads.
    return vec2(1.0, 0.0);
#endif
    if slot == 0u { return vec2(1.0, 0.0); }
    if layer >= LIGHT_LAYERS { return vec2(1.0, 0.0); }
    let newest = min(u32(params.light_ease.y), 1u);
    let now = light_cell(newest, slot, layer);
    if params.light_ease.z > 0.5 && params.light_ease.x < 1.0 {
        return mix(light_cell(1u - newest, slot, layer), now, params.light_ease.x);
    }
    return now;
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
#ifdef SHADOW_CASTER
    return vec2(1.0, 0.0);
#endif
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
#ifdef SHADOW_CASTER
    return vec2(1.0, 0.0);
#endif
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
    @location(9) @interpolate(flat) split: vec3<f32>,
    @location(10) @interpolate(flat) fade_t: vec3<f32>,
    // How brightly this vertex takes its own albedo. One everywhere except
    // down a grass blade, where the root is darker than the tip.
    @location(11) shade: f32,
    // The voxel field at this vertex, INTERPOLATED - which is the point of it.
    @location(12) voxel: vec2<f32>,
    // The column tier slot plus one, or zero off the tier: what a column-pass
    // face asks for the material of the layer it stands on.
    @location(13) @interpolate(flat) slot: u32,
    // Whether rain reaches this face: one where the air in front of it has no
    // solid layer of its column above it, zero under rock. Per face, from the
    // column's own runs; `pbd_core::column::Column::open_to_sky` is the rule.
    @location(14) @interpolate(flat) rain_open: f32,
    // Which partition this instance is drawn for (`detail-fade`): PART_BOTH,
    // PART_NEW or PART_OLD.
    @location(15) @interpolate(flat) part_mark: u32,
}
// `flora-v2` design 12.5: whether a tree a player felled, or picked bare, is
// in the uniform's short lists (`trees_taken::TreesTaken`, nearest first).
fn tree_felled(id: u32) -> bool {
    if id == 0u { return false; }
    for (var i = 0u; i < 16u; i++) {
        if any(params.felled[i] == vec4<u32>(id)) { return true; }
    }
    return false;
}
fn tree_picked(id: u32) -> bool {
    if id == 0u { return false; }
    for (var i = 0u; i < 8u; i++) {
        if any(params.picked[i] == vec4<u32>(id)) { return true; }
    }
    return false;
}

// clutter:shared begin
// What the ground clutter is rolled from, shared with `pbd_core::scatter`,
// which a GPU test (`clutter_gpu_tests.rs`) holds to it: the hash, the rolls
// and their salts, the grounds, and where and whether a piece a hand can
// take stands. Standalone: it reads no binding.
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
const SALT_MUSH: u32 = 0x56u;
const SALT_FERN: u32 = 0x57u;
const SALT_FRUIT: u32 = 0x58u;
fn grassy(material: u32) -> bool { return material==2u || material==3u || material==7u; }
fn bare(material: u32) -> bool { return material==1u || material==4u || material==5u || material==6u; }
// Where a piece salted `salt` stands on a cell: the reference's lerp from the
// cap's centre toward a hashed corner, at the cap's radius.
fn clutter_seat(id: u32, salt: u32, degree: u32, axis: vec3<f32>,
        corners: array<vec4<f32>,6>, radius: f32) -> vec3<f32> {
    let k = u32(roll(id,salt+1u)*f32(degree));
    let f0 = 0.15+0.60*roll(id,salt+2u);
    return (axis*(1.-f0)+corners[min(k,degree-1u)].xyz*f0)*radius;
}
// Whether a piece a hand can take grows on a cell: 0 a pebble, 1 a bush,
// 2 a dead shrub, 3 a fern, at `chance` (a fern's with its biome's weight).
fn clutter_grows(piece: u32, id: u32, material: u32, chance: f32) -> bool {
    let green = grassy(material);
    let dry = bare(material);
    if piece == 0u {
        return (green||dry) && roll(id,SALT_ROCK) < select(chance*0.3,chance,dry);
    }
    if piece == 1u { return green && roll(id,SALT_BUSH) < chance; }
    if piece == 2u { return (material==4u||material==6u) && roll(id,SALT_SHRUB) < chance; }
    return green && roll(id,SALT_FERN) < chance;
}
// clutter:shared end
// How far a piece a hand took is sunk into the ground, 0 whole to 1 gone
// (`gather-by-hand`): its byte in the taken cells' list, nought off it.
fn clutter_sunk(id: u32, piece: u32) -> f32 {
    for (var i = 0u; i < 16u; i++) {
        let lane = params.clutter_taken[i];
        if lane.x == 0u && lane.z == 0u { break; }
        if lane.x == id { return f32((lane.y >> (8u*piece)) & 0xffu)/255.; }
        if lane.z == id { return f32((lane.w >> (8u*piece)) & 0xffu)/255.; }
    }
    return 0.;
}
const TAU: f32 = 6.28318530718;

// `flora-v2`: the plants' own material codes, past the ground's (0-15) and
// before the lamps' (240-): faces nothing in a column is ever made of.
const TUFT_CODE: u32 = 16u;
const FERN_CODE: u32 = 18u;
const MUSH_STEM_CODE: u32 = 19u;
const CAP_RED_CODE: u32 = 20u;
const CAP_BROWN_CODE: u32 = 21u;
const FLOWER_CODE: u32 = 22u;
// `flora-v2` design 9.6: a fruit or a pine cone, its kind in the uv's
// integer part.
const FRUIT_CODE: u32 = 23u;
// Design 9.4: set over the material word of a cap whose cell stands a tree,
// above the biome byte, so the fallen leaves gather under it.
const UNDER_TREE_BIT: u32 = 0x10000u;

// Whether this cell stands a tree: the visibility pass's rule
// (`has_nearby_foliage`), the same lines, for the cap under it (design 9.4).
// `pbd_core::flora::grows_tree` is the CPU's copy and a test holds all three.
fn stands_tree(cell: Cell) -> bool {
    // Nor a cell water stands on at any tide (`flora-v2` 12.11): bit 25,
    // `pbd_core::flora::FLOODED_BIT`.
    if ((cell.metadata.y >> 24u) & 3u) != 0u { return false; }
    let roll = hash(cell.spare.x) & 0xffu;
    let material = cell.metadata.y & 0xffu;
    let biome = (cell.metadata.y >> 8u) & 0xffu;
    // `flora-v2` design 12.1: the world's tree table, from the uniform: the
    // high half of `plant_means[biome].w` (`pbd_core::flora::TreeTable::packed`),
    // so an old world keeps its trees and version 10 grows M13's forests.
    let entry = params.plant_means[min(biome, 7u)].w;
    let density = (entry >> 16u) & 0xffu;
    let grass = material==2u || material==3u || material==7u;
    let sand = ((entry >> 24u) & 1u) == 1u && material==1u;
    let pine = ((entry >> 25u) & 1u) == 1u && material==6u;
    let tree = (grass||sand||pine) && roll < density;
    // A tree felled for its wood is gone (design 12.5).
    let standing = tree && !tree_felled(cell.spare.x);
    return standing;
}

// A tree's crown, metres over its cell and widths as a share of the cell:
// the trunk's top, then the two leaf layers' bottom, top, and bottom and top
// widths. The tree draw and its fruit read the one shape.
struct Crown {
    trunk_top: f32,
    lo0: f32, hi0: f32, w0a: f32, w0b: f32,
    lo1: f32, hi1: f32, w1a: f32, w1b: f32,
}
fn crown_of(id: u32, biome: u32) -> Crown {
    let roll = hash(id);
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
    var c: Crown;
    c.trunk_top = trunk;
    c.lo0 = trunk; c.hi0 = trunk+1.; c.w0a = 0.; c.w0b = 0.;
    c.lo1 = trunk+1.; c.hi1 = trunk+2.; c.w1a = 0.; c.w1b = 0.;
    if biome == 7u {
        c.trunk_top = 1.;
        let top = trunk+3.;
        let mid = (1.+top)*0.5;
        c.lo0 = 1.; c.hi0 = mid; c.w0a = 0.95; c.w0b = 0.55;
        c.lo1 = mid; c.hi1 = top; c.w1a = 0.55; c.w1b = 0.15;
    } else {
        c.w0a = leaf_width(id,0u); c.w0b = c.w0a;
        c.w1a = leaf_width(id,1u); c.w1b = c.w1a;
    }
    return c;
}

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


// ---- `flora-v2` design 10: the detailed trees ----
//
// The near draws start at NEAR_TREE_FIRST_VERTEX with the species over bit
// 16 (`planet.rs`, `near_args`), and read their instances from the species'
// region of the near list. `pbd_core::tree_kit` packs the kit; its layout
// constants are these and a test holds them.
const NEAR_TREE_FIRST_VERTEX: u32 = 1048576u;
const NEAR_CAP: u32 = 4096u;
const KIT_SPECIES_WORDS: u32 = 16u;
const KIT_VARIANT_BASE: u32 = 128u;
const KIT_VARIANT_WORDS: u32 = 16u;
const KIT_VERTEX_WORDS: u32 = 6u;
const TREE_VARIANTS: u32 = 4u;
const IMPOSTOR_VIEW_W: u32 = 48u;
const IMPOSTOR_H: u32 = 72u;
const IMPOSTOR_W: u32 = 432u;
// The bake's ninth view, the tree from straight above, which the crown cap
// shows (`crossed-tree-cards` design 3).
const IMPOSTOR_TOP_VIEW: u32 = 8u;
// How far up the tree's height the crown cap lies.
const IMPOSTOR_CAP_HEIGHT: f32 = 0.55;
// The rings' cross-fade: the last share of each ring (the mockup's 0.3).
// How far up a far tree's standing normal leans (design 13.4).
const IMPOSTOR_LIFT: f32 = 0.35;
const TREE_BARK_CODE: u32 = 24u;
const TREE_LEAF_CODE: u32 = 25u;
const IMPOSTOR_CODE: u32 = 26u;
// Words from an impostor strip to its relief strip (`tree-impostor-depth`):
// `pbd_core::tree_kit::IMPOSTOR_RELIEF`, which a test holds.
const IMPOSTOR_RELIEF: u32 = 31104u;
// The species and variant ride over the material word's biome byte.
const SPECIES_SHIFT: u32 = 17u;
const VARIANT_SHIFT: u32 = 21u;
const SALT_SPECIES: u32 = 0x59u;
const SALT_VARIANT: u32 = 0x5au;
const SALT_TURN: u32 = 0x5bu;
const SALT_SIZE: u32 = 0x5cu;
// The mockup's wind (`uWind`, `uGust`) and the trees' sway over 8 m.
const TREE_WIND: f32 = 0.35;
const TREE_GUST: f32 = 0.5;
const TREE_WIND_H: f32 = 8.0;

fn kit_f(i: u32) -> f32 { return bitcast<f32>(kit[i]); }
fn kit_v3(i: u32) -> vec3<f32> { return vec3(kit_f(i), kit_f(i + 1u), kit_f(i + 2u)); }

// Which species a tree is, by its biome's mix: the visibility pass's
// `tree_species`, the same lines (`pbd_core::tree_kit::pick`).
fn tree_species(id: u32, biome: u32) -> u32 {
    let r = roll(id, SALT_SPECIES);
    if biome == 1u { return 4u; }
    if biome == 2u { return select(1u, 0u, r < 0.6); }
    if biome == 3u { return 7u; }
    if biome == 4u { return select(3u, 2u, r < 0.55); }
    if biome == 5u { return select(1u, 6u, r < 0.75); }
    if biome == 7u { return select(1u, 5u, r < 0.85); }
    return 0u;
}

// A tree's frame on the body: its base, up, its own x and z axes turned by
// its roll, and its size (`pbd_core::tree_kit::pick`).
struct TreeFrame {
    base: vec3<f32>, up: vec3<f32>, x: vec3<f32>, z: vec3<f32>,
    species: u32, variant: u32, size: f32, header: u32,
}
fn tree_frame(id: u32, biome: u32, base: vec3<f32>, axis: vec3<f32>, tangent: vec3<f32>, bitangent: vec3<f32>) -> TreeFrame {
    var t: TreeFrame;
    t.base = base;
    t.up = axis;
    t.species = tree_species(id, biome);
    t.variant = min(u32(roll(id, SALT_VARIANT)*f32(TREE_VARIANTS)), TREE_VARIANTS - 1u);
    let turn = roll(id, SALT_TURN)*TAU;
    t.x = tangent*cos(turn) + bitangent*sin(turn);
    t.z = cross(t.x, axis);
    t.size = 0.85 + roll(id, SALT_SIZE)*0.35;
    t.header = KIT_VARIANT_BASE + (t.species*TREE_VARIANTS + t.variant)*KIT_VARIANT_WORDS;
    return t;
}
fn tree_point(t: TreeFrame, p: vec3<f32>) -> vec3<f32> {
    return t.base + (t.x*p.x + t.up*p.y + t.z*p.z)*t.size;
}
fn tree_dir(t: TreeFrame, n: vec3<f32>) -> vec3<f32> {
    return normalized(t.x*n.x + t.up*n.y + t.z*n.z);
}
// The mockup's sway: a bend growing with the square of the height over the
// base, a slow sway and a gust rolling across the land, and a cross wobble.
fn tree_sway(t: TreeFrame, w: vec3<f32>, amp: f32) -> vec3<f32> {
    let time = params.settings.z;
    let wind = normalized(cross(t.up, normalized(vec3(0.31, 0.83, 0.47))));
    let side = cross(t.up, wind);
    let h = max(dot(w - t.base, t.up), 0.0)/TREE_WIND_H;
    let ph = f32(hash(bitcast<u32>(t.base.x)) & 1023u)/1023.0*TAU;
    var gust = sin(dot(w, wind)*0.06 - time*1.6)*0.5 + 0.5;
    gust = gust*gust*gust*TREE_GUST;
    let sway = TREE_WIND*(0.35 + 0.3*sin(time*1.9 + ph)) + gust*TREE_WIND*1.6;
    let bend = h*h*amp;
    return w + wind*bend*sway - t.up*bend*sway*sway*0.25*TREE_WIND_H
        + side*bend*TREE_WIND*0.18*sin(time*2.7 + ph*1.7);
}
// A tree's cross-fade is a moment in time, never a place (design 13.3;
// owner, 2026-11-16: "dithering should either stay or go, dont have it
// linger if the player isnt moving"). Which representation a tree at `base`
// takes seen from `eye`: 0 the near model, 1 the mid cards, 2 the far
// cards, 3 none. Hard thresholds, in the middle of the old rings'
// cross-fades (0.85 of each ring, `TREE_SWITCH`), so the visibility pass,
// which lists by the camera itself, still holds both representations while
// the point they are decided at trails it (`tree_fade_lag_m`).
const TREE_SWITCH: f32 = 0.85;
fn tree_level(eye: vec3<f32>, base: vec3<f32>) -> u32 {
    let d = distance(eye, base);
    if d > params.settings.w - TREE_FADE_MARGIN_M - 0.5*params.fade.x { return 3u; }
    if d < params.flora_more.z*TREE_SWITCH { return 0u; }
    if d < params.flora_more.w*TREE_SWITCH { return 1u; }
    return 2u;
}
// How representation `r` of the tree at `base` is drawn (`out_height`, which
// `tree_dropped` reads): 1 whole, 1 - t going out, 2 + t coming in, 0 not at
// all. Whole wherever the two points the fade runs between agree, which is
// everywhere one fade after the camera stops.
fn tree_show(base: vec3<f32>, r: u32) -> f32 {
    let was = tree_level(params.tree_fade[0].xyz, base);
    let now = tree_level(params.tree_fade[1].xyz, base);
    let t = clamp(params.tree_fade[0].w, 0.0, 1.0);
    if was == now { return select(0.0, 1.0, r == now); }
    if r == now { return 2.0 + t; }
    if r == was { return 1.0 - t; }
    return 0.0;
}
// The same for what the mid and the far ring both draw, the X's two cards
// and the crown cap (`crossed-tree-cards` design 4): whole while the tree is
// in either ring, so going from mid to far fades only the diagonal cards.
fn ring_show(base: vec3<f32>) -> f32 {
    let was = tree_level(params.tree_fade[0].xyz, base);
    let now = tree_level(params.tree_fade[1].xyz, base);
    let t = clamp(params.tree_fade[0].w, 0.0, 1.0);
    let a = was == 1u || was == 2u;
    let b = now == 1u || now == 2u;
    if a && b { return 1.0; }
    if b { return 2.0 + t; }
    if a { return 1.0 - t; }
    return 0.0;
}



@vertex
fn vertex(@builtin(vertex_index) vertex: u32, @builtin(instance_index) instance: u32) -> VertexOut {
    // A near tree's draw reads its species' region of the near list.
    var index = instance;
    if vertex >= NEAR_TREE_FIRST_VERTEX {
        index = min(((vertex - NEAR_TREE_FIRST_VERTEX) >> 16u), 7u)*NEAR_CAP + min(instance, NEAR_CAP - 1u);
    }
    let listed = visible[index];
    let mark = listed >> PART_SHIFT;
    let part = partition_of(mark);
    let cell = cells[listed & PART_MASK];
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
    var cut_wall = false;
    var material = cell.metadata.y & 0xffu;
    // One everywhere but down a grass blade, whose root is darker than its tip.
    var out_shade = 1.;
    // Bits over the material word's biome byte (`UNDER_TREE_BIT`).
    var out_flags = 0u;
    // The height a face measures its depth from: the cell's surface, except on
    // a cave run's flank, where it is that run's own top.
    var out_height = height;
    // The voxel field's answer at THIS vertex, one outside the column tier
    // where there is no field to ask. Per vertex rather than per cell, which
    // is the whole of what "baked vertex colours" means here: a face grades
    // across itself because its corners were sampled apart.
    var out_voxel = vec2(1., 0.);
    // A tree's own (`flora-v2` design 10.4): its dither offset, a leaf
    // card's id; carried where the partition's split plane is for the rest.
    var tree_split = vec3(0.0);
    // Set for a glowing flower's head while it burns, and applied last, over
    // the field the clutter takes (`lamps-and-lanterns` decision 8).
    var bloom = false;
    // Open to the rain unless the column pass says otherwise: the terrain
    // pass's cap IS its column's top, its height-field walls stand only off
    // the tier where nothing overhangs, and trees and clutter stand on the cap.
    var out_rain = 1.;
    // An instance drawn for the old partition is a record of the set a landing
    // dissolves from (`two-set-dissolve`), and the slot it carries is in that
    // set's column tier, whose buffers the landing rewrote: it is drawn as
    // the heightfield its record holds, with no column, for the dissolve's
    // second.
    let lit_slot = select(column_slot(cell), 0u, mark == PART_OLD);
    if vertex < 18u {
        // The fallen leaves gather on a cap whose cell stands a tree
        // (`flora-v2` design 9.4).
        if stands_tree(cell) { out_flags = UNDER_TREE_BIT; }
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
        let slot = lit_slot;
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
                if covered_by_finer_in(neighbor, level, part) {
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
        // Built wherever the owners' fades differ (`distance-lod-fade`): in
        // the pixels whose dither value falls between them one owner is fine
        // and the other coarse, and the fragment keeps it only there. It
        // faces the owner further out of the fine band.
        let a_t = band_t(cell.owner_a.xyz, level, part);
        let b_t = band_t(cell.owner_b.xyz, level, part);
        if level > base_level() && abs(a_t - b_t) > 1e-6 {
            cut_wall = true;
            let coarse = select(cell.owner_a.xyz, cell.owner_b.xyz, a_t > b_t);
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
    } else if vertex >= NEAR_TREE_FIRST_VERTEX {
        // ---- A near detailed tree (`flora-v2` design 10.2): the mockup's
        // model, pulled vertex by vertex from the kit, in the tree's own
        // frame turned and sized by its roll; then its fruit on the
        // model's own anchors (design 10.6).
        kind = 2u;
        let id = cell.spare.x;
        let biome = (cell.metadata.y >> 8u) & 0xffu;
        let t = tree_frame(id, biome, axis*radius, axis, tangent, bitangent);
        out_flags = (t.species << SPECIES_SHIFT) | (t.variant << VARIANT_SHIFT);
        tree_split = vec3(f32(hash(id) & 3u), 0.0, 0.0);
        let local = (vertex - NEAR_TREE_FIRST_VERTEX) & 0xffffu;
        // Outgoing across the near ring's last 0.3 (design 10.4).
        out_height = tree_show(t.base, 0u);
        position = t.base;
        let count = kit[t.header + 1u];
        let vmax = kit[t.species*KIT_SPECIES_WORDS + 12u];
        if local < count {
            let first = kit[t.header];
            let tri = local/3u;
            let corner = local%3u;
            var ps = array<vec3<f32>,3>(vec3(0.0), vec3(0.0), vec3(0.0));
            for (var c = 0u; c < 3u; c++) {
                ps[c] = tree_point(t, kit_v3(first + (tri*3u + c)*KIT_VERTEX_WORDS));
            }
            let leaf = ((kit[first + tri*3u*KIT_VERTEX_WORDS + 5u] >> 16u) & 1u) == 1u;
            // A leaf card is seen from both sides, as the mockup's are: its
            // triangle turns to face the eye (the pass culls back faces).
            var pick = corner;
            if leaf && dot(cross(ps[1] - ps[0], ps[2] - ps[0]), params.camera.xyz - ps[0]) < 0.0 {
                pick = array<u32,3>(0u, 2u, 1u)[corner];
            }
            let w = first + (tri*3u + pick)*KIT_VERTEX_WORDS;
            let uvw = kit[w + 4u];
            uv = vec2(f32(uvw & 0xffffu), f32(uvw >> 16u))/4096.0;
            let word = kit[w + 5u];
            let card = f32(word & 0xffu)/255.0;
            normal = tree_dir(t, unpack4x8snorm(kit[w + 3u]).xyz);
            var p = tree_sway(t, ps[pick], select(0.1, 0.16, leaf));
            if leaf {
                // The card's own flutter, masked: its root stays on the branch.
                let time = params.settings.z;
                let ph = f32(hash(id) & 1023u)/1023.0*TAU;
                let sw = clamp(uv.y, 0.0, 1.0)*clamp(uv.y, 0.0, 1.0);
                let flutter = vec3(sin(time*7.0 + card*40.0 + ph), cos(time*6.0 + card*31.0), sin(time*5.0 + card*23.0));
                p += (t.x*flutter.x + t.up*flutter.y + t.z*flutter.z)*0.05*TREE_WIND*1.2*sw;
            }
            position = p;
            material = select(TREE_BARK_CODE, TREE_LEAF_CODE, leaf);
            // The mockup's root-to-tip AO.
            out_shade = mix(0.45, 1.0, f32((word >> 8u) & 0xffu)/255.0);
            tree_split.y = card;
            // A vine's card (design 12.4): its own green and cut.
            tree_split.z = f32((word >> 17u) & 1u);
        } else if local >= vmax && local < vmax + 60u {
            // ---- fruit (`flora-v2` design 9.6, 10.6): four hanging from the
            // model's anchors (its last twigs, "on wood, never floating"),
            // and two fallen on the cap under the crown. Only a tree of a
            // fruiting biome, on its own roll; anything else collapses to
            // the base like a part a tree lacks.
            let f = local - vmax;
            let b = min(biome, 7u);
            // The low half is the fruit; the high half the tree table (12.1).
            let bits = params.plant_means[b].w & 0xffffu;
            let anchors = kit[t.header + 3u];
            if bits != 0u && anchors > 0u && roll(id,SALT_FRUIT) < params.flora_fall.w {
                // Neighbouring trees tend to bear the same kind: a patch is a
                // forty-metre stretch of the body.
                let grove = vec3<i32>(floor(axis*params.settings.x/40.0));
                let p = hash(u32(grove.x)*73856093u ^ u32(grove.y)*19349663u ^ u32(grove.z)*83492791u);
                let count = countOneBits(bits);
                var pick = (p + select(0u, 1u, roll(id,SALT_FRUIT+1u) < 0.25)) % count;
                var which = 0u;
                for (var q = 0u; q < 9u; q++) {
                    if ((bits >> q) & 1u) == 1u {
                        if pick == 0u { which = q; break; }
                        pick -= 1u;
                    }
                }
                let cone = which == 8u;
                // The season keeps the fruit (the mockup's: summer 0.85, all
                // in autumn, 0.08 in winter); a pine keeps its cones.
                let season = season_at(axis);
                let keep = select(mix(0.85, 1.01, season.x)*(1.0-0.92*season.y), 1.0, cone);
                let item = select(4u + (f-48u)/6u, f/12u, f < 48u);
                // A tree picked bare keeps only what fell (design 12.5).
                if roll(id,SALT_FRUIT+2u+item) < keep && !(f < 48u && tree_picked(id)) {
                    let size = select(0.40, 0.46, cone)*(0.9+0.3*roll(id,SALT_FRUIT+30u+item));
                    var corners = array<vec3<f32>,4>(axis,axis,axis,axis);
                    if f < 48u {
                        // A hanging fruit: two crossed cards, the top edge at
                        // an anchor, its stem on the wood.
                        let at = min(u32(roll(id,SALT_FRUIT+10u+item)*f32(anchors)), anchors - 1u);
                        let top = tree_sway(t, tree_point(t, kit_v3(kit[t.header + 2u] + at*3u)), 0.1);
                        let a = roll(id,SALT_FRUIT+40u+item)*TAU+f32((f%12u)/6u)*TAU*0.25;
                        let dir = tangent*cos(a)+bitangent*sin(a);
                        let half_w = size*0.5;
                        corners = array<vec3<f32>,4>(top-axis*size-dir*half_w,top-axis*size+dir*half_w,
                            top+dir*half_w,top-dir*half_w);
                    } else {
                        // A fallen one, flat on the cap under the crown.
                        let side_k = u32(roll(id,SALT_FRUIT+10u+item)*f32(degree)) % degree;
                        let along = roll(id,SALT_FRUIT+20u+item);
                        let rim = normalized(mix(cell.corners[side_k].xyz, cell.corners[(side_k+1u)%degree].xyz, along));
                        let at = normalized(axis+(rim-axis)*(0.25+0.45*roll(id,SALT_FRUIT+50u+item)));
                        let c = at*(radius+0.03);
                        let a = roll(id,SALT_FRUIT+60u+item)*TAU;
                        let u = (tangent*cos(a)+bitangent*sin(a))*(size*0.4);
                        let w = cross(axis, u);
                        corners = array<vec3<f32>,4>(c-u-w,c+u-w,c+u+w,c-u+w);
                    }
                    let i = f%6u;
                    let facing = dot(cross(corners[1]-corners[0],corners[3]-corners[0]),params.camera.xyz-corners[0]);
                    let forward = array<u32,6>(0u,1u,2u,0u,2u,3u);
                    let reversed = array<u32,6>(0u,2u,1u,0u,3u,2u);
                    var c = forward[i];
                    if facing < 0. { c = reversed[i]; }
                    position = corners[c];
                    let k = f32(which);
                    let card_uv = array<vec2<f32>,4>(vec2(k,0.),vec2(k+0.999,0.),vec2(k+0.999,1.),vec2(k,1.));
                    uv = card_uv[c];
                    normal = axis;
                    material = FRUIT_CODE;
                }
            }
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
        // Everything this pass draws is under rock until a face below proves
        // it has the sky over the air in front of it.
        out_rain = 0.;
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
                    // A floor is rained on only if no run of this column stands
                    // over it: a cave floor is dry, and so is the ground under a
                    // block put over it. A ceiling faces down and never is.
                    var top_run = true;
                    for (var k = r + 1u; k < COLUMN_RUNS; k++) {
                        if run_present(rec.runs[k]) { top_run = false; }
                    }
                    out_rain = select(0., 1., up && top_run);
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
                // ---- A lamp: a torch, a slim post standing on the floor
                // with a lit head on it, or two boxes for every other light
                // (`lamps-and-lanterns` task 5.2).
                //
                // Drawn HERE rather than in the clutter, because a torch is at
                // a layer rather than on the surface: one can stand on a cave
                // floor forty metres down, which is the whole reason to carry
                // one.
                let t = v-(COLUMN_CAP_VERTICES+6u*COLUMN_SIDE_VERTICES);
                let layer = lamp_layer(rec);
                let which = lamp_kind(rec);
                if layer != 0u && which == LAMP_TORCH && t < 30u {
                    let foot = params.settings.x+COLUMN_BASE_M+f32(layer-1u);
                    kind = KIND_LAMP;
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
                        material = LAMP_WOOD_CODE | (cell.metadata.y & 0xff00u);
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
                        material = LAMP_WOOD_CODE | (cell.metadata.y & 0xff00u);
                        out_voxel = vec2(0.0, LAMP_FLAME + FLAME_RISE*select(0.0, 1.0, c == 2u));
                    }
                } else if layer != 0u && which != LAMP_TORCH {
                    let foot = params.settings.x+COLUMN_BASE_M+f32(layer-1u);
                    kind = KIND_LAMP;
                    // The frame: across the cell, and up. A wall lantern turns
                    // to face the side of its cell that has a wall at its
                    // layer, found here from the neighbours' runs, so nothing
                    // is stored and a lantern follows its wall.
                    var x = tangent;
                    var y = bitangent;
                    var reach = 0.0;
                    if which == LAMP_WALL {
                        for (var side = 0u; side < degree; side++) {
                            let there = column_side(rec, side);
                            let solid = there == NO_NEIGHBOR || solid_at(there + 1u, layer - 1u);
                            if solid && reach == 0.0 {
                                let mid = normalized(cell.corners[side].xyz + cell.corners[(side+1u)%degree].xyz);
                                let across = mid - axis*dot(mid, axis);
                                x = normalized(across);
                                y = normalized(cross(axis, x));
                                reach = length(across)*foot;
                            }
                        }
                        if reach == 0.0 { reach = 0.6; }
                    }
                    let box = t/36u;
                    let i = t%36u;
                    // Two boxes: the frame (post, bracket, chain, stand, wax)
                    // and the light (lantern, fire, flame).
                    var centre = vec3(0.);
                    var half = vec3(0.);
                    // A brazier's and a candle's light is a bare flame; the
                    // three lanterns' is lit glass in an iron frame.
                    let flame = which == LAMP_BRAZIER || which == LAMP_CANDLE || which == LAMP_CAMPFIRE;
                    var code = LAMP_IRON_CODE;
                    if box == 1u && !flame { code = LAMP_GLASS_CODE; }
                    let glow = box == 1u && lamp_lit(rec);
                    if which == LAMP_POST {
                        if box == 0u { centre = axis*(foot+0.75); half = vec3(0.05,0.05,0.75); }
                        else { centre = axis*(foot+1.68); half = vec3(0.15,0.15,0.18); }
                    } else if which == LAMP_WALL {
                        if box == 0u { centre = axis*(foot+0.95)+x*(reach-0.2); half = vec3(0.2,0.03,0.03); }
                        else { centre = axis*(foot+0.72)+x*(reach-0.36); half = vec3(0.13,0.13,0.15); }
                    } else if which == LAMP_HANGING {
                        if box == 0u { centre = axis*(foot+0.8); half = vec3(0.02,0.02,0.2); }
                        else { centre = axis*(foot+0.45); half = vec3(0.13,0.13,0.15); }
                    } else if which == LAMP_BRAZIER {
                        // An iron basket on its stand, and the fire in it.
                        if box == 0u { centre = axis*(foot+0.3); half = vec3(0.26,0.26,0.3); }
                        else { centre = axis*(foot+0.82); half = vec3(0.17,0.17,0.22); }
                    } else if which == LAMP_CAMPFIRE {
                        // A low ring of stones, wide on the ground, and the
                        // fire on it (`cooking` design S2).
                        if box == 0u { centre = axis*(foot+0.09); half = vec3(0.40,0.40,0.09); code = LAMP_STONE_CODE; }
                        else { centre = axis*(foot+0.42); half = vec3(0.19,0.19,0.24); }
                    } else {
                        if box == 0u { centre = axis*(foot+0.12); half = vec3(0.045,0.045,0.12); code = LAMP_WAX_CODE; }
                        else { centre = axis*(foot+0.29); half = vec3(0.022,0.022,0.05); }
                    }
                    let part = lamp_box(i, centre, x, y, axis, half);
                    position = part.position;
                    normal = part.normal;
                    uv = part.uv;
                    // The light's own box, while it burns, is drawn as the
                    // light it is; everything else takes what the field has
                    // in its cell, which is mostly its own lamp.
                    material = code | (cell.metadata.y & 0xff00u);
                    let burning = select(LAMP_PANES, LAMP_FLAME + FLAME_RISE*part.rise, flame);
                    out_voxel = select(light_at(lit_slot, layer-1u), vec2(0.0, burning), glow);
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
                    // A flank faces the neighbour's air: rained on only if that
                    // is the neighbour's TOP gap, the one with nothing over it.
                    // A terrace step is wet; the wall of a cave is not.
                    out_rain = select(0., 1., gap.y >= COLUMN_TOP_M);
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
    } else if vertex < CLUTTER_FIRST_VERTEX {
        // ---- A tree's crossed cards and crown cap (`flora-v2` design 10.3,
        // `crossed-tree-cards`), baked from its near model. The tree is the
        // finest cell's at this cell's centre (`distance-lod-fade`): it rolls
        // on that id, so a coarse level draws the same tree the fine one did.
        // The compute pass selects the cells; no tree vertex is invoked for a
        // cell without one.
        kind = 2u;
        let id = cell.spare.x;
        let biome = (cell.metadata.y >> 8u) & 0xffu;
        let t = tree_frame(id, biome, axis*radius, axis, tangent, bitangent);
        out_flags = (t.species << SPECIES_SHIFT) | (t.variant << VARIANT_SHIFT);
        tree_split = vec3(f32(hash(id) & 3u), 0.0, 0.0);
        material = IMPOSTOR_CODE;
        position = t.base;
        normal = axis;
        // Past the fine band a tree stands for four (`distance-lod-fade`):
        // as wide as the prism grew, and half as much taller, so a far
        // forest keeps its cover.
        var grown = 1.0;
        for (var l = finest_level(); l > finest_level() - 3u; l--) {
            grown = mix(2.0*grown, grown, band_t(axis, l, part));
        }
        let half_w = kit_f(t.header + 5u)*0.5*t.size*grown;
        let tall = t.size*sqrt(grown);
        let y0 = kit_f(t.header + 6u)*tall;
        let y1 = kit_f(t.header + 7u)*tall;
        let to_eye = params.camera.xyz - t.base;
        let eye = vec2(dot(to_eye, t.x), dot(to_eye, t.z));
        let eye_l = length(eye);
        let dir = select(vec2(0.0, 1.0), eye/max(eye_l, 1e-6), eye_l > 1e-3);
        // Lit as a standing tree, not a floor (design 13.4; owner: "distant
        // trees appear to be glowing evenly"): toward the eye across the
        // ground and a little up, as the near crown's leaf cards face it. The
        // tree's up axis lit a card with the zenith's whole sky, flat. One
        // normal for every card and the cap, so no plane of a tree is lit
        // apart from the others. Each shown card and the cap then carry
        // their baked view's level direction instead (`tree-impostor-depth`),
        // and the fragment rebuilds this standing normal from the eye.
        normal = normalized(t.x*dir.x + t.z*dir.y + axis*IMPOSTOR_LIFT);
        let v = vertex - 60u;
        let k = v/6u;
        // Four cards crossed through the trunk at fixed angles on the tree's
        // own turn, never turned to the eye (owner, 2026-11-17: "2 - 4
        // billboards with no rotation"), then the crown cap. The far ring
        // draws cards 0 and 2, an X, and the cap; the mid ring all five.
        // Whole, or fading in or out by time (design 13.3).
        var f = ring_show(t.base);
        if k == 1u || k == 3u { f = tree_show(t.base, 1u); }
        if f > 0.0 {
            out_height = f;
            var corners = array<vec3<f32>,4>(t.base, t.base, t.base, t.base);
            var view = 0u;
            var flip = false;
            var shown = true;
            if k < 4u {
                // Each card shows the view baked from its own side (seen
                // from behind, the view from there).
                let phi = f32(k)*TAU/8.0;
                let along = t.x*cos(phi) - t.z*sin(phi);
                flip = dot(eye, vec2(sin(phi), cos(phi))) < 0.0;
                view = select(k, k + 4u, flip);
                corners = array<vec3<f32>,4>(
                    t.base + axis*y0 - along*half_w, t.base + axis*y0 + along*half_w,
                    t.base + axis*y1 + along*half_w, t.base + axis*y1 - along*half_w);
            } else {
                // The crown cap (design 3): level, a little over halfway up,
                // showing the bake from straight above; drawn only while the
                // eye is above it, where it is edge-on as the eye crosses.
                let cap = t.base + axis*(kit_f(t.header + 4u)*IMPOSTOR_CAP_HEIGHT*tall);
                shown = dot(params.camera.xyz - cap, axis) > 0.0;
                view = IMPOSTOR_TOP_VIEW;
                corners = array<vec3<f32>,4>(
                    cap + (t.z - t.x)*half_w, cap + (t.z + t.x)*half_w,
                    cap + (t.x - t.z)*half_w, cap - (t.x + t.z)*half_w);
            }
            if shown {
                for (var c = 0u; c < 4u; c++) {
                    corners[c] = tree_sway(t, corners[c], 0.08);
                }
                let i = v%6u;
                let facing = dot(cross(corners[1]-corners[0],corners[3]-corners[0]),params.camera.xyz-corners[0]);
                let forward = array<u32,6>(0u,1u,2u,0u,2u,3u);
                let reversed = array<u32,6>(0u,2u,1u,0u,3u,2u);
                var c = forward[i];
                if facing < 0. { c = reversed[i]; }
                position = corners[c];
                var u = array<f32,4>(0.0, 0.999, 0.999, 0.0)[c];
                if flip { u = 0.999 - u; }
                uv = vec2(f32(view) + u, select(0.0, 1.0, c >= 2u));
                // `tree-impostor-depth`: the baked view's own direction,
                // laid level, which is the frame its relief strip keeps the
                // near model's normals in; the fragment lights each pixel by
                // them. The cap's view (8) is bearing 0's frame, the tree's
                // own, so every card and the cap rebuild the same normals.
                let th = f32(view)*TAU/8.0;
                normal = t.x*sin(th) + t.z*cos(th);
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
        let v = vertex-CLUTTER_FIRST_VERTEX;
        let id = cell.metadata.w;
        let biome = min((cell.metadata.y >> 8u) & 0xffu, 7u);
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
            // ---- grass: a pixel tuft, two crossed cards in the slot's twelve
            // vertices (`flora-v2` design 1; the only grass since 2026-11-16,
            // the owner: "grass should be pixel tufts, remove the other grass").
            let j = v/12u;
            let i = v%12u;
            let tufts = u32(round((0.6+0.4*roll(id,SALT_GRASS+1u))*params.flora.x));
            if green && roll(id,SALT_GRASS) < params.clutter_chance.x && j < tufts {
                let sj = SALT_GRASS+16u+j*8u;
                let k = u32(roll(id,sj)*f32(degree));
                let f0 = 0.15+0.60*roll(id,sj+1u);
                let bottom = (axis*(1.-f0)+cell.corners[min(k,degree-1u)].xyz*f0)*radius;
                let size = 0.8+0.5*roll(id,sj+3u);
                let half_w = params.flora.y*size*0.5;
                let h = params.flora.z*size;
                let a = roll(id,sj+2u)*TAU+f32(i/6u)*TAU*0.25;
                let dir = tangent*cos(a)+bitangent*sin(a);
                // The tip row sways along the card; the root stays put.
                let sway = sin(1.7*params.settings.z+roll(id,sj+6u)*TAU)*0.06*h;
                let top = bottom+axis*h+dir*sway;
                quad = array<vec3<f32>,4>(bottom-dir*half_w,bottom+dir*half_w,
                    top+dir*half_w,top-dir*half_w);
                // The variant in the integer part, the card in the fraction.
                let variant = f32(u32(roll(id,sj+5u)*3.));
                quad_uv = array<vec2<f32>,4>(vec2(variant,0.),vec2(variant+0.999,0.),
                    vec2(variant+0.999,1.),vec2(variant,1.));
                material = TUFT_CODE;
                is_quad = true;
                grown = h > 0.0001;
            }
        } else if v < 234u {
            // ---- flower: a slim stem and a bright four-triangle head.
            let i = v-216u;
            // A cell the CPU chose for a glowing flower grows one whatever
            // its own roll said: the record's bit is the one decider, and the
            // rolls below only place and turn it (decision 8).
            let glowing = (cell.spare.z & GLOW_FLOWER_BIT) != 0u;
            if green && (glowing || roll(id,SALT_FLOWER) < params.clutter_chance.y) {
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
                    // The head takes one of its biome's flower colours
                    // (`flora-v2` design 3), carried in the uv's integer part.
                    material = FLOWER_CODE;
                    let bits = u32(params.flora_biome[biome].z) & 15u;
                    var pick = u32(roll(id,SALT_FLOWER+4u)*f32(max(countOneBits(bits),1u)));
                    var colour = 0u;
                    for (var q = 0u; q < 4u; q++) {
                        if ((bits >> q) & 1u) == 1u {
                            if pick == 0u { colour = q; break; }
                            pick -= 1u;
                        }
                    }
                    if glowing {
                        material = BLOOM_CODE;
                        bloom = lit_slot != 0u
                            && (columns[lit_slot-1u].more[3] & GLOW_LIT_BIT) != 0u;
                    }
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
                    if !glowing { uv.x += f32(colour); }
                }
                grown = h > 0.0001;
            }
        } else if v < 288u {
            // ---- pebble: one squat hexagonal prism on bare ground, and
            // sparsely on grass at the reference's own third.
            if clutter_grows(0u,id,material,params.clutter_chance.z) {
                let bottom = clutter_seat(id,SALT_ROCK,degree,axis,cell.corners,radius);
                let rad = params.clutter_size.z*(0.6+0.8*roll(id,SALT_ROCK+3u));
                let h = rad*(0.4+0.5*roll(id,SALT_ROCK+4u))*fade*(1.-clutter_sunk(id,0u));
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
            if clutter_grows(1u,id,material,params.clutter_chance.w) {
                let bottom = clutter_seat(id,SALT_BUSH,degree,axis,cell.corners,radius);
                let size = params.clutter_size.w*(0.8+0.4*roll(id,SALT_BUSH+3u));
                let phase = roll(id,SALT_BUSH+4u)*TAU;
                material = 9u;
                let second = v >= 342u;
                let lump = select(bottom,
                    bottom+(tangent*cos(phase)+bitangent*sin(phase))*(size*1.1),second);
                let rad = select(size,size*0.6,second);
                let h = select(size*0.9,size*0.55,second)*fade*(1.-clutter_sunk(id,1u));
                let turn = select(phase,phase+0.7,second);
                let part = select(v-288u,v-342u,second);
                let piece = prism_vertex(part,lump,axis,tangent,bitangent,rad,h,turn);
                position = piece.position;
                normal = piece.normal;
                uv = piece.uv;
                grown = h > 0.0001;
            }
        } else if v < 432u {
            // ---- dead shrub: a splay of dry twigs, which on desert sand and
            // tundra snow is the only ground cover there is.
            let i = v-396u;
            if clutter_grows(2u,id,material,params.clutter_more.y) {
                let twig = i/6u;
                let bottom = clutter_seat(id,SALT_SHRUB,degree,axis,cell.corners,radius);
                let phase = roll(id,SALT_SHRUB+3u)*TAU;
                let ang = phase+f32(twig)*TAU/6.+roll(id,SALT_SHRUB+8u+twig)*0.5;
                let h = params.clutter_more.z*(0.6+0.7*roll(id,SALT_SHRUB+16u+twig))*fade
                    *(1.-clutter_sunk(id,2u));
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
        } else if v < 540u {
            // ---- mushroom (`flora-v2` design 3): a pale stem prism and a
            // squat cap prism wider than it, at the biome's weight.
            let chance = min(params.flora_more.x*params.flora_biome[biome].y, 1.);
            if green && roll(id,SALT_MUSH) < chance {
                let k = u32(roll(id,SALT_MUSH+1u)*f32(degree));
                let f0 = 0.15+0.60*roll(id,SALT_MUSH+2u);
                let bottom = (axis*(1.-f0)+cell.corners[min(k,degree-1u)].xyz*f0)*radius;
                let stem_h = (0.12+0.08*roll(id,SALT_MUSH+3u))*fade;
                let cap_r = 0.09+0.05*roll(id,SALT_MUSH+4u);
                let phase = roll(id,SALT_MUSH+5u)*TAU;
                var piece: Piece;
                if v < 486u {
                    piece = prism_vertex(v-432u,bottom,axis,tangent,bitangent,0.03,stem_h,phase);
                    material = MUSH_STEM_CODE;
                } else {
                    piece = prism_vertex(v-486u,bottom+axis*(stem_h*0.85),axis,tangent,bitangent,
                        cap_r,cap_r*0.45*fade,phase+0.5);
                    material = select(CAP_BROWN_CODE,CAP_RED_CODE,params.tuft_ramps[biome].w == 1u);
                }
                position = piece.position;
                normal = piece.normal;
                uv = piece.uv;
                grown = stem_h > 0.0001;
            }
        } else {
            // ---- fern (`flora-v2` design 3): four fronds arching out of a
            // hashed seat, each cut to its shape by a pixel mask.
            let chance = min(params.flora_more.y*params.flora_biome[biome].x, 1.);
            if clutter_grows(3u,id,material,chance) {
                let frond = (v-540u)/6u;
                let bottom = clutter_seat(id,SALT_FERN,degree,axis,cell.corners,radius);
                let size = 0.8+0.4*roll(id,SALT_FERN+3u);
                let ang = roll(id,SALT_FERN+4u)*TAU+f32(frond)*TAU*0.25
                    +(roll(id,SALT_FERN+8u+frond)-0.5)*0.6;
                let out_dir = tangent*cos(ang)+bitangent*sin(ang);
                let side = tangent*cos(ang+1.5708)+bitangent*sin(ang+1.5708);
                let len = 0.6*size*fade*(1.-clutter_sunk(id,3u));
                let sway = sin(1.5*params.settings.z+f32(frond)*2.1+roll(id,SALT_FERN+5u)*TAU)*0.04*len;
                let tip = bottom+out_dir*(len*0.8)+axis*(len*0.55)+side*sway;
                let hw = 0.12*size;
                quad = array<vec3<f32>,4>(bottom-side*hw,bottom+side*hw,tip+side*hw,tip-side*hw);
                quad_uv = array<vec2<f32>,4>(vec2(f32(frond),0.),vec2(f32(frond)+0.999,0.),
                    vec2(f32(frond)+0.999,1.),vec2(f32(frond),1.));
                material = FERN_CODE;
                is_quad = true;
                grown = len > 0.0001;
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
    // Trees and ground clutter take the field at their own cell
    // (`lamps-and-lanterns` decision 9): clutter at the air over the cap, as
    // the cap's own centre does, and a tree at the layer of its own height,
    // so a canopy over a lantern is lit from below. Off the tier there is no
    // field, and they keep the open sky the far terrain is drawn with.
    if (kind == 2u || kind == 3u) && lit_slot != 0u {
        let ground = air_above(lit_slot, height);
        let own = light_layer(length(position) - params.settings.x);
        out_voxel = light_at(lit_slot, select(ground, max(own, ground), kind == 2u));
    }
    if bloom { out_voxel = vec2(0.0, LAMP_BLOOM); }
    var out: VertexOut;
    out.position = position;
    out.clip = params.clip_from_body*vec4(position,1.);
    out.normal = normal;
    out.uv = uv;
    // The material in the low byte and the cell's BIOME above it, exactly as
    // the record packs them: the fragment needs the biome to pick its sheet,
    // and carrying it in a word it already has costs no interpolator.
    out.material = (material & 0xffu) | (cell.metadata.y & 0xff00u) | out_flags
        | (cell.metadata.y & PLACED_SHEET_MASK);
    out.height = out_height;
    // The low half only: the high half carries the column tier slot.
    out.skylight = f32(cell.metadata.z & 0xffffu)/65535.;
    out.seed = cell.metadata.w;
    out.kind = kind;
    out.part_mark = mark;
    out.level = level;
    // The partition's per-pixel test (`distance-lod-fade`): the plane between
    // the owners (which owner a fragment of a split midpoint cell belongs to),
    // and the owners' fades and the finer level's fade at the centre. A cut
    // wall's finer fade is carried negative, less one, to mark it.
    out.split = select(cell.owner_a.xyz - cell.owner_b.xyz, tree_split, kind == 2u);
    let next_t = select(0.0, band_t(axis, level + 1u, part), level < finest_level());
    out.fade_t = vec3<f32>(band_t(cell.owner_a.xyz, level, part), band_t(cell.owner_b.xyz, level, part),
        select(next_t, -1.0 - next_t, cut_wall));
    out.shade = out_shade;
    out.voxel = out_voxel;
    out.slot = lit_slot;
    out.rain_open = out_rain;
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
// `farming-and-smelting`: tilled soil and a station block are materials of
// their own, drawn from one sheet wherever they lie: the fields sheet's
// tilled soil, the mountains sheet's slate bricks. `planet_terrain.rs`'s
// TILLED and FORGE, and the biomes' order in `Biome`; a test holds them.
const TILLED_CODE = 14u;
const FORGE_CODE = 15u;
const FIELDS_BIOME = 2u;
const MOUNTAINS_BIOME = 6u;
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
    // A furrow's side is the earth it was turned from.
    if cap == TILLED_CODE { return DIRT_CODE; }
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
// `lighting-model` (owner: "amount of rain drops on ground should vary based
// on rain intensity"): the same rings, each drop of each cycle kept with a
// chance `keep` picked anew every cycle, on a grid offset by `salt`.
fn rain_ripple_layer(uv: vec2<f32>, t: f32, keep: f32, salt: f32) -> vec2<f32> {
    let p0 = floor(uv);
    var circles = vec2(0.);
    for (var j=-1; j<=1; j++) {
        for (var i=-1; i<=1; i++) {
            let pi = p0+vec2(f32(i),f32(j));
            let p = pi+rr_hash22(pi+salt);
            let ph = 0.3*t+rr_hash12(pi+salt);
            let tt = fract(ph);
            if rr_hash12(pi*1.37+floor(ph)*7.13+salt) >= keep { continue; }
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
// As many drops as it is raining: a drizzle dots a puddle, a storm boils it
// (a second, offset grid above half), none once it stops.
fn rain_rings(uv: vec2<f32>, t: f32, rain: f32) -> vec2<f32> {
    var g = rain_ripple_layer(uv, t, clamp(rain*2.0, 0.0, 1.0), 0.0);
    if rain > 0.5 { g += rain_ripple_layer(uv+vec2(0.5,0.37), t*1.07, rain*2.0-1.0, 41.0); }
    return g;
}
// GGX with Smith-Schlick visibility and Schlick Fresnel at F0 0.02 (water):
// the light of colour `c` from direction `l` off a film of roughness^2 `a`.
fn ggx_d(nh: f32, a: f32) -> f32 {
    let a2 = a*a;
    let d = nh*nh*(a2-1.0)+1.0;
    return a2/(3.14159*d*d);
}
fn spec_light(n: vec3<f32>, v: vec3<f32>, l: vec3<f32>, c: vec3<f32>, a: f32) -> vec3<f32> {
    let nl = max(dot(n,l),0.0);
    if nl <= 0.0 { return vec3(0.0); }
    let h = normalized(v+l);
    let nv = max(dot(n,v),1e-3);
    let k = a*0.5;
    let vis = 0.25/((nl*(1.0-k)+k)*(nv*(1.0-k)+k));
    let f = 0.02+0.98*pow(1.0-max(dot(v,h),0.0),5.0);
    return c*ggx_d(max(dot(n,h),0.0),a)*vis*f*nl;
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

// The sky colour the ground hazes toward and mirrors when wet, taken through
// the same overcast the dome is: under full cover the blue goes grey by the
// sky's blue cut and the whole of it dims by the sky's dim. One function, so
// the haze on a far ridge and the sky in a puddle are the same sky.
fn ground_sky(sun_elevation: f32, cover: f32) -> vec3<f32> {
    let clear = mix(vec3(0.10,0.20,0.29),vec3(0.32,0.49,0.57),max(sun_elevation,0.));
    let grey = vec3(dot(clear,vec3(0.2126,0.7152,0.0722)));
    return mix(clear,grey,cover*params.rain[5].x)*(1.-cover*params.rain[5].y);
}
// Two octaves of value noise for the puddle mask, on cells of the given size.
fn puddle_noise(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let w = f*f*(3.-2.*f);
    let a = mix(rr_hash12(i),rr_hash12(i+vec2(1.,0.)),w.x);
    let b = mix(rr_hash12(i+vec2(0.,1.)),rr_hash12(i+vec2(1.,1.)),w.x);
    return mix(a,b,w.y);
}

// The puddle mask's mean and standard deviation, measured by sampling a
// transcription of the two `puddle_noise` octaves over 60,000 points: the
// identity of the noise rather than a tuning knob.
const PUDDLE_MASK_MEAN: f32 = 0.5003;
const PUDDLE_MASK_SPREAD: f32 = 0.1587;

// An ordered 4 x 4 Bayer threshold for a pixel, 0..1: the screen-door mask a
// fading piece is drawn through (`detail-fade`). Fixed to the screen, so a
// piece that stands still at one fade holds one still pattern.
fn bayer4(pixel: vec2<f32>) -> f32 {
    let p = vec2<u32>(pixel) & vec2<u32>(3u);
    let m = array<u32,16>(0u,8u,2u,10u, 12u,4u,14u,6u, 3u,11u,1u,9u, 15u,7u,13u,5u);
    return (f32(m[p.y*4u + p.x]) + 0.5)/16.0;
}

// How much of a tree at `position` is drawn under partition `part`, 0..1:
// the room left before the edge of the trees' outermost band or the foliage
// range, over `tree_fade_m` (`detail-fade`).
fn tree_shown(radial: vec3<f32>, position: vec3<f32>, part: Partition) -> f32 {
    var edge = params.settings.w;
    for (var level = finest_level() - 2u; level <= finest_level(); level++) {
        let c = band_cos_in(part.bands, level);
        if c <= 1.0 {
            edge = min(edge, acos(clamp(c, -1.0, 1.0))*params.settings.x);
            break;
        }
    }
    let from_anchor = acos(clamp(dot(radial, part.anchor), -1.0, 1.0))*params.settings.x;
    let left = min(edge - from_anchor, params.settings.w - distance(position, params.camera.xyz))
        - TREE_FADE_MARGIN_M;
    return clamp(left/params.fade.x, 0.0, 1.0);
}

// ---- `flora-v2`: the plants' pixel shapes and their season. ----

// The plants' season at a place (design 5): `pbd_core::flora::season_at`,
// whose constants a test holds these to. One calendar, half a year on across
// the equator, none in the tropics. x autumn colour, y bare, z straw, w the
// fallen leaves (design 9.4).
const SEASON_TROPIC = vec2<f32>(0.12, 0.45);
const SEASON_TURN = vec2<f32>(0.27, 0.15);
const SEASON_BARE = vec2<f32>(0.5, 0.13);
const SEASON_STRAW = vec2<f32>(0.47, 0.22);
fn season_bump(x: f32, at: vec2<f32>) -> f32 {
    let d = abs(x - at.x);
    return 1.0 - smoothstep(0.0, at.y, min(d, 1.0 - d));
}
fn season_at(radial: vec3<f32>) -> vec4<f32> {
    let own = fract(params.flora.w + select(0.0, 0.5, radial.y < 0.0));
    let strength = smoothstep(SEASON_TROPIC.x, SEASON_TROPIC.y, abs(radial.y));
    let bare = season_bump(own, SEASON_BARE);
    // w: the fallen leaves (design 9.4), landing through autumn, thinned to
    // LITTER_WINTER by midwinter, gone before spring.
    let fall = smoothstep(SEASON_LITTER.x, SEASON_LITTER.y, own)
        *(1.0 - smoothstep(SEASON_LITTER.z, SEASON_LITTER.w, own));
    return strength*vec4(season_bump(own, SEASON_TURN), bare,
        season_bump(own, SEASON_STRAW), fall*(1.0 - (1.0 - LITTER_WINTER)*bare));
}

// One texel of a plant card, as a number 0..1: the card (the uv's integer
// part), the texel on its 16 px grid, the cell and a salt.
fn card_texel(uv: vec2<f32>, seed: u32, salt: u32) -> f32 {
    let card = u32(max(floor(uv.x), 0.0));
    let tx = u32(floor(fract(uv.x)*16.0));
    let ty = u32(floor(clamp(uv.y, 0.0, 0.999)*16.0));
    return random(seed*131u + card*977u + tx*17u + ty*257u + salt*7919u);
}

// Whether a pixel of the pixel tuft is a blade (design 1): the Play demo's
// nine one-pixel blades on a 16 px card, at hashed columns 1..14 and heights
// 5..13, the odd ones stepping a pixel over above row 6. The variant is the
// uv's integer part; y counts up from the root.
fn tuft_on(uv: vec2<f32>) -> bool {
    let variant = u32(max(floor(uv.x), 0.0));
    let px = i32(floor(fract(uv.x)*16.0));
    let py = i32(floor(clamp(uv.y, 0.0, 0.999)*16.0));
    for (var b = 0u; b < 9u; b++) {
        let h = hash(variant*97u + b*13u + 0x7f4au);
        let x = 1 + i32(h % 14u);
        let tall = 5 + i32((h >> 8u) % 9u);
        let step_over = select(0, 1, py > 6 && (b & 1u) == 1u);
        if py < tall && px == x + step_over { return true; }
    }
    return false;
}

// Whether a pixel of a fern frond is cut away (design 3): a stem up the
// middle and leaflets either side in alternate rows, inside a frond's taper.
fn fern_cut(uv: vec2<f32>) -> bool {
    let u = fract(uv.x);
    let v = clamp(uv.y, 0.0, 1.0);
    let tx = floor(u*16.0);
    let ty = floor(v*16.0);
    let off = abs(tx - 7.5);
    if off < 1.0 { return false; }
    let width = 7.5*sin(3.14159*min(v*1.05, 1.0))*(1.0 - 0.3*v);
    if off > width { return true; }
    return (u32(ty + floor(off*0.5)) % 2u) == 1u;
}

// A `0xRRGGBB` sRGB colour as the linear albedo the atlas's texels are.
fn rgb8(c: u32) -> vec3<f32> {
    let srgb = vec3(f32((c >> 16u) & 0xffu), f32((c >> 8u) & 0xffu), f32(c & 0xffu))/255.0;
    return pow(srgb, vec3(2.2));
}
// The four flower colours, in `pbd_core::flora`'s bit order: red, yellow,
// white, blue.
fn flower_colour(i: u32) -> vec3<f32> {
    if i == 0u { return vec3(0.62, 0.06, 0.05); }
    if i == 1u { return vec3(0.85, 0.62, 0.06); }
    if i == 2u { return vec3(0.82, 0.82, 0.74); }
    return vec3(0.12, 0.22, 0.75);
}
const STRAW = vec3<f32>(0.50, 0.38, 0.15);
const TWIG = vec3<f32>(0.16, 0.12, 0.09);

// The plants' own colours and their season (design 1, 3, 5), over the
// albedo the tile gave: a tuft's ramp, a flower's colour, a mushroom's, the
// autumn and the bare winter of deciduous leaves, and straw in the grass.
fn flora_albedo(input: VertexOut, albedo_in: vec3<f32>, grain: f32, variation: f32) -> vec3<f32> {
    let cap = input.material & 0xffu;
    let biome = min((input.material >> 8u) & 0xffu, 7u);
    let leafy = cap == 9u;
    let grass = cap == TUFT_CODE || (input.kind == 3u && (cap == 2u || cap == 3u || cap == 7u));
    var albedo = albedo_in;
    if cap == TUFT_CODE {
        let ramp = params.tuft_ramps[biome];
        let py = u32(floor(clamp(input.uv.y, 0.0, 0.999)*16.0));
        albedo = rgb8(select(select(ramp.z, ramp.y, py < 8u), ramp.x, py < 4u))*variation;
    }
    if cap == FLOWER_CODE {
        albedo = flower_colour(min(u32(max(floor(input.uv.x), 0.0)), 3u))*(0.75 + 0.5*grain);
    }
    if cap == MUSH_STEM_CODE { albedo = vec3(0.72, 0.66, 0.52)*(0.8 + 0.4*grain); }
    if cap == FRUIT_CODE { albedo = fruit_colour(input.uv, fruit_pixel(input.uv)); }
    if cap == CAP_BROWN_CODE { albedo = vec3(0.30, 0.16, 0.07)*(0.7 + 0.6*grain); }
    if cap == CAP_RED_CODE {
        // White flecks on the red: a few texels of the cap's own grid.
        let fleck = random(input.seed*7u + u32(floor(input.uv.x*6.0)) + u32(floor(input.uv.y*6.0))*13u) < 0.2;
        albedo = select(vec3(0.58, 0.05, 0.03), vec3(0.85, 0.82, 0.74), fleck)*(0.85 + 0.3*grain);
    }
    if !(leafy || grass) { return albedo; }
    let season = season_at(normalized(input.position));
    // The tufts and blades dry with the ground under them (design 12.6).
    if grass { albedo = mix(albedo, STRAW*(0.7 + 0.6*grain)*variation, ground_season(normalized(input.position)).x*0.65); }
    if leafy && params.flora_biome[biome].w > 0.5 {
        // Autumn: a texel at a time turns gold, orange or red, more of them
        // as the season deepens; winter fades the prism's leaves toward
        // bare twigs.
        let texel = floor(fract(input.uv)*32.0);
        let h = random(input.seed*31u + u32(texel.x) + u32(texel.y)*37u);
        if h < season.x*1.1 {
            var turned = vec3(0.70, 0.40, 0.04);
            if h < season.x*0.7 { turned = vec3(0.68, 0.18, 0.03); }
            if h < season.x*0.3 { turned = vec3(0.45, 0.06, 0.03); }
            albedo = turned*(0.7 + 0.6*grain);
        }
        albedo = mix(albedo, TWIG*(0.7 + 0.6*grain), season.y*0.85);
    }
    return albedo;
}

// ---- `flora-v2` design 9: the approved M13 mockup's plants. ----

// How much of its texel's own contrast a plant keeps (the mockup's `detail`):
// leaves and bushes, then trunks, twigs and pebbles. `pbd_core::flora`'s
// LEAF_DETAIL and SOLID_DETAIL; a test holds them.
const LEAF_DETAIL: f32 = 0.45;
const SOLID_DETAIL: f32 = 0.7;
// A vine's own green and cut (design 12.4): the crown's leaf colour turned a
// little yellower and lighter, and a leafy rope rather than a twig's leaves.
// `pbd_core::tree_kit`'s VINE_TINT and VINE_MASK; a test holds them.
const VINE_TINT = vec3<f32>(1.08, 1.12, 0.62);
const VINE_MASK: u32 = 3u;
const BEACH_BIOME: u32 = 1u;
// The savanna's dry olive (`foliage.html`, `RAMPS.savannaLeaf`): linear
// luminance from 0.07 to 0.23 through three sRGB colours.
fn olive(c: vec3<f32>) -> vec3<f32> {
    let l = dot(c, vec3(0.2126, 0.7152, 0.0722));
    let t = clamp((l - 0.07)/(0.23 - 0.07), 0.0, 1.0);
    let a = rgb8(0x3e3a1au);
    let b = rgb8(0x7a7432u);
    let d = rgb8(0xb7a95au);
    return select(mix(b, d, t*2.0 - 1.0), mix(a, b, t*2.0), t < 0.5);
}
// A plant's texel pulled toward its tile's mean colour (design 9.2). A
// beach crown or bush wears the fields' leaves through the dry olive, as the
// mockup's savanna acacias do (design 9.7): the beach's own leaf tile is pale
// beach grass, which on a crown reads as frost.
fn plant_art(texel: vec3<f32>, uv: vec2<f32>, tile: vec2<f32>, code: u32, biome_in: u32) -> vec3<f32> {
    let biome = min(biome_in, 7u);
    let means = params.plant_means[biome];
    var art = texel;
    var mean = rgb8(select(select(means.z, means.y, code == 8u), means.x, code == 9u));
    if code == 9u && biome == BEACH_BIOME {
        art = olive(pixel_tile(uv, tile, tileset_slot(FIELDS_BIOME)));
        mean = olive(rgb8(params.plant_means[FIELDS_BIOME].x));
    }
    return mix(mean, art, select(SOLID_DETAIL, LEAF_DETAIL, code == 9u));
}

// Fallen leaves (design 9.4), the mockup's terrain block: the cap's 32-texel
// grid cut into slots of about two texels, two leaves a slot, each with its
// own moment in the fall; a leaf down by now is an ellipse of texels in one
// of six fallen-leaf colours, the first down browned, a lit half and a shaded
// half, more of them under a tree. Only in deciduous biomes.
const SEASON_LITTER = vec4<f32>(0.15, 0.27, 0.55, 0.68);
const LITTER_WINTER: f32 = 0.6;
fn h12(p: vec2<f32>) -> f32 { return fract(sin(dot(p, vec2(127.1, 311.7)))*43758.5453); }
fn fallen_leaves(input: VertexOut, albedo: vec3<f32>) -> vec3<f32> {
    let biome = min((input.material >> 8u) & 0xffu, 7u);
    if params.flora_biome[biome].w < 0.5 || params.flora_fall.x <= 0.0 { return albedo; }
    let radial = normalized(input.position);
    if dot(normalized(input.normal), radial) < 0.55 { return albedo; }
    let season = season_at(radial);
    if season.w <= 0.0 { return albedo; }
    let under = select(0.0, 1.0, (input.material & UNDER_TREE_BIT) != 0u);
    let gather = params.flora_fall.y;
    let reach = params.flora_fall.x*season.w*mix(1.0 - gather*0.85, 1.0, under)*(0.8 + 0.45*under*gather);
    let tp = floor(input.uv*32.0) + 0.5;
    // The cell, from its seed: a leaf pattern is the cell's own. Small
    // numbers, so the sine hash keeps its precision.
    let sh = hash(input.seed);
    let cc = vec2(f32(sh & 63u), f32((sh >> 6u) & 63u));
    let gs = max(1.6, 2.4*params.flora_fall.z);
    var top = -1.0;
    var hk = vec2(0.0);
    var side = 0.0;
    let s0 = floor(tp/gs);
    for (var j = -1; j <= 1; j++) {
        for (var i = -1; i <= 1; i++) {
            for (var n = 0; n < 2; n++) {
                let sl = s0 + vec2(f32(i), f32(j));
                let key = sl*1.37 + cc*7.31 + f32(n)*53.7;
                let when = h12(key + 31.7);
                if when >= reach || when <= top { continue; }
                let ctr = (sl + 0.15 + 0.7*vec2(h12(key + 1.3), h12(key + 2.9)))*gs;
                let an = h12(key + 4.1)*3.14159;
                let ax = vec2(cos(an), sin(an));
                let d = tp - ctr;
                let a = dot(d, ax);
                let b = dot(d, vec2(-ax.y, ax.x));
                let l = gs*0.62*(0.75 + 0.5*h12(key + 5.7));
                let w = l*0.5;
                if a*a/(l*l) + b*b/(w*w) < 1.0 { top = when; hk = key; side = b; }
            }
        }
    }
    if top < 0.0 { return albedo; }
    // Real fallen leaves: rust, ochre, oak brown, tan, a little dull red and
    // old gold; never a saturated orange.
    let hue = h12(hk + 3.1);
    var c = vec3(0.451, 0.192, 0.149);
    if hue < 0.88 { c = vec3(0.722, 0.553, 0.294); }
    if hue < 0.74 { c = vec3(0.671, 0.518, 0.396); }
    if hue < 0.60 { c = vec3(0.490, 0.275, 0.212); }
    if hue < 0.42 { c = vec3(0.659, 0.388, 0.243); }
    if hue < 0.22 { c = vec3(0.553, 0.271, 0.149); }
    // The first down have browned, and in the bare months all of them, wet.
    c = mix(c, vec3(0.353, 0.235, 0.165), smoothstep(0.0, 0.8, reach - top)*0.45);
    c = mix(c, vec3(0.29, 0.19, 0.13), season.y*0.7);
    let lc = pow(c, vec3(2.2))*select(0.86, 1.08, side > 0.0);
    return mix(lc, albedo, 0.12);
}

// A fruit's three tones, dark to light (`pbd_core::flora::FRUIT_RAMPS`, the
// mockup's `fruit_art.py` palette): apple, pear, plum, cherry, mango, papaya,
// marula, baobab, pine cone. A test holds the two tables together.
fn fruit_ramp(k: u32) -> vec3<u32> {
    switch k {
        case 0u: { return vec3(0xa61e1fu, 0xd12d41u, 0xfa3a64u); }
        case 1u: { return vec3(0xc56d25u, 0xfaaf5cu, 0xeab798u); }
        case 2u: { return vec3(0x4f1d83u, 0x7d2dbdu, 0x9e3ba2u); }
        case 3u: { return vec3(0x841c1du, 0xd12d41u, 0xfa3a64u); }
        case 4u: { return vec3(0xd4462bu, 0xfb902fu, 0xfaaf5cu); }
        case 5u: { return vec3(0xc56d25u, 0xfe6420u, 0xfb902fu); }
        case 6u: { return vec3(0xc56d25u, 0xfaaf5cu, 0xeab798u); }
        case 7u: { return vec3(0xa09f91u, 0xcfaa81u, 0xeab798u); }
        default: { return vec3(0x522b2fu, 0x7d4636u, 0xab8465u); }
    }
}
// One pixel of a fruit sprite on its 16 px card (design 9.6), as a tone:
// 0 empty, 1-3 the fruit's dark, mid and light, 4 the stem's wood. Lit from
// the top left, no outline: the silhouette's edge is the fruit's own tones.
fn fruit_pixel(uv: vec2<f32>) -> u32 {
    let k = min(u32(max(floor(uv.x), 0.0)), 8u);
    let px = floor(fract(uv.x)*16.0) + 0.5;
    let py = floor(clamp(uv.y, 0.0, 0.999)*16.0) + 0.5;
    // Centre, radius and how much taller than wide, per kind.
    var c = vec2(8.0, 7.0);
    var r = 4.0;
    var tall = 1.0;
    if k == 1u { c = vec2(8.0, 6.0); r = 4.0; tall = 1.3; }
    if k == 2u { r = 3.5; tall = 1.1; }
    if k == 3u { c = vec2(6.5, 6.0); r = 2.4; }
    if k == 4u { r = 4.0; tall = 1.25; }
    if k == 5u { c = vec2(8.0, 6.5); r = 3.5; tall = 1.6; }
    if k == 6u { r = 3.2; }
    if k == 7u { c = vec2(8.0, 6.5); r = 3.2; tall = 1.8; }
    if k == 8u { c = vec2(8.0, 7.5); r = 3.4; tall = 1.8; }
    var d = vec2(px, py) - c;
    var inside = d.x*d.x + (d.y/tall)*(d.y/tall) <= r*r;
    // A cherry hangs in a pair.
    if k == 3u && !inside {
        d = vec2(px, py) - vec2(10.0, 5.0);
        inside = dot(d, d) <= r*r;
    }
    if inside {
        if k == 8u {
            // A cone's scales: a darker row every third, offset each row.
            let row = u32(py);
            if (u32(px) + row) % 3u == 0u { return 1u; }
            return select(2u, 3u, d.x < 0.0 && d.y > 0.0);
        }
        let lit = dot(normalize(d + vec2(0.001)), vec2(-0.6, 0.8))*length(d)/r;
        if lit > 0.35 { return 3u; }
        if lit < -0.3 { return 1u; }
        return 2u;
    }
    // The stem, from the top of the fruit to the card's top: what it hangs by.
    if k == 3u {
        // Two stems that meet at the top.
        if py > 7.0 && abs(px - (6.5 + (py - 7.0)*0.25)) < 0.6 { return 4u; }
        if py > 7.0 && abs(px - (10.0 - (py - 7.0)*0.19)) < 0.6 { return 4u; }
        return 0u;
    }
    if abs(px - 8.5) < 0.6 && py > c.y + r*tall - 0.5 { return 4u; }
    return 0u;
}
fn fruit_colour(uv: vec2<f32>, tone: u32) -> vec3<f32> {
    if tone == 4u { return rgb8(0x725244u); }
    let ramp = fruit_ramp(min(u32(max(floor(uv.x), 0.0)), 8u));
    return rgb8(select(select(ramp.z, ramp.y, tone == 2u), ramp.x, tone == 1u));
}


// ---- `flora-v2` design 10: the detailed trees' pixels ----

const KIT_TILE_BASE: u32 = 640u;
// A near leaf card's mean root-to-tip shade (`mix(0.45, 1, ao)` over the
// mockup's 0.7 at the root to 1 at the tip).
const NEAR_CROWN_SHADE: f32 = 0.87;

// A texel of a tree's texture (`pbd_core::tree_kit::Tex`): an atlas tile, or
// one of the kit's own, at a uv as the mockup samples it (v counted up from
// the tile's foot), nearest; linear.
fn tree_texel(tex: u32, uv: vec2<f32>) -> vec3<f32> {
    let x = u32(floor(fract(uv.x)*32.0)) & 31u;
    let y = u32(floor(fract(1.0 - uv.y)*32.0)) & 31u;
    if (tex & 0x80000000u) != 0u {
        let sheet = (tex >> 8u) & 0xffu;
        let col = (tex >> 4u) & 0xfu;
        let row = tex & 0xfu;
        let k = max(textureDimensions(atlas).x/512u, 1u);
        let p = vec2(((sheet%4u)*128u + col*32u + x)*k, ((sheet/4u)*128u + row*32u + y)*k);
        return textureLoad(atlas, vec2<i32>(p), 0).rgb;
    }
    let w = kit[KIT_TILE_BASE + min(tex, 3u)*1024u + y*32u + x];
    return pow(unpack4x8unorm(w).rgb, vec3(2.2));
}

// The mockup's `maskOf`: a card's shape cut from its tile by a pixel mask
// (1 leaves alternating up a twig, 2 a needle spray, 4 a frond). v = 0 is
// the root edge, on the branch.
fn leaf_mask(uv: vec2<f32>, m: u32) -> bool {
    let q = (floor(clamp(uv, vec2(0.0), vec2(0.999))*32.0) + 0.5)/32.0;
    let x = q.x - 0.5;
    let y = q.y;
    let ax = abs(x);
    if m == 1u {
        if ax < 0.04 && y < 0.8 { return true; }
        for (var k = 0u; k < 5u; k++) {
            let fk = f32(k);
            let s = f32(k%2u)*2.0 - 1.0;
            var c = vec2(s*0.2*(1.0 - fk*0.1), 0.2 + fk*0.14);
            var a2 = normalized(vec3(s*0.8, 1.0, 0.0)).xy;
            if k == 4u { c = vec2(0.0, 0.8); a2 = vec2(0.0, 1.0); }
            let d = vec2(x, y) - c;
            let a = dot(d, a2);
            let b = dot(d, vec2(-a2.y, a2.x));
            let wd = 0.11*(1.0 - abs(a)/0.17*0.7);
            if abs(a) < 0.17 && abs(b) < wd { return true; }
        }
        return false;
    }
    if m == 3u {
        // A vine (design 12.4): a stem the card's whole length and three
        // broad leaves on it, side to side, so a strand of cards reads as
        // one leafy rope.
        if ax < 0.06 { return true; }
        for (var k = 0u; k < 3u; k++) {
            let s = f32(k%2u)*2.0 - 1.0;
            let d = vec2(x - s*0.18, y - (f32(k) + 0.5)/3.0)/vec2(0.2, 0.2);
            if dot(d, d) < 1.0 { return true; }
        }
        return false;
    }
    if m == 2u {
        if ax < 0.035 && y < 0.95 { return true; }
        if ax > 0.46*(1.0 - y*0.8) || y > 0.95 { return false; }
        return fract(y*10.0 - ax*1.8) >= 0.45;
    }
    let env = 0.46*pow(sin(3.14159*clamp(y, 0.0, 1.0)), 0.7);
    if ax < 0.035 && y < 0.97 { return true; }
    if ax > env { return false; }
    return fract(y*12.0 - ax*2.5) >= 0.35;
}

fn tree_header(input: VertexOut) -> u32 {
    let species = min((input.material >> SPECIES_SHIFT) & 0xfu, 7u);
    let variant = (input.material >> VARIANT_SHIFT) & 3u;
    return KIT_VARIANT_BASE + (species*TREE_VARIANTS + variant)*KIT_VARIANT_WORDS;
}
fn tree_flags(input: VertexOut) -> u32 {
    return kit[min((input.material >> SPECIES_SHIFT) & 0xfu, 7u)*KIT_SPECIES_WORDS + 3u];
}

// A baked impostor's pixel (design 10.3): the view in the uv's integer part,
// the pixel by the uv's fraction. A deciduous tree's leafy bake gives way to
// its bare one a texel at a time as winter comes (design 10.6).
fn impostor_pixel(input: VertexOut) -> u32 {
    return kit[impostor_index(input)];
}
fn impostor_index(input: VertexOut) -> u32 {
    let header = tree_header(input);
    let view = min(u32(max(floor(input.uv.x), 0.0)), IMPOSTOR_TOP_VIEW);
    let px = view*IMPOSTOR_VIEW_W + min(u32(fract(input.uv.x)*f32(IMPOSTOR_VIEW_W)), IMPOSTOR_VIEW_W - 1u);
    let py = min(u32(clamp(input.uv.y, 0.0, 0.9999)*f32(IMPOSTOR_H)), IMPOSTOR_H - 1u);
    var at = kit[header + 8u];
    if (tree_flags(input) & 1u) != 0u {
        let bare = season_at(normalized(input.position)).y;
        if random(input.seed*977u + px*131u + py*7919u) < bare { at = kit[header + 9u]; }
    }
    return at + py*IMPOSTOR_W + px;
}

// `tree-impostor-depth`: a far tree is lit by the near model's own normals
// and its root-to-tip AO, baked beside each view (`pbd_core::tree_bake::
// bake_with_relief`), so a crown has its sunlit side and its shaded side as
// the near crown does. `level` is the baked view's direction laid level (the
// vertex's normal); the relief's x runs across it, y up the tree, z toward
// the view. Mixed by `light_ease.w` with the standing normal of `flora-v2`
// 13.4 and the near crown's mean shade (0: the flat card). Returns the
// normal and the shade over the crown's matched albedo.
var<private> impostor_shade: f32 = 1.0;
fn impostor_relief(input: VertexOut, level_in: vec3<f32>) -> vec3<f32> {
    let up = normalized(input.position);
    let level = normalized(level_in - up*dot(level_in, up));
    let across = cross(up, level);
    let index = impostor_index(input);
    let word = kit[index + IMPOSTOR_RELIEF];
    let r = unpack4x8snorm(word).xyz;
    let baked = normalized(across*r.x + up*r.y + level*r.z);
    // The standing normal is `crossed-tree-cards` 5's: toward the eye
    // across the ground and a little up, one for every card and the cap, so
    // with the relief at nought no plane is lit apart from the others.
    let to_eye = params.camera.xyz - input.position;
    let flat_eye = to_eye - up*dot(to_eye, up);
    let toward = select(level, flat_eye/max(length(flat_eye), 1e-6), length(flat_eye) > 1e-3);
    let standing = normalized(toward + up*IMPOSTOR_LIFT);
    let k = clamp(params.light_ease.w, 0.0, 1.0);
    // The near pass's `mix(0.45, 1, ao)`; the crown's albedo was already met
    // to the near crown's under its MEAN of that, so a crown pixel takes its
    // own over the mean, and bark its own.
    let ao = mix(0.45, 1.0, f32(word >> 24u)/255.0);
    let crown = (kit[index] >> 24u) == 255u;
    impostor_shade = mix(1.0, select(ao, ao/NEAR_CROWN_SHADE, crown), k);
    return normalized(mix(standing, baked, k));
}

// The mockup's autumn on a leaf: the whole card (or a far tree's crown) to
// one colour between rust and orange by its id, more of them as the turn
// deepens.
fn tree_autumn(albedo: vec3<f32>, tree: f32, card: f32, turn: f32) -> vec3<f32> {
    let lum = dot(albedo, vec3(0.3, 0.55, 0.15));
    let k = h12(vec2(floor(tree*97.0), floor(card*61.0)));
    let aut = mix(vec3(0.46, 0.10, 0.025), vec3(0.66, 0.34, 0.04), k)*(0.55 + lum*2.6);
    return mix(albedo, aut, turn*smoothstep(0.1, 0.6, k + 0.35));
}

// The ground follows the seasons (design 12.6): `pbd_core::flora::
// ground_season`, the trees' calendar. A grass texel keeps its own light and
// dark; dry multiplies its luminance into the dried-grass ochre, frost lifts
// it toward a pale rime. x dry, y frost.
const GROUND_DRY = vec3<f32>(1.25, 1.0, 0.45);
const GROUND_FROST = vec3<f32>(0.8, 0.84, 0.9);
const GROUND_DRY_MIX: f32 = 0.6;
const GROUND_FROST_MIX: f32 = 0.45;
fn ground_season(radial: vec3<f32>) -> vec2<f32> {
    let s = season_at(radial);
    return vec2(max(s.z, s.y), s.y*smoothstep(SNOW_COLD.x, SNOW_COLD.y, abs(radial.y)));
}
fn ground_tint(albedo: vec3<f32>, radial: vec3<f32>) -> vec3<f32> {
    let g = ground_season(radial);
    if g.x <= 0.0 && g.y <= 0.0 { return albedo; }
    let lum = dot(albedo, vec3(0.299, 0.587, 0.114));
    let dried = mix(albedo, lum*GROUND_DRY, g.x*GROUND_DRY_MIX);
    return mix(dried, GROUND_FROST*(0.6 + 0.8*lum), g.y*GROUND_FROST_MIX);
}

// Snow on the branches (design 12.3): `pbd_core::flora::winter_cover`, the
// season's bare months times the cold of the latitude, laid on the up-facing
// side of a near tree's bark and leaves and on the top of a baked crown. A
// pine keeps its needles and carries the most; bare twigs a thin line.
const SNOW_COLD = vec2<f32>(0.574, 0.766);
const BRANCH_SNOW = vec3<f32>(0.80, 0.83, 0.88);
fn winter_cover(radial: vec3<f32>) -> f32 {
    return season_at(radial).y*smoothstep(SNOW_COLD.x, SNOW_COLD.y, abs(radial.y));
}
fn tree_snow(input: VertexOut, albedo: vec3<f32>, code: u32, deciduous: bool) -> vec3<f32> {
    let radial = normalized(input.position);
    let cover = winter_cover(radial);
    if cover <= 0.0 { return albedo; }
    var on = 0.0;
    if code == IMPOSTOR_CODE {
        // A baked view's rows run bottom up: the crown's top third; the
        // view from above is all top.
        on = smoothstep(0.55, 0.9, input.uv.y);
        if floor(input.uv.x) >= f32(IMPOSTOR_TOP_VIEW) { on = 1.0; }
    } else {
        on = smoothstep(0.15, 0.6, dot(normalize(input.normal), radial));
    }
    // In clumps a texel across, not a wash.
    let clump = 0.75 + 0.25*h12(floor(input.uv*16.0) + vec2(f32(input.seed & 255u), 0.0));
    let amount = select(0.95, 0.6, deciduous);
    // The snow keeps the scene's brightness (one art style): lit as the
    // ground's own snow is, never a flat white.
    return mix(albedo, BRANCH_SNOW, clamp(cover*on*amount*clump, 0.0, 1.0));
}

// A detailed tree's albedo: its bark or leaf texel pulled to the texture's
// mean (design 9.2's 0.7 and 0.45), the acacia's leaves in the dry olive, or
// a baked impostor's pixel; with the season laid on live, and its snow.
fn tree_albedo(input: VertexOut) -> vec3<f32> {
    let code = input.material & 0xffu;
    let flags = kit[min((input.material >> SPECIES_SHIFT) & 0xfu, 7u)*KIT_SPECIES_WORDS + 3u];
    return tree_snow(input, tree_albedo_bare(input), code, (flags & 1u) != 0u);
}
fn tree_albedo_bare(input: VertexOut) -> vec3<f32> {
    let code = input.material & 0xffu;
    let sh = min((input.material >> SPECIES_SHIFT) & 0xfu, 7u)*KIT_SPECIES_WORDS;
    let flags = kit[sh + 3u];
    let deciduous = (flags & 1u) != 0u;
    let tree = random(input.seed);
    if code == IMPOSTOR_CODE {
        let pixel = impostor_pixel(input);
        var albedo = pow(unpack4x8unorm(pixel).rgb, vec3(2.2));
        // A crown pixel (alpha 255; the bark's is 254). The mockup pulls a
        // baked crown's green pixels back toward the near crown by a factor
        // it measured; here the bake snaps to the crown's own tones as well
        // as Sweaft (`tree_bake::leaf_ramp`), and measured its crown's mean
        // (words 10..12), whose brightness is met to the near leaves' under
        // their root-to-tip AO, so near, mid and far are one colour.
        if (pixel >> 24u) == 255u {
            let header = tree_header(input);
            let baked = vec3(kit_f(header + 10u), kit_f(header + 11u), kit_f(header + 12u));
            let near = vec3(kit_f(sh + 8u), kit_f(sh + 9u), kit_f(sh + 10u))*NEAR_CROWN_SHADE;
            let weights = vec3(0.2126, 0.7152, 0.0722);
            albedo *= clamp(dot(near, weights)/max(dot(baked, weights), 1e-4), 0.5, 2.0);
            if deciduous {
                let season = season_at(normalized(input.position));
                albedo = tree_autumn(albedo, tree, 0.0, max(season.x, season.y));
            }
        }
        return albedo*impostor_shade;
    }
    if code == TREE_LEAF_CODE {
        var albedo = tree_texel(kit[sh + 1u], input.uv);
        if (flags & 2u) != 0u { albedo = olive(albedo); }
        albedo = mix(vec3(kit_f(sh + 8u), kit_f(sh + 9u), kit_f(sh + 10u)), albedo, LEAF_DETAIL);
        if input.split.z > 0.5 { albedo *= VINE_TINT; }
        // The last leaves hanging on into winter keep their autumn colour.
        let season = season_at(normalized(input.position));
        if deciduous { albedo = tree_autumn(albedo, tree, input.split.y, max(season.x, season.y)); }
        return albedo;
    }
    return mix(vec3(kit_f(sh + 4u), kit_f(sh + 5u), kit_f(sh + 6u)), tree_texel(kit[sh], input.uv), SOLID_DETAIL);
}

// Whether a pixel of a detailed tree is left out (design 10.3, 10.4, 10.6):
// an impostor's clear pixel, a card's cut-out, a deciduous card fallen for
// winter, and the rings' cross-fade, the outgoing level keeping the pixels
// whose threshold is under 1 - t and the incoming the rest.
fn tree_dropped(input: VertexOut) -> bool {
    let code = input.material & 0xffu;
    if code == IMPOSTOR_CODE && (impostor_pixel(input) >> 24u) < 128u { return true; }
    if code == TREE_LEAF_CODE {
        let sh = min((input.material >> SPECIES_SHIFT) & 0xfu, 7u)*KIT_SPECIES_WORDS;
        if !leaf_mask(input.uv, select(kit[sh + 2u], VINE_MASK, input.split.z > 0.5)) { return true; }
        if (kit[sh + 3u] & 1u) != 0u
            && h12(vec2(floor(input.split.y*255.0), f32(input.seed & 1023u))) < season_at(normalized(input.position)).y {
            return true;
        }
    }
    if (code >= TREE_BARK_CODE && code <= IMPOSTOR_CODE) || code == FRUIT_CODE {
        let f = input.height;
        let th = bayer4(input.clip.xy + vec2(1.0, 2.0)*input.split.x);
        if f < 1.5 {
            if th >= f { return true; }
        } else if th < 1.0 - (f - 2.0) {
            return true;
        }
    }
    return false;
}

// Whether a pixel of this piece is left out: a landing's dissolve, a band's
// cross-fade, a split midpoint cell's far half, a tree thinning at the edge of
// its range (`detail-fade`, `distance-lod-fade`). The main pass and the sun's
// cascades ask the same question, so a shadow is cast by what is drawn.
fn dropped(input: VertexOut) -> bool {
    let radial = normalized(input.position);
    let mask = bayer4(input.clip.xy);
    // `flora-v2`: the plants' cut-outs, and the clutter's dither at the
    // edge of its reach. The
    // reach is shortened by the most a piece stands from its cell's centre,
    // so a cell is wholly faded before the pass stops listing it.
    let code = input.material & 0xffu;
    if code == TUFT_CODE && !tuft_on(input.uv) { return true; }
    if code == FERN_CODE && fern_cut(input.uv) { return true; }
    if code == FRUIT_CODE && fruit_pixel(input.uv) == 0u { return true; }
    if input.kind == 3u || code == FRUIT_CODE {
        // At the reach's edge by time, as the trees (`flora-v2` 13.3): shown
        // inside the middle of the old thinning band, seen from each end of
        // the trees' fade, and dithered between only while that fade runs.
        let edge = params.clutter.x - 2.0 - 0.5*max(params.clutter.y, 0.0);
        let was = distance(input.position, params.tree_fade[0].xyz) < edge;
        let now = distance(input.position, params.tree_fade[1].xyz) < edge;
        let t = clamp(params.tree_fade[0].w, 0.0, 1.0);
        if was == now {
            if !now { return true; }
        } else if now == (mask >= t) {
            return true;
        }
    }
    if input.kind == 2u && tree_dropped(input) { return true; }
    // A landing's dissolve (`detail-fade`): each pixel shows the new
    // partition where its mask is under the dissolve's progress and the old
    // one elsewhere. Its own mask, interleaved gradient noise, so it does not
    // run in step with the distance cross-fade's.
    let dissolve = fract(52.9829189*fract(dot(input.clip.xy, vec2<f32>(0.06711056, 0.00583715))));
    // The old half is drawn from the old set's own records, kept for the
    // dissolve (`two-set-dissolve`), so it is always there: no pixel has to
    // be kept by the new partition for want of the old one, and no ring of
    // the landing switches at once.
    if input.part_mark != PART_BOTH {
        let new_half = dissolve < params.fade.y;
        if input.part_mark == PART_NEW && !new_half { return true; }
        if input.part_mark == PART_OLD && new_half { return true; }
    }
    // The partition in this pixel's dither class (`distance-lod-fade`): the
    // cell is drawn where its owner is in its band (the nearer owner, for a
    // split midpoint cell) and the finer band has not taken its centre. Across
    // a ring the pixels divide between the two levels by distance; outside
    // one, this is the hard partition as before. A cut wall stands where the
    // owners fall on different sides.
    let cut = input.fade_t.z < 0.0;
    let next_t = select(input.fade_t.z, -1.0 - input.fade_t.z, cut);
    if next_t > mask { return true; }
    if input.level > base_level() {
        if cut {
            if (input.fade_t.x > mask) == (input.fade_t.y > mask) { return true; }
        } else if input.kind == 2u {
            if min(input.fade_t.x, input.fade_t.y) <= mask { return true; }
        } else {
            let owner_t = select(input.fade_t.y, input.fade_t.x, dot(radial, input.split) >= 0.0);
            if owner_t <= mask { return true; }
        }
    }
    // A tree at the edge of the foliage range no longer thins out through
    // the mask by distance, which left a band of half-drawn trees standing
    // round a still camera: it is gone past the middle of the old thinning
    // band, by the time fade (`tree_level`, design 13.3).
    return false;
}

// The depth the sun's cascades are drawn with (`sun-shadows` decision 3).
// `lighting-model` L1: the sun after the air it came through. The arithmetic
// is `pbd_core::sky_light::sun_air`, and a test holds these constants to it.
const SKY_TAU = vec3<f32>(0.040, 0.100, 0.240);
const SKY_TAU_M = 0.07;
const SUN_NOW = vec3<f32>(1.12, 1.03, 0.87);
const SUN_NORM = vec3<f32>(0.880787, 0.821865, 0.699257);
fn airmass(e: f32) -> f32 {
    let h = asin(clamp(e, 0.0, 1.0));
    return 1.0/(sin(h) + 0.50572*pow(h*57.29578 + 6.07995, -1.6364));
}
fn sun_air(e: f32) -> vec3<f32> {
    let t = exp(-(SKY_TAU + SKY_TAU_M)*airmass(e));
    return t/SUN_NORM*SUN_NOW*smoothstep(-0.02, 0.01, e);
}
// L2: irradiance (over pi) from the sky's nine coefficients, for a normal
// already in the local frame (y up, the sun in the x-y plane); `deck` mixes
// the clear set toward the overcast one. `pbd_core::sky_light::sh_irradiance`.
fn sh_irradiance(n: vec3<f32>, deck: f32) -> vec3<f32> {
    let x = n.x; let y = n.y; let z = n.z;
    let b = array<f32, 9>(0.282095, 0.488603*y, 0.488603*z, 0.488603*x,
        1.092548*x*y, 1.092548*y*z, 0.315392*(3.0*z*z - 1.0), 1.092548*x*z, 0.546274*(x*x - y*y));
    var e = vec3<f32>(0.0);
    for (var k = 0u; k < 9u; k++) {
        e += mix(params.sky_sh[k].xyz, params.sky_sh[k + 9u].xyz, deck)*b[k];
    }
    return e;
}
// The sky's RADIANCE toward a local direction, out of the same nine
// coefficients: each band un-convolved (1, 3/2, 4). Soft, as an L2 sky is;
// what the wet film mirrors where nothing nearer is found.
fn sh_radiance(d: vec3<f32>, deck: f32) -> vec3<f32> {
    let x = d.x; let y = d.y; let z = d.z;
    let b = array<f32, 9>(0.282095, 1.5*0.488603*y, 1.5*0.488603*z, 1.5*0.488603*x,
        4.0*1.092548*x*y, 4.0*1.092548*y*z, 4.0*0.315392*(3.0*z*z - 1.0), 4.0*1.092548*x*z, 4.0*0.546274*(x*x - y*y));
    var e = vec3<f32>(0.0);
    for (var k = 0u; k < 9u; k++) {
        e += mix(params.sky_sh[k].xyz, params.sky_sh[k + 9u].xyz, deck)*b[k];
    }
    return max(e, vec3(0.0));
}
// A world normal in a fragment's own local frame: up its radial, x toward its
// own sun's azimuth, so the sky reads right anywhere on the curved body.
fn sky_frame(n: vec3<f32>, up: vec3<f32>, sun: vec3<f32>) -> vec3<f32> {
    var sx = sun - up*dot(sun, up);
    let l = length(sx);
    var ax = select(normalized(cross(up, vec3(0.0, 0.0, 1.0)) + vec3(1e-4, 0.0, 0.0)), sx/l, l > 1e-4);
    let az = cross(ax, up);
    return vec3(dot(n, ax), dot(n, up), dot(n, az));
}

// ---- `lighting-model` L4: the sun's bounce ----------------------------------
// The bounce in one cell of a slot (plus one, as records carry it), zero
// outside its window or off the tier.
fn bounce_at(slot: u32, layer: u32) -> vec3<f32> {
    if slot == 0u || slot > BOUNCE_BASES { return vec3(0.0); }
    let base = bounce[slot - 1u];
    if layer < base || layer - base >= BOUNCE_SPAN { return vec3(0.0); }
    let k = layer - base;
    let at = BOUNCE_BASES + (slot - 1u)*BOUNCE_WORDS + k/2u;
    if at >= arrayLength(&bounce) { return vec3(0.0); }
    let v = (bounce[at] >> ((k % 2u)*16u)) & 0xffffu;
    return vec3(f32((v >> 11u) & 31u)/31.0, f32((v >> 5u) & 63u)/63.0, f32(v & 31u)/31.0)*BOUNCE_MAX;
}
// The column holding `q`: `slot`, or the neighbour across whichever border
// plane `q` is furthest past (a step being all a point half a metre off a
// face can take). The plane test, not the nearer of two directions: both of
// those dots are one to within f32's last bit, and comparing them flipped
// from pixel to pixel as a fine speckle.
fn column_of(q: vec3<f32>, slot: u32) -> u32 {
    let rec = columns[slot - 1u];
    let c = centres[slot - 1u].xyz;
    var best = slot;
    var past = 0.0;
    for (var s = 0u; s < column_degree(rec); s++) {
        let ns = column_neighbour(rec, s);
        if ns == 0u { continue; }
        let nc = centres[ns - 1u].xyz;
        let plane = nc - c;
        let here = dot(q - (c + nc)*(0.5*length(q)), plane)/max(length(plane), 1e-12);
        if here > past { past = here; best = ns; }
    }
    return best;
}
// The column holding `q` and the one across the border nearest it, with how
// much of that second one to blend in: a half on the border, none 0.8 m in.
// What a cell's light is blended across, so its hex edge is never drawn.
// Scored against `slot` and its neighbours: a point half a metre off a face
// is never further.
fn column_pair(q: vec3<f32>, slot: u32) -> vec3<f32> {
    let rec = columns[slot - 1u];
    let c = centres[slot - 1u].xyz;
    var first = slot;
    var first_score = 0.0;
    var second = 0u;
    var second_score = -1e9;
    for (var s = 0u; s < column_degree(rec); s++) {
        let ns = column_neighbour(rec, s);
        if ns == 0u { continue; }
        let nc = centres[ns - 1u].xyz;
        let plane = nc - c;
        let score = dot(q - (c + nc)*(0.5*length(q)), plane)/max(length(plane), 1e-12);
        if score > first_score {
            second = first;
            second_score = first_score;
            first = ns;
            first_score = score;
        } else if score > second_score {
            second = ns;
            second_score = score;
        }
    }
    if second == 0u { return vec3(f32(first), 0.0, 0.0); }
    let w = 0.5*(1.0 - smoothstep(0.0, 0.8, first_score - second_score));
    return vec3(f32(first), f32(second), w);
}
// A sample's altitude in layers, snapped to the whole layer when it lies
// within `LAYER_SNAP_M` of one (`wall-foot-speckle`). A floor's sample half a
// metre up sits exactly on a whole layer, and at 4,800 m `length` steps every
// 0.49 mm, so `floor` flipped from pixel to pixel between the layer under the
// floor and the one over it. Where `column_pair` blends in a wall's column the
// two differ (the wall's rock is empty in the bake), and the flip drew dark
// speckle along every wall's foot.
const LAYER_SNAP_M: f32 = 0.01;
fn sample_layer(alt: f32) -> f32 {
    let whole = round(alt);
    return select(alt, whole, abs(alt - whole) < LAYER_SNAP_M);
}
// The bounce half a metre off a face, between the two layers it falls
// between; a layer that is rock (none) gives way to the other.
fn bounce_near(position: vec3<f32>, n: vec3<f32>, slot: u32) -> vec3<f32> {
    let q = position + n*0.5;
    let pair = column_pair(q, slot);
    let alt = sample_layer(length(q) - params.settings.x - COLUMN_BASE_M - 0.5);
    if alt < 0.0 { return vec3(0.0); }
    let l0 = u32(floor(alt));
    let f = fract(alt);
    let here = bounce_column(u32(pair.x), l0, f);
    if pair.z <= 0.0 { return here; }
    return mix(here, bounce_column(u32(pair.y), l0, f), pair.z);
}
fn bounce_column(s: u32, l0: u32, f: f32) -> vec3<f32> {
    let a = bounce_at(s, l0);
    let b = bounce_at(s, l0 + 1u);
    if dot(a, a) == 0.0 { return b; }
    if dot(b, b) == 0.0 { return a; }
    return mix(a, b, f);
}
// ---- `lighting-model` L7: the lamp bake -------------------------------------
// One cell's baked lamp light on a face at `p` turned to `n`: each lamp the
// cell sees, by N.L and the windowed inverse square (`lamp_bake::falloff`),
// plus the cell's bounce; w one where some lamp is in reach (the bake lights
// it), zero where none is, and -1 outside the cell's window.
fn lamp_cell(half: u32, slot: u32, layer: u32, p: vec3<f32>, n: vec3<f32>) -> vec4<f32> {
    let off = half*LAMP_HALF_WORDS;
    let base = lamp_bake[off + LAMP_LIST_WORDS + slot - 1u];
    if layer < base || layer - base >= BOUNCE_SPAN { return vec4(0.0, 0.0, 0.0, -1.0); }
    let cell = off + LAMP_LIST_WORDS + BOUNCE_BASES
        + ((slot - 1u)*BOUNCE_SPAN + layer - base)*LAMP_CELL_WORDS;
    let seen = lamp_bake[cell];
    let more = lamp_bake[cell + 1u];
    var direct = 0.0;
    for (var k = 0u; k < 4u; k++) {
        let i = (seen >> (8u*k)) & 0xffu;
        if i == 255u { break; }
        let at = off + i*4u;
        let lamp = vec3(bitcast<f32>(lamp_bake[at]), bitcast<f32>(lamp_bake[at + 1u]),
            bitcast<f32>(lamp_bake[at + 2u]));
        let to = lamp - p;
        let d = max(length(to), 1e-3);
        let nl = dot(n, to/d);
        if d > LAMP_REACH_M || nl <= 0.0 { continue; }
        let w = max(1.0 - pow(d/LAMP_REACH_M, 4.0), 0.0);
        direct += nl*w*w/(1.0 + (d/2.2)*(d/2.2))*LAMP_BAKE_GAIN*bitcast<f32>(lamp_bake[at + 3u]);
    }
    let v = more & 0xffffu;
    let bounce = vec3(f32((v >> 11u) & 31u)/31.0, f32((v >> 5u) & 63u)/63.0, f32(v & 31u)/31.0)
        *LAMP_BOUNCE_MAX;
    return vec4(vec3(direct) + bounce, select(0.0, 1.0, (more & 0x10000u) != 0u));
}
// The baked light half a metre off a face, between the two layers it falls
// between, or -1 in w where the bake does not hold it (the flood lights it).
fn lamp_near(half: u32, position: vec3<f32>, n: vec3<f32>, slot: u32) -> vec4<f32> {
    let q = position + n*0.5;
    let pair = column_pair(q, slot);
    let alt = sample_layer(length(q) - params.settings.x - COLUMN_BASE_M - 0.5);
    if alt < 0.0 { return vec4(0.0, 0.0, 0.0, -1.0); }
    let l0 = u32(floor(alt));
    let here = lamp_column(half, u32(pair.x), l0, fract(alt), position, n);
    if pair.z <= 0.0 || here.w < 0.0 { return here; }
    let there = lamp_column(half, u32(pair.y), l0, fract(alt), position, n);
    if there.w < 0.0 { return here; }
    return mix(here, there, pair.z);
}
fn lamp_column(half: u32, s: u32, l0: u32, f: f32, position: vec3<f32>, n: vec3<f32>) -> vec4<f32> {
    let a = lamp_cell(half, s, l0, position, n);
    let b = lamp_cell(half, s, l0 + 1u, position, n);
    if a.w < 0.0 || b.w < 0.0 { return vec4(0.0, 0.0, 0.0, -1.0); }
    // Rock holds nothing: the other layer answers.
    if solid_at(s, l0) { return b; }
    if solid_at(s, l0 + 1u) { return a; }
    return mix(a, b, f);
}

// ---- `lighting-model` L8: the reflected ray walks the columns -------------
//
// The screen cannot see what the camera cannot: an eave's underside, a
// slab's, the far side of a wall. So a wet face's reflected ray first walks
// the block grid itself, cell to cell (a DDA over hex prisms: out through
// whichever of the six side planes or the two layer spheres it meets first),
// up to 72 cells or `reflect.y` metres. Where it reaches a solid cell, that
// face is lit as the terrain is: the sky by the field's light in the air in
// front of it, the sun with its cascades and the cloud over it, the lamps by
// the field, and a lit lantern glows. Where it finds nothing it says so, and
// the film mirrors the sky. Off the tier there is nothing to walk.
fn column_neighbour(rec: ColumnRec, side: u32) -> u32 {
    let raw = column_side(rec, side);
    return select(raw + 1u, 0u, raw == NO_NEIGHBOR);
}
// A face's flat colour by its render code, for what a reflection shows at
// the size it shows it: the means of the tiles the terrain draws.
fn vox_albedo(code: u32, up: f32) -> vec3<f32> {
    if code == 1u || code == 4u { return vec3(0.61,0.48,0.25); }
    if code == 5u { return vec3(0.31,0.34,0.33); }
    if code == 6u { return vec3(0.80,0.90,0.91); }
    if code == 8u { return vec3(0.16,0.105,0.055); }
    if grassy(code) {
        if up > 0.5 { return vec3(0.12,0.32,0.075); }
        return GROUND_MEAN;
    }
    if code == 9u { return vec3(0.085,0.24,0.060); }
    return GROUND_MEAN;
}
fn reflect_trace(origin: vec3<f32>, dir: vec3<f32>, start: u32, sun: vec3<f32>,
                 cover: f32) -> vec4<f32> {
    if start == 0u || params.reflect.x < 0.5 { return vec4(0.0); }
    let radius = params.settings.x;
    var slot = start;
    var p = origin;
    var layer = light_layer(length(p) - radius);
    var walked = 0.0;
    for (var i = 0u; i < 72u; i++) {
        if layer >= LIGHT_LAYERS { return vec4(0.0); }
        let rec = columns[slot - 1u];
        // Rising (its altitude only grows from here on) and at or above the
        // highest solid layer of every column it could still enter: nothing
        // is left to hit, so the walk ends with the miss it would have ended
        // with after the rest of its steps (`lighting-perf` 2.1).
        if dot(p, dir) >= 0.0 && layer >= reach_top(rec) { return vec4(0.0); }
        let c = centres[slot - 1u].xyz;
        let degree = column_degree(rec);
        var t_exit = 1e9;
        var next = slot;
        var next_layer = layer;
        var across = vec3(0.0);
        for (var s = 0u; s < degree; s++) {
            let ns = column_neighbour(rec, s);
            if ns == 0u { continue; }
            // Through the point midway between the columns at this height:
            // through the planet's centre, the plane would hang on the radial
            // part of `plane`, which is below f32's resolution.
            let nc = centres[ns - 1u].xyz;
            let plane = nc - c;
            let mid = (c + nc)*(0.5*length(p));
            let dn = dot(dir, plane);
            if dn > 1e-9 {
                let t = -dot(p - mid, plane)/dn;
                if t < t_exit { t_exit = max(t, 0.0); next = ns; next_layer = layer; across = plane; }
            }
        }
        // The layer's two spheres.
        let b = dot(p, dir);
        let pp = dot(p, p);
        let r_hi = radius + COLUMN_BASE_M + f32(layer) + 1.0;
        let up_disc = b*b - (pp - r_hi*r_hi);
        if up_disc >= 0.0 {
            let t = -b + sqrt(up_disc);
            if t >= 0.0 && t < t_exit { t_exit = t; next = slot; next_layer = layer + 1u; across = vec3(0.0); }
        }
        let r_lo = r_hi - 1.0;
        let lo_disc = b*b - (pp - r_lo*r_lo);
        if lo_disc >= 0.0 && layer > 0u {
            let t = -b - sqrt(lo_disc);
            if t >= 0.0 && t < t_exit { t_exit = t; next = slot; next_layer = layer - 1u; across = vec3(0.0); }
        }
        if t_exit > 1e8 { return vec4(0.0); }
        walked += t_exit;
        if walked > params.reflect.y { return vec4(0.0); }
        p += dir*(t_exit + 1e-4);
        let air_slot = slot;
        let air_layer = layer;
        slot = next;
        layer = next_layer;
        if layer >= LIGHT_LAYERS { return vec4(0.0); }
        if solid_at(slot, layer) {
            let up = normalized(p);
            var hn = -normalized(across);
            if dot(across, across) < 1e-12 {
                hn = select(up, -up, layer > air_layer);
            }
            let e = dot(up, sun);
            let daylight = smoothstep(-0.13, 0.20, e);
            let field = light_at(air_slot, air_layer);
            let hrec = columns[slot - 1u];
            var col = vec3(0.0);
            if lamp_lit(hrec) && lamp_layer(hrec) == layer + 1u {
                col = vec3(1.0, 0.80, 0.46)*params.light.y;
            } else {
                let alb = vox_albedo(material_at(slot, layer), dot(hn, up));
                let amb = max(max(sh_irradiance(sky_frame(hn, up, sun), cover), vec3(0.0))*field.x,
                    vec3(0.006, 0.008, 0.011));
                var key = vec3(0.0);
                let facing = max(dot(hn, sun), 0.0)*smoothstep(-0.0145, 0.02, e);
                if facing > 0.0 {
                    key = sun_air(e)*facing*cloud_sun(p, sun)
                        *sun_shadow_soft(sun_map, sun_cmp, sun_cascades, p, hn, 0.0, facing)*field.x;
                }
                let lamps = TORCH_TINT*(lamp_strength(field.y)*TORCH_GAIN)*lamp_share(field.x, daylight);
                col = alb*(amb + key + lamps);
            }
            // Faded over the last quarter of the reach, never cut.
            return vec4(col, 1.0 - smoothstep(0.75*params.reflect.y, params.reflect.y, walked));
        }
    }
    return vec4(0.0);
}

// ---- `lighting-model` L8 (10.2): the screen trace ---------------------------
// Where the walk over the columns finds no block (a house, a tree, the sea),
// the reflected ray marches through last frame's colour and distances,
// reprojected with last frame's matrix. Read at an explicit level: a loop
// that can stop early has no screen derivatives (Direct3D's X3595, the
// owner's laptop). A step that lands within three texels of where the ray
// started is skipped (its own surface, which drew lines), and a hit needs
// the ray to have CROSSED behind the surface since the last step, not only to
// be behind it (which smeared). It fades at the screen's edge and at the end
// of the march, never cut.
fn ssr_uv(p: vec3<f32>) -> vec3<f32> {
    let clip = params.ssr_clip*vec4(p, 1.0);
    if clip.w <= 1e-4 { return vec3(-1.0); }
    return vec3(clip.x/clip.w*0.5 + 0.5, 0.5 - clip.y/clip.w*0.5, 1.0);
}
fn ssr_trace(origin: vec3<f32>, dir: vec3<f32>) -> vec4<f32> {
    if params.ssr_eye.w < 0.5 || params.reflect.w < 0.5 { return vec4(0.0); }
    let size = vec2<f32>(textureDimensions(ssr_history));
    let home = ssr_uv(origin).xy;
    var t = 0.04;
    var behind_before = -1.0;
    let reach = params.reflect.y;
    // As the sea's (`lighting-perf` 2.2): the last step lands 16.75 m out
    // (t += 0.05 + 0.12 t, 32 times); where that projects within the three
    // texels every step skips, no step can hit.
    let far = ssr_uv(origin + dir*min(16.8, reach));
    if far.z > 0.0 && length((far.xy - home)*size) < 3.0 { return vec4(0.0); }
    for (var i = 0u; i < 32u; i++) {
        t += 0.05 + t*0.12;
        if t > reach { break; }
        let p = origin + dir*t;
        let at = ssr_uv(p);
        if at.z < 0.0 || any(at.xy < vec2(0.0)) || any(at.xy > vec2(1.0)) { break; }
        if length((at.xy - home)*size) < 3.0 { continue; }
        let texel = textureLoad(ssr_history, vec2<i32>(at.xy*size), 0);
        if texel.w <= 0.0 { break; }
        let behind = distance(p, params.ssr_eye.xyz) - texel.w;
        let thickness = clamp(0.25 + 0.06*t, 0.25, 1.2);
        if behind > 0.0 && behind_before < 0.0 && behind < thickness {
            let edge = min(min(at.x, 1.0 - at.x), min(at.y, 1.0 - at.y));
            let fade = smoothstep(0.0, 0.08, edge)*(1.0 - smoothstep(0.75*reach, reach, t));
            return vec4(texel.rgb, fade);
        }
        behind_before = behind;
    }
    return vec4(0.0);
}

// ---- `lighting-perf` 2.8: L8's traces at half size ----------------------
// `reflect_gather` runs at `reflect_render_scale` of the view after the
// terrain's pass (`planet_reflect.rs`). Each texel takes the first wet pixel
// of its 2 x 2 that asked (`fragment_split`, sample 0), puts it back where
// it stands from the depth, and runs the same walk and screen trace the
// fragment ran inline. Out: the hit, and the ray and distance it was traced
// for, so the compose weighs it by both. The camera is MSAA; at one sample
// the terrain traces inline and this does not run.
struct ReflectView {
    // Body from clip, less the eye: a pixel's position relative to the eye.
    body_from_clip: mat4x4<f32>,
    // xyz the eye (body frame); w the half-size target's scale.
    eye: vec4<f32>,
    // xy the full size, zw the half size, pixels.
    size: vec4<f32>,
}
@group(3) @binding(0) var reflect_request: texture_multisampled_2d<u32>;
@group(3) @binding(1) var reflect_depth: texture_depth_multisampled_2d;
@group(3) @binding(2) var<uniform> reflect_view: ReflectView;

struct GatherOut {
    @location(0) hit: vec4<f32>,
    @location(1) ray: vec4<f32>,
}
@fragment
fn reflect_gather(@builtin(position) at: vec4<f32>) -> GatherOut {
    let full = vec2<i32>(reflect_view.size.xy);
    let base = vec2<i32>(floor(at.xy/reflect_view.eye.w));
    var offsets = array<vec2<i32>,4>(vec2(0,0), vec2(1,1), vec2(1,0), vec2(0,1));
    var px = vec2<i32>(-1);
    var req = vec4<u32>(0u);
    for (var k = 0; k < 4; k++) {
        let q = min(base + offsets[k], full - 1);
        let r = textureLoad(reflect_request, q, 0);
        if (r.y & REQUEST_FLAG) != 0u { px = q; req = r; break; }
    }
    if px.x < 0 { return GatherOut(vec4(0.0), vec4(0.0)); }
    let slot = req.x | ((req.y & 0x7fffu) << 16u);
    let dir = oct_decode(req.zw);
    let depth = textureLoad(reflect_depth, px, 0);
    let ndc = vec2((f32(px.x) + 0.5)/reflect_view.size.x*2.0 - 1.0,
                   1.0 - (f32(px.y) + 0.5)/reflect_view.size.y*2.0);
    let h = reflect_view.body_from_clip*vec4(ndc, depth, 1.0);
    let rel = h.xyz/h.w;
    let p = reflect_view.eye.xyz + rel;
    // The fragment stepped 2 cm along its normal, which is not written out.
    // A step toward the eye leaves the face whichever way it faces, and one
    // up clears a floor's layer as the normal's did: at a glancing view the
    // first alone is a millimetre up, and a floor rebuilt from the depth a
    // hair low starts the walk inside its own solid layer.
    let origin = p - normalized(rel)*0.02 + normalized(p)*0.02;
    let cover = clamp(textureSampleLevel(weather_cloud,weather_sampler,normalized(p),0.0).x,0.,1.);
    var hit = reflect_trace(origin, dir, slot, params.sun.xyz, cover);
    if hit.a <= 0.0 { hit = ssr_trace(origin, dir); }
    return GatherOut(hit, vec4(dir, length(rel)));
}

// The distance haze and the limb, the last of every terrain fragment.
// ---- `night-lights-from-space`: the night side's far lights -------------
//
// Each settlement, and each group of the player's lights past the threshold,
// is a warm cluster of points over a soft glow, added to the ground before
// its haze (`finish`), so the clouds drawn over the ground hide it. The CPU
// decides each light's strength now (the sky's night term there, gone near
// the camera where the town itself stands): `night_lights::FarLights::pack`.
var<private> far_glow: vec3<f32> = vec3<f32>(0.0);
const FAR_LIGHT_TINT: vec3<f32> = vec3<f32>(1.0, 0.70, 0.36);
// How bright a point of a light of strength one is, HDR.
const FAR_LIGHT_GAIN: f32 = 3.0;
// A point's own size, metres; a pixel's footprint widens it.
const FAR_POINT_M: f32 = 2.5;
// The share of a town's grid cells, at its centre, that hold a lit point.
const FAR_LIT_SHARE: f32 = 0.5;
// The glow under the points, as a share of a point: what a town is from so
// far off that its points are under a pixel.
const FAR_GLOW_SHARE: f32 = 0.02;
fn far_night_lights(position: vec3<f32>, foot: f32) -> vec3<f32> {
    let count = min(u32(params.night.x), 64u);
    if count == 0u { return vec3(0.0); }
    // Not under the sea: the water drawn over a harbour's quay would tint
    // its lights as the seabed.
    if length(position) < params.water_absorption.w { return vec3(0.0); }
    let radial = normalized(position);
    // A point's blob: its own size, or 1.5 pixels where that is wider. A
    // widened point is dimmer by as much, so a far town reads as sparks.
    let blob = max(FAR_POINT_M, foot*1.5);
    let dilute = FAR_POINT_M/blob;
    var sum = 0.0;
    for (var i = 0u; i < count; i++) {
        let a = params.night_lights[2u*i];
        let c = dot(radial, a.xyz);
        if c < a.w { continue; }
        let b = params.night_lights[2u*i + 1u];
        let radius = max(b.x, 1.0);
        let spacing = max(b.w, 1.0);
        let seed = u32(b.z);
        // The light's tangent plane, metres from its centre.
        let up = a.xyz;
        let axis = select(vec3(0.0, 1.0, 0.0), vec3(1.0, 0.0, 0.0), abs(up.y) > 0.9);
        let e1 = normalized(cross(axis, up));
        let e2 = cross(up, e1);
        let off = (radial - up*c)*params.settings.x;
        let uv = vec2(dot(off, e1), dot(off, e2));
        let rr = length(uv)/radius;
        // Gone at the edge of its reach, so it never ends on a ring.
        let edge = 1.0 - smoothstep(0.75, 1.0, sqrt(max(1.0 - c, 0.0)/max(1.0 - a.w, 1e-9)));
        // Up close, the points themselves: one at most in each cell of the
        // grid, by the cell's hash, likelier toward the centre and in some
        // quarters (blocks of four cells) than others, so a town is ragged
        // rather than a disc.
        var points = 0.0;
        let base = floor(uv/spacing);
        for (var dy = -1; dy <= 1; dy++) {
            for (var dx = -1; dx <= 1; dx++) {
                let cell = base + vec2(f32(dx), f32(dy));
                let ci = vec2<i32>(cell);
                let h = hash((seed*0x9e3779b9u) ^ (bitcast<u32>(ci.x)*0x85ebca6bu)
                    ^ (bitcast<u32>(ci.y)*0xc2b2ae35u));
                let q = hash((seed*0x27d4eb2fu) ^ (bitcast<u32>(ci.x >> 2u)*0x165667b1u)
                    ^ (bitcast<u32>(ci.y >> 2u)*0xd3a2646cu));
                let quarter = 0.25 + f32(q & 1023u)/1023.0;
                let cr = length((cell + 0.5)*spacing)/radius;
                if f32(h & 1023u)/1023.0 > FAR_LIT_SHARE*quarter*exp(-2.5*cr*cr) { continue; }
                let at = (cell + vec2(f32((h >> 10u) & 255u), f32((h >> 18u) & 255u))/255.0)*spacing;
                let d = length(uv - at)/blob;
                points += (0.6 + 0.8*f32((h >> 26u) & 63u)/63.0)*exp(-2.0*d*d);
            }
        }
        // From afar, where a pixel holds several cells, what they average to.
        let mean = FAR_LIT_SHARE*0.75*exp(-2.5*rr*rr)*1.5708*(blob/spacing)*(blob/spacing);
        let p = mix(points, mean, smoothstep(0.5, 1.5, blob/spacing));
        sum += b.y*edge*(p*dilute + FAR_GLOW_SHARE*exp(-4.0*rr*rr));
    }
    if sum <= 0.0 { return vec3(0.0); }
    // A deck over the light hides most of it, by the cover the weather maps
    // hold there, before the clouds pass draws the deck itself over it.
    let cover = textureSampleLevel(weather_cloud, weather_sampler, radial, 0.0).x;
    return FAR_LIGHT_TINT*(FAR_LIGHT_GAIN*sum*(1.0 - 0.97*smoothstep(0.2, 0.85, cover)));
}

fn finish(lit: vec3<f32>, radial: vec3<f32>, toward_camera: vec3<f32>,
          distance_to_camera: f32, sun_elevation: f32, daylight: f32, cover: f32,
          skylight: f32) -> vec4<f32> {
    // The far lights go under the haze like the ground they stand on.
    var color = lit + far_glow;
#ifdef TERRAIN_OFF_FOG
    return vec4(color, 1.0);
#endif
    // Tenebris-style limb and distance haze, all in the same body-local frame.
    let altitude = max(length(params.camera.xyz)-params.settings.x,0.);
    let air = exp(-altitude/1050.);
    // Both take the FIELD: a face the sky does not reach has no atmosphere
    // between it and the eye, and without this a cave wall forty metres
    // underground fogged toward the sky colour exactly as a ridge at the same
    // distance did. Tenebris gates its rim by `v_sky_light` for the same
    // reason; the haze here is the same term one step earlier.
    //
    // The air thickens under cloud (`cloud_fog_add` per unit cover) and in
    // rain (up to `rain_fog_mult`), and hazes toward the overcast sky.
    let haze_density = 0.00036*(1.+cover*params.rain[4].z)
        *mix(1.,params.rain[4].w,clamp(params.weather.y,0.,1.));
    let fog = (1.-exp(-distance_to_camera*haze_density))*air*daylight*skylight;
    color=mix(color,ground_sky(sun_elevation,cover),fog*0.55);
    // `lighting-perf` 2.8: what the haze leaves of anything added before it,
    // for the half-size reflections' compose.
    split_k = 1.-fog*0.55;
    let rim = pow(1.-clamp(dot(radial,toward_camera),0.,1.),4.);
    color+=vec3(0.07,0.16,0.25)*rim*(0.25+0.75*daylight)*(1.-air)*0.55*skylight;
    return vec4(color,1.);
}

@fragment
fn shadow_fragment(input: VertexOut) {
    if dropped(input) { discard; }
}

@fragment
fn fragment(input: VertexOut) -> @location(0) vec4<f32> {
    return shade_terrain(input);
}

// `lighting-perf` 2.8: the terrain with L8's traces left to a half-size pass
// (`planet_reflect.rs`). The colour is the miss (no hit), as `fragment` draws
// it where nothing is hit; beside it, a wet pixel's request (its column slot
// and its reflected ray) and what a hit would replace, through the haze, so
// `reflect_compose.wgsl` adds `k*fr*H*h - h*k*B` back exactly.
struct SplitOut {
    @location(0) color: vec4<f32>,
    @location(1) request: vec4<u32>,
    @location(2) replaced: vec4<f32>,
}
@fragment
fn fragment_split(input: VertexOut) -> SplitOut {
    split_on = true;
    let color = shade_terrain(input);
    return SplitOut(color, split_request, vec4(split_b*split_k, split_fr*split_k));
}

// What `fragment_split` writes, set where `shade` decides it.
var<private> split_on: bool = false;
var<private> split_request: vec4<u32> = vec4<u32>(0u);
var<private> split_fr: f32 = 0.0;
var<private> split_b: vec3<f32> = vec3<f32>(0.0);
var<private> split_k: f32 = 1.0;
// A unit vector to two `unorm16` (octahedral) and back.
fn oct_encode(v: vec3<f32>) -> vec2<u32> {
    var p = v.xy/(abs(v.x) + abs(v.y) + abs(v.z));
    if v.z < 0.0 {
        p = (1.0 - abs(p.yx))*select(vec2(-1.0), vec2(1.0), p >= vec2(0.0));
    }
    return vec2<u32>(round(clamp(p*0.5 + 0.5, vec2(0.0), vec2(1.0))*65535.0));
}
fn oct_decode(e: vec2<u32>) -> vec3<f32> {
    let p = vec2<f32>(e)/65535.0*2.0 - 1.0;
    var v = vec3(p, 1.0 - abs(p.x) - abs(p.y));
    if v.z < 0.0 {
        v = vec3((1.0 - abs(v.yx))*select(vec2(-1.0), vec2(1.0), v.xy >= vec2(0.0)), v.z);
    }
    return normalized(v);
}
const REQUEST_FLAG: u32 = 0x8000u;

// Caustics on the seabed (`ocean-environments` B5): the bright web the waves
// focus the sun into. The point is traced up along the sun to the surface, so
// the web leans with the sun as the light does, and snapped to 0.25 m to
// match the nearest-sampled textures. Two warped three-wave sums, each bright
// where it crosses zero, give a web that drifts and never tiles; `cell` is
// its spacing, m. Returns 0..about 1.
// The web's mean over the bed, what a pixel spanning many of its blocks sees.
const CAUSTIC_MEAN: f32 = 0.34;
fn caustic_web(position: vec3<f32>, radial: vec3<f32>, sun: vec3<f32>, depth: f32, t: f32, cell: f32) -> f32 {
    let sun_up = max(dot(sun, radial), 0.2);
    let surface = position + (sun - radial*dot(sun, radial))*(depth/sun_up)*0.75;
    // Body-fixed coordinates, the two axes across the radial's main one (a
    // cube face), so the web stays put as the eye moves.
    let a = abs(radial);
    var flat = surface.xz;
    if a.x >= a.y && a.x >= a.z { flat = surface.yz; } else if a.z >= a.y { flat = surface.xy; }
    var p = floor(flat*4.0)*0.25/max(cell, 0.05)*6.2832;
    var web = 0.0;
    for (var layer = 0; layer < 2; layer++) {
        let s = 1.0 + f32(layer)*0.63;
        let q = p*s + vec2(sin(p.y*0.37*s + t*0.31), cos(p.x*0.41*s - t*0.27))*1.7;
        let wave = sin(q.x + t*0.9) + sin(dot(q, vec2(-0.5, 0.866)) - t*0.7)
            + sin(dot(q, vec2(-0.5, -0.866)) + t*0.8);
        web += pow(clamp(1.0 - abs(wave)*0.5, 0.0, 1.0), 4.0);
        p = vec2(p.x*0.8 - p.y*0.6, p.x*0.6 + p.y*0.8);
    }
    return web*0.6;
}

fn shade_terrain(input: VertexOut) -> vec4<f32> {
    // A pixel's footprint on the ground, metres: taken before anything can
    // branch, since a derivative needs every pixel of its quad. The new wet
    // film fades rings under it and widens its lobe by it (specular AA).
    let foot = length(fwidth(input.position));
    if dropped(input) { discard; }
#ifdef TERRAIN_OFF_FLAT
    return vec4(0.5, 0.5, 0.5, 1.0);
#endif
    let radial = normalized(input.position);
    var n = normalized(input.normal);
    if input.kind == 2u && (input.material & 0xffu) == IMPOSTOR_CODE {
        n = impostor_relief(input, input.normal);
    }
    let sun = params.sun.xyz;
    let toward_camera = normalized(params.camera.xyz-input.position);
    let distance_to_camera = distance(input.position,params.camera.xyz);
#ifndef TERRAIN_OFF_NIGHT
    far_glow = far_night_lights(input.position, foot);
#endif
    let sun_elevation = dot(radial,sun);
    let daylight = smoothstep(-0.13,0.20,sun_elevation);
    // Overcast: the cover over the player takes the sun down by
    // `overcast_sun_dim` and the sky fill by `overcast_amb_dim` at full cover,
    // Tenebris's model. The sun loses more than the fill, so the contrast
    // between a lit face and a shaded one collapses too: a storm is flat grey
    // light, not a dim noon.
    let cover = clamp(params.weather.z,0.,1.);
    // The sun is cut by the cloud between this face and the sun, and the
    // sky's fill by the cover over this face: both off the weather maps, so a
    // storm on the far hills darkens the far hills and not the meadow under
    // a clear sky. `overcast_amb_dim` stays the fill's knob.
#ifdef TERRAIN_OFF_CLOUD
    let here_cover = 0.0;
    let sun_dim = 1.0;
#else
    let here_cover = clamp(textureSampleLevel(weather_cloud,weather_sampler,radial,0.0).x,0.,1.);
    let sun_dim = cloud_sun(input.position,sun);
#endif
    let fill_dim = 1.-here_cover*params.rain[4].y;
    // The sun ends at the horizon (`sun-shadows` decision 5), and reaches a
    // face only where its cascades see it (decision 4). A face turned from it
    // is unlit by `dot(n, sun)` already, and its shadow is not sampled.
    let sunlight = smoothstep(-0.0145,0.02,sun_elevation);
    let facing = max(dot(n,sun),0.0)*sunlight;
    var shade = 1.0;
    if facing > 0.0 {
        // `lighting-model` L3: `light.z` is the sun's apparent radius as a
        // tangent (zero: today's filter), wider under the cover over this
        // face, so an overcast day softens its shadows (`light.w`).
        let spread = params.light.z*(1.0 + params.light.w*here_cover);
#ifndef TERRAIN_OFF_SHADOW
        shade = sun_shadow_soft(sun_map, sun_cmp, sun_cascades, input.position, n, spread, dot(n,sun));
#endif
    }
    let direct = facing*sun_dim*shade;
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
    let face_altitude = length(input.position)-params.settings.x;
    // A block somebody PLACED is drawn from its own sheet, never the biome's
    // (`placed-blocks-keep-material`): dirt on desert sand is the meadow's
    // earth, not the desert's. The cap carries its record's.
    let placed_cap = (input.material & PLACED_SHEET_MASK) >> PLACED_SHEET_SHIFT;
    if input.kind==0u && placed_cap != 0u { slot = placed_cap - 1u; }
    if input.kind==1u {
        // The heightfield's own wall, which stands only where a cell has no
        // column: no layer to ask, so the depth rule, which is the
        // generator's own stack.
        let depth = max(input.height-face_altitude,0.);
        code = face_code(cap,depth);
        // Its top metre is the placed block's own side; under it, the
        // generated ground.
        if placed_cap != 0u && depth <= 1.0 { slot = placed_cap - 1u; }
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
        var layer = light_layer(face_altitude);
        if facing > 0.5 { layer = light_layer(face_altitude-0.5); }
        else if facing < -0.5 { layer = light_layer(face_altitude+0.5); }
        else { layer = min(layer, max(light_layer(input.height-0.5), 0u)); }
        let own = material_at(input.slot, layer);
        code = own;
        let placed = placed_sheet_at(input.slot, layer);
        if placed != 0u { slot = placed - 1u; }
        if abs(facing) < 0.5 {
            if grassy(own) { code = GRASS_SIDE_CODE; }
            if own == 6u { code = SNOW_SIDE_CODE; }
            if own == TILLED_CODE { code = DIRT_CODE; }
        } else if facing < -0.5 && own == TILLED_CODE {
            code = DIRT_CODE;
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
    if code==14u { base=vec3(0.36,0.25,0.16); tile=vec2(2.,3.); }
    if code==15u { base=vec3(0.30,0.31,0.33); tile=vec2(1.,3.); }
    // `flora-v2`: a fern frond wears its biome's leaves (the jungle sheet's
    // fern ground is fronds on bare soil, which a cut frond shows as brown);
    // a mushroom and a flower head the stone tile's grain under their own
    // colour.
    if code==FERN_CODE { base=vec3(0.085,0.24,0.060); tile=vec2(0.,2.); }
    if code>=MUSH_STEM_CODE && code<=FRUIT_CODE { tile=vec2(3.,0.); }
    if code==TILLED_CODE { slot = tileset_slot(FIELDS_BIOME); }
    if code==FORGE_CODE { slot = tileset_slot(MOUNTAINS_BIOME); }
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
    if code>=LAMP_IRON_CODE { tile = vec2(3.,0.); }
    if code>=DIRT_CODE && code<=SNOW_SIDE_CODE {
        tile = vec2(2.,0.);
        if code>=GRASS_SIDE_CODE { tile = vec2(1.,0.); }
        far = GROUND_MEAN;
    }
#ifdef TERRAIN_OFF_TEXTURE
    var art = base;
#else
    var art = pixel_tile(input.uv,tile,slot);
#endif
    // `flora-v2` design 10: a detailed tree's own texel, or its impostor's,
    // which needs no far fade (it is already the far picture).
    if input.kind == 2u && cap >= TREE_BARK_CODE && cap <= IMPOSTOR_CODE {
        art = tree_albedo(input);
        far = art;
    }
    // `flora-v2` design 9.2, 9.7: a plant in its tile's palette but not its
    // full contrast, and a beach crown in the dry olive.
    if (input.kind==2u || input.kind==3u) && (code==9u || code==8u || code==5u) {
        art = plant_art(art, input.uv, tile, code, (input.material >> 8u) & 0xffu);
    }
    // `shade` is one everywhere but down a grass blade, where the root sits at
    // the configured fraction of full light and eases to the tip. That gradient
    // is what makes a sward read as lush rather than as flat green paper.
    var albedo = mix(art,far,smoothstep(180.,1400.,distance_to_camera))
        *cell_variation*input.shade;
    // A lamp's parts: iron, wax, and a lantern's glass while it is out, with
    // the stone tile's grain left in so they are pixel art and not paint.
    let grain = dot(art, vec3(0.299, 0.587, 0.114));
    let up = dot(n, radial);
    if cap == BLOOM_CODE { albedo = vec3(0.55, 0.90, 0.82)*(0.75 + 0.5*grain); }
#ifndef TERRAIN_OFF_SEASON
    albedo = flora_albedo(input, albedo, grain, cell_variation);
#endif
    // Design 12.6: the ground's grass (a top, the sod lip down a column's
    // side, a terrace) dries and frosts with the season.
    if (input.kind == 0u || input.kind == 1u || input.kind == 4u)
        && (code == 2u || code == 3u || code == 7u || code == GRASS_SIDE_CODE) {
#ifndef TERRAIN_OFF_SEASON
        albedo = ground_tint(albedo, normalized(input.position));
#endif
    }
    // Design 9.4: the year's fallen leaves, in the cap's own texels.
    if input.kind == 0u && cap != 0u {
#ifndef TERRAIN_OFF_SEASON
        albedo = fallen_leaves(input, albedo);
#endif
    }
    if input.kind == KIND_LAMP {
        var part = vec3(0.16, 0.15, 0.14);
        if cap == LAMP_WAX_CODE { part = vec3(0.86, 0.80, 0.66); }
        if cap == LAMP_WOOD_CODE { part = vec3(0.36, 0.22, 0.12); }
        if cap == LAMP_STONE_CODE { part = vec3(0.44, 0.42, 0.39); }
        if cap == LAMP_GLASS_CODE && !lantern_frame(input.uv, up) { part = vec3(0.20, 0.25, 0.27); }
        albedo = part*(0.75 + 0.5*grain);
    }
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
    if input.kind==1u || input.kind==4u || input.kind==KIND_LAMP {
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
    // The overcast dims the SKY's share of the fill and leaves the floor under
    // it alone: that floor is what keeps a cave legible, and a cave is not
    // darker for a cloud over the hill it is dug into.
    var color = albedo*(fill*max(AMBIENT_FLOOR,mix(night,1.,daylight)*skylight*fill_dim)
        + vec3(1.12,1.03,0.87)*direct*skylight)*gain;
    // The light this face takes, for the seabed's tint below (`water-lit`).
    var face_light = vec3(1.0);
    if params.light.x > 0.5 {
        // `lighting-model` L1 + L2: the sun takes the colour of the air it came
        // through at THIS face's own elevation, and the shade is lit by the sky
        // that is there (its nine coefficients, mixed by the cover over this
        // face), through the field's sky light. No wall gain and no fixed
        // fill: the sky's own light says what a wall and a cap get.
        let amb = max(max(sh_irradiance(sky_frame(n, radial, sun), here_cover), vec3(0.0))*skylight,
            vec3(0.006, 0.008, 0.011));
        face_light = amb + sun_air(sun_elevation)*max(dot(n,sun),0.0)*sun_dim*shade*skylight;
        color = albedo*face_light;
        // L4: the sun thrown back off the faces round this one (baked in the
        // field), as dim as the cloud makes the sun here. The field does not
        // say where it came from, and most of the air over a floor was lit by
        // that floor: a face turned up takes a fifth, so a floor is not lit by
        // itself, while walls and undersides take it all.
#ifndef TERRAIN_OFF_BOUNCE
        if params.reflect.z > 0.0 && input.slot != 0u {
#else
        if false {
#endif
            let toward_floor = 1.0 - 0.8*max(dot(n, radial), 0.0);
            color += albedo*bounce_near(input.position, n, input.slot)
                *(params.reflect.z*sun_dim*toward_floor);
        }
    }
    // A lamp, ADDED. Warm, because everything that burns is, and over the
    // albedo so a torch lights the ground it stands on rather than painting a
    // flat orange patch over it. This is the term that makes a night worth
    // carrying a light through.
    let flood = albedo*TORCH_TINT*(lamp_strength(lamp)*TORCH_GAIN)*lamp_share(skylight, daylight);
    var torch = flood;
#ifndef TERRAIN_OFF_LAMP
    // `gpu-cuts` 4: none where a lamp's light cannot show (`lamp_share` is
    // nought in the open by day), so its lookups are skipped there.
    if params.light.x > 0.5 && input.slot != 0u && params.lamp_bake.y + params.lamp_bake.z > 0.5
        && lamp_share(skylight, daylight) > 0.0 {
#else
    if false {
#endif
        // `lighting-model` L7: the baked lamps where the bake holds this face
        // and a lamp is in reach, the flood elsewhere; the newest bake fades
        // in over the one before it (or over the flood), never a cell at a
        // time.
        let new_half = u32(params.lamp_bake.w);
        var newer = flood;
        var older = flood;
        if params.lamp_bake.z > 0.5 {
            let l = lamp_near(new_half, input.position, n, input.slot);
            if l.w > 0.0 { newer = albedo*TORCH_TINT*TORCH_GAIN*l.rgb*lamp_share(skylight, daylight); }
        }
        if params.lamp_bake.y > 0.5 && params.lamp_bake.x < 1.0 {
            let l = lamp_near(1u - new_half, input.position, n, input.slot);
            if l.w > 0.0 { older = albedo*TORCH_TINT*TORCH_GAIN*l.rgb*lamp_share(skylight, daylight); }
        }
        torch = mix(older, newer, params.lamp_bake.x);
    }
    color += torch;
    // A lamp's own light is drawn as the light it is, whatever lights the
    // cell around it. A lantern is lit panes in its iron frame; a brazier's
    // fire and a candle's flame are white-gold at the root and orange at the
    // tip. The tile's grain is left in both.
    // A glowing flower's head at night: its own cool light, whatever the
    // field around it holds.
    if lamp > LAMP_BLOOM - 0.25 {
        color = vec3(0.62, 1.0, 0.88)*(0.85 + 0.3*grain)*GLOW_HEAD_GAIN;
    } else if input.kind == KIND_LAMP && lamp > LAMP_PANES - 0.5 {
        if lamp > LAMP_FLAME - 0.5 {
            let height = clamp((lamp - LAMP_FLAME)/FLAME_RISE, 0.0, 1.0);
            // Kept under the tonemapper's shoulder, which washes anything
            // brighter toward white: a fire should read as orange, not as a
            // lit block.
            color = mix(vec3(0.98, 0.66, 0.22), vec3(0.86, 0.28, 0.05), height)*(0.80 + 0.30*grain);
        } else if lantern_frame(input.uv, up) {
            color = vec3(0.16, 0.15, 0.14)*TORCH_TINT*TORCH_GAIN*(0.75 + 0.5*grain);
        } else {
            color = vec3(1.0, 0.80, 0.46)*(0.82 + 0.35*grain);
        }
        // The new model's display transform has room above white: a lit pane
        // and a flame glow there, and bloom (`lighting-model` L6).
        if params.light.x > 0.5 && !lantern_frame(input.uv, up) { color *= params.light.y; }
    }
    // Lightning: the storm's cloud lit from inside, which lights everything
    // under open sky a cool white for the length of a flicker. Gated by the
    // sky light, so a cave does not flash with the storm over it.
    color += albedo*vec3(0.80,0.85,1.0)*(max(params.weather.w,0.)*skylight);
    // Submerged terrain: Tenebris's hex.fs absorption, the sheet's own
    // absorption and deep colour so the seabed tints the way its sea does.
    //
    // WHERE the water is, is the COLUMN's answer wherever there is a column.
    // A height field has only one: the radius, so every fragment under sea
    // level was tinted as sea and a cave carved under land below sea level
    // was a dry room full of ocean. The column says that pocket is air.
    // Capped at the sheet, which is where the sea's own surface is drawn, so
    // a column whose water reaches sea level reads exactly as it did.
    // The tide (`water_deep.w`, at the camera) lifts or lowers the sheet, and
    // the tint with it; a column's own water is the sea where it reaches sea
    // level, and takes the tide too (`ocean-environments` 5.1).
    // Water standing over the sea's level (a river, a pool, a flowing sheet)
    // is its own: the tint stops at ITS surface, the flowing level included,
    // and never at the sea's (owner, 2026-11-16: "underwater ground tint is
    // happening before the water line"). Each fragment is tested against it
    // by its own radius.
    let tided = params.water_absorption.w + params.water_deep.w;
    var water_surface = tided;
    if input.slot != 0u {
        let column_top = column_water_top_m(columns[input.slot-1u]);
        if column_top < -0.5 {
            water_surface = min(water_surface, params.settings.x + column_top);
        } else if column_top > 0.5 {
            water_surface = params.settings.x + column_top;
        }
    }
    let water_depth = max(water_surface - length(input.position), 0.);
    if water_depth > 0. {
        // The daylight left at this depth (downwelling): the deep goes dark
        // inside the column (`ocean-environments` B2).
        let attenuation = exp(-params.water_absorption.rgb*water_depth);
        // Caustics: the sun focused by the waves, a moving web on what the
        // sun reaches, as strong as the light left here and gone by
        // `lod_spare.z` (B5). On `direct`, so cloud, shade and night take it.
        if params.lod_spare.x > 0. && params.light.x > 0.5 && direct > 0. {
            let fade = 1. - smoothstep(0., max(params.lod_spare.z, 1e-3), water_depth);
            if fade > 0. {
                // Faded to its mean where a pixel spans its 0.25 m blocks
                // (`ocean-environments` B19): point-sampled past that, the
                // far bed of a river turned to per-pixel speckle.
                let web = mix(caustic_web(input.position, radial, sun, water_depth, params.settings.z, params.lod_spare.y),
                    CAUSTIC_MEAN, smoothstep(0.12, 0.35, foot));
                color += albedo*sun_air(sun_elevation)*direct*skylight*attenuation
                    *(web*params.lod_spare.x*fade);
            }
        }
        // The water's deep albedo lit as this face is (`water-lit`): the deep
        // colour is an albedo now, and bare it would be a near-black seabed
        // under the new model, as unlit it glowed under the old one.
        color = mix(color*attenuation, params.water_deep.rgb*face_light*attenuation,
            (1.-dot(attenuation,vec3(1./3.)))*0.5);
    }
    // Rain wetness, gated by whether rain REACHES this face - the column's
    // answer, not the sky light's, because light spreads sideways into a cave
    // a cell at a time and rain does not - and by being above the waterline
    // (no rings on the seabed).
    let above_water = 1.-step(0.001,water_depth);
#ifdef TERRAIN_OFF_WET
    let wet_amt = 0.0;
#else
    let wet_amt = clamp(params.weather.x,0.,1.)*input.rain_open*above_water;
#endif
    // A plant is not wet ground (`flora-v2` 13.2; owner, 2026-11-16: "grass
    // blades shouldnt have rain shader on them", "no rain drops on tree
    // leaves either"): a detailed tree and every piece of clutter only
    // darken a little as the ground soaks. No puddle, rings, sheet,
    // rivulets, mirror or glint: on a card standing up with the tree's up
    // axis for its normal those drew the sky in the shape of a tree.
    let plant = input.kind == 2u || input.kind == 3u;
    if plant && wet_amt > 0.001 {
        color *= mix(1., clamp(params.rain_near.z,0.,1.), wet_amt);
    }
    if wet_amt > 0.001 && !plant {
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
        let k_mirror = params.rain[2].z;
        let k_glint_power = params.rain[2].w;
        let k_glint_strength = params.rain[3].x;
        let k_puddle_scale = max(params.rain[3].y,0.01);
        let k_puddle_share = params.rain[3].z;
        let k_grass_rings = params.rain[3].w;
        let wet_up = radial;
        let face = dot(n,wet_up);
        let top_w = smoothstep(0.35,0.85,face);
        // A wall takes rivulets; an UNDERSIDE takes none, since nothing runs
        // down a face that points at the ground.
        let side_w = (1.-smoothstep(0.1,0.5,face))*smoothstep(-0.5,-0.1,face);
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
        // Where water STANDS: a flat face, not grass or leaves, under a noise
        // mask that spreads as the ground soaks. Everything that says "water"
        // - the sheet's waves, the rings, the full mirror - lives inside it;
        // outside it the face is only wet. A meadow in rain is dark grass, not
        // a tiled floor of identical rings.
        let floor_like = !grassy(code) && code != 9u && code != GRASS_SIDE_CODE;
        let flat = smoothstep(0.90,0.97,face);
        let soak = clamp(params.weather.x,0.,1.);
        let mask = puddle_noise(uv/k_puddle_scale)*0.65+puddle_noise(uv/(k_puddle_scale*0.37)+vec2(17.,5.))*0.35;
        // The mask is NOT uniform - two octaves of value noise bunch round a
        // half - so the threshold is its own quantile: its mean and spread,
        // measured off a transcription of these two lines, and the logistic
        // stand-in for the normal quantile. Taken as uniform, a share of 0.35
        // puddled 13.9% of a soaked floor and 0.6% of a half-soaked one.
        let wet_share = clamp(k_puddle_share*soak,1e-4,0.9999);
        let edge = PUDDLE_MASK_MEAN+PUDDLE_MASK_SPREAD*log((1.-wet_share)/wet_share)/1.702;
        let puddle = select(0.,flat*smoothstep(edge-0.03,edge+0.03,mask),floor_like);
        // Raindrop rings land on EVERY wet upward face, grass at a lighter
        // touch. They bend the mirrored sky rather than darken, so they read
        // as water; putting them only in puddles left a meadow in a storm
        // with none, and a meadow is where the game starts.
        let rings = top_w*max(puddle,select(k_grass_rings,1.,floor_like));
        // `wet-near-detail` (owner: "We don't need [rain] drops [further] out
        // than that"): the sheet, the rings and the rivulets only within the
        // reach, fading out over the band before it; past it the work is
        // skipped, not only weighed out. Today's model draws them everywhere.
        var near_detail = 1.;
        if params.light.x > 0.5 {
            let reach = max(params.rain_near.x,0.);
            near_detail = 1.-smoothstep(reach-max(params.rain_near.y,0.),reach,distance_to_camera);
        }
        var pg = vec2(0.);
        if rings > 0.001 && near_detail > 0.001 {
            // A continuous rippled sheet so the whole wet face reads as a
            // normal map, then raindrop impact rings on top of it.
            pg += vec2(
                cos(uv.x*k_wave_scale+wet_t*k_wave_speed)
                    +0.7*cos((uv.x+uv.y)*k_wave_scale*0.6-wet_t*k_wave_speed*0.8),
                cos(uv.y*k_wave_scale-wet_t*k_wave_speed*0.9)
                    +0.7*cos((uv.x-uv.y)*k_wave_scale*0.6+wet_t*k_wave_speed*0.7)
            )*(k_wave_strength*puddle);
            if params.wet.w > 0.5 {
                // The new model: as many drops as it is raining, faded once a
                // ring falls under a pixel, or the rings turn to speckle.
                pg += rain_rings(uv*k_ripple_scale,wet_t,clamp(params.weather.y,0.,1.))
                    *(k_ripple_strength*rings);
                pg /= 1.+pow(foot*k_ripple_scale*3.,2.);
            } else {
                pg += rain_ripple_grad(uv*k_ripple_scale,wet_t)*(k_ripple_strength*rings);
            }
            pg *= near_detail;
        }
        var pert = ax_a*pg.x+ax_b*pg.y;
        if side_w > 0.001 && near_detail > 0.001 {
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
            pert += (across_dir*dn.x+down_dir*dn.y)*(k_flow_strength*side_w*near_detail);
        }
        let wet = wet_amt*max(top_w,side_w);
        let wet_n = normalized(n+pert);
        // Wet darkens AND saturates: water fills the grain, so the colour
        // deepens toward its own hue rather than toward grey. The new film
        // darkens a little more, and a puddle more again.
        if params.light.x > 0.5 {
            color *= mix(1.,k_wet_darken*0.85,wet)*mix(1.,0.7,puddle*wet_amt);
        } else {
            color *= mix(1.,k_wet_darken,wet);
        }
        let grey = dot(color,vec3(0.2126,0.7152,0.0722));
        color = max(mix(vec3(grey),color,1.+0.25*wet),vec3(0.));
        // The sky, mirrored. Fresnel against the view along the RIPPLED
        // normal, so a ring is the reflection bending - brighter on one side,
        // darker on the other - where the old sheen could only subtract sky
        // light and printed every ring as a dark line. A reflected ray that
        // dips under the horizon sees the ground, not the sky. Full weight in
        // a puddle, a quarter on a merely wet face.
        let cos_v = clamp(dot(wet_n,toward_camera),0.,1.);
        let fresnel = 0.02+0.98*pow(1.-cos_v,5.);
        let r = reflect(-toward_camera,wet_n);
        if params.light.x > 0.5 {
            // `lighting-model`, wet ground (owner: "kinda dull"): a glossy
            // film. GGX at the wet or the puddle roughness, widened where the
            // rippled normal turns fast across a pixel (specular AA), that
            // mirrors the real sky (L2) and takes highlights from the sun and
            // from every lamp near the eye. The masks above are today's.
            let rough0 = mix(params.wet.x,params.wet.y,puddle);
            let turn = dot(pg,pg)*min(foot*k_ripple_scale*2.,1.);
            let rough = clamp(sqrt(rough0*rough0+2.*turn),rough0,0.6);
            let a = rough*rough;
            let r_up = dot(r,wet_up);
            // The sky's nine coefficients are soft and dim at the horizon,
            // which is where a puddle seen from eye height looks: never
            // darker than the sky the haze takes (`ground_sky`), the dome's
            // own colour there.
            var env = max(sh_radiance(sky_frame(normalized(r-wet_up*min(r_up,0.)),wet_up,sun),here_cover),
                ground_sky(sun_elevation,cover)*mix(0.12,1.,daylight));
            if r_up < 0. { env *= 0.35; }
            env *= mix(0.4,1.,skylight);
            // L8: what the reflected ray really reaches, where it finds it.
            let fr = fresnel*(1.-rough)*k_mirror*mix(0.45,1.,puddle)*wet;
            var hit = vec4(0.);
            let request = fr > 0.03 && split_on;
            if request {
                // Traced at half size (`lighting-perf` 2.8): this pixel draws
                // the miss and asks.
                let ray = oct_encode(r);
                split_request = vec4(input.slot & 0xffffu,
                    ((input.slot >> 16u) & 0x7fffu) | REQUEST_FLAG, ray.x, ray.y);
            } else if fr > 0.03 {
                hit = reflect_trace(input.position+n*0.02, r, input.slot, sun, here_cover);
                // Voxels first (owner); the screen where they find nothing.
                if hit.a <= 0.0 { hit = ssr_trace(input.position+n*0.02, r); }
            }
            let env_miss = env;
            env = mix(env,hit.rgb,hit.a);
            color = color*(1.-fr)+env*fr;
            // The key light's glint, wider under cover so a storm keeps a soft
            // sheen; hidden where the reflection finds a surface in front of it.
            let a_key = max(a,0.02+0.25*here_cover);
            let key = sun_air(sun_elevation)*sun_dim*sunlight;
            let glint = min(spec_light(wet_n,toward_camera,sun,key*3.14159,a_key),vec3(3.))
                *shade*wet;
            color += glint*(1.-hit.a);
            // Each lamp's own highlight, where its light reaches this face (the
            // field's block channel says so: none through a wall).
            var ls = vec3(0.);
            for (var i = 0u; i < 8u; i++) {
                let lamp_at = params.lamps[i];
                if lamp_at.w <= 0. { continue; }
                let q = lamp_at.xyz-input.position;
                let d = length(q);
                if d < 16. {
                    ls += spec_light(wet_n,toward_camera,q/d,
                        TORCH_TINT*TORCH_GAIN*lamp_at.w*11.6/(4.84+d*d)*(1.-d/16.),max(a,0.02));
                }
            }
            let lamp_glints = min(ls,vec3(6.))*params.wet.z*wet*lamp_share(skylight,daylight)
                *clamp(lamp*3.,0.,1.);
            color += lamp_glints*(1.-0.8*hit.a);
            if request {
                split_fr = fr;
                split_b = fr*env_miss + glint + 0.8*lamp_glints;
            }
            return finish(color, radial, toward_camera, distance_to_camera, sun_elevation, daylight, cover, skylight);
        }
        let r_up = dot(r,wet_up);
        let sky_seen = ground_sky(sun_elevation,cover)*mix(0.12,1.,daylight)
            *mix(0.55,1.,smoothstep(-0.05,0.35,r_up));
        let mirror = fresnel*k_mirror*mix(0.25,1.,puddle)*wet_amt*top_w;
        color = mix(color,sky_seen,clamp(mirror,0.,1.));
        // The sun glint goes where the sun goes: under full cover it is
        // `overcast_sun_dim` of itself, so a storm has none and a passing
        // shower sparkles.
        let glint = pow(max(dot(wet_n,sun),0.),max(k_glint_power,1.));
        color += vec3(k_glint_strength)*(glint*wet*sunlight*sun_dim*shade);
    }
    return finish(color, radial, toward_camera, distance_to_camera, sun_elevation, daylight, cover, skylight);
}
