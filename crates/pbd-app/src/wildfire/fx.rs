//! A fire's flames, embers and smoke, and its charred trees (`wildfire`
//! 4.2), drawn from [`FireNow`]. Derived, never saved.
//!
//! - **Flames** are the held torch's own pixel sheet on crossed cards, two
//!   to five a burning cell by what burns, flickering, dying down as the
//!   cell burns out; a crown fire's stand in the crown.
//! - **Embers** rise from each flame and drift with the wind.
//! - **Smoke** is a pixel puff sheet (`textures/fire/smoke.png`) rising and
//!   leaning downwind, lit by the field (so its underside glows over the
//!   flames at night and it is dark in the shade).
//! - **Snags** are the tree's own model (`tree_kit`): a crown fire's bark
//!   alone, charred; a ground fire's whole tree with its foot charred and
//!   its leaves browned; then a sapling of its species growing to full size
//!   by day 60. Lit by the field, as the terrain is.
//!
//! Every mesh is in the render frame about an anchor at the fire, so a far
//! body's f32 never holds a planet-sized number.

use super::{FireNow, Snag};
use crate::field_light::{LitByField, LitLikeTerrain};
use crate::planet::PlanetRenderFrame;
use bevy::asset::RenderAssetUsages;
use bevy::image::{ImageAddressMode, ImageSampler, ImageSamplerDescriptor};
use bevy::math::DVec3;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use pbd_core::fire::TreeLook;
use pbd_core::tree_kit::{self, Tex};

pub fn build(app: &mut App) {
    app.add_systems(
        Update,
        (looks, draw_flames, draw_snags)
            .chain()
            .after(super::show)
            .run_if(crate::world_open),
    );
    app.add_systems(
        Update,
        prewarm.run_if(resource_exists::<crate::loading::Warming>),
    );
}

/// The fire's materials.
#[derive(Resource, Clone)]
pub struct Looks {
    pub flame: Handle<StandardMaterial>,
    pub ember: Handle<StandardMaterial>,
    pub smoke: Handle<StandardMaterial>,
}

#[derive(Component)]
struct FlameMesh;
#[derive(Component)]
struct EmberMesh;
#[derive(Component)]
struct SmokeMesh;
#[derive(Component)]
struct SnagMesh;

/// The flame sheet's columns and rows (`held_light/flame.png`).
const SHEET: (f32, f32) = (8.0, 3.0);
const FLAME_FPS: f32 = 11.0;
const SMOKE_RISE_M: f32 = 14.0;
const SMOKE_LIFE_S: f32 = 7.0;
const EMBER_RISE_M: f32 = 5.0;
const EMBER_LIFE_S: f32 = 2.2;

fn image(bytes: &[u8], images: &mut Assets<Image>) -> Handle<Image> {
    let pixels = image::load_from_memory(bytes)
        .expect("a fire texture decodes")
        .to_rgba8();
    pixel_image(pixels, images)
}

fn pixel_image(pixels: image::RgbaImage, images: &mut Assets<Image>) -> Handle<Image> {
    let mut image = Image::from_dynamic(
        image::DynamicImage::ImageRgba8(pixels),
        true,
        RenderAssetUsages::default(),
    );
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::Repeat,
        address_mode_v: ImageAddressMode::Repeat,
        ..ImageSamplerDescriptor::nearest()
    });
    images.add(image)
}

fn looks(
    mut commands: Commands,
    existing: Option<Res<Looks>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    if existing.is_some() {
        return;
    }
    let looks = make_looks(&mut images, &mut materials);
    for (material, which) in [(looks.flame.clone(), 0), (looks.ember.clone(), 1)] {
        let mut e = commands.spawn((
            Mesh3d(meshes.add(empty())),
            MeshMaterial3d(material),
            Transform::default(),
            Visibility::Hidden,
            bevy::light::NotShadowCaster,
            Name::new(if which == 0 {
                "fire flames"
            } else {
                "fire embers"
            }),
        ));
        if which == 0 {
            e.insert(FlameMesh);
        } else {
            e.insert(EmberMesh);
        }
    }
    commands.spawn((
        Mesh3d(meshes.add(empty())),
        MeshMaterial3d(looks.smoke.clone()),
        Transform::default(),
        Visibility::Hidden,
        bevy::light::NotShadowCaster,
        LitByField,
        LitLikeTerrain,
        SmokeMesh,
        Name::new("fire smoke"),
    ));
    commands.insert_resource(looks);
}

