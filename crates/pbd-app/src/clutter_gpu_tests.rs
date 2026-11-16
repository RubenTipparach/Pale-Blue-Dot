//! The ground clutter's WGSL, run (`gather-by-hand` task 1):
//! `planet_surface.wgsl`'s shared clutter block (`clutter_grows`,
//! `clutter_seat`) on a GPU adapter against `pbd_core::scatter`, the Rust
//! the hand's pick-up reads. A port that rolled differently would offer a
//! stone where none is drawn, or none where one is. Skipped, loudly, where
//! there is no adapter.

use bevy::math::Vec3;
use pbd_core::scatter::{self, Kind};
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

fn shared_source(shader: &str) -> String {
    let begin = shader
        .find("// clutter:shared begin")
        .expect("the shader marks its shared clutter block");
    let end = shader
        .find("// clutter:shared end")
        .expect("the shader ends its shared clutter block");
    shader[begin..end].to_string()
}

/// The chance every piece is rolled against in the probe: high, so most
/// cells grow something and the seats are compared often.
const CHANCE: f32 = 0.5;

/// Eight vec4 a probe in, four out (a seat and whether it grows, a piece).
const MAIN: &str = r#"
@group(0) @binding(0) var<storage, read> probes: array<vec4<f32>>;
@group(0) @binding(1) var<storage, read_write> out: array<vec4<f32>>;
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let n = arrayLength(&out)/4u;
    let i = gid.x;
    if (i >= n) { return; }
    let head = probes[8u*i];
    let id = bitcast<u32>(head.x);
    let material = bitcast<u32>(head.y);
    let degree = bitcast<u32>(head.z);
    let radius = head.w;
    let axis = probes[8u*i+1u].xyz;
    var corners: array<vec4<f32>,6>;
    for (var k = 0u; k < 6u; k++) { corners[k] = probes[8u*i+2u+k]; }
    let salts = array<u32,4>(SALT_ROCK, SALT_BUSH, SALT_SHRUB, SALT_FERN);
    for (var p = 0u; p < 4u; p++) {
        let seat = clutter_seat(id, salts[p], degree, axis, corners, radius);
        let grows = select(0.0, 1.0, clutter_grows(p, id, material, CHANCE));
        out[4u*i+p] = vec4<f32>(seat, grows);
    }
}
"#;

fn run(probes: &[[[f32; 4]; 8]]) -> Option<Vec<[f32; 4]>> {
    let shader = include_str!("../../../assets/shaders/planet_surface.wgsl");
    let (device, queue) = device()?;
    let source = format!(
        "{}\nconst CHANCE: f32 = {CHANCE:?};\n{MAIN}",
        shared_source(shader)
    );
    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("clutter parity"),
        source: wgpu::ShaderSource::Wgsl(source.into()),
    });
    let bytes: Vec<u8> = probes
        .iter()
        .flatten()
        .flatten()
        .flat_map(|f| f.to_le_bytes())
        .collect();
    let input = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: None,
        contents: &bytes,
        usage: wgpu::BufferUsages::STORAGE,
    });
    let outputs = probes.len() * 4;
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
                resource: input.as_entire_binding(),
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

/// Every piece's growth and seat, from the shader's own block and from the
/// port, over cells of every degree, material and size on a 4,800 m
/// planet: the same decision every time, and seats within a centimetre.
#[test]
fn the_shader_and_the_port_put_the_clutter_in_the_same_places() {
    let mut cells = Vec::new();
    for n in 0..2_000u32 {
        let id = scatter::hash(n.wrapping_mul(7919) ^ 0x5eed);
        let degree = if n % 7 == 0 { 5 } else { 6 };
        let material = n % 8;
        let a = n as f32 * 0.37;
        let axis = Vec3::new(a.cos(), (a * 0.61).sin(), a.sin()).normalize();
        let t = axis.any_orthonormal_vector();
        let b = axis.cross(t);
        let corners: Vec<Vec3> = (0..degree)
            .map(|k| {
                let c = k as f32 * std::f32::consts::TAU / degree as f32;
                (axis + (t * c.cos() + b * c.sin()) * 0.00034).normalize()
            })
            .collect();
        let radius = 4_800.0 + (n % 40) as f32 - 10.0;
        cells.push((id, material, degree, axis, corners, radius));
    }
    let probes: Vec<[[f32; 4]; 8]> = cells
        .iter()
        .map(|(id, material, degree, axis, corners, radius)| {
            let mut p = [[0.0; 4]; 8];
            p[0] = [
                f32::from_bits(*id),
                f32::from_bits(*material),
                f32::from_bits(*degree as u32),
                *radius,
            ];
            p[1] = axis.extend(0.0).to_array();
            for (k, c) in corners.iter().enumerate() {
                p[2 + k] = c.extend(0.0).to_array();
            }
            p
        })
        .collect();
    let Some(out) = run(&probes) else {
        eprintln!("SKIPPED: no GPU adapter for the clutter parity test");
        return;
    };
    let (mut grown, mut worst) = (0, 0.0f32);
    for (i, (id, material, _, axis, corners, radius)) in cells.iter().enumerate() {
        let cell = scatter::Cell {
            id: *id,
            material: *material,
            biome: 0,
            axis: *axis,
            corners,
            radius: *radius,
        };
        for kind in Kind::ALL {
            let [x, y, z, g] = out[4 * i + kind as usize];
            let grows = scatter::grows(kind, *id, *material, CHANCE);
            assert_eq!(
                g == 1.0,
                grows,
                "cell {i} {kind:?}: the shader and the port disagree"
            );
            let off = Vec3::new(x, y, z).distance(scatter::seat(kind, &cell));
            assert!(off < 0.01, "cell {i} {kind:?}: seats {off} m apart");
            worst = worst.max(off);
            grown += usize::from(grows);
        }
    }
    assert!(
        grown > 1_000,
        "only {grown} pieces grew: the probe proves little"
    );
    eprintln!("clutter parity: {grown} pieces, seats at most {worst} m apart");
}
