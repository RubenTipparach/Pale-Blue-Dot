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

/// The shader's shared part: what stands between its markers.
fn shared_source() -> String {
    let shader = include_str!("../../../assets/shaders/map_live.wgsl");
    let begin = shader
        .find("// map_live:shared begin")
        .expect("map_live.wgsl marks its shared part");
    let end = shader
        .find("// map_live:shared end")
        .expect("map_live.wgsl ends its shared part");
    shader[begin..end].to_string()
}

/// Each probe (a map position and a sun direction) through the shipped WGSL:
/// the direction `unproject` gives, and the daylight there.
fn run(probes: &[(Vec2, Vec3)]) -> Option<Vec<[f32; 4]>> {
    let (device, queue) = device()?;
    let source = format!(
        "{}\n{}",
        shared_source(),
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
"#
    );
    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("map parity"),
        source: wgpu::ShaderSource::Wgsl(source.into()),
    });
    let mut input: Vec<f32> = Vec::new();
    for (uv, sun) in probes {
        input.extend_from_slice(&[uv.x, uv.y, 0.0, 0.0]);
        input.extend_from_slice(&sun.extend(0.0).to_array());
    }
    let bytes: Vec<u8> = input.iter().flat_map(|f| f.to_le_bytes()).collect();
    let probe_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: None,
        contents: &bytes,
        usage: wgpu::BufferUsages::STORAGE,
    });
    let size = (probes.len() * 16) as u64;
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
        pass.dispatch_workgroups((probes.len() as u32).div_ceil(64), 1, 1);
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
