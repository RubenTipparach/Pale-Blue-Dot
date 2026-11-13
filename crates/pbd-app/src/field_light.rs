//! What lights the things the terrain pass does not draw: the held tool and
//! hand, the ship, a fish, the float (`lamps-and-lanterns` decision 10).
//!
//! One field lights everything. The terrain pass reads the column tier's
//! light field directly; these read the same field through
//! `ColumnTier::sample` and turn it into light with the terrain's own terms
//! (`pbd_core::light::sky_fill`, `lamp_light`), so a tool in a cave is as
//! dark as the cave and a ship beside a lantern is lit on the lantern's side.

use crate::planet::{PLANET_RADIUS, PlanetContact, PlanetFine};
use bevy::camera::primitives::Aabb;
use bevy::pbr::{ExtendedMaterial, MaterialExtension};
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, BufferUsages};
use bevy::render::storage::{GpuShaderStorageBuffer, ShaderStorageBuffer};
use bevy::shader::ShaderRef;
use pbd_core::light;

/// Put on the root of anything lit PBR that should take the field: a craft,
/// a fish school, the float. Every mesh under it is given its own copy of its
/// material, extended with the field (`FieldLit`).
#[derive(Component, Default, Clone, Copy)]
pub struct LitByField;

/// Put beside [`LitByField`] on a root whose meshes are lit as the terrain
/// is, by the terrain's own fill and sun rather than Bevy's picture: a town's
/// pieces (`sun-shadows` decision 7), which are the ground's own kind of
/// thing and stand beside it.
#[derive(Component, Default, Clone, Copy)]
pub struct LitLikeTerrain;

/// The most lights a building's rooms carry (`cities-in-the-world` decision
/// 7a): a hearth, a stair's sconces and its candles. `field_lit.wgsl`'s
/// arrays are the same length, and a test holds them together.
pub const MAX_ROOM_LIGHTS: usize = 24;

/// What burns in a building's rooms, on each of its room meshes: the lights
/// its faces take on top of the field's (`cities-in-the-world` decision 7a).
#[derive(Component, Clone, Debug, Default)]
pub struct RoomLights(pub Vec<pbd_core::settlement::pieces::RoomLight>);

impl RoomLights {
    /// As `field_lit.wgsl` reads them, in the planet's frame: where and how
    /// far, colour times power and whether only by night, and the band of
    /// height each lights. The fires first, so a building with more candles
    /// than room keeps its hearth and sconces. At most [`MAX_ROOM_LIGHTS`].
    pub fn pack(&self) -> Vec<LightRow> {
        let mut lights: Vec<_> = self.0.iter().collect();
        lights.sort_by_key(|l| !l.kind.all_day());
        lights
            .into_iter()
            .take(MAX_ROOM_LIGHTS)
            .map(|l| {
                let [r, g, b] = l.kind.colour();
                LightRow {
                    at: l.at.extend(l.kind.reach_m()),
                    colour: Vec4::new(
                        r * l.power,
                        g * l.power,
                        b * l.power,
                        if l.kind.all_day() { 0.0 } else { 1.0 },
                    ),
                    span: Vec4::new(-l.below_m, l.above_m, 0.0, 0.0),
                }
            })
            .collect()
    }
}

/// How far a field-lit thing has faded out, 0 drawn whole to 1 gone, on it
/// or on its parent: a town standing up or dropped at the edge of its range
/// (`cities-in-the-world` slice 4a). It is drawn through the terrain's own
/// screen-door mask, so it never appears or vanishes in one frame.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq)]
pub struct Faded(pub f32);

/// How a room's faces are lit where the field cannot say (`sun-shadows`
/// decision 7): the voxel field has never heard of a house's walls.
#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub struct SkyShare {
    /// The share of the sky that reaches it, 0..1.
    pub sky: f32,
    /// How much of the sun that came in by its door and windows bounces
    /// round it, warm, times the shader's `ROOM_BOUNCE`: zero outdoors.
    pub bounce: f32,
}

/// One field-lit mesh's own row of `field_lit.wgsl`'s `rows`, found by its
/// [`MeshTag`] (`crab-breeze-lessons` rank 2): what differs from one mesh to
/// the next. What is the same for every mesh is in [`FieldView`].
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FieldRow {
    /// The mesh's bounds in the render frame.
    pub low: Vec4,
    pub high: Vec4,
    /// x how far it has faded out ([`Faded`]).
    pub fade: Vec4,
    /// The field at the bounds' eight corners, corner `x + 2y + 4z`.
    pub sky: [Vec4; 2],
    pub block: [Vec4; 2],
    /// x one where it is lit as the terrain is ([`LitLikeTerrain`]); y the
    /// share of the sky that reaches it and z the sun bounced round a room
    /// ([`SkyShare`]; one and zero outdoors).
    pub look: Vec4,
    /// x the first of its room lights in `room_lights`, y how many.
    pub lights: Vec4,
}

/// One room light as `field_lit.wgsl` reads it ([`RoomLights::pack`]): where
/// (planet frame) and how far; colour times power, w one for a candle; the
/// band of height round it that it lights.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LightRow {
    pub at: Vec4,
    pub colour: Vec4,
    pub span: Vec4,
}

/// What every field-lit mesh reads alike, written once a frame.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FieldView {
    /// The planet's centre in the render frame.
    pub centre: Vec4,
    /// The way to the sun.
    pub sun: Vec4,
    /// `lighting-model`: x one for the new model, y the cover over the
    /// camera, z a lamp's glow, w the sun's apparent radius (L3).
    pub model: Vec4,
    /// The sky's light (L2), clear then under a full deck
    /// ([`crate::tone::SkyLight`]).
    pub sky_sh: [Vec4; 18],
}

/// Bytes of a run of vec4s as WGSL lays them out in storage: packed, 16 a
/// vec4, little-endian.
fn vec4_bytes<'a>(vec4s: impl IntoIterator<Item = &'a Vec4>) -> Vec<u8> {
    vec4s
        .into_iter()
        .flat_map(|v| v.to_array())
        .flat_map(f32::to_le_bytes)
        .collect()
}

impl FieldRow {
    const BYTES: u64 = 9 * 16;
    fn vec4s(&self) -> [Vec4; 9] {
        [
            self.low,
            self.high,
            self.fade,
            self.sky[0],
            self.sky[1],
            self.block[0],
            self.block[1],
            self.look,
            self.lights,
        ]
    }
}

impl LightRow {
    const BYTES: u64 = 3 * 16;
    fn vec4s(&self) -> [Vec4; 3] {
        [self.at, self.colour, self.span]
    }
}

impl FieldView {
    const BYTES: u64 = 21 * 16;
    fn bytes(&self) -> Vec<u8> {
        vec4_bytes(
            [&self.centre, &self.sun, &self.model]
                .into_iter()
                .chain(self.sky_sh.iter()),
        )
    }
}

/// The extension that lights a PBR material from the field, and shades it
/// with the sun's cascades (`sun-shadows` decision 4). One is shared by every
/// field-lit mesh of a base material (`crab-breeze-lessons` rank 2): its
/// buffers are written in place each frame, so the material itself never
/// changes and the meshes under it batch.
#[derive(Asset, AsBindGroup, Reflect, Debug, Clone, Default)]
pub struct FieldLit {
    #[storage(100, read_only)]
    pub view: Handle<ShaderStorageBuffer>,
    #[texture(101, sample_type = "depth", dimension = "2d_array")]
    #[sampler(102, sampler_type = "comparison")]
    pub sun_map: Handle<Image>,
    #[storage(103, read_only)]
    pub sun_cascades: Handle<ShaderStorageBuffer>,
    #[storage(104, read_only)]
    pub rows: Handle<ShaderStorageBuffer>,
    #[storage(105, read_only)]
    pub room_lights: Handle<ShaderStorageBuffer>,
}

