//! The lens drops' WGSL, run (`lens-drop-refraction`): `water.wgsl`'s drop
//! field and the place each lens pixel looks through it, on a GPU adapter,
//! over a whole frame of lens pixels in full rain at the shipped knobs. A
//! parser can say the drops compile; only running them says a drop over the
//! sky shows the sky behind it and not the trees below it. Skipped, loudly,
//! where there is no adapter at all.

use crate::config::WeatherSettings;
use wgpu::util::DeviceExt;

/// The lens frame the drops are run over: 1.6 wide, as the owner's 1440x900.
const W: u32 = 720;
const H: u32 = 450;

fn device() -> Option<(wgpu::Device, wgpu::Queue)> {
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: wgpu::Backends::VULKAN | wgpu::Backends::METAL,
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

/// One lens pixel as the shipped WGSL sees it.
#[derive(Clone, Copy, Debug)]
struct Pixel {
    /// Where it looks, as an offset in the lens's uv (y down).
    offset: [f32; 2],
    /// The drops' coverage and lens height here, and where it looks.
    cover: f32,
    height: f32,
    seen_cover: f32,
    seen_height: f32,
    /// How far the drop's surface here faces down.
    lower: f32,
}

/// Every pixel of a `W` x `H` lens at drop time `t` with the lens's aspect
/// `aspect`. With `fixed_space` the pixels' drop space is the 1.6-wide
/// frame's whatever `aspect` says, so two aspects look at the same drops.
fn run(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    settings: &WeatherSettings,
    aspect: f32,
    t: f32,
    fixed_space: bool,
) -> Vec<Pixel> {
    let source = format!(
        "{}\n{}",
        crate::shader_tests::lens_drops_source(),
        r#"
@group(0) @binding(1) var<storage, read_write> out: array<vec4<f32>>;
@compute @workgroup_size(8, 8)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let size = vec2<u32>(u32(view.probe.z), u32(view.probe.w));
    if (id.x >= size.x || id.y >= size.y) { return; }
    let uv = (vec2<f32>(id.xy) + 0.5)/vec2<f32>(size);
    let scale = max(view.mist_look.w, 0.05);
    // The levels `lens()` gives the drops in full rain.
    let rain = 1.0;
    let l0 = smoothstep(-0.5,1.0,rain)*2.0;
    let l1 = smoothstep(0.25,0.75,rain);
    let l2 = smoothstep(0.0,0.5,rain);
    let t = view.probe.x;
    var drop_uv = rd_space(uv)*scale;
    if (view.probe.y > 0.5) { drop_uv = vec2<f32>((uv.x-0.5)*1.6, 0.5-uv.y)*scale; }
    let lens = rd_lens(drop_uv,t,l0,l1,l2);
    let seen = rd_drop_parts(rd_space(uv+lens.offset)*scale,t,l0,l1,l2);
    let i = 2u*(id.y*size.x+id.x);
    out[i] = vec4<f32>(lens.offset, lens.field.x, lens.field.y);
    out[i+1u] = vec4<f32>(seen.x, seen.y, lens.lower, 0.0);
}
"#
    );
    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("lens drops"),
        source: wgpu::ShaderSource::Wgsl(source.into()),
    });
    let s = settings;
    let uniform: [f32; 16] = [
        s.rain_lens_density,
        s.rain_lens_bend,
        s.rain_lens_speed,
        s.rain_lens_size,
        aspect,
        0.0,
        0.0,
        0.0,
        s.lens_mist_beads,
        s.lens_mist_blur,
        s.lens_mist_whiten,
        s.rain_lens_scale,
        t,
        if fixed_space { 1.0 } else { 0.0 },
        W as f32,
        H as f32,
    ];
    let bytes: Vec<u8> = uniform.iter().flat_map(|f| f.to_le_bytes()).collect();
    let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: None,
        contents: &bytes,
        usage: wgpu::BufferUsages::UNIFORM,
    });
    let size = (W * H * 2 * 16) as u64;
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
                resource: out.as_entire_binding(),
            },
        ],
    });
    let mut encoder = device.create_command_encoder(&Default::default());
    {
        let mut pass = encoder.begin_compute_pass(&Default::default());
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &bind, &[]);
        pass.dispatch_workgroups(W.div_ceil(8), H.div_ceil(8), 1);
    }
    encoder.copy_buffer_to_buffer(&out, 0, &read, 0, size);
    queue.submit([encoder.finish()]);
    let slice = read.slice(..);
    slice.map_async(wgpu::MapMode::Read, |_| {});
    device
        .poll(wgpu::PollType::wait_indefinitely())
        .expect("the GPU finishes");
    let data = slice.get_mapped_range();
    let floats: Vec<f32> = data
        .chunks_exact(4)
        .map(|b| f32::from_le_bytes(b.try_into().expect("four bytes")))
        .collect();
    floats
        .chunks_exact(8)
        .map(|p| Pixel {
            offset: [p[0], p[1]],
            cover: p[2],
            height: p[3],
            seen_cover: p[4],
            seen_height: p[5],
            lower: p[6],
        })
        .collect()
}

