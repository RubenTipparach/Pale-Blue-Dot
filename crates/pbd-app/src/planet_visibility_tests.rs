//! Executes the shipped visibility shader against synthetic geometry. Readback
//! belongs only to this test harness; terrain authority never depends on it.

use super::*;
use bevy::{
    math::DVec3,
    math::UVec4,
    render::{
        renderer::initialize_renderer,
        settings::{WgpuSettings, WgpuSettingsPriority},
    },
};
use std::time::Duration;

const RADIUS: f32 = 10_000.;

struct VisibilityGpu {
    device: RenderDevice,
    queue: RenderQueue,
    layout: BindGroupLayout,
    clear: ComputePipeline,
    compact: ComputePipeline,
}

#[derive(Debug, PartialEq, Eq)]
struct VisibleCells {
    terrain: Vec<u32>,
    foliage: Vec<u32>,
    /// Water sheets listed; the synthetic columns sit above the sea, so it
    /// is zero for them and only the real-record test sees any.
    water: u32,
}

impl VisibilityGpu {
    fn new() -> Self {
        let settings = WgpuSettings {
            priority: WgpuSettingsPriority::Compatibility,
            ..default()
        };
        let resources = bevy::tasks::block_on(initialize_renderer(
            settings.backends.unwrap(),
            None,
            &settings,
        ));
        let device = resources.0;
        let queue = resources.1;
        eprintln!(
            "Visibility regression GPU: {} ({:?})",
            resources.2.name, resources.2.backend
        );
        // Use the production Rust layout and the actual WGSL asset together.
        let layout = device
            .create_bind_group_layout("visibility regression layout", &compute_layout().entries);
        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("visibility regression pipeline layout"),
            bind_group_layouts: &[&layout],
            push_constant_ranges: &[],
        });
        let shader = device.create_and_validate_shader_module(ShaderModuleDescriptor {
            label: Some("actual planet_visibility.wgsl"),
            source: ShaderSource::Wgsl(
                include_str!("../../../assets/shaders/planet_visibility.wgsl").into(),
            ),
        });
        let pipeline = |entry_point| {
            device.create_compute_pipeline(&RawComputePipelineDescriptor {
                label: Some(entry_point),
                layout: Some(&pipeline_layout),
                module: &shader,
                entry_point: Some(entry_point),
                compilation_options: default(),
                cache: None,
            })
        };
        let clear = pipeline("clear_indirect");
        let compact = pipeline("compact_visible");
        Self {
            device,
            queue,
            layout,
            clear,
            compact,
        }
    }

    fn run(&self, cells: &[GpuCell], params: PlanetParams) -> VisibleCells {
        self.run_with_capacities(cells, params, [cells.len(); 2])
    }

    fn run_with_capacities(
        &self,
        cells: &[GpuCell],
        params: PlanetParams,
        capacities: [usize; 2],
    ) -> VisibleCells {
        assert!(!cells.is_empty(), "storage binding needs at least one cell");
        let mut uniform = UniformBuffer::from(params);
        uniform.write_buffer(&self.device, &self.queue);
        let cell_buffer = self.device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("synthetic planet columns"),
            contents: bytemuck::cast_slice(cells),
            usage: BufferUsages::STORAGE,
        });
        let output = |label, words| {
            self.device.create_buffer_with_data(&BufferInitDescriptor {
                label: Some(label),
                contents: bytemuck::cast_slice(&vec![u32::MAX; words]),
                usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC,
            })
        };
        let terrain = output("terrain visibility under test", capacities[0]);
        let foliage = output("foliage visibility under test", capacities[1]);
        // The water list is bound and cleared like the others; these synthetic
        // columns sit above sea level, so it stays empty and its count is
        // asserted to be zero.
        let water = output("water visibility under test", capacities[0]);
        let args = output("indirect arguments under test", 12);
        let bind_group = self.device.create_bind_group(
            "visibility regression inputs",
            &self.layout,
            &BindGroupEntries::sequential((
                &uniform,
                cell_buffer.as_entire_binding(),
                terrain.as_entire_binding(),
                args.as_entire_binding(),
                foliage.as_entire_binding(),
                water.as_entire_binding(),
            )),
        );
        let list_bytes = capacities.map(|count| (count * size_of::<u32>()) as u64);
        let readback = self.device.create_buffer(&BufferDescriptor {
            label: Some("test-only visibility readback"),
            size: 48 + list_bytes[0] + list_bytes[1],
            usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor::default());
        {
            let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor::default());
            pass.set_bind_group(0, &*bind_group, &[]);
            pass.set_pipeline(&self.clear);
            pass.dispatch_workgroups(1, 1, 1);
            pass.set_pipeline(&self.compact);
            // Deliberately launch beyond both count and array length to test
            // the tail guards, including a completely spare workgroup.
            pass.dispatch_workgroups((cells.len() as u32).div_ceil(128) + 1, 1, 1);
        }
        encoder.copy_buffer_to_buffer(&args, 0, &readback, 0, 48);
        encoder.copy_buffer_to_buffer(&terrain, 0, &readback, 48, list_bytes[0]);
        encoder.copy_buffer_to_buffer(&foliage, 0, &readback, 48 + list_bytes[0], list_bytes[1]);
        let submitted = self.queue.submit([encoder.finish()]);
        let (sender, receiver) = std::sync::mpsc::channel();
        let slice = readback.slice(..);
        slice.map_async(MapMode::Read, move |result| sender.send(result).unwrap());
        self.device
            .poll(PollType::Wait {
                submission_index: Some(submitted),
                timeout: Some(Duration::from_secs(30)),
            })
            .expect("visibility compute must finish");
        receiver
            .recv_timeout(Duration::from_secs(30))
            .expect("readback callback must run")
            .expect("visibility readback must map");
        let mapped = slice.get_mapped_range();
        let words: &[u32] = bytemuck::cast_slice(&mapped);
        assert_eq!([words[0], words[2], words[3]], [60, 0, 0]);
        assert_eq!([words[4], words[6], words[7]], [108, 60, 0]);
        assert_eq!([words[8], words[10], words[11]], [18, 0, 0]);
        let collect = |start: usize, count: u32, capacity: usize| {
            assert!(count as usize <= capacity, "draw exceeds ID capacity");
            let mut ids = words[start..start + count as usize].to_vec();
            assert!(ids.iter().all(|id| (*id as usize) < cells.len()));
            ids.sort_unstable();
            assert!(ids.windows(2).all(|pair| pair[0] != pair[1]));
            // The shader must never publish IDs after the indirect draw count.
            assert!(
                words[start + count as usize..start + capacity]
                    .iter()
                    .all(|word| *word == u32::MAX)
            );
            ids
        };
        VisibleCells {
            terrain: collect(12, words[1], capacities[0]),
            foliage: collect(12 + capacities[0], words[5], capacities[1]),
            water: words[9],
        }
    }
}