impl MaterialExtension for FieldLit {
    fn fragment_shader() -> ShaderRef {
        "shaders/field_lit.wgsl".into()
    }
}

/// A PBR material lit by the field.
pub type FieldLitMaterial = ExtendedMaterial<StandardMaterial, FieldLit>;

/// Rows the buffers are made with; they double when outgrown, and every
/// shared material is then given the new buffer.
const ROWS_START: usize = 4096;
const LIGHTS_START: usize = 8192;

/// Where one field-lit mesh's data lives: its row, and the run of room
/// lights it may fill.
#[derive(Clone, Copy, Debug)]
struct Slot {
    row: u32,
    first: u32,
    cap: u32,
}

/// What changed this frame, for the render world to write
/// (`crab-breeze-lessons` rank 2). Emptied as the next frame's field systems
/// start.
#[derive(Resource, Clone, Default)]
pub struct FieldUploads {
    view: FieldView,
    rows: Vec<(u32, FieldRow)>,
    lights: Vec<(u32, LightRow)>,
    view_buffer: Handle<ShaderStorageBuffer>,
    rows_buffer: Handle<ShaderStorageBuffer>,
    lights_buffer: Handle<ShaderStorageBuffer>,
}

/// Every field-lit mesh's row and room lights, as the CPU holds them, and the
/// one material each base material is drawn with (`crab-breeze-lessons`
/// rank 2).
#[derive(Resource, Default)]
pub struct FieldRows {
    rows: Vec<FieldRow>,
    free: Vec<u32>,
    lights: Vec<LightRow>,
    slots: std::collections::HashMap<Entity, Slot>,
    shared: std::collections::HashMap<AssetId<StandardMaterial>, Handle<FieldLitMaterial>>,
    row_capacity: usize,
    light_capacity: usize,
    uploads: FieldUploads,
}

/// A zeroed storage buffer the render world writes into.
fn storage_buffer(bytes: u64) -> ShaderStorageBuffer {
    let mut buffer = ShaderStorageBuffer::new(
        &vec![0u8; bytes as usize],
        bevy::asset::RenderAssetUsages::RENDER_WORLD,
    );
    buffer.buffer_description.usage |= BufferUsages::COPY_DST;
    buffer
}

impl FieldRows {
    fn new(buffers: &mut Assets<ShaderStorageBuffer>) -> Self {
        Self {
            row_capacity: ROWS_START,
            light_capacity: LIGHTS_START,
            uploads: FieldUploads {
                view_buffer: buffers.add(storage_buffer(FieldView::BYTES)),
                rows_buffer: buffers.add(storage_buffer(FieldRow::BYTES * ROWS_START as u64)),
                lights_buffer: buffers.add(storage_buffer(LightRow::BYTES * LIGHTS_START as u64)),
                ..default()
            },
            ..default()
        }
    }

    /// How many meshes hold a row now.
    pub fn len(&self) -> usize {
        self.slots.len()
    }

    /// Whether no mesh holds a row.
    pub fn is_empty(&self) -> bool {
        self.slots.is_empty()
    }

    /// How many materials the field-lit meshes are drawn with.
    pub fn materials(&self) -> usize {
        self.shared.len()
    }

    /// The row a mesh reads, if it has one.
    pub fn row(&self, entity: Entity) -> Option<FieldRow> {
        self.slots.get(&entity).map(|s| self.rows[s.row as usize])
    }

    /// The room lights a mesh reads.
    pub fn room_lights(&self, entity: Entity) -> &[LightRow] {
        self.slots.get(&entity).map_or(&[], |s| {
            let first = s.first as usize;
            let count = self.rows[s.row as usize].lights.y as usize;
            &self.lights[first..first + count]
        })
    }

    fn set_row(&mut self, row: u32, value: FieldRow) {
        self.rows[row as usize] = value;
        self.uploads.rows.push((row, value));
    }

    /// A row for a new mesh, the buffer doubled if it is full.
    fn take_row(&mut self, buffers: &mut Assets<ShaderStorageBuffer>) -> (u32, bool) {
        if let Some(row) = self.free.pop() {
            return (row, false);
        }
        let row = self.rows.len() as u32;
        self.rows.push(FieldRow::default());
        let grown = self.rows.len() > self.row_capacity;
        if grown {
            self.row_capacity = (self.row_capacity * 2).max(ROWS_START);
            self.uploads.rows_buffer =
                buffers.add(storage_buffer(FieldRow::BYTES * self.row_capacity as u64));
            // The new buffer starts empty: every row goes again.
            self.uploads.rows = (0..self.rows.len() as u32)
                .zip(self.rows.iter().copied())
                .collect();
        }
        (row, grown)
    }

    /// Room for `count` lights at the end of the lights, the buffer doubled
    /// (and the lights packed again) if they do not fit.
    fn take_lights(
        &mut self,
        count: u32,
        buffers: &mut Assets<ShaderStorageBuffer>,
    ) -> (u32, bool) {
        if self.lights.len() + count as usize > self.light_capacity {
            // Pack the live meshes' lights together, dropping what the gone
            // ones left, and double the room if that is not enough.
            let mut packed = Vec::with_capacity(self.lights.len());
            let mut slots: Vec<_> = self.slots.iter_mut().collect();
            slots.sort_by_key(|(_, s)| s.first);
            for (_, slot) in slots {
                let first = slot.first as usize;
                let first_new = packed.len() as u32;
                packed.extend_from_slice(&self.lights[first..first + slot.cap as usize]);
                slot.first = first_new;
                let mut row = self.rows[slot.row as usize];
                row.lights.x = first_new as f32;
                self.rows[slot.row as usize] = row;
                self.uploads.rows.push((slot.row, row));
            }
            self.lights = packed;
            while self.lights.len() + count as usize > self.light_capacity {
                self.light_capacity = (self.light_capacity * 2).max(LIGHTS_START);
            }
            self.uploads.lights_buffer =
                buffers.add(storage_buffer(LightRow::BYTES * self.light_capacity as u64));
            let first = self.lights.len() as u32;
            self.lights
                .resize(self.lights.len() + count as usize, LightRow::default());
            self.uploads.lights = (0..self.lights.len() as u32)
                .zip(self.lights.iter().copied())
                .collect();
            return (first, true);
        }
        let first = self.lights.len() as u32;
        self.lights
            .resize(self.lights.len() + count as usize, LightRow::default());
        (first, false)
    }

    fn set_lights(&mut self, first: u32, lights: &[LightRow]) {
        for (i, l) in lights.iter().enumerate() {
            let at = first + i as u32;
            self.lights[at as usize] = *l;
            self.uploads.lights.push((at, *l));
        }
    }

    /// The material a base material's meshes share, made the first time.
    fn shared(
        &mut self,
        base: &Handle<StandardMaterial>,
        standard: &StandardMaterial,
        shadows: &crate::planet::shadow::SunShadowMaps,
        lit: &mut Assets<FieldLitMaterial>,
    ) -> Handle<FieldLitMaterial> {
        let uploads = &self.uploads;
        self.shared
            .entry(base.id())
            .or_insert_with(|| {
                lit.add(ExtendedMaterial {
                    base: standard.clone(),
                    extension: FieldLit {
                        view: uploads.view_buffer.clone(),
                        sun_map: shadows.map.clone(),
                        sun_cascades: shadows.cascades.clone(),
                        rows: uploads.rows_buffer.clone(),
                        room_lights: uploads.lights_buffer.clone(),
                    },
                })
            })
            .clone()
    }

