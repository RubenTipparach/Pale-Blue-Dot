//! The world map's WGSL, run (`world-map` task 3.4): `map_live.wgsl`'s
//! `unproject` and `daylight` on a GPU adapter against `pbd_core::geo` and
//! `Clock::daylight`, the Rust they transcribe. The map's live layers are
//! looked up at the direction `unproject` gives, so a shader that projected
//! even slightly differently would put the night and the clouds in the wrong
//! place over a base the CPU drew. Skipped, loudly, where there is no adapter.

use bevy::math::{Vec2, Vec3};
use pbd_core::daylight::Clock;
use pbd_core::geo;
use wgpu::util::DeviceExt;

fn device() -> Option<(wgpu::Device, wgpu::Queue)> {
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: wgpu::Backends::VULKAN,
        ..Default::default()
    });
    let adapter = bevy::tasks::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::LowPower,
        force_fallback_adapter: false,
        compatible_surface: None,
    }))
    .ok()?;
    bevy::tasks::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default())).ok()
}

/// A shader's shared part: what stands between its `<name>:shared` markers.
fn shared_source(shader: &str, name: &str) -> String {
    let begin = shader
        .find(&format!("// {name}:shared begin"))
        .expect("the shader marks its shared part");
    let end = shader
        .find(&format!("// {name}:shared end"))
        .expect("the shader ends its shared part");
    shader[begin..end].to_string()
}

/// Each probe (a map position and a sun direction) through the shipped WGSL:
/// the direction `unproject` gives, and the daylight there.
fn run(probes: &[(Vec2, Vec3)]) -> Option<Vec<[f32; 4]>> {
    let shader = include_str!("../../../assets/shaders/map_live.wgsl");
    let mut input: Vec<[f32; 4]> = Vec::new();
    for (uv, sun) in probes {
        input.push([uv.x, uv.y, 0.0, 0.0]);
        input.push(sun.extend(0.0).to_array());
    }
    compute(
        &shared_source(shader, "map_live"),
        r#"
@group(0) @binding(0) var<storage, read> probes: array<vec4<f32>>;
@group(0) @binding(1) var<storage, read_write> out: array<vec4<f32>>;
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if (i >= arrayLength(&out)) { return; }
    let probe = probes[2u * i];
    let sun = probes[2u * i + 1u].xyz;
    let up = unproject(probe.xy);
    out[i] = vec4<f32>(up, daylight(sun, up));
}
"#,
        &input,
        probes.len(),
    )
}

/// Run `main` over `inputs` with the shared WGSL before it, and read back
/// `outputs` vec4s. `None` where there is no adapter.
fn compute(shared: &str, main: &str, inputs: &[[f32; 4]], outputs: usize) -> Option<Vec<[f32; 4]>> {
    let (device, queue) = device()?;
    let source = format!("{shared}\n{main}");
    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("map parity"),
        source: wgpu::ShaderSource::Wgsl(source.into()),
    });
    let bytes: Vec<u8> = inputs
        .iter()
        .flatten()
        .flat_map(|f| f.to_le_bytes())
        .collect();
    let probe_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: None,
        contents: &bytes,
        usage: wgpu::BufferUsages::STORAGE,
    });
    let size = (outputs * 16) as u64;
    let out = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    let read = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: None,
        layout: None,
        module: &module,
        entry_point: Some("main"),
        compilation_options: Default::default(),
        cache: None,
    });
    let bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &pipeline.get_bind_group_layout(0),
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: probe_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: out.as_entire_binding(),
            },
        ],
    });
    let mut encoder = device.create_command_encoder(&Default::default());
    {
        let mut pass = encoder.begin_compute_pass(&Default::default());
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &bind, &[]);
        pass.dispatch_workgroups((outputs as u32).div_ceil(64), 1, 1);
    }
    encoder.copy_buffer_to_buffer(&out, 0, &read, 0, size);
    queue.submit([encoder.finish()]);
    let slice = read.slice(..);
    slice.map_async(wgpu::MapMode::Read, |_| {});
    device
        .poll(wgpu::PollType::wait_indefinitely())
        .expect("the GPU finishes");
    let data = slice.get_mapped_range();
    Some(
        data.chunks_exact(16)
            .map(|b| {
                std::array::from_fn(|k| {
                    f32::from_le_bytes(b[4 * k..4 * k + 4].try_into().expect("four bytes"))
                })
            })
            .collect(),
    )
}

