// GPU Jacobi form of Tenebris WorldLight's seed-and-propagate rule, expanded
// to RGB + sky nibbles. This is an optional render cache, not authoritative data.
// Reset BOTH ping-pong buffers from seeds after removals, then dispatch relax
// with distinct previous/next buffers. Never bind one buffer to both bindings.
struct LightCell { emission: u32, flags: u32, padding0: u32, padding1: u32 }
struct Neighbors { ids: array<u32, 8> }
struct Params { count: u32, padding0: u32, padding1: u32, padding2: u32 }
@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<storage, read> cells: array<LightCell>;
@group(0) @binding(2) var<storage, read> neighbors: array<Neighbors>;
@group(0) @binding(3) var<storage, read> previous: array<u32>;
@group(0) @binding(4) var<storage, read_write> next: array<u32>;

fn unpack(packed: u32) -> vec4<u32> {
    return vec4<u32>(packed & 15u, (packed>>4u)&15u, (packed>>8u)&15u, (packed>>12u)&15u);
}
fn pack(light: vec4<u32>) -> u32 { return light.x | (light.y<<4u) | (light.z<<8u) | (light.w<<12u); }
fn seed(id: u32) -> vec4<u32> {
    let c = cells[id];
    var value = unpack(c.emission & 0xfffu);
    // bit 0 opaque; bit 1 direct, unobstructed sky; bit 2 frozen halo.
    if ((c.flags & 3u) == 2u) { value.w = 15u; }
    return value;
}
@compute @workgroup_size(64)
fn initialize(@builtin(global_invocation_id) gid: vec3<u32>) {
    let id = gid.x;
    if (id >= params.count || id >= arrayLength(&cells) || id >= arrayLength(&next)) { return; }
    // Caller supplies the frozen halo (GPU exchange or initial CPU boundary
    // seeds); it stays in place in both buffers during this region's solve.
    if ((cells[id].flags & 4u) != 0u) { return; }
    next[id] = pack(seed(id));
}
@compute @workgroup_size(64)
fn relax(@builtin(global_invocation_id) gid: vec3<u32>) {
    let id = gid.x;
    if (id >= params.count || id >= arrayLength(&cells) || id >= arrayLength(&neighbors) || id >= arrayLength(&next)) { return; }
    if ((cells[id].flags & 4u) != 0u) { return; }
    var value = seed(id);
    if ((cells[id].flags & 1u) == 0u) {
        for (var side = 0u; side < 8u; side++) {
            let neighbor = neighbors[id].ids[side];
            if (neighbor < arrayLength(&previous)) {
                // max before subtract avoids unsigned underflow at dark cells.
                value = max(value, max(unpack(previous[neighbor]), vec4<u32>(1u)) - vec4<u32>(1u));
            }
        }
    }
    next[id] = pack(value);
}