fn make_looks(images: &mut Assets<Image>, materials: &mut Assets<StandardMaterial>) -> Looks {
    let unlit = |texture: Option<Handle<Image>>| StandardMaterial {
        base_color: Color::WHITE,
        base_color_texture: texture,
        unlit: true,
        cull_mode: None,
        double_sided: true,
        ..default()
    };
    let flame = materials.add(StandardMaterial {
        // Under the tonemapper's shoulder, as the held flame is, so it
        // reads orange and not white.
        base_color: Color::linear_rgb(0.75, 0.75, 0.75),
        alpha_mode: AlphaMode::Mask(0.5),
        ..unlit(Some(image(
            include_bytes!("../../../../assets/textures/held_light/flame.png"),
            images,
        )))
    });
    let ember = materials.add(unlit(None));
    let smoke = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        base_color_texture: Some(image(
            include_bytes!("../../../../assets/textures/fire/smoke.png"),
            images,
        )),
        alpha_mode: AlphaMode::Blend,
        perceptual_roughness: 1.0,
        reflectance: 0.0,
        cull_mode: None,
        double_sided: true,
        ..default()
    });
    Looks {
        flame,
        ember,
        smoke,
    }
}

fn empty() -> Mesh {
    Build::default().mesh()
}

/// Triangles with normals, uvs and vertex colours.
#[derive(Default)]
struct Build {
    p: Vec<[f32; 3]>,
    n: Vec<[f32; 3]>,
    uv: Vec<[f32; 2]>,
    c: Vec<[f32; 4]>,
    i: Vec<u32>,
}

impl Build {
    fn quad(&mut self, v: [Vec3; 4], n: Vec3, uv: [[f32; 2]; 4], c: [f32; 4]) {
        let base = self.p.len() as u32;
        for k in 0..4 {
            self.p.push(v[k].to_array());
            self.n.push(n.to_array());
            self.uv.push(uv[k]);
            self.c.push(c);
        }
        self.i
            .extend([base, base + 1, base + 2, base, base + 2, base + 3]);
    }

    fn mesh(self) -> Mesh {
        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
        );
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, self.p);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, self.n);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, self.uv);
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, self.c);
        mesh.insert_indices(Indices::U32(self.i));
        mesh
    }
}

/// One decision about one thing, 0..1.
fn roll(key: u32, salt: u32) -> f32 {
    tree_kit::roll(key, salt)
}

/// A card standing at `base` about `up`, turned to face `eye`, `w` wide and
/// `h` tall, leaning by `lean` at its top.
fn facing(base: Vec3, up: Vec3, eye: Vec3) -> (Vec3, Vec3) {
    let mut toward = eye - base;
    toward -= up * toward.dot(up);
    let toward = toward.normalize_or(Vec3::Z);
    (up.cross(toward).normalize_or(Vec3::X), toward)
}

/// The flames of one burning cell: tongues of the torch's flame sheet.
#[allow(clippy::too_many_arguments)]
fn flame_tongues(
    b: &mut Build,
    base: Vec3,
    up: Vec3,
    eye: Vec3,
    lean: Vec3,
    f: &super::Flame,
    t: f32,
) {
    use pbd_core::fire::Plant;
    let (count, size, lift) = match f.plant {
        Plant::Grass => (3, 1.0, 0.0),
        Plant::Shrub => (4, 1.5, 0.0),
        _ if f.crown => (5, 3.2, 3.5),
        _ => (4, 1.7, 0.0),
    };
    // Dying down as it burns out.
    let strength = (f.left / 0.3).clamp(0.15, 1.0);
    let tangent = up.any_orthonormal_vector();
    let bitangent = up.cross(tangent);
    for k in 0..count {
        let a = roll(f.key, 0x70 + k) * std::f32::consts::TAU;
        let r = 0.3 + 0.9 * roll(f.key, 0x80 + k);
        let at = base + (tangent * a.cos() + bitangent * a.sin()) * r + up * lift;
        let flick = crate::held::light::flicker(t, roll(f.key, 0x90 + k) * 10.0, 0.16, FLAME_FPS);
        let h = size * strength * (0.7 + 0.6 * roll(f.key, 0xa0 + k)) * flick;
        let w = h * 0.6;
        let (right, toward) = facing(at, up, eye);
        let frame = ((t * FLAME_FPS).floor() as i64 + i64::from(k) * 3).rem_euclid(8) as f32;
        let row = (k % 3) as f32;
        let uv = |x: f32, y: f32| {
            [
                (frame + (x + 0.5).clamp(0.002, 0.998)) / SHEET.0,
                (row + (1.0 - y).clamp(0.01, 0.99)) / SHEET.1,
            ]
        };
        let p = |x: f32, y: f32| at + right * (x * w) + up * (y * h) + lean * (y * y * h);
        b.quad(
            [p(-0.5, 0.0), p(0.5, 0.0), p(0.5, 1.0), p(-0.5, 1.0)],
            toward,
            [uv(-0.5, 0.0), uv(0.5, 0.0), uv(0.5, 1.0), uv(-0.5, 1.0)],
            [1.0; 4],
        );
    }
}