    /// Every shared material given the buffers again, after one has grown.
    fn rebind(&self, lit: &mut Assets<FieldLitMaterial>) {
        for handle in self.shared.values() {
            if let Some(material) = lit.get_mut(handle) {
                material.extension.rows = self.uploads.rows_buffer.clone();
                material.extension.room_lights = self.uploads.lights_buffer.clone();
            }
        }
    }
}

/// Lights the ship, the fish and the float from the field.
pub struct FieldLightPlugin;

impl Plugin for FieldLightPlugin {
    fn build(&self, app: &mut App) {
        let rows = app
            .world_mut()
            .get_resource_mut::<Assets<ShaderStorageBuffer>>()
            .map(|mut buffers| FieldRows::new(&mut buffers))
            .unwrap_or_default();
        app.add_plugins(MaterialPlugin::<FieldLitMaterial>::default())
            .insert_resource(rows)
            .add_systems(
                PostUpdate,
                (
                    take_the_field,
                    drop_the_field,
                    light_from_the_field,
                    repack_room_lights,
                )
                    .chain()
                    .after(bevy::transform::TransformSystems::Propagate)
                    .run_if(crate::world_open),
            )
            .add_systems(Update, bake_town_outsides)
            .add_systems(Update, fill_follows_the_day);
        if let Some(render) = app.get_sub_app_mut(bevy::render::RenderApp) {
            render
                .init_resource::<FieldMirror>()
                .add_systems(bevy::render::ExtractSchedule, extract_the_field)
                .add_systems(
                    bevy::render::Render,
                    write_the_field.in_set(bevy::render::RenderSystems::PrepareResources),
                );
        }
    }
}

/// Give each new mesh under a `LitByField` root the field-lit material its
/// base material's meshes share, and a row of its own, found by its
/// [`MeshTag`] (`crab-breeze-lessons` rank 2): each is lit at its own
/// bounds, and the builders never need to know.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn take_the_field(
    mut commands: Commands,
    added: Query<
        (
            Entity,
            &MeshMaterial3d<StandardMaterial>,
            Option<&RoomLights>,
        ),
        Added<MeshMaterial3d<StandardMaterial>>,
    >,
    roots: Query<(), With<LitByField>>,
    terrain_like: Query<(), With<LitLikeTerrain>>,
    parents: Query<&ChildOf>,
    standard: Res<Assets<StandardMaterial>>,
    shadows: Option<Res<crate::planet::shadow::SunShadowMaps>>,
    mut lit: ResMut<Assets<FieldLitMaterial>>,
    mut buffers: ResMut<Assets<ShaderStorageBuffer>>,
    mut rows: ResMut<FieldRows>,
) {
    let rows = &mut *rows;
    rows.uploads.rows.clear();
    rows.uploads.lights.clear();
    let Some(shadows) = shadows else {
        return;
    };
    let mut grown = false;
    for (entity, material, room_lights) in &added {
        let marked = std::iter::once(entity)
            .chain(parents.iter_ancestors(entity))
            .any(|at| roots.contains(at));
        if !marked {
            continue;
        }
        let Some(base) = standard.get(&material.0) else {
            continue;
        };
        // A flame draws its own light, unlit.
        if base.unlit {
            continue;
        }
        let like_terrain = std::iter::once(entity)
            .chain(parents.iter_ancestors(entity))
            .any(|at| terrain_like.contains(at));
        let handle = rows.shared(&material.0, base, &shadows, &mut lit);
        let lights = room_lights.map(RoomLights::pack).unwrap_or_default();
        // A mesh that carries room lights may gain some (a craft's lantern),
        // so it keeps room for all of them.
        let cap = if room_lights.is_some() {
            MAX_ROOM_LIGHTS as u32
        } else {
            0
        };
        let (row, row_grown) = rows.take_row(&mut buffers);
        let (first, lights_grown) = rows.take_lights(cap, &mut buffers);
        grown |= row_grown || lights_grown;
        rows.set_lights(first, &lights);
        rows.set_row(
            row,
            FieldRow {
                look: Vec4::new(if like_terrain { 1.0 } else { 0.0 }, 1.0, 0.0, 0.0),
                lights: Vec4::new(first as f32, lights.len() as f32, 0.0, 0.0),
                ..default()
            },
        );
        rows.slots.insert(entity, Slot { row, first, cap });
        commands
            .entity(entity)
            .remove::<MeshMaterial3d<StandardMaterial>>()
            .insert((MeshMaterial3d(handle), bevy::mesh::MeshTag(row)));
    }
    if grown {
        rows.rebind(&mut lit);
    }
}

/// Give back the rows of the meshes that are gone, and every so often the
/// shared materials whose base material is gone with them.
fn drop_the_field(
    mut gone: RemovedComponents<MeshMaterial3d<FieldLitMaterial>>,
    standard: Res<Assets<StandardMaterial>>,
    mut rows: ResMut<FieldRows>,
    mut frames: Local<u32>,
) {
    let rows = &mut *rows;
    for entity in gone.read() {
        if let Some(slot) = rows.slots.remove(&entity) {
            rows.free.push(slot.row);
        }
    }
    *frames = frames.wrapping_add(1);
    if *frames % 600 == 0 {
        rows.shared.retain(|base, _| standard.get(*base).is_some());
    }
}

/// The bounds of a mesh in the render frame, from its local box and where it
/// is; half a metre round its origin where it has no box yet.
fn world_bounds(transform: &GlobalTransform, aabb: Option<&Aabb>) -> (Vec3, Vec3) {
    let Some(aabb) = aabb else {
        let at = transform.translation();
        return (at - Vec3::splat(0.5), at + Vec3::splat(0.5));
    };
    let (centre, half) = (Vec3::from(aabb.center), Vec3::from(aabb.half_extents));
    let mut low = Vec3::splat(f32::MAX);
    let mut high = Vec3::splat(f32::MIN);
    for i in 0..8 {
        let sign = Vec3::new(
            if i & 1 == 0 { -1.0 } else { 1.0 },
            if i & 2 == 0 { -1.0 } else { 1.0 },
            if i & 4 == 0 { -1.0 } else { 1.0 },
        );
        let corner = transform.transform_point(centre + half * sign);
        low = low.min(corner);
        high = high.max(corner);
    }
    (low, high)
}

/// What a field-lit mesh's corners last read (`town-frame-time` decision 1):
/// the corners where it stood, the points they read (ground-clamped), the
/// field they were read from and what it said. The generator's ground is
/// asked again only when the mesh moves, the field only when it changes.
struct Corners {
    raw: [Vec3; 8],
    points: [Vec3; 8],
    key: (usize, u64, bool),
    sky: [Vec4; 2],
    block: [Vec4; 2],
    /// The frame it was last used in, so a despawned mesh's entry goes.
    seen: u32,
}