/// Radial corners represent the real storage format; unused pentagon data is
/// intentionally absurd so reading the sixth slot affects visibility.
fn column(position: Vec3, degree: u32, width: f32, wall_depth: f32) -> GpuCell {
    let axis = position.normalize();
    let height = position.length() - RADIUS;
    let mut corners = [[0., 0., -1., -100_000.]; 6];
    for (index, corner) in corners.iter_mut().take(degree as usize).enumerate() {
        let angle = std::f32::consts::PI + std::f32::consts::TAU * index as f32 / degree as f32;
        let ray = (position + Vec3::new(angle.cos(), angle.sin(), 0.) * width).normalize();
        *corner = ray.extend(height - wall_depth).to_array();
    }
    // A synthetic column is a finest-level cell that owns itself, so the
    // partition rule keeps every one of them and the tests below exercise the
    // horizon, frustum and foliage decisions on their own.
    GpuCell {
        direction_height: axis.extend(height).to_array(),
        corners,
        metadata: [degree | (TEST_FINEST_LEVEL << 8), 5, 65_535, 0],
        owner_a: axis.extend(height).to_array(),
        owner_b: axis.extend(height).to_array(),
        floors: [height; 4],
        spare: [0.; 4],
    }
}

/// The base level the synthetic params declare; the finest is four above it.
const TEST_BASE_LEVEL: u32 = 0;
const TEST_FINEST_LEVEL: u32 = TEST_BASE_LEVEL + 4;

