// The weather maps the shaders read, mixed between the two published states
// of the atmosphere the view is running between (`smooth-weather` decision
// 4). The simulation publishes a state a second; uploading each straight into
// the maps made every cloud edge tick once a second and jump when a held step
// caught up. Instead the two states of the pair are uploaded once each and
// this pass writes their mix into the cube maps every frame, so every shader
// that samples them (the clouds, the sea, the ground's cloud shadows) sees
// the weather move continuously, and none of them changes.
//
// `planet_weather_gpu_tests` runs this on an adapter against the CPU's mix.

struct Blend {
    // x: how far from `from` to `to`, 0..1.
    t: vec4<f32>,
};

@group(0) @binding(0) var<uniform> blend: Blend;
@group(0) @binding(1) var cloud_from: texture_2d_array<f32>;
@group(0) @binding(2) var cloud_to: texture_2d_array<f32>;
@group(0) @binding(3) var cloud_out: texture_storage_2d_array<rgba16float, write>;
@group(0) @binding(4) var wind_from: texture_2d_array<f32>;
@group(0) @binding(5) var wind_to: texture_2d_array<f32>;
@group(0) @binding(6) var wind_out: texture_storage_2d_array<rgba16float, write>;

@compute @workgroup_size(8, 8, 1)
fn blend_maps(@builtin(global_invocation_id) id: vec3<u32>) {
    let size = textureDimensions(cloud_out);
    if (id.x >= size.x || id.y >= size.y || id.z >= textureNumLayers(cloud_out)) {
        return;
    }
    let at = vec2<i32>(id.xy);
    let layer = i32(id.z);
    let t = clamp(blend.t.x, 0.0, 1.0);
    textureStore(
        cloud_out,
        at,
        layer,
        mix(textureLoad(cloud_from, at, layer, 0), textureLoad(cloud_to, at, layer, 0), t),
    );
    textureStore(
        wind_out,
        at,
        layer,
        mix(textureLoad(wind_from, at, layer, 0), textureLoad(wind_to, at, layer, 0), t),
    );
}