/// Sample the field at every field-lit mesh's eight corners, each frame
/// (`lamps-and-lanterns` decision 2), and write its row. A mesh that has not
/// moved, under a field that has not changed, keeps what its corners read
/// last (`town-frame-time` decision 1), and its row is written only when it
/// changes.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn light_from_the_field(
    fine: Option<Res<PlanetFine>>,
    contact: Option<Res<PlanetContact>>,
    sun: Option<Res<crate::sky::Sun>>,
    frame: Option<Res<crate::planet::PlanetRenderFrame>>,
    meshes: Query<
        (
            Entity,
            &GlobalTransform,
            Option<&Aabb>,
            Option<&SkyShare>,
            Option<&ChildOf>,
        ),
        With<MeshMaterial3d<FieldLitMaterial>>,
    >,
    faded: Query<&Faded>,
    sky_light: Option<Res<crate::tone::SkyLight>>,
    mut rows: ResMut<FieldRows>,
    mut read: Local<(u32, std::collections::HashMap<Entity, Corners>)>,
) {
    let sky_light = sky_light.map(|s| *s).unwrap_or_default();
    let centre = frame
        .map(|frame| frame.center.as_vec3())
        .unwrap_or(Vec3::ZERO);
    let toward_sun = sun.map(|sun| sun.direction()).unwrap_or(Vec3::Y);
    let rows = &mut *rows;
    rows.uploads.view = FieldView {
        centre: centre.extend(0.0),
        sun: toward_sun.extend(0.0),
        model: Vec4::new(
            if sky_light.new_model { 1.0 } else { 0.0 },
            sky_light.cover,
            sky_light.lamp_glow,
            sky_light.shadow_spread,
        ),
        sky_sh: sky_light.sh,
    };
    let key = match (&fine, &contact) {
        (Some(fine), Some(contact)) => (
            std::sync::Arc::as_ptr(&fine.set) as usize,
            fine.set.columns.light_version(),
            contact.serves(&fine.set),
        ),
        _ => (0, 0, false),
    };
    let (count, cache) = &mut *read;
    *count = count.wrapping_add(1);
    let now = *count;
    // MEASUREMENT (`town-frame-time`): this system's time and how many rows
    // it rewrote, each frame it passes 2 ms.
    let started = std::time::Instant::now();
    let (mut seen, mut rewritten) = (0usize, 0usize);
    for (entity, transform, aabb, share, parent) in &meshes {
        let Some(slot) = rows.slots.get(&entity).copied() else {
            continue;
        };
        seen += 1;
        let current = rows.rows[slot.row as usize];
        let (low, high) = world_bounds(transform, aabb);
        let gone = parent
            .and_then(|p| faded.get(p.parent()).ok())
            .map_or(0.0, |f| f.0.clamp(0.0, 1.0));
        let like_terrain = current.look.x > 0.5;
        let raw: [Vec3; 8] = std::array::from_fn(|i| {
            Vec3::new(
                if i & 1 == 0 { low.x } else { high.x },
                if i & 2 == 0 { low.y } else { high.y },
                if i & 4 == 0 { low.z } else { high.z },
            ) - centre
        });
        let entry = cache.entry(entity).or_insert_with(|| Corners {
            // NaN is never near, so the first frame reads everything.
            raw: [Vec3::NAN; 8],
            points: raw,
            key,
            sky: [Vec4::ONE; 2],
            block: [Vec4::ZERO; 2],
            seen: now,
        });
        entry.seen = now;
        let still = entry
            .raw
            .iter()
            .zip(&raw)
            .all(|(a, b)| (*a - *b).abs().max_element() <= 1e-3);
        if !still {
            entry.raw = raw;
            entry.points = raw.map(|mut at| {
                // A town's mesh spans the town, and its box, square to the
                // render frame's axes, reaches into the ground wherever the
                // town's up is tilted from them: at Theringford two corners
                // of every piece's box read the rock's dark
                // (`cities-in-the-world` slice 4a). A corner under the ground
                // reads the field just over it instead: the town stands on
                // the ground, never in it.
                if like_terrain && let Some(up) = at.try_normalize() {
                    let ground = PLANET_RADIUS
                        + pbd_core::column::surface_m(crate::planet::terrain_config(), up)
                        + GROUND_CLEAR_M;
                    if at.length() < ground {
                        at = up * ground;
                    }
                }
                at
            });
        }
        if !still || entry.key != key {
            entry.key = key;
            for (i, at) in entry.points.iter().enumerate() {
                let (sky, block) = match (&fine, &contact) {
                    (Some(fine), Some(contact)) => field_at(fine, contact, *at),
                    _ => (1.0, 0.0),
                };
                entry.sky[i / 4][i % 4] = sky;
                entry.block[i / 4][i % 4] = block;
            }
        }
        let mut row = FieldRow {
            low: low.extend(0.0),
            high: high.extend(0.0),
            fade: Vec4::new(gone, 0.0, 0.0, 0.0),
            sky: entry.sky,
            block: entry.block,
            look: current.look,
            lights: current.lights,
        };
        row.look.y = share.map_or(1.0, |s| s.sky);
        row.look.z = share.map_or(0.0, |s| s.bounce);
        let moved = (current.low - row.low).abs().max_element() > 1e-3
            || current.look != row.look
            || current.fade != row.fade
            || (current.high - row.high).abs().max_element() > 1e-3
            || (0..2).any(|k| {
                (current.sky[k] - row.sky[k]).abs().max_element() > 1e-3
                    || (current.block[k] - row.block[k]).abs().max_element() > 1e-3
            });
        if moved {
            rows.set_row(slot.row, row);
            rewritten += 1;
        }
    }
    // Every so often, forget the meshes that are gone.
    if now % 600 == 0 {
        cache.retain(|_, c| c.seen == now);
    }
    crate::planet::lod::spent(
        &format!("field corners: {seen} meshes, {rewritten} rows rewritten"),
        started,
    );
}

/// Pack a mesh's room lights again when they change: a light a craft carries
/// moves with it (`sail-the-cog` step 3, part 4).
fn repack_room_lights(
    meshes: Query<
        (Entity, &RoomLights),
        (Changed<RoomLights>, With<MeshMaterial3d<FieldLitMaterial>>),
    >,
    mut rows: ResMut<FieldRows>,
) {
    for (entity, lights) in &meshes {
        let Some(slot) = rows.slots.get(&entity).copied() else {
            continue;
        };
        let mut packed = lights.pack();
        packed.truncate(slot.cap as usize);
        rows.set_lights(slot.first, &packed);
        let mut row = rows.rows[slot.row as usize];
        if row.lights.y != packed.len() as f32 {
            row.lights.y = packed.len() as f32;
            rows.set_row(slot.row, row);
        }
    }
}

/// The render world's copy of every row, light and the view, and how much of
/// each the GPU has not been given yet.
#[derive(Resource, Default)]
struct FieldMirror {
    view: FieldView,
    rows: Vec<FieldRow>,
    lights: Vec<LightRow>,
    dirty_rows: Option<(u32, u32)>,
    dirty_lights: Option<(u32, u32)>,
    buffers: FieldUploads,
    /// The GPU buffers last written, so a new one is written whole.
    written: [Option<bevy::render::render_resource::BufferId>; 3],
}

fn widen(range: &mut Option<(u32, u32)>, at: u32) {
    *range = Some(range.map_or((at, at + 1), |(a, b)| (a.min(at), b.max(at + 1))));
}

/// Copy this frame's changes into the render world's mirror. The uploads
/// are emptied as the next frame's field systems start; while the world is
/// loading they are not, and the same rows are copied again, harmlessly.
fn extract_the_field(
    rows: bevy::render::Extract<Option<Res<FieldRows>>>,
    mut mirror: ResMut<FieldMirror>,
) {
    let Some(rows) = rows.as_ref() else {
        return;
    };
    let up = &rows.uploads;
    let mirror = &mut *mirror;
    mirror.view = up.view;
    mirror.buffers.view_buffer = up.view_buffer.clone();
    mirror.buffers.rows_buffer = up.rows_buffer.clone();
    mirror.buffers.lights_buffer = up.lights_buffer.clone();
    for (at, row) in &up.rows {
        let i = *at as usize;
        if mirror.rows.len() <= i {
            mirror.rows.resize(i + 1, FieldRow::default());
        }
        mirror.rows[i] = *row;
        widen(&mut mirror.dirty_rows, *at);
    }
    for (at, light) in &up.lights {
        let i = *at as usize;
        if mirror.lights.len() <= i {
            mirror.lights.resize(i + 1, LightRow::default());
        }
        mirror.lights[i] = *light;
        widen(&mut mirror.dirty_lights, *at);
    }
}

