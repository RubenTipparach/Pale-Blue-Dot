//! The sea's WGSL, run: `sea.wgsl`'s `sea_surface` on a GPU adapter against
//! `pbd_core::sea::LocalSea`, the Rust it transcribes. A parser can say the
//! shader is valid; only running it can say it computes the same sea the hulls
//! float on. Skipped, loudly, where there is no adapter at all.

use bevy::math::Vec3;
use pbd_core::sea::{SeaSettings, SeaTable};
use wgpu::util::DeviceExt;

const RADIUS: f32 = 4799.5;

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

/// Every point, run through the shipped WGSL, heights out.
fn run(sea: &pbd_core::sea::SeaGpu, probes: &[(Vec3, Vec3, f32)]) -> Option<Vec<f32>> {
    let (device, queue) = device()?;
    let source = format!(
        "{}\n{}",
        crate::shader_tests::sea_source(),
        r#"
@group(0) @binding(0) var<uniform> sea: SeaView;
@group(0) @binding(1) var<storage, read> probes: array<vec4<f32>>;
@group(0) @binding(2) var<storage, read_write> heights: array<f32>;
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if (i >= arrayLength(&heights)) { return; }
    let p = probes[2u * i].xyz;
    let n = probes[2u * i + 1u];
    heights[i] = sea_surface(sea, p, n.xyz, n.w, 0.0).height;
}
"#
    );
    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("sea parity"),
        source: wgpu::ShaderSource::Wgsl(source.into()),
    });
    let mut uniform: Vec<f32> = Vec::new();
    let lanes = sea
        .directions
        .iter()
        .chain(&sea.bands)
        .chain(&sea.phase)
        .chain([&sea.heading, &sea.limits]);
    for lane in lanes {
        uniform.extend_from_slice(&lane.to_array());
    }
    let mut input: Vec<f32> = Vec::new();
    for (p, n, depth) in probes {
        input.extend_from_slice(&p.extend(0.0).to_array());
        input.extend_from_slice(&n.extend(*depth).to_array());
    }
    let bytes = |v: &[f32]| v.iter().flat_map(|f| f.to_le_bytes()).collect::<Vec<u8>>();
    let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: None,
        contents: &bytes(&uniform),
        usage: wgpu::BufferUsages::UNIFORM,
    });
    let probe_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: None,
        contents: &bytes(&input),
        usage: wgpu::BufferUsages::STORAGE,
    });
    let size = (probes.len() * 4) as u64;
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
                resource: uniform_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: probe_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
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
        data.chunks_exact(4)
            .map(|b| f32::from_le_bytes(b.try_into().expect("four bytes")))
            .collect(),
    )
}

#[test]
fn the_shipped_wgsl_computes_the_sea_the_hulls_float_on() {
    let table = SeaTable::new(SeaSettings::default(), 25.0);
    let seconds = 7_654.321;
    let mut failures = Vec::new();
    let mut compared = 0;
    for (wind, heading) in [(11.0, Vec3::X), (3.0, Vec3::Z), (18.0, Vec3::ZERO)] {
        let state = table.state(wind, heading);
        // Directions all over the sphere, each at a few points and depths.
        let mut probes = Vec::new();
        for i in 0..40 {
            let y = 1.0 - (i as f32 + 0.5) / 20.0;
            let a = i as f32 * 2.399;
            let r = (1.0 - y * y).max(0.0).sqrt();
            let n = Vec3::new(r * a.cos(), y, r * a.sin());
            for (offset, depth) in [(0.0, 400.0), (13.7, 3.0), (-41.2, 30.0)] {
                let p = (n * RADIUS + n.any_orthonormal_vector() * offset).normalize() * RADIUS;
                probes.push((p, p.normalize(), depth));
            }
        }
        // The state's heading is tangent at the place in the engine; here a
        // fixed vector stands in, projected by the code under test itself.
        let gpu = table.gpu(&state, seconds);
        let Some(heights) = run(&gpu, &probes) else {
            eprintln!("SKIPPED: no Vulkan adapter, the sea's WGSL was not run");
            return;
        };
        for ((p, n, depth), gpu_height) in probes.iter().zip(heights) {
            let local = table.local(&state, *n, *depth, seconds);
            let cpu = local.height(*p, RADIUS);
            compared += 1;
            if (cpu - gpu_height).abs() > 1e-3 {
                failures.push(format!(
                    "U {wind} at {n} depth {depth}: CPU {cpu} GPU {gpu_height}"
                ));
            }
        }
    }
    eprintln!("compared {compared} points on the GPU");
    assert!(
        failures.is_empty(),
        "{} of {compared}:\n{}",
        failures.len(),
        failures.join("\n")
    );
}