/// A flame's embers: small bright squares rising and drifting.
fn embers(b: &mut Build, base: Vec3, up: Vec3, eye: Vec3, drift: Vec3, f: &super::Flame, t: f32) {
    for k in 0..4u32 {
        let phase = (t / EMBER_LIFE_S + roll(f.key, 0xb0 + k)).fract();
        let side = up.any_orthonormal_vector() * (roll(f.key, 0xc0 + k) - 0.5) * 1.6
            + up.cross(up.any_orthonormal_vector()) * (roll(f.key, 0xd0 + k) - 0.5) * 1.6;
        let wobble = (t * 3.1 + k as f32).sin() * 0.15;
        let at = base + side + up * (0.4 + phase * EMBER_RISE_M) + drift * phase + side * wobble;
        let s = 0.06 * (1.0 - phase * 0.6);
        let (right, toward) = facing(at, up, eye);
        let glow = 1.0 - phase;
        let c = [1.6 * glow + 0.3, 0.55 * glow + 0.05, 0.1 * glow, 1.0];
        b.quad(
            [
                at - right * s - up * s,
                at + right * s - up * s,
                at + right * s + up * s,
                at - right * s + up * s,
            ],
            toward,
            [[0.0; 2]; 4],
            c,
        );
    }
}

/// A flame's smoke: puffs rising and leaning downwind, growing and fading.
fn smoke(b: &mut Build, base: Vec3, up: Vec3, eye: Vec3, drift: Vec3, f: &super::Flame, t: f32) {
    for k in 0..2u32 {
        let phase = (t / SMOKE_LIFE_S + roll(f.key, 0xe0 + k)).fract();
        let at = base + up * (1.5 + phase * SMOKE_RISE_M) + drift * (phase * phase);
        let s = 0.8 + phase * 2.6;
        let to_eye = (eye - at).normalize_or(Vec3::Z);
        let right = up.cross(to_eye).normalize_or(Vec3::X);
        let top = to_eye.cross(right);
        let frame = (phase * 4.0).floor().min(3.0);
        let alpha = (1.0 - phase) * (phase * 6.0).min(1.0) * 0.75;
        let c = [0.55, 0.53, 0.5, alpha];
        let uv = |x: f32, y: f32| [(frame + x) / 4.0, 1.0 - y];
        b.quad(
            [
                at - right * s - top * s,
                at + right * s - top * s,
                at + right * s + top * s,
                at - right * s + top * s,
            ],
            to_eye,
            [uv(0.0, 0.0), uv(1.0, 0.0), uv(1.0, 1.0), uv(0.0, 1.0)],
            c,
        );
    }
}