/// Write what changed into the GPU's buffers in place: the shared materials'
/// bind groups never change. A buffer the GPU has not seen before is
/// written whole.
fn write_the_field(
    mut mirror: ResMut<FieldMirror>,
    storage: Res<bevy::render::render_asset::RenderAssets<GpuShaderStorageBuffer>>,
    queue: Res<bevy::render::renderer::RenderQueue>,
) {
    let mirror = &mut *mirror;
    if let Some(gpu) = storage.get(&mirror.buffers.view_buffer) {
        queue.write_buffer(&gpu.buffer, 0, &mirror.view.bytes());
        mirror.written[0] = Some(gpu.buffer.id());
    }
    if let Some(gpu) = storage.get(&mirror.buffers.rows_buffer) {
        let whole = mirror.written[1] != Some(gpu.buffer.id());
        let range = if whole {
            Some((0, mirror.rows.len() as u32))
        } else {
            mirror.dirty_rows
        };
        if let Some((a, b)) = range {
            let b = b.min((gpu.buffer.size() / FieldRow::BYTES) as u32);
            if a < b {
                let vec4s: Vec<Vec4> = mirror.rows[a as usize..b as usize]
                    .iter()
                    .flat_map(FieldRow::vec4s)
                    .collect();
                let bytes = vec4_bytes(&vec4s);
                queue.write_buffer(&gpu.buffer, a as u64 * FieldRow::BYTES, &bytes);
            }
        }
        mirror.written[1] = Some(gpu.buffer.id());
        mirror.dirty_rows = None;
    }
    if let Some(gpu) = storage.get(&mirror.buffers.lights_buffer) {
        let whole = mirror.written[2] != Some(gpu.buffer.id());
        let range = if whole {
            Some((0, mirror.lights.len() as u32))
        } else {
            mirror.dirty_lights
        };
        if let Some((a, b)) = range {
            let b = b.min((gpu.buffer.size() / LightRow::BYTES) as u32);
            if a < b {
                let vec4s: Vec<Vec4> = mirror.lights[a as usize..b as usize]
                    .iter()
                    .flat_map(LightRow::vec4s)
                    .collect();
                let bytes = vec4_bytes(&vec4s);
                queue.write_buffer(&gpu.buffer, a as u64 * LightRow::BYTES, &bytes);
            }
        }
        mirror.written[2] = Some(gpu.buffer.id());
        mirror.dirty_lights = None;
    }
}

/// How far over the ground a town's buried bound reads the field, metres:
/// half a layer, in the air of the cell on top.
const GROUND_CLEAR_M: f32 = 0.5;

/// A town's outside mesh, lit from the field vertex by vertex
/// (`cities-in-the-world`, "Finding: the stair towers have no light"): the
/// fine set and the relight it was last baked from. Its mesh carries the
/// field's `(sky, block)` in its second UV channel, `-1` until baked.
/// What a town's outside was baked from, and the bake in progress
/// (`frame-budget` 4.3).
#[derive(Component, Default, Clone, Debug)]
pub struct BakedField {
    /// The field the vertices carry now.
    pub baked: Option<BakeSource>,
    /// The town's cap: its mean direction and angular radius, taken once.
    bounds: Option<(Vec3, f32)>,
    /// A bake being made a slice a frame, swapped in whole when done.
    pending: Option<PendingBake>,
}

/// Where a bake's light came from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BakeSource {
    /// No column of the fine set reaches the town: every vertex reads open
    /// sky, whichever set it is.
    Open,
    /// The set (by pointer) and its light version.
    Field(usize, u64),
}

#[derive(Clone, Debug)]
struct PendingBake {
    source: BakeSource,
    done: Vec<[f32; 2]>,
}

/// What a town's outside reads before it is baked: the shader takes the
/// corners where it finds this.
pub const UNBAKED: [f32; 2] = [-1.0, 0.0];

/// How far in front of its face a vertex reads the field, metres: in the
/// air the face looks into, not the wall it is on.
const BAKE_FRONT_M: f32 = 0.3;

/// The most main-thread time the bakes take in a frame. A vertex costs about
/// 0.45 µs (a height and a field sample), a walled town about 158,000 of them:
/// baked whole it was a 67-75 ms frame on every landing (measured 2026-10-28).
/// Sliced, a town is baked over about 35 frames and swapped in whole.
const BAKE_BUDGET: std::time::Duration = std::time::Duration::from_micros(2_000);

/// How many vertices a slice bakes between looks at the clock.
const BAKE_SLICE: usize = 512;

/// How far past a town's cap a column of the set can still light it: two
/// tile widths and the read in front of a face.
const BAKE_REACH_M: f32 = 6.0 + BAKE_FRONT_M;

/// Each vertex's `(sky, block)`: the field at the point `BAKE_FRONT_M` in
/// front of it along its normal, positions and normals in the planet's
/// frame. `field` answers a point.
pub fn bake_vertices(
    positions: &[[f32; 3]],
    normals: &[[f32; 3]],
    field: impl Fn(Vec3) -> (f32, f32),
) -> Vec<[f32; 2]> {
    positions
        .iter()
        .zip(normals)
        .map(|(p, n)| {
            let (sky, block) = field(Vec3::from(*p) + Vec3::from(*n) * BAKE_FRONT_M);
            [sky, block]
        })
        .collect()
}

/// A town's cap from its vertices: their mean direction and the widest angle
/// from it.
fn cap_of(positions: &[[f32; 3]]) -> Option<(Vec3, f32)> {
    let mean = positions
        .iter()
        .fold(Vec3::ZERO, |sum, p| {
            sum + Vec3::from(*p).normalize_or_zero()
        })
        .try_normalize()?;
    let widest = positions
        .iter()
        .map(|p| mean.dot(Vec3::from(*p).normalize_or(mean)).clamp(-1.0, 1.0))
        .fold(1.0f32, f32::min)
        .acos();
    Some((mean, widest))
}

