// A town's pieces drawn into a sun cascade, depth only (`sun-shadows`
// decision 3): each town's triangles in the planet's frame, one static buffer
// a town, moved into the cascade's box by its matrix.

struct Caster {
    clip_from_body: mat4x4<f32>,
}

@group(0) @binding(0) var<uniform> caster: Caster;

@vertex
fn vertex(@location(0) position: vec3<f32>) -> @builtin(position) vec4<f32> {
    return caster.clip_from_body * vec4<f32>(position, 1.0);
}