/// Every frame there is fire: the flames, embers and smoke, rebuilt about
/// an anchor at the first flame.
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
fn draw_flames(
    now: Res<FireNow>,
    frame: Res<PlanetRenderFrame>,
    time: Res<Time>,
    cameras: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut parts: Query<
        (
            &Mesh3d,
            &mut Transform,
            &mut Visibility,
            Has<FlameMesh>,
            Has<EmberMesh>,
        ),
        Or<(With<FlameMesh>, With<EmberMesh>, With<SmokeMesh>)>,
    >,
) {
    let Some(anchor) = now.flames.first().map(|f| f.at) else {
        for (_, _, mut v, _, _) in &mut parts {
            *v = Visibility::Hidden;
        }
        return;
    };
    let Some(eye) = cameras
        .iter()
        .find(|(c, _)| c.is_active)
        .map(|(_, g)| g.translation())
    else {
        return;
    };
    let origin = anchor + frame.center;
    // The eye about the anchor: small numbers only.
    let eye = (eye.as_dvec3() - origin).as_vec3();
    let t = time.elapsed_secs();
    let wind = now.wind;
    let (mut flames, mut sparks, mut puffs) =
        (Build::default(), Build::default(), Build::default());
    for f in &now.flames {
        let base = (f.at - anchor).as_vec3();
        let up = f.at.normalize().as_vec3();
        let lean = (wind - up * wind.dot(up)) * 0.04;
        flame_tongues(&mut flames, base, up, eye, lean, f, t);
        embers(&mut sparks, base, up, eye, wind * 0.4, f, t);
        smoke(&mut puffs, base, up, eye, wind * 1.6, f, t);
    }
    let mut built = [Some(flames.mesh()), Some(sparks.mesh()), Some(puffs.mesh())];
    for (mesh, mut at, mut v, is_flame, is_ember) in &mut parts {
        let which = if is_flame {
            0
        } else if is_ember {
            1
        } else {
            2
        };
        if let Some(m) = built[which].take()
            && let Some(slot) = meshes.get_mut(&mesh.0)
        {
            *slot = m;
        }
        at.translation = origin.as_vec3();
        *v = Visibility::Inherited;
    }
}

/// The tree a snag is, its bark and leaf textures as images.
fn tile_image(tex: Tex, leaf_mask: Option<u32>, tint: [f32; 3]) -> Option<image::RgbaImage> {
    let assets = crate::config::config_dir().join("..");
    let tile: image::RgbaImage = match tex {
        Tex::Atlas { sheet, col, row } => {
            let atlas = image::open(assets.join("tilesets/atlas.png"))
                .ok()?
                .to_rgba8();
            let k = atlas.width() / 512;
            let mut out = image::RgbaImage::new(32, 32);
            for y in 0..32 {
                for x in 0..32 {
                    let px = ((sheet % 4) * 128 + col * 32 + x) * k;
                    let py = ((sheet / 4) * 128 + row * 32 + y) * k;
                    out.put_pixel(x, y, *atlas.get_pixel(px, py));
                }
            }
            out
        }
        Tex::Kit(k) => image::open(assets.join(tree_kit::KIT_TILES[k as usize]))
            .ok()?
            .to_rgba8(),
    };
    let mut out = tile;
    for y in 0..32 {
        for x in 0..32 {
            let p = out.get_pixel_mut(x, y);
            for (c, k) in p.0.iter_mut().zip(tint) {
                *c = (f32::from(*c) * k).clamp(0.0, 255.0) as u8;
            }
            if let Some(m) = leaf_mask {
                // v = 0 the root edge: the image's bottom row.
                let uv = [(x as f32 + 0.5) / 32.0, 1.0 - (y as f32 + 0.5) / 32.0];
                p.0[3] = if leaf_kept(uv, m) { 255 } else { 0 };
            }
        }
    }
    Some(out)
}