fn smoothstep(a: f32, b: f32, x: f32) -> f32 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// A drop on the lens bends only what is behind it: over the whole frame,
/// every pixel in a drop looks within the bend times the largest drop's
/// radius of itself, toward the drop's centre (up its lens height) and at a
/// point a drop covers, and the light gathers on the lower half of each drop.
/// Tenebris's lens, measured the same way, looked a median 70 px away at
/// 1440x900, and two thirds of its drop pixels at a point no drop covers.
#[test]
fn a_drop_on_the_lens_shows_what_is_behind_it() {
    let Some((device, queue)) = device() else {
        eprintln!("SKIPPED: no Vulkan or Metal adapter, the lens drops were not run");
        return;
    };
    let settings = WeatherSettings::default();
    let size = settings.rain_lens_size.max(0.05);
    let scale = settings.rain_lens_scale.max(0.05);
    // The near drop's radius in pixels of this frame: 0.4 of a 1/12 cell of
    // the drop space, which is screen heights times the scale.
    let near_px = 0.4 / 12.0 / size / scale * H as f32;
    let reach_px = settings.rain_lens_bend * near_px;
    for t in [0.7f32, 3.7, 11.3] {
        let pixels = run(&device, &queue, &settings, W as f32 / H as f32, t, false);
        let in_drop: Vec<&Pixel> = pixels
            .iter()
            .filter(|p| smoothstep(0.3, 1.0, p.cover) > 0.05)
            .collect();
        let share = in_drop.len() as f32 / pixels.len() as f32;
        assert!(
            (0.05..0.4).contains(&share),
            "t {t}: {share} of the frame in drops; the drops are not there"
        );
        let px = |p: &Pixel| (p.offset[0] * W as f32, p.offset[1] * H as f32);
        let mut moving = 0;
        let mut uphill = 0;
        let mut inside = 0;
        let mut lit_below = 0;
        let mut farthest = 0.0f32;
        for p in &in_drop {
            let (x, y) = px(p);
            let length = x.hypot(y);
            farthest = farthest.max(length);
            assert!(
                length <= reach_px * 1.001 + 1e-3,
                "t {t}: a drop pixel looks {length} px away, past the cap {reach_px} px"
            );
            if p.seen_cover > 0.3 {
                inside += 1;
            }
            if length > 0.25 {
                moving += 1;
                if p.seen_height >= p.height {
                    uphill += 1;
                }
                // Looking up at the centre is being below it.
                if (y < 0.0) == (p.lower > 0.0) {
                    lit_below += 1;
                }
            }
        }
        let n = in_drop.len() as f32;
        let m = moving.max(1) as f32;
        eprintln!(
            "t {t}: {share:.3} of the frame in drops, {:.3} of them moving, farthest {farthest:.2} px (cap {reach_px:.2}); \
             uphill {:.4}, inside a drop {:.4}, lit below {:.4}",
            moving as f32 / n,
            uphill as f32 / m,
            inside as f32 / n,
            lit_below as f32 / m,
        );
        assert!(moving as f32 > 0.5 * n, "t {t}: the drops bend nothing");
        assert!(
            uphill as f32 >= 0.97 * m,
            "t {t}: {uphill} of {moving} moving drop pixels look toward their drop's centre"
        );
        assert!(
            inside as f32 >= 0.99 * n,
            "t {t}: {inside} of {} drop pixels look at a point a drop covers",
            in_drop.len()
        );
        assert_eq!(lit_below, moving, "t {t}: light gathered on an upper edge");
    }
}

/// The offset is measured alike across and down: the same drops seen at two
/// aspects bend each pixel by the same number of pixels each way. Tenebris's
/// added a share of the screen's width across, 1.6 times its pixels down.
#[test]
fn a_drop_bends_alike_across_and_down() {
    let Some((device, queue)) = device() else {
        eprintln!("SKIPPED: no Vulkan or Metal adapter, the lens drops were not run");
        return;
    };
    let settings = WeatherSettings::default();
    let narrow = run(&device, &queue, &settings, 1.0, 3.7, true);
    let wide = run(&device, &queue, &settings, 2.0, 3.7, true);
    let mut compared = 0;
    for (a, b) in narrow.iter().zip(&wide) {
        // Pixels across are the offset times the width, aspect times H.
        let across = (a.offset[0] * H as f32, b.offset[0] * 2.0 * H as f32);
        let down = (a.offset[1] * H as f32, b.offset[1] * H as f32);
        assert!(
            (across.0 - across.1).abs() <= 1e-4 + 1e-4 * across.0.abs(),
            "across {across:?}"
        );
        assert!(
            (down.0 - down.1).abs() <= 1e-4 + 1e-4 * down.0.abs(),
            "down {down:?}"
        );
        if across.0.abs() > 0.25 {
            compared += 1;
        }
    }
    assert!(compared > 1000, "only {compared} pixels bent across");
}