/// Bake every town's outside from the field whenever the light it reads can
/// have changed: a fine set that reaches it landing, a dig, a lamp, dusk and
/// dawn. A town no column of the set reaches reads open sky from any set and
/// is baked open once, not again on every landing (`frame-budget` 4.3). A
/// bake that reads the field is made a slice a frame, under `BAKE_BUDGET`,
/// and swapped in whole, so the town never shows half of each. A point under
/// the ground reads the air just over it, as a corner does.
fn bake_town_outsides(
    fine: Option<Res<PlanetFine>>,
    contact: Option<Res<PlanetContact>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut outsides: Query<(&Mesh3d, &mut BakedField)>,
) {
    let (Some(fine), Some(contact)) = (fine, contact) else {
        return;
    };
    if !contact.serves(&fine.set) {
        return;
    }
    let key = BakeSource::Field(
        std::sync::Arc::as_ptr(&fine.set) as usize,
        fine.set.columns.light_version(),
    );
    let config = crate::planet::terrain_config();
    let field = |at: Vec3| {
        let Some(up) = at.try_normalize() else {
            return (1.0, 0.0);
        };
        let ground = PLANET_RADIUS + pbd_core::column::surface_m(config, up) + GROUND_CLEAR_M;
        field_at(
            &fine,
            &contact,
            if at.length() < ground {
                up * ground
            } else {
                at
            },
        )
    };
    let started = std::time::Instant::now();
    let mut baked_vertices = 0usize;
    for (mesh, mut state) in &mut outsides {
        let state = &mut *state;
        // What this town's light comes from under this set.
        if state.bounds.is_none() {
            let Some(m) = meshes.get(&mesh.0) else {
                continue;
            };
            let Some(positions) = m
                .attribute(Mesh::ATTRIBUTE_POSITION)
                .and_then(|a| a.as_float3())
            else {
                continue;
            };
            state.bounds = cap_of(positions);
        }
        let Some((centre, angle)) = state.bounds else {
            continue;
        };
        let want = if fine
            .set
            .columns
            .reaches(centre, angle + BAKE_REACH_M / PLANET_RADIUS)
        {
            key
        } else {
            BakeSource::Open
        };
        if state.baked == Some(want) {
            state.pending = None;
            continue;
        }
        let Some(m) = meshes.get_mut(&mesh.0) else {
            continue;
        };
        let Some(count) = m.attribute(Mesh::ATTRIBUTE_POSITION).map(|a| a.len()) else {
            continue;
        };
        if want == BakeSource::Open {
            // Open sky everywhere: no field to read, nothing to slice.
            m.insert_attribute(Mesh::ATTRIBUTE_UV_1, vec![[1.0f32, 0.0]; count]);
            state.baked = Some(BakeSource::Open);
            state.pending = None;
            continue;
        }
        if started.elapsed() >= BAKE_BUDGET {
            continue;
        }
        let (Some(positions), Some(normals)) = (
            m.attribute(Mesh::ATTRIBUTE_POSITION)
                .and_then(|a| a.as_float3()),
            m.attribute(Mesh::ATTRIBUTE_NORMAL)
                .and_then(|a| a.as_float3()),
        ) else {
            continue;
        };
        if state.pending.as_ref().is_none_or(|p| p.source != want) {
            state.pending = Some(PendingBake {
                source: want,
                done: Vec::with_capacity(count),
            });
        }
        let Some(pending) = state.pending.as_mut() else {
            continue;
        };
        while pending.done.len() < count && started.elapsed() < BAKE_BUDGET {
            let from = pending.done.len();
            let to = (from + BAKE_SLICE).min(count);
            pending.done.extend(bake_vertices(
                &positions[from..to],
                &normals[from..to],
                field,
            ));
            baked_vertices += to - from;
        }
        if pending.done.len() == count {
            let done = std::mem::take(&mut pending.done);
            state.pending = None;
            m.insert_attribute(Mesh::ATTRIBUTE_UV_1, done);
            state.baked = Some(want);
        }
    }
    if baked_vertices > 0 {
        crate::planet::lod::spent(&format!("town bake: {baked_vertices} vertices"), started);
    }
}

/// Bevy's own ambient, what lights a PBR face turned from the sun, is the
/// terrain's cap fill for the hour where the camera is (`sun-shadows`
/// decision 7): a shaded wall of a ship is lit as a shaded cliff is, not by
/// Bevy's default. Divided by the camera's exposure, which Bevy multiplies
/// every light by.
fn fill_follows_the_day(
    sun: Option<Res<crate::sky::Sun>>,
    frame: Option<Res<crate::planet::PlanetRenderFrame>>,
    cameras: Query<(&Camera, &GlobalTransform, Option<&bevy::camera::Exposure>), With<Camera3d>>,
    ambient: Option<ResMut<GlobalAmbientLight>>,
) {
    let (Some(sun), Some(frame), Some(mut ambient)) = (sun, frame, ambient) else {
        return;
    };
    let Some((_, at, exposure)) = cameras.iter().find(|(camera, ..)| camera.is_active) else {
        return;
    };
    let up = (at.translation().as_dvec3() - frame.center).as_vec3();
    let fill = light::sky_fill(1.0, sun.clock.daylight(up));
    let [r, g, b] = light::SKY_FILL;
    let brightness = fill / exposure.copied().unwrap_or_default().exposure();
    if (ambient.brightness - brightness).abs() > 1e-3 * brightness.max(1.0) {
        ambient.color = Color::linear_rgb(r, g, b);
        ambient.brightness = brightness;
    }
}

/// Both channels of the field at a point in the planet's frame, 0..1
/// `(sky, block)`. Off the tier, or while the contact serves another set, it
/// answers the open sky and no lamp, which is what the far terrain is drawn
/// with.
pub fn field_at(fine: &PlanetFine, contact: &PlanetContact, point: Vec3) -> (f32, f32) {
    let Some(direction) = point.try_normalize() else {
        return (1.0, 0.0);
    };
    if !contact.serves(&fine.set) {
        return (1.0, 0.0);
    }
    let record = contact.finest_cell(direction);
    fine.set
        .columns
        .sample(record, direction, point.length() - PLANET_RADIUS)
}

