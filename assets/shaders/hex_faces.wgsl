// Pale Blue Dot: bounded GPU face extraction. Original implementation informed
// by swarm-demo's persistent compute->draw buffers; provenance in docs/shader-port.md.
// Dispatch reset(1), emit(ceil(owned_count/64)), finalize(1), IN THAT ORDER.
// Each dispatch is a separate pipeline entry point. No global barrier within emit.
struct Cell {
    column: u32, material: u32, light: u32, flags: u32,
    radii: vec2<f32>, padding: vec2<u32>,
}
struct Column { corners: array<vec4<f32>, 6>, center_degree: vec4<f32> }
struct Neighbors { ids: array<u32, 8> }
struct Face { cell: u32, face: u32, material: u32, light: u32 }
struct Params { owned_count: u32, capacity: u32, padding0: u32, padding1: u32 }
struct Counts { requested: atomic<u32>, overflow: atomic<u32>, padding0: u32, padding1: u32 }
struct DrawIndirect { vertex_count: u32, instance_count: u32, first_vertex: u32, first_instance: u32 }
@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<storage, read> cells: array<Cell>;
@group(0) @binding(2) var<storage, read> neighbors: array<Neighbors>;
@group(0) @binding(3) var<storage, read> columns: array<Column>;
@group(0) @binding(4) var<storage, read_write> faces: array<Face>;
@group(0) @binding(5) var<storage, read_write> counts: Counts;
@group(0) @binding(6) var<storage, read_write> draw: DrawIndirect;
@group(0) @binding(7) var<storage, read> baked_light: array<u32>;

// flags bit 0: covers a neighboring opaque face; bit 1: owned drawable cell.
// Halo cells are readable but never emitted. Neighbor 0xffffffff is UNKNOWN,
// hence closed; 0xfffffffe is explicitly known open sky (not an unloaded chunk).
const UNKNOWN: u32 = 0xffffffffu;
const OPEN_SKY: u32 = 0xfffffffeu;

@compute @workgroup_size(1)
fn reset() {
    atomicStore(&counts.requested, 0u);
    atomicStore(&counts.overflow, 0u);
    draw = DrawIndirect(18u, 0u, 0u, 0u);
}

@compute @workgroup_size(64)
fn emit(@builtin(global_invocation_id) gid: vec3<u32>) {
    let id = gid.x;
    // Prevent atomic u32 wrapping even if invalid host parameters arrive.
    let owned = min(params.owned_count, 0x1fffffffu);
    if (id >= owned || id >= arrayLength(&cells) || id >= arrayLength(&neighbors)) { return; }
    let cell = cells[id];
    if ((cell.flags & 2u) == 0u || cell.material == 0u || cell.column >= arrayLength(&columns)) { return; }
    let degree = u32(clamp(columns[cell.column].center_degree.w, 5.0, 6.0));
    for (var side = 0u; side < degree + 2u; side++) {
        let neighbor = neighbors[id].ids[side];
        if (neighbor == UNKNOWN) { continue; }
        var exposed_light = 0xf000u;
        if (neighbor != OPEN_SKY) {
            if (neighbor >= arrayLength(&cells) || neighbor >= arrayLength(&baked_light)) { continue; }
            if ((cells[neighbor].flags & 1u) != 0u) { continue; }
            exposed_light = baked_light[neighbor];
        }
        let slot = atomicAdd(&counts.requested, 1u);
        let capacity = min(params.capacity, arrayLength(&faces));
        if (slot < capacity) {
            faces[slot] = Face(id, side, cell.material, exposed_light);
        } else {
            atomicStore(&counts.overflow, 1u);
        }
    }
}

@compute @workgroup_size(1)
fn finalize() {
    let count = min(atomicLoad(&counts.requested), min(params.capacity, arrayLength(&faces)));
    // Never publish a partial replacement. Renderer must retain the old chunk
    // allocation until this job succeeds (that publication logic is not here).
    draw = DrawIndirect(18u, select(count, 0u, atomicLoad(&counts.overflow) != 0u), 0u, 0u);
}
