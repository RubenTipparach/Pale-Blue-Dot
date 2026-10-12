//! The weather maps' blend pass, run: `weather_blend.wgsl` on a GPU adapter,
//! mixing two half-float cube maps as the render world uploads them, against
//! the mix the CPU makes (`smooth-weather` task 4.2). A parser can say the
//! shader is valid; only running it says it writes the mix where the clouds
//! read it. Skipped, loudly, where there is no adapter at all.

use crate::atmosphere::MAP_SIZE;
use crate::planet::weather_maps::half_bits;

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

fn half_to_f32(h: u16) -> f32 {
    let sign = if h & 0x8000 != 0 { -1.0 } else { 1.0 };
    let e = ((h >> 10) & 0x1f) as i32;
    let m = (h & 0x3ff) as f32;
    sign * if e == 0 {
        m / 1024.0 * 2f32.powi(-14)
    } else {
        (1.0 + m / 1024.0) * 2f32.powi(e - 15)
    }
}

/// A texel's value in a test map: distinct per channel, face and place.
fn texel(seed: f32, i: usize) -> [f32; 4] {
    let x = i as f32;
    [
        (x * 0.013 + seed).sin(),
        (x * 0.007 - seed).cos() * 3.0,
        seed * 10.0 - x * 0.001,
        (x * 0.0021 + seed * 0.5).fract(),
    ]
}

#[test]
fn the_blend_pass_writes_the_mix_of_the_pair() {
    let Some((device, queue)) = device() else {
        eprintln!("SKIPPED: no Vulkan adapter, the weather blend was not run");
        return;
    };
    let size = MAP_SIZE as u32;
    let texels = 6 * MAP_SIZE * MAP_SIZE;
    let extent = wgpu::Extent3d {
        width: size,
        height: size,
        depth_or_array_layers: 6,
    };
    let texture = |usage| {
        device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage,
            view_formats: &[],
        })
    };
    let read_usage = wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST;
    let out_usage = wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_SRC;
    let maps: Vec<Vec<[f32; 4]>> = [0.2f32, 1.7, -0.6, 2.4]
        .iter()
        .map(|seed| (0..texels).map(|i| texel(*seed, i)).collect())
        .collect();
    let upload = |t: &wgpu::Texture, values: &[[f32; 4]]| {
        let bytes: Vec<u8> = values
            .iter()
            .flatten()
            .flat_map(|v| half_bits(*v).to_le_bytes())
            .collect();
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: t,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &bytes,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(size * 8),
                rows_per_image: Some(size),
            },
            extent,
        );
    };
    let inputs: Vec<wgpu::Texture> = maps
        .iter()
        .map(|values| {
            let t = texture(read_usage);
            upload(&t, values);
            t
        })
        .collect();
    let outs = [texture(out_usage), texture(out_usage)];
    let t = 0.3f32;
    let uniform = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: 16,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let bytes: Vec<u8> = [t, 0.0, 0.0, 0.0]
        .iter()
        .flat_map(|v| v.to_le_bytes())
        .collect();
    queue.write_buffer(&uniform, 0, &bytes);
    let source = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../assets/shaders/weather_blend.wgsl"
    ))
    .expect("the shader");
    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("weather blend"),
        source: wgpu::ShaderSource::Wgsl(source.into()),
    });
    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: None,
        layout: None,
        module: &module,
        entry_point: Some("blend_maps"),
        compilation_options: Default::default(),
        cache: None,
    });
    let array = |t: &wgpu::Texture| {
        t.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            ..Default::default()
        })
    };
    let views = [
        array(&inputs[0]),
        array(&inputs[1]),
        array(&outs[0]),
        array(&inputs[2]),
        array(&inputs[3]),
        array(&outs[1]),
    ];
    let mut entries = vec![wgpu::BindGroupEntry {
        binding: 0,
        resource: uniform.as_entire_binding(),
    }];
    for (k, view) in views.iter().enumerate() {
        entries.push(wgpu::BindGroupEntry {
            binding: k as u32 + 1,
            resource: wgpu::BindingResource::TextureView(view),
        });
    }
    let bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &pipeline.get_bind_group_layout(0),
        entries: &entries,
    });
    let bytes = (texels * 8) as u64;
    let reads: Vec<wgpu::Buffer> = (0..2)
        .map(|_| {
            device.create_buffer(&wgpu::BufferDescriptor {
                label: None,
                size: bytes,
                usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        })
        .collect();
    let mut encoder = device.create_command_encoder(&Default::default());
    {
        let mut pass = encoder.begin_compute_pass(&Default::default());
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &bind, &[]);
        let groups = size.div_ceil(8);
        pass.dispatch_workgroups(groups, groups, 6);
    }
    for (out, read) in outs.iter().zip(&reads) {
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: out,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: read,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(size * 8),
                    rows_per_image: Some(size),
                },
            },
            extent,
        );
    }
    queue.submit([encoder.finish()]);
    for read in &reads {
        read.slice(..).map_async(wgpu::MapMode::Read, |_| {});
    }
    device
        .poll(wgpu::PollType::wait_indefinitely())
        .expect("the GPU finishes");
    let mut worst = 0.0f32;
    for (map, read) in reads.iter().enumerate() {
        let data = read.slice(..).get_mapped_range();
        let got: Vec<f32> = data
            .chunks_exact(2)
            .map(|b| half_to_f32(u16::from_le_bytes([b[0], b[1]])))
            .collect();
        let (from, to) = (&maps[2 * map], &maps[2 * map + 1]);
        for (i, value) in got.iter().enumerate() {
            let (a, b) = (from[i / 4][i % 4], to[i / 4][i % 4]);
            // The CPU's mix of what the GPU was given: each end as a half.
            let (a, b) = (half_to_f32(half_bits(a)), half_to_f32(half_bits(b)));
            let want = a + (b - a) * t;
            let error = (value - want).abs() / want.abs().max(1.0);
            worst = worst.max(error);
            assert!(error < 2e-3, "map {map} value {i}: GPU {value}, CPU {want}");
        }
    }
    eprintln!("the blend pass matches the CPU's mix on every texel, worst {worst:.2e}");
}