fn params(count: usize, camera_height: f32, half_width: f32) -> PlanetParams {
    let camera = Vec3::new(0., 0., RADIUS + camera_height);
    PlanetParams {
        clip_from_body: Mat4::orthographic_rh(
            -half_width,
            half_width,
            -half_width,
            half_width,
            10.,
            3000.,
        ) * Mat4::look_at_rh(camera, Vec3::Z * RADIUS, Vec3::Y),
        camera: camera.extend(1.),
        sun: Vec3::new(0.3, 0.6, 0.7).normalize().extend(1.),
        settings: Vec4::new(RADIUS, count as f32, 0., 2300.),
        water_absorption: Vec3::new(0.6, 0.2, 0.1).extend(RADIUS - 0.5),
        water_deep: Vec3::new(0.02, 0.10, 0.22).extend(0.),
        weather: Vec4::ZERO,
        rain: [Vec4::ZERO; 4],
        // Every slot is in the base, so all of them are live; the bands are
        // wider than the sphere, so every owner is fine and nothing is covered.
        lod_offsets: UVec4::new(count as u32, 1, 0, 0),
        lod_counts: UVec4::ZERO,
        lod: Vec3::Z.extend(TEST_BASE_LEVEL as f32),
        bands: Vec4::splat(-2.),
    }
}

fn expect(
    gpu: &VisibilityGpu,
    cells: &[GpuCell],
    params: PlanetParams,
    terrain: &[u32],
    foliage: &[u32],
) {
    assert_eq!(
        gpu.run(cells, params),
        VisibleCells {
            terrain: terrain.to_vec(),
            foliage: foliage.to_vec(),
            water: 0,
        }
    );
}