/// `planet_surface.wgsl`'s `leaf_mask` (the mockup's `maskOf`): a card's
/// shape cut from its tile.
pub fn leaf_kept(uv: [f32; 2], m: u32) -> bool {
    let q = [
        ((uv[0].clamp(0.0, 0.999) * 32.0).floor() + 0.5) / 32.0,
        ((uv[1].clamp(0.0, 0.999) * 32.0).floor() + 0.5) / 32.0,
    ];
    let x = q[0] - 0.5;
    let y = q[1];
    let ax = x.abs();
    match m {
        1 => {
            if ax < 0.04 && y < 0.8 {
                return true;
            }
            for k in 0..5u32 {
                let fk = k as f32;
                let s = (k % 2) as f32 * 2.0 - 1.0;
                let (c, a2) = if k == 4 {
                    (Vec2::new(0.0, 0.8), Vec2::new(0.0, 1.0))
                } else {
                    (
                        Vec2::new(s * 0.2 * (1.0 - fk * 0.1), 0.2 + fk * 0.14),
                        Vec3::new(s * 0.8, 1.0, 0.0).normalize().truncate(),
                    )
                };
                let d = Vec2::new(x, y) - c;
                let a = d.dot(a2);
                let b = d.dot(Vec2::new(-a2.y, a2.x));
                let wd = 0.11 * (1.0 - a.abs() / 0.17 * 0.7);
                if a.abs() < 0.17 && b.abs() < wd {
                    return true;
                }
            }
            false
        }
        2 => {
            if ax < 0.035 && y < 0.95 {
                return true;
            }
            if ax > 0.46 * (1.0 - y * 0.8) || y > 0.95 {
                return false;
            }
            (y * 10.0 - ax * 1.8).fract() >= 0.45
        }
        _ => {
            let env = 0.46 * (std::f32::consts::PI * y.clamp(0.0, 1.0)).sin().powf(0.7);
            if ax < 0.035 && y < 0.97 {
                return true;
            }
            if ax > env {
                return false;
            }
            (y * 12.0 - ax * 2.5).fract() >= 0.35
        }
    }
}

/// A snag's or sapling's mesh in its own frame: the tree's model scaled,
/// bark (and leaves where it has them), with the foot charred.
fn tree_mesh(snag: &Snag, leaves: bool) -> Option<(Mesh, Option<Mesh>)> {
    let kit = crate::planet::trees::kit();
    let model = kit
        .models
        .iter()
        .find(|m| m.species == snag.tree.species && m.variant == snag.tree.variant)?;
    let grown = match snag.look {
        TreeLook::Sapling { grown } => 0.2 + 0.8 * grown,
        _ => 1.0,
    };
    let scale = snag.tree.size * grown;
    let turn = Quat::from_rotation_y(snag.tree.turn);
    let charred = matches!(snag.look, TreeLook::Snag | TreeLook::Scorched);
    let part = |range: std::ops::Range<usize>, foot: bool| {
        let mut b = Build::default();
        let vs = &model.vertices[range];
        for (k, v) in vs.iter().enumerate() {
            let p = turn * Vec3::from(v.position) * scale;
            let n = turn * Vec3::from(v.normal);
            let ao = 0.55 + 0.45 * v.ao;
            // The foot black where a ground fire burned round it.
            let char_k = if foot && charred {
                1.0 - (v.position[1] / 2.5).clamp(0.0, 1.0) * 0.6
            } else {
                0.0
            };
            let shade = ao * (1.0 - 0.8 * char_k);
            b.p.push(p.to_array());
            b.n.push(n.normalize_or(Vec3::Y).to_array());
            b.uv.push(v.uv);
            b.c.push([shade, shade, shade, 1.0]);
            b.i.push(k as u32);
        }
        b.mesh()
    };
    let bark = part(0..model.bark, true);
    let crown = (leaves && model.vertices.len() > model.bark)
        .then(|| part(model.bark..model.vertices.len() - model.vines, false));
    Some((bark, crown))
}

/// The snags' materials by species, state and part, and the version they
/// were made for.
type Made = (
    u64,
    std::collections::BTreeMap<(u32, u8, u8), Handle<StandardMaterial>>,
);