/// The light a sample lays over a white surface, linear RGB: the sky's fill
/// for how much of it is day where the point is, and the lamps' warm light
/// where the sun does not drown it (`lamps-and-lanterns` decision 15).
pub fn light_of((sky, block): (f32, f32), daylight: f32) -> Vec3 {
    let fill = light::sky_fill(sky, daylight);
    let lamp = light::lamp_light(block);
    Vec3::splat(fill) + Vec3::from_array(lamp) * light::lamp_share(sky, daylight)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `cities-in-the-world`, "Finding: the stair towers have no light": a
    /// baked vertex reads the field in the air in front of its face, so a
    /// plank beside a lamp is lit and one 10 m off is not, and a face turned
    /// toward the lamp reads nearer it than one turned away.
    #[test]
    fn a_town_vertex_bakes_the_field_in_front_of_its_face() {
        let lamp = Vec3::new(0.0, 1.0, 0.0);
        // A made-up field: the lamp's light falls off over 6 m.
        let field = |p: Vec3| (1.0, (1.0 - p.distance(lamp) / 6.0).max(0.0));
        let positions = [
            [1.0, 0.0, 0.0],
            [10.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
        ];
        let normals = [
            [0.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [-1.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
        ];
        let baked = bake_vertices(&positions, &normals, field);
        assert_eq!(baked.len(), 4);
        assert!(baked[0][1] > 0.5, "beside the lamp: {:?}", baked[0]);
        assert_eq!(baked[1][1], 0.0, "10 m off");
        assert!(
            baked[2][1] > baked[3][1],
            "the face toward the lamp reads nearer it: {:?} {:?}",
            baked[2],
            baked[3]
        );
        assert!(
            baked.iter().all(|b| b[0] == 1.0),
            "the sky as the field gives it"
        );
        const { assert!(UNBAKED[0] < 0.0, "unbaked reads the corners") };
    }

    /// A fading town is drawn through the terrain's own mask
    /// (`cities-in-the-world` slice 4a), so the two dissolve alike.
    #[test]
    fn a_fading_town_takes_the_terrains_mask() {
        // A checkout may carry either line ending (`core.autocrlf`).
        let town = include_str!("../../../assets/shaders/field_lit.wgsl").replace("\r\n", "\n");
        let terrain =
            include_str!("../../../assets/shaders/planet_surface.wgsl").replace("\r\n", "\n");
        let body = |s: &str| {
            let i = s.find("fn bayer4").expect("a bayer4");
            let j = s[i..].find("\n}\n").expect("its end");
            s[i..i + j].to_string()
        };
        assert_eq!(body(&town), body(&terrain));
        assert!(town.contains("if bayer4(in.position.xy) < field.fade.x {"));
    }

    /// `field_lit.wgsl` carries the terrain's light constants, and they are
    /// the core's: a ship and the ground it stands on are lit by one set of
    /// numbers.
    #[test]
    fn the_field_lit_shader_carries_the_terrain_light_constants() {
        let shader = include_str!("../../../assets/shaders/field_lit.wgsl").replace("\r\n", "\n");
        for line in [
            format!("const AMBIENT_FLOOR: f32 = {:.2};", light::AMBIENT_FLOOR),
            format!("const NIGHT_FILL: f32 = {:.2};", light::NIGHT_FILL),
            format!(
                "const TORCH_TINT: vec3<f32> = vec3<f32>({:.2}, {:.2}, {:.2});",
                light::TORCH_TINT[0],
                light::TORCH_TINT[1],
                light::TORCH_TINT[2]
            ),
            format!("const TORCH_GAIN: f32 = {:.2};", light::TORCH_GAIN),
            "    let g = f * (2.0 - f);\n    return g * g;".to_string(),
            // Decision 15 (survey L4): the lamp fades where the sun reaches,
            // the core's `light::lamp_share` written out.
            "    return 1.0 - clamp(daylight, 0.0, 1.0) * clamp(sky, 0.0, 1.0);".to_string(),
            // ... over a town's piece by its room's share of the sky too.
            "* lamp_share(sky * field.look.y, daylight);".to_string(),
            "let daylight = smoothstep(-0.13, 0.20, elevation);".to_string(),
            // `sun-shadows` decision 5: the sun ends at the horizon, the
            // terrain's own curve for its direct term.
            "let sunlight = smoothstep(-0.0145, 0.02, elevation);".to_string(),
            // Decision 14 and `sun-shadows` decision 4: the sun only where it
            // is up, the sky reaches and the cascades see it.
            "let sun_up = sunlight * sky * shadow;".to_string(),
            // `sun-shadows` decision 7: a town is lit as the terrain is, by
            // its numbers.
            "const SUN_TINT: vec3<f32> = vec3<f32>(1.12, 1.03, 0.87);".to_string(),
            "const WALL_FILL: vec3<f32> = vec3<f32>(0.30, 0.32, 0.34);".to_string(),
            "const WALL_NIGHT: f32 = 0.20;".to_string(),
            "const WALL_GAIN: f32 = 0.95;".to_string(),
            format!(
                "const SKY_FILL: vec3<f32> = vec3<f32>({:.2}, {:.2}, {:.2});",
                light::SKY_FILL[0],
                light::SKY_FILL[1],
                light::SKY_FILL[2]
            ),
            "var ambient = base * SKY_FILL * max(AMBIENT_FLOOR, mix(NIGHT_FILL, 1.0, daylight) * sky);"
                .to_string(),
            "out.color.rgb * sun_up + ambient * (1.0 - sun_up) + mirrored + lamp".to_string(),
            // `crab-breeze-lessons` rank 2: a mesh's own row, found by its
            // tag, and what every mesh reads alike.
            "@binding(104) var<storage, read> rows: array<Field>;".to_string(),
            "field = rows[mesh_functions::get_tag(in.instance_index)];".to_string(),
            "@binding(100) var<storage, read> scene: Scene;".to_string(),
            // `cities-in-the-world` decision 7a: a building's own lights, as
            // many as a row carries, lit the mockup's way.
            "@binding(105) var<storage, read> room: array<RoomLight>;".to_string(),
            format!("let count = min(u32(field.lights.y), {MAX_ROOM_LIGHTS}u);"),
            "q * q * (0.3 + 0.7 * max(facing, 0.0)) * d * d / (d * d + 0.36)".to_string(),
            "let fire = 0.9 + 0.3 * night;".to_string(),
            "let candle = 1.8 * clamp((night - 0.25) / 0.35, 0.0, 1.0);".to_string(),
        ] {
            assert!(
                shader.contains(&line),
                "field_lit.wgsl should carry `{line}`"
            );
        }
        // A town's outside reads the field baked into its vertices, where
        // it has been baked (`BakedField`).
        assert!(
            shader.contains("if in.uv_b.x >= 0.0 {")
                && shader.contains("sky = in.uv_b.x;")
                && shader.contains("block = in.uv_b.y;"),
            "field_lit.wgsl reads the baked field"
        );
        // A room face turned away from a building's light still takes its
        // 0.3 share (`cities-in-the-world`, "Finding: the stair towers have
        // no light").
        assert!(
            !shader.contains("if facing < -0.15"),
            "a room light's backside takes the 0.3 share"
        );
        // The fill's colour is the terrain's cap fill, which that shader
        // writes without spaces.
        let terrain = include_str!("../../../assets/shaders/planet_surface.wgsl");
        let fill = format!(
            "var fill = vec3({:.2},{:.2},{:.2});",
            light::SKY_FILL[0],
            light::SKY_FILL[1],
            light::SKY_FILL[2]
        );
        assert!(
            terrain.contains(&fill),
            "planet_surface.wgsl should carry `{fill}`"
        );
        // And its wall's fill, night and gain, and its sun's tint and curve,
        // which a town takes as its own.
        for line in [
            "fill = vec3(0.30,0.32,0.34);",
            "night = 0.20;",
            "gain = 0.95;",
            "vec3(1.12,1.03,0.87)*direct",
            "let sunlight = smoothstep(-0.0145,0.02,sun_elevation);",
        ] {
            assert!(
                terrain.contains(line),
                "planet_surface.wgsl should carry `{line}`"
            );
        }
    }

    /// `sun-shadows` decision 7: Bevy's ambient, what lights a PBR face
    /// turned from the sun, is the terrain's cap fill for the hour where the
    /// camera stands: the whole of it at noon, the night's share at midnight.
    #[test]
    fn a_face_turned_from_the_sun_takes_the_skys_fill() {
        let sun = crate::sky::Sun::default();
        let overhead = sun.direction();
        let mut world = World::new();
        world.insert_resource(sun);
        world.insert_resource(crate::planet::PlanetRenderFrame::default());
        world.insert_resource(GlobalAmbientLight::default());
        let camera = world
            .spawn((
                Camera3d::default(),
                GlobalTransform::from_translation(overhead * PLANET_RADIUS),
            ))
            .id();
        let mut follow = IntoSystem::into_system(fill_follows_the_day);
        follow.initialize(&mut world);
        follow.run((), &mut world).unwrap();
        let exposure = bevy::camera::Exposure::default().exposure();
        let at_noon = world.resource::<GlobalAmbientLight>().clone();
        assert!(
            (at_noon.brightness * exposure - 1.0).abs() < 1e-4,
            "{at_noon:?}"
        );
        assert_eq!(
            at_noon.color.to_linear().to_f32_array_no_alpha(),
            light::SKY_FILL
        );
        world
            .entity_mut(camera)
            .insert(GlobalTransform::from_translation(-overhead * PLANET_RADIUS));
        follow.run((), &mut world).unwrap();
        let at_midnight = world.resource::<GlobalAmbientLight>().brightness * exposure;
        assert!(
            (at_midnight - light::NIGHT_FILL).abs() < 1e-4,
            "{at_midnight}"
        );
    }

    /// The rows, lights and view the CPU writes are laid out as
    /// `field_lit.wgsl` reads them: the same number of vec4s, read off the
    /// shader's own structs (`crab-breeze-lessons` rank 2).
    #[test]
    fn the_rows_are_the_shaders_structs() {
        let shader = include_str!("../../../assets/shaders/field_lit.wgsl").replace("\r\n", "\n");
        let vec4s = |name: &str| -> u64 {
            let start = shader
                .find(&format!("struct {name} {{"))
                .unwrap_or_else(|| panic!("struct {name}"));
            let body = &shader[start..start + shader[start..].find("\n}").expect("its end")];
            body.lines()
                .map(|l| l.split("//").next().unwrap_or("").trim())
                .filter(|l| l.contains("vec4<f32>"))
                .map(|l| match l.split("vec4<f32>, ").nth(1) {
                    Some(n) => n.trim_end_matches(">,").parse::<u64>().expect("a length"),
                    None => 1,
                })
                .sum()
        };
        assert_eq!(vec4s("Field") * 16, FieldRow::BYTES);
        assert_eq!(
            FieldRow::default().vec4s().len() as u64 * 16,
            FieldRow::BYTES
        );
        assert_eq!(vec4s("RoomLight") * 16, LightRow::BYTES);
        assert_eq!(vec4s("Scene") * 16, FieldView::BYTES);
        assert_eq!(FieldView::default().bytes().len() as u64, FieldView::BYTES);
    }

    /// Meshes of one base material share one field-lit material, each with
    /// its own row by its tag; a gone mesh's row is given to the next; and a
    /// mesh fading writes its row, never a material (`crab-breeze-lessons`
    /// rank 2).
    #[test]
    fn meshes_of_one_material_share_it_and_keep_their_own_rows() {
        let mut world = World::new();
        let mut standard = Assets::<StandardMaterial>::default();
        let stone = standard.add(StandardMaterial::default());
        let timber = standard.add(StandardMaterial::default());
        world.insert_resource(standard);
        world.insert_resource(Assets::<FieldLitMaterial>::default());
        let mut buffers = Assets::<ShaderStorageBuffer>::default();
        let rows = FieldRows::new(&mut buffers);
        world.insert_resource(buffers);
        world.insert_resource(rows);
        world.insert_resource(crate::planet::shadow::SunShadowMaps {
            map: Handle::default(),
            cascades: Handle::default(),
        });
        let root = world
            .spawn((LitByField, LitLikeTerrain, Transform::default()))
            .id();
        let piece = |world: &mut World, material: &Handle<StandardMaterial>| {
            let e = world
                .spawn((
                    MeshMaterial3d(material.clone()),
                    Transform::default(),
                    GlobalTransform::default(),
                ))
                .id();
            world.entity_mut(root).add_child(e);
            e
        };
        let a = piece(&mut world, &stone);
        let b = piece(&mut world, &stone);
        let c = piece(&mut world, &timber);
        let mut take = IntoSystem::into_system(take_the_field);
        take.initialize(&mut world);
        take.run((), &mut world).unwrap();
        world.flush();
        let material = |world: &World, e: Entity| {
            world
                .get::<MeshMaterial3d<FieldLitMaterial>>(e)
                .expect("field-lit")
                .0
                .clone()
        };
        let tag = |world: &World, e: Entity| world.get::<bevy::mesh::MeshTag>(e).expect("a tag").0;
        assert_eq!(
            material(&world, a),
            material(&world, b),
            "one material for one base"
        );
        assert_ne!(
            material(&world, a),
            material(&world, c),
            "another for another"
        );
        assert_eq!(world.resource::<Assets<FieldLitMaterial>>().len(), 2);
        assert_ne!(tag(&world, a), tag(&world, b), "a row each");
        let rows = world.resource::<FieldRows>();
        assert_eq!((rows.len(), rows.materials()), (3, 2));
        assert_eq!(
            rows.row(a).expect("a's row").look.x,
            1.0,
            "lit as the terrain"
        );

        // A fade reaches the row, and no material is touched.
        world.entity_mut(root).insert(Faded(0.5));
        let before: Vec<_> = world
            .resource::<Assets<FieldLitMaterial>>()
            .iter()
            .map(|(id, m)| (id, m.extension.rows.id()))
            .collect();
        let mut light = IntoSystem::into_system(light_from_the_field);
        light.initialize(&mut world);
        light.run((), &mut world).unwrap();
        let rows = world.resource::<FieldRows>();
        assert_eq!(rows.row(a).expect("a's row").fade.x, 0.5);
        assert!(rows.uploads.rows.iter().any(|(r, _)| *r == tag(&world, a)));
        let after: Vec<_> = world
            .resource::<Assets<FieldLitMaterial>>()
            .iter()
            .map(|(id, m)| (id, m.extension.rows.id()))
            .collect();
        assert_eq!(before, after);

        // A gone mesh's row goes to the next.
        let freed = tag(&world, b);
        world.despawn(b);
        let mut drop = IntoSystem::into_system(drop_the_field);
        drop.initialize(&mut world);
        drop.run((), &mut world).unwrap();
        let d = piece(&mut world, &timber);
        take.run((), &mut world).unwrap();
        world.flush();
        assert_eq!(tag(&world, d), freed);
        assert_eq!(world.resource::<FieldRows>().len(), 3);
    }

    /// A building with more lights than the shader reads keeps its fires
    /// and drops candles (`cities-in-the-world` decision 7a).
    #[test]
    fn room_lights_pack_fires_first_and_no_more_than_the_shader_reads() {
        use pbd_core::settlement::pieces::{LightKind, RoomLight};
        let light = |kind, x: f32| RoomLight {
            kind,
            at: Vec3::new(x, 0.0, 0.0),
            power: 1.0,
            below_m: 1.0,
            above_m: 2.0,
        };
        // Twenty candles first, then ten fires: thirty, more than fit.
        let lights: Vec<RoomLight> = (0..20)
            .map(|i| light(LightKind::Candle, i as f32))
            .chain((0..10).map(|i| light(LightKind::Hearth, 100.0 + i as f32)))
            .collect();
        let packed = RoomLights(lights).pack();
        assert_eq!(packed.len(), MAX_ROOM_LIGHTS);
        assert!(
            packed[..10].iter().all(|l| l.colour.w == 0.0),
            "the fires first"
        );
        assert!(
            packed[10..].iter().all(|l| l.colour.w == 1.0),
            "then candles"
        );
    }

    /// A box's bounds in the render frame hold all eight of its corners,
    /// turned and moved.
    #[test]
    fn a_turned_box_is_bounded_by_its_corners() {
        let aabb = Aabb::from_min_max(Vec3::new(-1.0, -0.5, -2.0), Vec3::new(1.0, 0.5, 2.0));
        let transform = GlobalTransform::from(
            Transform::from_xyz(10.0, 0.0, 0.0).with_rotation(Quat::from_rotation_y(0.5)),
        );
        let (low, high) = world_bounds(&transform, Some(&aabb));
        for corner in [
            Vec3::new(-1.0, -0.5, -2.0),
            Vec3::new(1.0, 0.5, 2.0),
            Vec3::new(1.0, -0.5, -2.0),
        ] {
            let at = transform.transform_point(corner);
            assert!(
                at.cmpge(low - 1e-4).all() && at.cmple(high + 1e-4).all(),
                "{at}"
            );
        }
    }

    /// The open sky at noon is the full day, unchanged; a sealed cave is the
    /// floor whatever the hour; night in the open is the night's fill; and a
    /// torch beside it adds warm light, more red than blue.
    #[test]
    fn the_sky_the_cave_the_night_and_a_torch() {
        assert_eq!(light_of((1.0, 0.0), 1.0), Vec3::ONE, "noon in the open");
        let cave = light_of((0.0, 0.0), 1.0);
        assert_eq!(cave, Vec3::splat(light::AMBIENT_FLOOR), "a cave at noon");
        assert_eq!(light_of((0.0, 0.0), 0.0), cave, "and at midnight");
        let night = light_of((1.0, 0.0), 0.0);
        assert_eq!(
            night,
            Vec3::splat(light::NIGHT_FILL),
            "midnight in the open"
        );
        let torch = light_of((1.0, 14.0 / 15.0), 0.0);
        assert!(torch.x > night.x * 5.0, "a torch lights the night: {torch}");
        assert!(torch.x > torch.z, "warm: {torch}");
    }
}