#[test]
#[ignore = "requires a GPU; run cargo test -p pbd-app --lib actual_gpu_visibility -- --ignored --nocapture"]
fn actual_gpu_visibility_preserves_geometry_and_selects_foliage() {
    let gpu = VisibilityGpu::new();

    let at = |x, y, z, degree, width, wall| column(Vec3::new(x, y, z), degree, width, wall);
    let cells = [
        at(0., 0., RADIUS, 6, 2., 0.),
        at(14., 0., RADIUS, 6, 5., 0.), // Centre outside, hex corner inside.
        at(-14., 0., RADIUS, 6, 5., 0.),
        at(0., 14., RADIUS, 6, 6., 0.),
        at(0., -14., RADIUS, 6, 6., 0.),
        at(14., 0., RADIUS, 5, 5., 0.), // Pentagon crossing the right plane.
        at(0., 0., RADIUS + 94., 6, 2., 15.), // Only the wall crosses near.
        at(40., 0., RADIUS, 5, 2., 0.), // Sixth corner must not enlarge bounds.
        at(-40., 0., RADIUS, 6, 2., 0.),
        at(0., 40., RADIUS, 6, 2., 0.),
        at(0., -40., RADIUS, 6, 2., 0.),
        at(0., 0., RADIUS + 120., 6, 2., 0.), // Entirely behind near plane.
        at(0., 0., RADIUS + 100., 6, 2., 0.), // At camera, outside near plane.
    ];
    expect(
        &gpu,
        &cells,
        params(cells.len(), 100., 10.),
        &[0, 1, 2, 3, 4, 5, 6],
        &[],
    );

    // Far-plane and horizon rejection use physically positive surface heights.
    let cells = [
        at(0., 0., RADIUS, 6, 2., 0.),
        at(0., 0., -RADIUS, 6, 2., 0.),
    ];
    let mut wide = params(2, 100., 30_000.);
    wide.clip_from_body = Mat4::orthographic_rh(-30_000., 30_000., -30_000., 30_000., 10., 40_000.)
        * Mat4::look_at_rh(wide.camera.truncate(), Vec3::Z * RADIUS, Vec3::Y);
    expect(&gpu, &cells, wide, &[0], &[]);
    expect(&gpu, &cells, params(2, 3100., 30_000.), &[], &[]);

    // Match the desktop camera's infinite reverse-Z perspective convention.
    // The distant surface lies beyond the finite fixture's 3000 m far plane;
    // the degenerate infinite far plane must still keep it. A side column and
    // a column entirely inside the 10 m near distance must both be rejected.
    let cells = [
        at(0., 0., RADIUS, 6, 2., 0.),         // Surface 5000 m away.
        at(6000., 0., RADIUS, 6, 2., 0.),      // Outside the 90-degree view.
        at(0., 0., RADIUS + 4995., 6, 2., 0.), // Only 5 m away.
        at(0., 0., RADIUS + 4900., 5, 2., 0.), // Visible foreground.
    ];
    let mut reverse = params(cells.len(), 5000., 10.);
    reverse.clip_from_body =
        Mat4::perspective_infinite_reverse_rh(std::f32::consts::FRAC_PI_2, 1., 10.)
            * Mat4::look_at_rh(reverse.camera.truncate(), Vec3::Z * RADIUS, Vec3::Y);
    expect(&gpu, &cells, reverse, &[0, 3], &[]);

    // Eligibility fixtures pin the actual existing hash/biome contract without
    // reimplementing the predicate in Rust: forest 5 passes and 0 fails; grass
    // 5 passes and 0 fails; scrub 15 passes and 0 fails.
    let mut cells: Vec<_> = [(3, 5), (3, 0), (2, 5), (2, 0), (7, 15), (7, 0), (5, 0)]
        .into_iter()
        .map(|(biome, seed)| {
            let mut cell = at(0., 0., RADIUS + 500., 6, 2., 0.);
            cell.metadata[1] = biome;
            cell.metadata[3] = seed;
            cell
        })
        .collect();
    for height in [201., 200., 199.] {
        let mut cell = at(0., 0., RADIUS + height, 5, 2., 0.);
        cell.metadata[1] = 3;
        cell.metadata[3] = 5;
        cells.push(cell);
    }
    expect(
        &gpu,
        &cells,
        params(cells.len(), 2500., 10.),
        &(0..10).collect::<Vec<_>>(),
        &[0, 2, 4, 7],
    );
    let mut disabled = params(cells.len(), 2500., 10.);
    disabled.settings.w = 0.;
    expect(&gpu, &cells, disabled, &(0..10).collect::<Vec<_>>(), &[]);

    // A crown may intersect the view while its cap is outside. The independent
    // foliage list keeps it; a distant or ineligible column gets no tree margin.
    // The largest rescaled tree fits inside 15 m of its base, so a trunk at
    // 20 m (cap corners at 18 to 22, outside the 10 m half-width) is kept by
    // its crown alone.
    let mut tree = at(20., 0., RADIUS, 6, 2., 0.);
    tree.metadata[1] = 3;
    tree.metadata[3] = 5;
    let rock = at(20., 0., RADIUS, 6, 2., 0.);
    let cells = [tree, rock];
    expect(&gpu, &cells, params(2, 100., 10.), &[], &[0]);
    expect(&gpu, &cells, params(2, 2500., 10.), &[], &[]);

    // Exercise atomic compaction across workgroups and both bounds checks.
    let cells = vec![at(0., 0., RADIUS, 6, 2., 0.); 129];
    let expected: Vec<u32> = (0..129).collect();
    expect(&gpu, &cells, params(cells.len(), 100., 10.), &expected, &[]);
    expect(&gpu, &cells, params(cells.len() + 128, 100., 10.), &[], &[]);
    expect(&gpu, &cells, params(1, 100., 10.), &[0], &[]);
    expect(&gpu, &cells, params(0, 100., 10.), &[], &[]);
    for capacities in [[1, cells.len()], [cells.len(), 1]] {
        assert_eq!(
            gpu.run_with_capacities(&cells, params(cells.len(), 100., 10.), capacities),
            VisibleCells {
                terrain: vec![],
                foliage: vec![],
                water: 0,
            },
            "either undersized output must suppress the complete generation",
        );
    }

    // Move the body and camera together, through the same preparation helper
    // used by rendering. The GPU must make identical visibility decisions.
    let cells = [
        at(0., 0., RADIUS, 6, 2., 0.),
        at(14., 0., RADIUS, 5, 5., 0.),
        at(40., 0., RADIUS, 6, 2., 0.),
        tree,
    ];
    let origin = gpu.run(&cells, params(cells.len(), 100., 10.));
    let offset = DVec3::new(1_000_000., -2_000_000., 3_000_000.);
    let camera = GlobalTransform::from(Transform::from_translation(
        (offset + DVec3::new(0., 0., f64::from(RADIUS + 100.))).as_vec3(),
    ));
    let projection = Mat4::orthographic_rh(-10., 10., -10., 10., 10., 3000.);
    let frame = PlanetRenderFrame { center: offset };
    for override_clip in [None, Some(projection * camera.to_matrix().inverse())] {
        let (position, clip) = frame.camera_and_clip(&camera, projection, override_clip);
        assert_eq!(position, Vec3::new(0., 0., RADIUS + 100.));
        let mut translated = params(cells.len(), 100., 10.);
        translated.camera = position.extend(1.);
        translated.clip_from_body = clip;
        assert_eq!(gpu.run(&cells, translated), origin);
    }
}