/// The charred trees: one entity a snag, made when [`FireNow`]'s snags
/// change.
#[allow(clippy::too_many_arguments)]
fn draw_snags(
    mut commands: Commands,
    now: Res<FireNow>,
    frame: Res<PlanetRenderFrame>,
    drawn: Query<(Entity, &SnagAt), With<SnagMesh>>,
    mut placed: Query<&mut Transform, With<SnagMesh>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut made: Local<Made>,
) {
    if made.0 != now.version {
        made.0 = now.version;
        for (e, _) in &drawn {
            commands.entity(e).despawn();
        }
        for snag in &now.snags {
            let leaves = !matches!(snag.look, TreeLook::Snag);
            let Some((bark, crown)) = tree_mesh(snag, leaves) else {
                continue;
            };
            let look = tree_kit::look(snag.tree.species);
            let state = match snag.look {
                TreeLook::Snag | TreeLook::Scorched => 0u8,
                _ => 1u8,
            };
            let species = snag.tree.species.index() as u8;
            let mut material = |part: u8, tex: Tex, mask: Option<u32>, tint: [f32; 3]| {
                made.1
                    .entry((u32::from(species), state, part))
                    .or_insert_with(|| {
                        let texture =
                            tile_image(tex, mask, tint).map(|img| pixel_image(img, &mut images));
                        materials.add(StandardMaterial {
                            base_color_texture: texture,
                            perceptual_roughness: 1.0,
                            reflectance: 0.0,
                            alpha_mode: if mask.is_some() {
                                AlphaMode::Mask(0.5)
                            } else {
                                AlphaMode::Opaque
                            },
                            cull_mode: if mask.is_some() {
                                None
                            } else {
                                Some(bevy::render::render_resource::Face::Back)
                            },
                            double_sided: mask.is_some(),
                            ..default()
                        })
                    })
                    .clone()
            };
            // Charred bark: the bark's own texels, burnt nearly black.
            let bark_tint = if state == 0 {
                [0.28, 0.25, 0.23]
            } else {
                [1.0; 3]
            };
            let leaf_tint = if state == 0 {
                [0.85, 0.55, 0.3]
            } else {
                [1.0; 3]
            };
            let at = snag.at + frame.center;
            let up = snag.at.normalize().as_vec3();
            let rotation = Quat::from_rotation_arc(Vec3::Y, up);
            let root = commands
                .spawn((
                    Transform::from_translation(at.as_vec3()).with_rotation(rotation),
                    Visibility::Inherited,
                    LitByField,
                    LitLikeTerrain,
                    SnagMesh,
                    SnagAt(snag.at),
                    Name::new("fire snag"),
                ))
                .id();
            let bark_m = material(0, look.bark, None, bark_tint);
            commands.entity(root).with_children(|p| {
                p.spawn((Mesh3d(meshes.add(bark)), MeshMaterial3d(bark_m)));
            });
            if let (Some(crown), Some(leaf)) = (crown, look.leaf) {
                let leaf_m = material(1, leaf, Some(look.mask), leaf_tint);
                commands.entity(root).with_children(|p| {
                    p.spawn((Mesh3d(meshes.add(crown)), MeshMaterial3d(leaf_m)));
                });
            }
        }
        return;
    }
    // Kept where they stand as the render frame moves.
    for (e, at) in &drawn {
        if let Ok(mut t) = placed.get_mut(e) {
            t.translation = (at.0 + frame.center).as_vec3();
        }
    }
}

/// Where a snag stands, planet frame (f64).
#[derive(Component, Clone, Copy)]
struct SnagAt(DVec3);

/// The fire's materials are pipelines of their own: each shown once, tiny,
/// in front of the eye behind the loading screen (`loading` decision 6).
fn prewarm(
    mut commands: Commands,
    looks: Option<Res<Looks>>,
    warming: Res<crate::loading::Warming>,
    mut meshes: ResMut<Assets<Mesh>>,
    cameras: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    mut done: Local<bool>,
) {
    if *done || warming.immediate() {
        return;
    }
    let Some(looks) = looks else {
        return;
    };
    let Some(eye) = cameras
        .iter()
        .find(|(c, _)| c.is_active)
        .map(|(_, g)| g.translation() + g.forward() * 2.0)
    else {
        return;
    };
    let mut b = Build::default();
    b.quad(
        [Vec3::ZERO, Vec3::X, Vec3::new(1.0, 1.0, 0.0), Vec3::Y],
        Vec3::Z,
        [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
        [1.0; 4],
    );
    let quad = meshes.add(b.mesh());
    let at = Transform::from_translation(eye).with_scale(Vec3::splat(1e-4));
    for material in [looks.flame.clone(), looks.ember.clone()] {
        commands.spawn((
            Mesh3d(quad.clone()),
            MeshMaterial3d(material),
            at,
            crate::loading::PrewarmProxy,
        ));
    }
    commands.spawn((
        Mesh3d(quad),
        MeshMaterial3d(looks.smoke.clone()),
        at,
        LitByField,
        LitLikeTerrain,
        crate::loading::PrewarmProxy,
    ));
    *done = true;
}