/// The shader turns map positions into the directions `geo` does, the poles,
/// the antimeridian and a wrapped `u` included, and shades the night where
/// `Clock::daylight` says it is night.
#[test]
fn the_map_shader_projects_as_geo_does() {
    let mut probes = Vec::new();
    let clocks: Vec<Clock> = [3.0, 9.0, 12.0, 18.5, 23.0]
        .iter()
        .map(|&h| Clock::at_hour(h))
        .collect();
    for (k, clock) in clocks.iter().enumerate() {
        for i in 0..60 {
            let u = (i as f32 * 0.618_034 + k as f32 * 0.1).fract() * 1.2 - 0.1;
            let v = (i as f32 * 0.377 + 0.05).fract();
            probes.push((Vec2::new(u, v), clock.sun(), *clock));
        }
        for uv in [
            Vec2::new(0.0, 0.5),
            Vec2::new(1.0, 0.5),
            Vec2::new(0.25, 0.0),
            Vec2::new(0.75, 1.0),
        ] {
            probes.push((uv, clock.sun(), *clock));
        }
    }
    let pairs: Vec<(Vec2, Vec3)> = probes.iter().map(|(uv, sun, _)| (*uv, *sun)).collect();
    let Some(out) = run(&pairs) else {
        eprintln!("SKIPPED: no Vulkan adapter, the map's WGSL was not run");
        return;
    };
    let mut failures = Vec::new();
    for ((uv, _, clock), gpu) in probes.iter().zip(&out) {
        let cpu = geo::unproject(*uv);
        let up = Vec3::new(gpu[0], gpu[1], gpu[2]);
        if cpu.distance(up) > 1e-4 {
            failures.push(format!("at {uv}: geo {cpu}, shader {up}"));
        }
        let day = clock.daylight(cpu);
        if (day - gpu[3]).abs() > 1e-3 {
            failures.push(format!(
                "daylight at {uv}, hour {}: Clock {day}, shader {}",
                clock.hour(),
                gpu[3]
            ));
        }
    }
    eprintln!("compared {} points on the GPU", probes.len());
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// The mockup's greying, in its own arithmetic on sRGB bytes: a canvas
/// "saturation" blend with grey leaves the luminosity 0.3 R + 0.59 G + 0.11 B
/// in every channel, and then 45% of (10, 16, 20) is laid over it
/// (`docs/mockups/world-map.html`, `draw()`).
fn mockup_grey(srgb: [u8; 3]) -> [f32; 3] {
    let [r, g, b] = srgb.map(f32::from);
    let luminosity = 0.3 * r + 0.59 * g + 0.11 * b;
    [10.0, 16.0, 20.0].map(|under| luminosity * 0.55 + under * 0.45)
}

fn linear(srgb: u8) -> f32 {
    let c = f32::from(srgb) / 255.0;
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

fn srgb(linear: f32) -> f32 {
    let c = if linear <= 0.003_130_8 {
        linear * 12.92
    } else {
        1.055 * linear.powf(1.0 / 2.4) - 0.055
    };
    c * 255.0
}

/// The map's base greys out under an overlay as the approved mockup greys it
/// (`world-map` decision 11): `map_image.wgsl`'s `greyed`, run on a GPU
/// adapter over the base's own kinds of colour, lands within a byte of the
/// mockup's canvas arithmetic. The deep sea (35, 90, 150) goes to the
/// mockup's charcoal (49, 51, 53).
#[test]
fn the_map_greys_as_the_mockup_does() {
    let shader = include_str!("../../../assets/shaders/map_image.wgsl");
    let colours: Vec<[u8; 3]> = vec![
        [35, 90, 150],
        [120, 190, 200],
        [150, 190, 80],
        [222, 150, 60],
        [30, 120, 60],
        [232, 214, 150],
        [225, 235, 245],
        [0, 0, 0],
        [255, 255, 255],
        [140, 130, 125],
    ];
    let inputs: Vec<[f32; 4]> = colours
        .iter()
        .map(|c| [linear(c[0]), linear(c[1]), linear(c[2]), 1.0])
        .collect();
    let Some(out) = compute(
        &shared_source(shader, "map_image"),
        r#"
@group(0) @binding(0) var<storage, read> probes: array<vec4<f32>>;
@group(0) @binding(1) var<storage, read_write> out: array<vec4<f32>>;
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if (i >= arrayLength(&out)) { return; }
    out[i] = vec4<f32>(greyed(probes[i].rgb), 1.0);
}
"#,
        &inputs,
        inputs.len(),
    ) else {
        eprintln!("SKIPPED: no Vulkan adapter, the map's greying WGSL was not run");
        return;
    };
    let mut failures = Vec::new();
    for (colour, gpu) in colours.iter().zip(&out) {
        let want = mockup_grey(*colour);
        let got = [srgb(gpu[0]), srgb(gpu[1]), srgb(gpu[2])];
        if want.iter().zip(&got).any(|(w, g)| (w - g).abs() > 1.0) {
            failures.push(format!("{colour:?}: mockup {want:?}, shader {got:?}"));
        }
    }
    let sea = mockup_grey([35, 90, 150]).map(f32::round);
    assert_eq!(sea, [49.0, 51.0, 53.0], "the deep sea's charcoal");
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