/// The partition on the real records: the base and the fine set around one
/// anchor, packed as `upload_planet` packs them, with a 12 m eye at the anchor
/// looking along the ground through a 90-degree perspective. Every listed
/// tile must be at the level its band says, so no coarse cap can poke up
/// through fine ground and no band is left empty.
#[test]
#[ignore = "requires a GPU; run cargo test -p pbd-app --lib partition -- --ignored --nocapture"]
fn the_partition_lists_each_tile_at_its_bands_level_on_the_real_records() {
    let gpu = VisibilityGpu::new();
    let anchor = Vec3::new(0.8776, 0.4794, 0.0).normalize();
    let base = lod::base_records(&topology::dual_sphere(lod::BASE_LEVEL as u32));
    let fine = lod::generate_fine(anchor);
    let capacity = fine.levels.iter().map(Vec::len).max().unwrap();
    let blank = GpuCell {
        direction_height: [0.; 4],
        corners: [[0.; 4]; 6],
        metadata: [0; 4],
        owner_a: [0.; 4],
        owner_b: [0.; 4],
        floors: [0.; 4],
        spare: [0.; 4],
    };
    let mut records = base.clone();
    for level in &fine.levels {
        records.extend_from_slice(level);
        records.extend(std::iter::repeat_n(blank, capacity - level.len()));
    }
    let slots = records.len();
    let tangent = Vec3::Y.cross(anchor).normalize();
    let lod_params = lod::LodParams::new(anchor);
    // The seam eye: 12 m up, along the ground, where the horizon is about
    // 340 m out so the base is below it. Then 3,000 m up looking down, where
    // every band is in view.
    let eyes = [
        (
            anchor * (terrain_radius(anchor) + 12.),
            tangent * 600. - anchor * 12.,
            anchor,
        ),
        (anchor * (terrain_radius(anchor) + 3000.), -anchor, Vec3::Y),
    ];
    let mut per_level = [0usize; 5];
    for (eye, (camera, look, up)) in eyes.into_iter().enumerate() {
        let mut params = params(slots, 0., 0.);
        params.camera = camera.extend(1.);
        params.clip_from_body =
            Mat4::perspective_infinite_reverse_rh(std::f32::consts::FRAC_PI_2, 1.6, 0.1)
                * Mat4::look_at_rh(camera, camera + look, up);
        params.settings = Vec4::new(PLANET_RADIUS, slots as f32, 0., lod::BAND_M[3]);
        params.water_absorption.w = PLANET_RADIUS - 0.5;
        params.lod_offsets = UVec4::new(base.len() as u32, capacity as u32, 0, 0);
        params.lod_counts = UVec4::from_array(fine.levels.each_ref().map(|l| l.len() as u32));
        params.lod = anchor.extend(lod::BASE_LEVEL as f32);
        params.bands = lod_params.bands;
        let listed = gpu.run_with_capacities(&records, params, [slots; 2]);
        assert!(!listed.terrain.is_empty());
        per_level = [0usize; 5];
        for &id in &listed.terrain {
            let cell = &records[id as usize];
            let level = cell.metadata[0] >> 8;
            let direction = Vec3::from_slice(&cell.direction_height[..3]);
            let distance_m = direction.dot(anchor).clamp(-1., 1.).acos() * PLANET_RADIUS;
            // A tile is never listed inside the band of the next finer level,
            // and a fine tile is never listed outside its own band, give or take
            // the coarser cell it is partitioned by.
            let slack = lod::tile_width_m(level.max(8) as u8 - 1);
            let k = level as usize - lod::BASE_LEVEL as usize;
            if level < lod::FINEST_LEVEL as u32 {
                assert!(
                    distance_m + slack >= lod::BAND_M[k],
                    "level {level} tile listed {distance_m:.0} m from the player, inside the {} m band",
                    lod::BAND_M[k]
                );
            }
            if level > lod::BASE_LEVEL as u32 {
                assert!(
                    distance_m - slack <= lod::BAND_M[k - 1],
                    "level {level} tile listed {distance_m:.0} m out, past its {} m band",
                    lod::BAND_M[k - 1]
                );
            }
            per_level[k] += 1;
        }
        eprintln!("eye {eye}: listed per level L7..L11: {per_level:?}");
        if eye == 0 {
            assert!(
                per_level[4] > 100,
                "the finest band is empty: {per_level:?}"
            );
            assert_eq!(per_level[0], 0, "the base is below a 12 m eye's horizon");
        }
    }
    assert!(
        per_level.iter().all(|&n| n > 0),
        "a band is empty: {per_level:?}"
    );
}
