//! A light held in hand (`glowing-mushrooms` Part C, design 14; M13 v3's
//! Glow tab, `docs/mockups/foliage.html#glow`): the torch, a lantern and the
//! glowing mushroom, drawn in the approved hand when one is in the selected
//! hotbar slot.
//!
//! - **The torch**: a six-sided haft in a bark texture, a head of rag wound
//!   round its top and charred at the crown, and a pixel flame: three
//!   view-facing cards on frames of one sheet, the outer keeping its red rim
//!   and the inner its white heart, with embers rising from it in the world
//!   and a soft glow round the head.
//! - **A lantern**, hung from its bail in the fist: an iron frame and four
//!   panes, lit from dusk to dawn, swinging a little behind the step.
//! - **The glowing mushroom**, held by a round stalk, its cap today's
//!   four-sided one (survey A3 is open), burning cyan at night.
//!
//! One flicker drives the flame's height, its glow and the carried light's
//! strength ([`flicker`]), so the light on the ground flickers in step with
//! the flame drawn. The hand is the tools' (the pickaxe's grip, which closes
//! on a stick), turned to each hold of the mockup's `HOLD` block. It is in
//! the right hand when the hand is otherwise empty, and mirrored into the
//! left beside a tool in the right.
//!
//! Everything here is drawn only: no rule reads it. The textures are
//! `tools/gen_held_lights.py`'s, the mockup's own drawings.

use super::{
    HandShading, Held, HeldConfig, HeldMaterials, HeldModel, HeldPose, Texture, hand_colours,
    hand_image, hand_mesh, models, seated_afloat,
};
use crate::carried_light::{HeldItem, HeldLightAt};
use crate::fish::ToolSlot;
use crate::walking::{WalkingCamera, WalkingReadout};
use bevy::asset::RenderAssetUsages;
use bevy::image::{ImageAddressMode, ImageSampler, ImageSamplerDescriptor};
use bevy::light::NotShadowCaster;
use bevy::math::DVec3;
use bevy::mesh::PrimitiveTopology;
use bevy::prelude::*;
use pbd_core::goods::Good;
use pbd_core::inventory::{Item, Tool};
use pbd_core::terrain::Material;
use serde::{Deserialize, Serialize};

/// Which light the hand holds.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LightKind {
    Torch,
    Lantern,
    Mushroom,
}

impl LightKind {
    pub const ALL: [LightKind; 3] = [LightKind::Torch, LightKind::Lantern, LightKind::Mushroom];

    /// The held look an item takes, if it has one. A brazier, a candle and a
    /// campfire give their light from the hand too, but have no held look.
    pub fn of(item: Item) -> Option<Self> {
        match item {
            Item::Block(Material::Torch) => Some(Self::Torch),
            Item::Block(
                Material::LanternPost | Material::LanternWall | Material::LanternHanging,
            ) => Some(Self::Lantern),
            Item::Good(good) if good == Good::GLOWING_MUSHROOM => Some(Self::Mushroom),
            _ => None,
        }
    }
}

/// One hold: where the fist closes, eye space m (x right, y up, -z ahead),
/// and which way the thing held runs from it (the haft's up; a lantern's
/// bail).
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Hold {
    pub grip: [f32; 3],
    pub up: [f32; 3],
}

/// The hold's idle sway (design 14): a step's side-to-side and bob while
/// walking, eased in and out, and a breath at rest.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Sway {
    /// Metres walked a step.
    pub step_m: f32,
    /// Side to side and bob, m; the roll, rad; the breath at rest, m.
    pub side_m: f32,
    pub bob_m: f32,
    pub roll_rad: f32,
    pub breathe_m: f32,
    /// How fast the walk's sway eases in and out, 1/s.
    pub ease_hz: f32,
}

/// How a light in hand is held and burns (`held.ron`'s `light`).
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct LightHold {
    /// The nearest the carried light is taken to the hand and the thing in
    /// it, m (`carried_hand_min_m`), so the hand holding a torch is lit warm
    /// and not blown out.
    pub hand_min_m: f32,
    /// The torch's flicker: the share of its light's strength it swings
    /// about, peak to mean. A lantern flickers a third as much; the
    /// mushroom not at all.
    pub torch_flicker: f32,
    /// The flame sheet's frames a second.
    pub flame_fps: f32,
    /// The held torch's flame, m tall.
    pub flame_m: f32,
    pub torch: Hold,
    pub lantern: Hold,
    pub mushroom: Hold,
    pub sway: Sway,
}

impl Default for LightHold {
    fn default() -> Self {
        // The mockup's `HOLD` block (M13 v3).
        Self {
            hand_min_m: 0.5,
            torch_flicker: 0.15,
            flame_fps: 11.0,
            flame_m: 0.19,
            torch: Hold {
                grip: [0.24, -0.33, -0.52],
                up: [-0.12, 1.0, -0.35],
            },
            lantern: Hold {
                grip: [0.25, -0.07, -0.62],
                up: [1.0, -0.05, 0.25],
            },
            mushroom: Hold {
                grip: [0.22, -0.31, -0.46],
                up: [-0.15, 1.0, -0.5],
            },
            sway: Sway {
                step_m: 0.75,
                side_m: 0.0,
                bob_m: 0.0,
                roll_rad: 0.0,
                breathe_m: 0.003,
                ease_hz: 6.0,
            },
        }
    }
}

impl LightHold {
    pub fn validate(&self) -> Result<(), String> {
        let ok = |v: f32, lo: f32, hi: f32| v.is_finite() && (lo..=hi).contains(&v);
        if !ok(self.hand_min_m, 0.1, 2.0) {
            return Err("held.light: hand_min_m is 0.1..2".into());
        }
        if !ok(self.torch_flicker, 0.0, 0.5) {
            return Err("held.light: torch_flicker is 0..0.5".into());
        }
        if !ok(self.flame_fps, 4.0, 24.0) || !ok(self.flame_m, 0.02, 1.0) {
            return Err("held.light: flame_fps is 4..24 and flame_m 0.02..1".into());
        }
        let s = &self.sway;
        if [
            s.step_m,
            s.side_m,
            s.bob_m,
            s.roll_rad,
            s.breathe_m,
            s.ease_hz,
        ]
        .iter()
        .any(|v| !v.is_finite() || *v < 0.0)
            || s.step_m <= 0.0
        {
            return Err("held.light: the sway's values are finite and not negative".into());
        }
        for hold in [self.torch, self.lantern, self.mushroom] {
            if Vec3::from(hold.up).length() < 1e-3 || !Vec3::from(hold.grip).is_finite() {
                return Err("held.light: each hold has a grip and a way up".into());
            }
        }
        Ok(())
    }

    fn hold(&self, kind: LightKind) -> Hold {
        match kind {
            LightKind::Torch => self.torch,
            LightKind::Lantern => self.lantern,
            LightKind::Mushroom => self.mushroom,
        }
    }
}

/// The mockup's `hh`: a hash of three integers in 0..1.
fn hh(a: i32, b: i32, s: i32) -> f32 {
    let h = a
        .wrapping_mul(374_761_393)
        .wrapping_add(b.wrapping_mul(668_265_263))
        .wrapping_add(s.wrapping_mul(1_442_695_041));
    let h = ((h ^ ((h as u32) >> 13) as i32) as u32).wrapping_mul(1_274_126_177);
    (h ^ (h >> 16)) as f32 / 4_294_967_296.0
}

/// The flicker at `t` s for a flame seeded `seed` (the mockup's `flick`):
/// smooth waves, and a step each time the flame sheet changes frame, so the
/// light jumps when the drawing does. About 1 +- `amount`.
pub fn flicker(t: f32, seed: f32, amount: f32, fps: f32) -> f32 {
    let frame = (t * fps + seed).floor() as i32;
    let waves = 0.05 * (t * 12.7 + seed).sin()
        + 0.04 * (t * 21.3 + seed * 2.1).sin()
        + 0.03 * (t * 5.9 + seed * 0.7).sin()
        + (hh(frame, 3, 77) - 0.5) * 0.08;
    // The mockup's waves peak at 0.16; `amount` scales them.
    1.0 + waves * (amount / 0.16)
}

/// The torch, in its own frame: y up the haft, the origin where the fist
/// closes on it (the mockup's `torchModel`). The haft runs from `bottom` to
/// [`HAFT_TOP`]; the head from [`HEAD_BOT`] to [`HEAD_TOP`], fattest at
/// [`HEAD_MID`]; the flame stands on its crown.
const HEAD_BOT: f32 = 0.19;
const HEAD_MID: f32 = 0.26;
const HEAD_TOP: f32 = 0.33;
const HAFT_TOP: f32 = 0.28;
const HAFT_BOTTOM: f32 = -0.17;
/// Where the flame stands, and the held torch's light burns.
const FLAME_AT: Vec3 = Vec3::new(0.0, HEAD_TOP - 0.025, 0.0);
/// Where a lantern's light burns, under the fist on its bail.
const LANTERN_AT: Vec3 = Vec3::new(0.0, -0.15, 0.0);
/// Where the mushroom's cap burns.
const CAP_AT: Vec3 = Vec3::new(0.0, 0.17, 0.0);

/// Triangles in an item's frame, with normals, uvs and vertex colours.
#[derive(Default)]
struct Build {
    p: Vec<[f32; 3]>,
    n: Vec<[f32; 3]>,
    uv: Vec<[f32; 2]>,
    c: Vec<[f32; 4]>,
}

impl Build {
    fn tri(&mut self, v: [Vec3; 3], n: Vec3, uv: [[f32; 2]; 3], c: [f32; 4]) {
        // Wound outward, whatever order the corners came in.
        let out = (v[1] - v[0]).cross(v[2] - v[0]).dot(n) >= 0.0;
        let order = if out { [0, 1, 2] } else { [0, 2, 1] };
        for i in order {
            self.p.push(v[i].to_array());
            self.n.push(n.to_array());
            self.uv.push(uv[i]);
            self.c.push(c);
        }
    }

    fn quad(&mut self, v: [Vec3; 4], n: Vec3, uv: [[f32; 2]; 4], c: [f32; 4]) {
        self.tri([v[0], v[1], v[2]], n, [uv[0], uv[1], uv[2]], c);
        self.tri([v[0], v[2], v[3]], n, [uv[0], uv[2], uv[3]], c);
    }

    /// A ring of `sides` flat faces about y from `y0` (radius `r0`) to `y1`
    /// (`r1`), the texture's u round it and v from `v0` to `v1`.
    #[allow(clippy::too_many_arguments)]
    fn ring(
        &mut self,
        sides: u32,
        y0: f32,
        y1: f32,
        r0: f32,
        r1: f32,
        v0: f32,
        v1: f32,
        turn: f32,
    ) {
        let at = |r: f32, y: f32, a: f32| Vec3::new(a.cos() * r, y, a.sin() * r);
        for i in 0..sides {
            let a0 = turn + i as f32 / sides as f32 * std::f32::consts::TAU;
            let a1 = turn + (i + 1) as f32 / sides as f32 * std::f32::consts::TAU;
            let am = 0.5 * (a0 + a1);
            let n = Vec3::new(am.cos(), (r0 - r1) / (y1 - y0).max(1e-4), am.sin()).normalize();
            let (u0, u1) = (i as f32 / sides as f32, (i + 1) as f32 / sides as f32);
            self.quad(
                [
                    at(r0, y0, a0),
                    at(r0, y0, a1),
                    at(r1, y1, a1),
                    at(r1, y1, a0),
                ],
                n,
                [[u0, v0], [u1, v0], [u1, v1], [u0, v1]],
                [1.0; 4],
            );
        }
    }

    /// A flat cap of `sides` triangles at `y`, facing `up` (+1 or -1).
    fn disc(&mut self, sides: u32, y: f32, r: f32, up: f32, uv: impl Fn(Vec3) -> [f32; 2]) {
        for i in 0..sides {
            let a0 = i as f32 / sides as f32 * std::f32::consts::TAU;
            let a1 = (i + 1) as f32 / sides as f32 * std::f32::consts::TAU;
            let p0 = Vec3::new(a0.cos() * r, y, a0.sin() * r);
            let p1 = Vec3::new(a1.cos() * r, y, a1.sin() * r);
            let c = Vec3::new(0.0, y + up * 0.004, 0.0);
            self.tri([c, p0, p1], Vec3::Y * up, [uv(c), uv(p0), uv(p1)], [1.0; 4]);
        }
    }

    /// A box at `c` of size `s`, its uvs planar at 1 cm a texel on an 8 px
    /// texture (the mockup's `boxMap`).
    fn cuboid(&mut self, c: Vec3, s: Vec3) {
        let (lo, hi) = (c - s * 0.5, c + s * 0.5);
        let k = 1.0 / 0.08;
        let faces: [(Vec3, [Vec3; 4], usize, usize); 6] = [
            (
                Vec3::Y,
                [
                    Vec3::new(lo.x, hi.y, lo.z),
                    Vec3::new(hi.x, hi.y, lo.z),
                    Vec3::new(hi.x, hi.y, hi.z),
                    Vec3::new(lo.x, hi.y, hi.z),
                ],
                0,
                2,
            ),
            (
                -Vec3::Y,
                [
                    Vec3::new(lo.x, lo.y, lo.z),
                    Vec3::new(hi.x, lo.y, lo.z),
                    Vec3::new(hi.x, lo.y, hi.z),
                    Vec3::new(lo.x, lo.y, hi.z),
                ],
                0,
                2,
            ),
            (
                Vec3::Z,
                [
                    Vec3::new(lo.x, lo.y, hi.z),
                    Vec3::new(hi.x, lo.y, hi.z),
                    Vec3::new(hi.x, hi.y, hi.z),
                    Vec3::new(lo.x, hi.y, hi.z),
                ],
                0,
                1,
            ),
            (
                -Vec3::Z,
                [
                    Vec3::new(lo.x, lo.y, lo.z),
                    Vec3::new(hi.x, lo.y, lo.z),
                    Vec3::new(hi.x, hi.y, lo.z),
                    Vec3::new(lo.x, hi.y, lo.z),
                ],
                0,
                1,
            ),
            (
                Vec3::X,
                [
                    Vec3::new(hi.x, lo.y, lo.z),
                    Vec3::new(hi.x, lo.y, hi.z),
                    Vec3::new(hi.x, hi.y, hi.z),
                    Vec3::new(hi.x, hi.y, lo.z),
                ],
                2,
                1,
            ),
            (
                -Vec3::X,
                [
                    Vec3::new(lo.x, lo.y, lo.z),
                    Vec3::new(lo.x, lo.y, hi.z),
                    Vec3::new(lo.x, hi.y, hi.z),
                    Vec3::new(lo.x, hi.y, lo.z),
                ],
                2,
                1,
            ),
        ];
        for (n, v, a, b) in faces {
            let uv = v.map(|p| [p[a] * k, p[b] * k]);
            self.quad(v, n, uv, [1.0; 4]);
        }
    }

    /// Shade each face by a light fixed in the item's frame, so it reads as
    /// round under unlit materials (the field's light is laid over it).
    fn shaded(mut self) -> Self {
        let l = Vec3::new(0.45, 0.8, 0.4).normalize();
        for (c, n) in self.c.iter_mut().zip(&self.n) {
            let k = 0.55 + 0.45 * Vec3::from(*n).dot(l).max(0.0);
            c[0] *= k;
            c[1] *= k;
            c[2] *= k;
        }
        self
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
        mesh
    }
}

fn haft() -> Mesh {
    let mut b = Build::default();
    b.ring(
        6,
        HAFT_BOTTOM,
        HAFT_TOP,
        0.017,
        0.02,
        (HAFT_TOP - HAFT_BOTTOM) / 0.45,
        0.0,
        0.3,
    );
    b.disc(6, HAFT_BOTTOM, 0.017, -1.0, |p| {
        [p.x * 12.0 + 0.5, p.z * 12.0 + 0.5]
    });
    b.shaded().mesh()
}

fn torch_head() -> Mesh {
    let mut b = Build::default();
    let v = |y: f32| (HEAD_TOP - y) / (HEAD_TOP - HEAD_BOT);
    b.ring(
        8,
        HEAD_BOT,
        HEAD_MID,
        0.032,
        0.043,
        v(HEAD_BOT),
        v(HEAD_MID),
        0.0,
    );
    b.ring(
        8,
        HEAD_MID,
        HEAD_TOP - 0.012,
        0.043,
        0.04,
        v(HEAD_MID),
        v(HEAD_TOP - 0.012),
        0.0,
    );
    b.ring(8, HEAD_TOP - 0.012, HEAD_TOP, 0.04, 0.03, 0.12, 0.02, 0.0);
    b.disc(8, HEAD_TOP, 0.03, 1.0, |p| {
        [p.x * 8.0 + 0.5, 0.01 + (p.z + 0.03) * 1.2]
    });
    b.disc(8, HEAD_BOT, 0.032, -1.0, |p| [p.x * 8.0 + 0.5, 0.9]);
    b.shaded().mesh()
}

/// The lantern's iron, hung from its bail: x along the bail, y up, the
/// origin where the fist closes on it (the mockup's `lanternModel`).
fn lantern_iron() -> Mesh {
    let mut b = Build::default();
    for (c, s) in [
        (Vec3::ZERO, Vec3::new(0.10, 0.009, 0.009)),
        (Vec3::new(-0.05, -0.035, 0.0), Vec3::new(0.007, 0.07, 0.007)),
        (Vec3::new(0.05, -0.035, 0.0), Vec3::new(0.007, 0.07, 0.007)),
        (Vec3::new(0.0, -0.064, 0.0), Vec3::new(0.035, 0.012, 0.035)),
        (Vec3::new(0.0, -0.078, 0.0), Vec3::new(0.12, 0.018, 0.12)),
        (Vec3::new(0.0, -0.222, 0.0), Vec3::new(0.125, 0.022, 0.125)),
    ] {
        b.cuboid(c, s);
    }
    for (dx, dz) in [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)] {
        b.cuboid(
            Vec3::new(dx * 0.052, -0.15, dz * 0.052),
            Vec3::new(0.014, 0.13, 0.014),
        );
    }
    b.shaded().mesh()
}

fn lantern_panes() -> Mesh {
    let mut b = Build::default();
    let (y0, y1, e) = (-0.211, -0.087, 0.05);
    let corners = [(-e, -e), (e, -e), (e, e), (-e, e)];
    for i in 0..4 {
        let (a, c) = (corners[i], corners[(i + 1) % 4]);
        let n = Vec3::new(a.0 + c.0, 0.0, a.1 + c.1).normalize();
        b.quad(
            [
                Vec3::new(a.0, y0, a.1),
                Vec3::new(c.0, y0, c.1),
                Vec3::new(c.0, y1, c.1),
                Vec3::new(a.0, y1, a.1),
            ],
            n,
            [[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]],
            [1.0; 4],
        );
    }
    b.mesh()
}

fn mushroom_stalk() -> Mesh {
    let mut b = Build::default();
    b.ring(6, HAFT_BOTTOM, 0.15, 0.015, 0.012, 1.0, 0.0, 0.2);
    b.disc(6, HAFT_BOTTOM, 0.015, -1.0, |p| [p.x * 20.0 + 0.5, 0.95]);
    b.shaded().mesh()
}

/// Today's cap (survey A3 is open): the flower slot's four triangles out to
/// 7 cm, its centre 3 cm up, over the stalk's top, and its underside.
fn mushroom_cap() -> Mesh {
    let mut b = Build::default();
    let (tip, top) = (Vec3::new(0.0, 0.15, 0.0), Vec3::new(0.0, 0.18, 0.0));
    let under = [0.55, 0.62, 0.60, 1.0];
    for t in 0..4 {
        let a0 = t as f32 * std::f32::consts::FRAC_PI_2;
        let a1 = a0 + std::f32::consts::FRAC_PI_2;
        let p0 = tip + Vec3::new(a0.cos(), 0.0, a0.sin()) * 0.07;
        let p1 = tip + Vec3::new(a1.cos(), 0.0, a1.sin()) * 0.07;
        let n = (p1 - p0).cross(top - p0).normalize();
        let n = if n.y < 0.0 { -n } else { n };
        b.tri(
            [p0, p1, top],
            n,
            [[0.1, 0.9], [0.9, 0.9], [0.5, 0.5]],
            [1.0; 4],
        );
        b.tri(
            [p0, p1, tip],
            -Vec3::Y,
            [[0.1, 0.9], [0.9, 0.9], [0.5, 0.5]],
            under,
        );
    }
    b.mesh()
}

/// A nearest-sampled, tiled texture from `tools/gen_held_lights.py`.
fn texture(bytes: &[u8], images: &mut Assets<Image>) -> Handle<Image> {
    let pixels = image::load_from_memory(bytes)
        .expect("assets/textures/held_light/*.png: tools/gen_held_lights.py writes them")
        .to_rgba8();
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

/// A held light: its kind, its rest pose (the pickaxe's grip turned to the
/// hold) and where its light burns in its own frame, tool units.
#[derive(Component)]
pub struct HeldLightView {
    kind: LightKind,
    rest: Transform,
    fist: Vec3,
    burns: Vec3,
}

/// The lantern under its bail, which swings.
#[derive(Component)]
pub(super) struct LanternSwing(Transform);

/// The lantern's panes, lit or dark.
#[derive(Component)]
pub(super) struct Panes;

/// The view's flame, glow and embers: meshes in eye space, rebuilt each
/// frame.
#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub(super) enum Fx {
    Flame,
    Glow,
    Embers,
}

/// What the held lights' draw keeps.
#[derive(Resource)]
pub(super) struct LightLooks {
    pane_lit: Handle<StandardMaterial>,
    pane_dark: Handle<StandardMaterial>,
    cap: Handle<StandardMaterial>,
    glow: Handle<StandardMaterial>,
}

/// An item's frame as a child of a holder: item y along the tool's x (the
/// handle), the origin on the fist, metres to tool units. `up_tool`, the
/// eye's up in tool units, sets which way the item's own up turns (a
/// lantern hangs under its bail); `None` for things along the handle.
fn item_frame(fist_tool: Vec3, scale: f32, up_tool: Option<Vec3>) -> Transform {
    let rotation = match up_tool {
        // A lantern: its x along the bail (the handle), its y the eye's up.
        Some(up) => {
            let x = Vec3::X;
            let y = (up - x * up.dot(x)).normalize_or(Vec3::Y);
            Quat::from_mat3(&Mat3::from_cols(x, y, x.cross(y)))
        }
        None => Quat::from_rotation_arc(Vec3::Y, Vec3::X),
    };
    Transform {
        translation: fist_tool,
        rotation,
        scale: Vec3::splat(1.0 / scale),
    }
}

/// The pickaxe's grip, turned and moved to `hold`: the rest pose of a held
/// light, and its fist, eye space.
fn hold_rest(pick: &HeldModel, hold: Hold) -> (Transform, Vec3) {
    let rest = pick.rest();
    let fist = Vec3::from(pick.fist);
    let handle = rest.rotation * Vec3::X;
    let turn = Quat::from_rotation_arc(handle.normalize(), Vec3::from(hold.up).normalize());
    let grip = Vec3::from(hold.grip);
    (
        Transform {
            translation: grip + turn * (rest.translation - fist),
            rotation: turn * rest.rotation,
            scale: rest.scale,
        },
        grip,
    )
}

/// Build the three held lights and their hands once the tools are built,
/// hidden.
#[allow(clippy::too_many_arguments)]
pub(super) fn spawn_lights(
    mut commands: Commands,
    settings: Res<HeldConfig>,
    cameras: Query<Entity, With<WalkingCamera>>,
    existing: Query<(), With<HeldLightView>>,
    held_materials: Option<ResMut<HeldMaterials>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let (Some(mut held_materials), Some(camera)) = (held_materials, cameras.iter().next()) else {
        return;
    };
    if !existing.is_empty() {
        return;
    }
    let s = &settings.0;
    // Drawn from both sides: the left hand is the right mirrored.
    let unlit = |texture: Option<Handle<Image>>| StandardMaterial {
        base_color: Color::WHITE,
        base_color_texture: texture,
        unlit: true,
        fog_enabled: false,
        cull_mode: None,
        double_sided: true,
        ..default()
    };
    let mut lit_by_field = Vec::new();
    let mut add = |m: StandardMaterial, field: bool| {
        let handle = materials.add(m);
        if field {
            lit_by_field.push(handle.clone());
        }
        handle
    };
    let hand_materials: Vec<Handle<StandardMaterial>> = Texture::ALL
        .iter()
        .map(|&t| add(unlit(Some(images.add(hand_image(t)))), true))
        .collect();
    let haft_m = add(
        unlit(Some(texture(
            include_bytes!("../../../../assets/textures/held_light/haft.png"),
            &mut images,
        ))),
        true,
    );
    let head_m = add(
        unlit(Some(texture(
            include_bytes!("../../../../assets/textures/held_light/head.png"),
            &mut images,
        ))),
        true,
    );
    let iron_m = add(
        unlit(Some(texture(
            include_bytes!("../../../../assets/textures/held_light/iron.png"),
            &mut images,
        ))),
        true,
    );
    let stalk_m = add(
        unlit(Some(texture(
            include_bytes!("../../../../assets/textures/held_light/stalk.png"),
            &mut images,
        ))),
        true,
    );
    // The panes and the cap are their own light at night; by day the field
    // lays its light over them (`hold_lights`).
    let pane_lit = add(
        StandardMaterial {
            base_color: Color::linear_rgb(0.9, 0.9, 0.9),
            ..unlit(Some(texture(
                include_bytes!("../../../../assets/textures/held_light/pane.png"),
                &mut images,
            )))
        },
        false,
    );
    let pane_dark = add(
        unlit(Some(texture(
            include_bytes!("../../../../assets/textures/held_light/pane_day.png"),
            &mut images,
        ))),
        true,
    );
    let cap = add(unlit(None), false);
    let flame_m = add(
        StandardMaterial {
            // Under the tonemapper's shoulder, as the placed flame is
            // (`planet_surface.wgsl`), so it reads orange, not white.
            base_color: Color::linear_rgb(0.75, 0.75, 0.75),
            alpha_mode: AlphaMode::Mask(0.5),
            ..unlit(Some(texture(
                include_bytes!("../../../../assets/textures/held_light/flame.png"),
                &mut images,
            )))
        },
        false,
    );
    // The glow is a soft light round the head, not pixel art: sampled
    // smoothly, as the mockup's shader draws it.
    let glow_texture = texture(
        include_bytes!("../../../../assets/textures/held_light/glow.png"),
        &mut images,
    );
    if let Some(image) = images.get_mut(&glow_texture) {
        image.sampler = ImageSampler::linear();
    }
    let glow = add(
        StandardMaterial {
            alpha_mode: AlphaMode::Add,
            ..unlit(Some(glow_texture))
        },
        false,
    );
    let embers = add(unlit(None), false);
    held_materials.0.extend(lit_by_field);
    commands.insert_resource(LightLooks {
        pane_lit: pane_lit.clone(),
        pane_dark,
        cap: cap.clone(),
        glow: glow.clone(),
    });
    let all = models();
    let pick = all.get(Tool::Pickaxe);
    let built = super::meshes(pick, all.spacing, &all.hand, s.stroke.sleeve_extra);
    let fist_tool = pick
        .rest()
        .compute_affine()
        .inverse()
        .transform_point3(pick.fist.into());
    for kind in LightKind::ALL {
        let (rest, fist) = hold_rest(pick, s.light.hold(kind));
        let up_tool = rest.rotation.inverse() * Vec3::Y;
        let arm = commands
            .spawn((Name::new("arm"), Transform::IDENTITY, Visibility::Inherited))
            .id();
        let frame = item_frame(
            fist_tool,
            pick.scale,
            (kind == LightKind::Lantern).then_some(up_tool),
        );
        let burns_item = match kind {
            LightKind::Torch => FLAME_AT,
            LightKind::Lantern => LANTERN_AT,
            LightKind::Mushroom => CAP_AT,
        };
        let holder = commands
            .spawn((
                Name::new(format!("{kind:?} in hand")),
                Held {
                    tool: Tool::Pickaxe,
                    rest,
                    fist,
                    wrist: built.wrist,
                    arm,
                    dirt: None,
                    bare: None,
                },
                HeldLightView {
                    kind,
                    rest,
                    fist,
                    burns: frame.transform_point(burns_item),
                },
                rest,
                Visibility::Hidden,
            ))
            .id();
        let parts: Vec<(Mesh, Handle<StandardMaterial>)> = match kind {
            LightKind::Torch => vec![(haft(), haft_m.clone()), (torch_head(), head_m.clone())],
            LightKind::Lantern => vec![
                (lantern_iron(), iron_m.clone()),
                (lantern_panes(), pane_lit.clone()),
            ],
            LightKind::Mushroom => vec![
                (mushroom_stalk(), stalk_m.clone()),
                (mushroom_cap(), cap.clone()),
            ],
        };
        // The lantern hangs from a pivot on the bail, which swings.
        let item = commands
            .spawn((Name::new("item"), frame, Visibility::Inherited))
            .id();
        if kind == LightKind::Lantern {
            commands.entity(item).insert(LanternSwing(frame));
        }
        for (i, (mesh, material)) in parts.into_iter().enumerate() {
            let mut part = commands.spawn((
                Mesh3d(meshes.add(mesh)),
                MeshMaterial3d(material),
                Transform::IDENTITY,
                Visibility::Inherited,
                NotShadowCaster,
            ));
            if kind == LightKind::Lantern && i == 1 {
                part.insert(Panes);
            }
            let part = part.id();
            commands.entity(item).add_child(part);
        }
        commands.entity(holder).add_children(&[item, arm]);
        let light = s.hand_light;
        for (on_arm, set) in [(false, &built.hand), (true, &built.arm)] {
            for texture in Texture::ALL {
                let part = &set[texture.index()];
                if part.positions.is_empty() {
                    continue;
                }
                let normals: Vec<Vec3> = part.normals.iter().map(|n| Vec3::from(*n)).collect();
                let tint: Vec<[f32; 3]> = part.flush.iter().map(|f| all.hand.tint(*f)).collect();
                let colours = hand_colours(&normals, &part.shade, &tint, rest.rotation, &light);
                let child = commands
                    .spawn((
                        Name::new(if on_arm { "arm part" } else { "hand part" }),
                        Mesh3d(meshes.add(hand_mesh(part, colours))),
                        MeshMaterial3d(hand_materials[texture.index()].clone()),
                        Transform::IDENTITY,
                        Visibility::Inherited,
                        NotShadowCaster,
                        HandShading {
                            holder,
                            on_arm,
                            normals,
                            shade: part.shade.clone(),
                            tint,
                            lit_at: None,
                        },
                    ))
                    .id();
                commands
                    .entity(if on_arm { arm } else { holder })
                    .add_child(child);
            }
        }
        commands.entity(camera).add_child(holder);
    }
    // The flame, its glow and the embers: in eye space under the camera.
    // Each starts as one tiny triangle with every attribute it will carry,
    // hidden, so the loading screen compiles its pipeline (a glow's additive
    // blend is a pipeline nothing else uses) before play.
    for (fx, material) in [(Fx::Flame, flame_m), (Fx::Glow, glow), (Fx::Embers, embers)] {
        let mut seed = Build::default();
        seed.tri(
            [
                Vec3::new(0.0, 0.0, -1.0),
                Vec3::new(1e-4, 0.0, -1.0),
                Vec3::new(0.0, 1e-4, -1.0),
            ],
            Vec3::Z,
            [[0.0, 0.0]; 3],
            [1.0; 4],
        );
        let mesh = seed.mesh();
        let fx_entity = commands
            .spawn((
                Name::new(format!("held {fx:?}")),
                fx,
                Mesh3d(meshes.add(mesh)),
                MeshMaterial3d(material),
                Transform::IDENTITY,
                Visibility::Hidden,
                NotShadowCaster,
                // Rebuilt every frame in eye space: never culled by a stale box.
                bevy::camera::visibility::NoFrustumCulling,
            ))
            .id();
        commands.entity(camera).add_child(fx_entity);
    }
}

impl std::fmt::Debug for Fx {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Fx::Flame => "flame",
            Fx::Glow => "glow",
            Fx::Embers => "embers",
        })
    }
}

/// One ember: where it is from the eye (the render frame's axes, so it
/// stays in the world as the view turns), how it moves, and its life.
#[derive(Clone, Copy, Default)]
struct Ember {
    off: Vec3,
    vel: Vec3,
    t: f32,
    max: f32,
    seed: f32,
}

/// What the held lights keep between frames.
#[derive(Default)]
pub(super) struct LightState {
    /// The eye in the planet's frame, f64: what the embers and the walk's
    /// sway are moved by.
    eye: Option<DVec3>,
    walk_phase: f32,
    walk_amp: f32,
    embers: Vec<Ember>,
    next: usize,
    spawn_acc: f32,
    rng: u32,
    /// The flame's base last frame (eye space), for its lean.
    flame_was: Option<Vec3>,
    lean: Vec3,
    /// Whether the flame's meshes were last written empty (`memory-leak`
    /// cut 7): with no flame and no ember alight they are left as they are,
    /// not written empty again and uploaded every frame.
    fx_empty: bool,
}

impl LightState {
    fn random(&mut self) -> f32 {
        self.rng = self.rng.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        self.rng as f32 / 4_294_967_296.0
    }
}

const EMBERS: usize = 48;

/// Show the light in hand, sway it, flicker its flame and light, and say
/// where its light burns.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub(super) fn hold_lights(
    settings: Res<HeldConfig>,
    item: Res<HeldItem>,
    tools: Res<ToolSlot>,
    walking: Option<Res<WalkingReadout>>,
    (aboard, view, vehicles): (
        Option<Res<crate::vehicles::Aboard>>,
        Option<Res<crate::vehicles::view::VehicleView>>,
        Query<&crate::vehicles::Vehicle>,
    ),
    (time, dusk, frame, looks, held_materials): (
        Res<Time>,
        Option<Res<crate::planet::lod::DuskLamps>>,
        Option<Res<crate::planet::PlanetRenderFrame>>,
        Option<Res<LightLooks>>,
        Option<Res<HeldMaterials>>,
    ),
    cameras: Query<&GlobalTransform, With<WalkingCamera>>,
    mut at: ResMut<HeldLightAt>,
    mut holders: Query<(&HeldLightView, &mut Transform, &mut Visibility)>,
    mut fx: Query<(&Fx, &Mesh3d, &mut Visibility), Without<HeldLightView>>,
    mut swings: Query<(&LanternSwing, &mut Transform), Without<HeldLightView>>,
    mut panes: Query<&mut MeshMaterial3d<StandardMaterial>, With<Panes>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut state: Local<LightState>,
) {
    let s = &settings.0.light;
    let dt = time.delta_secs().clamp(0.0, 0.1);
    let t = time.elapsed_secs();
    let on_foot = walking.is_some_and(|w| w.active)
        || seated_afloat(aboard.as_deref(), view.as_deref(), &vehicles);
    let kind = item.0.and_then(LightKind::of).filter(|_| on_foot);
    let lit = dusk.is_some_and(|d| d.lit);
    // Beside a tool in the right hand, the light is in the left.
    let left = tools.held().is_some();
    // The eye in the planet's frame (f64), how fast it walks, and its up.
    let eye = cameras.iter().next();
    let (speed, up_world, moved) = match (eye, frame.as_deref()) {
        (Some(eye), Some(frame)) => {
            let now = eye.translation().as_dvec3() - frame.center;
            let moved = state.eye.map_or(DVec3::ZERO, |was| now - was);
            state.eye = Some(now);
            let up = now.try_normalize().unwrap_or(DVec3::Y).as_vec3();
            // A jump (a load, a teleport) is not a walk.
            let moved = if moved.length() > 3.0 {
                DVec3::ZERO
            } else {
                moved
            };
            (
                (moved.as_vec3() / dt.max(1e-4)).length(),
                up,
                moved.as_vec3(),
            )
        }
        _ => (0.0, Vec3::Y, Vec3::ZERO),
    };
    let sway = s.sway;
    let walking_now = if speed > 0.3 { 1.0 } else { 0.0 };
    state.walk_amp += (walking_now - state.walk_amp) * (dt * sway.ease_hz).min(1.0);
    state.walk_phase += speed * dt / sway.step_m * std::f32::consts::PI;
    let (amp, ph) = (state.walk_amp, state.walk_phase);
    let pose = HeldPose {
        roll: sway.roll_rad * amp * ph.sin(),
        shift: Vec3::new(
            sway.side_m * amp * ph.sin(),
            -sway.bob_m * amp * (1.0 - (2.0 * ph).cos()) * 0.5 + sway.breathe_m * (t * 1.7).sin(),
            0.0,
        ),
        ..default()
    };
    let flick = match kind {
        Some(LightKind::Torch) => flicker(t, 0.0, s.torch_flicker, s.flame_fps),
        Some(LightKind::Lantern) if lit => flicker(t, 1.3, s.torch_flicker / 3.0, s.flame_fps),
        _ => 1.0,
    };
    let mut flame_base = None;
    let mut burns_eye = None;
    for (view, mut transform, mut visibility) in &mut holders {
        let shown = kind == Some(view.kind);
        let want = if shown {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if *visibility != want {
            *visibility = want;
        }
        if !shown {
            continue;
        }
        let mut posed = pose.apply(view.rest, view.fist);
        if left {
            // Mirrored across the view's middle: the left hand.
            let q = posed.rotation;
            posed.translation.x = -posed.translation.x;
            posed.rotation = Quat::from_xyzw(q.x, -q.y, -q.z, q.w);
            posed.scale.x = -posed.scale.x;
        }
        if *transform != posed {
            *transform = posed;
        }
        let burns = posed.transform_point(view.burns);
        burns_eye = Some(burns);
        if view.kind == LightKind::Torch {
            flame_base = Some(burns);
        }
    }
    at.eye = burns_eye.unwrap_or(crate::carried_light::HELD_LIGHT_EYE);
    at.flicker = flick;
    // The lantern swings behind the step; its panes burn from dusk to dawn.
    for (swing, mut transform) in &mut swings {
        let turn = Quat::from_rotation_x(0.10 * amp * (ph - 0.7).sin() + 0.015 * (t * 1.3).sin());
        let want = Transform {
            rotation: swing.0.rotation * turn,
            ..swing.0
        };
        if *transform != want {
            *transform = want;
        }
    }
    if let Some(looks) = looks.as_deref() {
        let want = if lit {
            &looks.pane_lit
        } else {
            &looks.pane_dark
        };
        for mut material in &mut panes {
            if material.0 != *want {
                material.0 = want.clone();
            }
        }
        // The cap: its own cool light at night (the plant's head,
        // `GLOW_HEAD_RGB` at the shader's gain), lit by the field by day.
        let field = held_materials
            .as_deref()
            .and_then(|h| h.0.first())
            .and_then(|m| materials.get(m))
            .map_or(LinearRgba::WHITE, |m| m.base_color.to_linear());
        let cap = if lit {
            let [r, g, b] = pbd_core::flora::GLOW_HEAD_RGB;
            LinearRgba::rgb(r, g, b) * 0.25 * 3.0
        } else {
            LinearRgba::rgb(0.80 * field.red, 0.92 * field.green, 0.90 * field.blue)
        };
        if let Some(m) = materials.get_mut(&looks.cap)
            && m.base_color.to_linear() != cap
        {
            m.base_color = cap.into();
        }
        let glow_k = (if lit { 0.5 } else { 0.08 }) * flick;
        if let Some(m) = materials.get_mut(&looks.glow) {
            m.base_color = Color::linear_rgba(glow_k, glow_k, glow_k, 1.0);
        }
    }
    // The flame leans back a little against the hand's motion.
    let rotation = eye.map_or(Quat::IDENTITY, |e| e.compute_transform().rotation);
    if let Some(base) = flame_base {
        if let Some(was) = state.flame_was {
            let vel = (base - was) / dt.max(1e-4) + rotation.inverse() * (moved / dt.max(1e-4));
            let lean = (-vel * 0.03).with_y(0.0).clamp_length_max(0.07);
            state.lean = state.lean.lerp(lean, (dt * 6.0).min(1.0));
        }
        state.flame_was = Some(base);
    } else {
        state.flame_was = None;
    }
    // Embers: spawned at the flame, rising and drifting in the world.
    let to_eye = rotation.inverse();
    let up_eye = to_eye * up_world;
    if let Some(base) = flame_base {
        state.spawn_acc += dt * 9.0;
        while state.spawn_acc > 1.0 {
            state.spawn_acc -= 1.0;
            let (a, b, c, d, e) = (
                state.random(),
                state.random(),
                state.random(),
                state.random(),
                state.random(),
            );
            let world_base = rotation * base;
            let side = Vec3::new(a - 0.5, 0.0, b - 0.5) * 0.05;
            let ember = Ember {
                off: world_base + rotation * side + up_world * (0.05 + c * 0.06),
                vel: rotation * Vec3::new((d - 0.5) * 0.15, 0.0, (e - 0.5) * 0.15)
                    + up_world * (0.35 + c * 0.4),
                max: 0.7 + d * 0.9,
                t: 0.7 + d * 0.9,
                seed: state.next as f32 * 1.7,
            };
            if state.embers.len() < EMBERS {
                state.embers.push(ember);
            } else {
                let i = state.next % EMBERS;
                state.embers[i] = ember;
            }
            state.next += 1;
        }
    }
    let side_a = rotation * Vec3::X;
    let side_b = rotation * Vec3::Z;
    for ember in &mut state.embers {
        if ember.t <= 0.0 {
            continue;
        }
        ember.t -= dt;
        ember.vel += up_world * (0.25 * dt)
            + side_a * ((t * 3.1 + ember.seed).sin() * 0.5 * dt)
            + side_b * ((t * 2.7 + ember.seed).cos() * 0.5 * dt);
        ember.vel *= 1.0 - 0.8 * dt;
        // In the world: the eye's own motion is taken off.
        ember.off += ember.vel * dt - moved;
    }
    let idle = flame_base.is_none() && state.embers.iter().all(|e| e.t <= 0.0);
    if idle && state.fx_empty {
        return;
    }
    state.fx_empty = idle;
    for (part, mesh, mut visibility) in &mut fx {
        let Some(mesh) = meshes.get_mut(&mesh.0) else {
            continue;
        };
        let mut b = Build::default();
        match part {
            Fx::Flame => {
                if let Some(base) = flame_base {
                    flame_cards(
                        &mut b,
                        base,
                        up_eye,
                        state.lean,
                        s.flame_m,
                        flick,
                        t,
                        s.flame_fps,
                    );
                }
            }
            Fx::Glow => {
                if let Some(base) = flame_base {
                    let size = s.flame_m * 3.0;
                    let centre = base + up_eye * (s.flame_m * 0.3);
                    let toward = -centre.normalize_or(Vec3::Z);
                    let right = up_eye.cross(toward).normalize_or(Vec3::X);
                    let up = toward.cross(right);
                    let h = size * 0.5;
                    b.quad(
                        [
                            centre - right * h - up * h,
                            centre + right * h - up * h,
                            centre + right * h + up * h,
                            centre - right * h + up * h,
                        ],
                        toward,
                        [[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]],
                        [1.0; 4],
                    );
                }
            }
            Fx::Embers => {
                for ember in state.embers.iter().filter(|e| e.t > 0.0) {
                    let p = to_eye * ember.off;
                    let life = ember.t / ember.max;
                    let c = if life > 0.62 {
                        [0.98, 0.686, 0.361, 1.0]
                    } else if life > 0.3 {
                        [0.996, 0.392, 0.125, 1.0]
                    } else {
                        [0.651, 0.118, 0.122, 1.0]
                    };
                    // A square pixel facing the eye, never under 2 px at
                    // 1080p from where it is.
                    let h = (0.0025 * p.length().max(0.2)).max(0.003);
                    let toward = -p.normalize_or(Vec3::Z);
                    let right = Vec3::Y.cross(toward).normalize_or(Vec3::X);
                    let up = toward.cross(right);
                    b.quad(
                        [
                            p - right * h - up * h,
                            p + right * h - up * h,
                            p + right * h + up * h,
                            p - right * h + up * h,
                        ],
                        toward,
                        [[0.0, 0.0]; 4],
                        c,
                    );
                }
            }
        }
        let empty = b.p.is_empty();
        let want = if empty {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        };
        if *visibility != want {
            *visibility = want;
        }
        if empty {
            continue;
        }
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, b.p);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, b.n);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, b.uv);
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, b.c);
    }
}

/// The flame's three cards (the mockup's `FLAME_VS`): each facing the eye
/// about the view's up, each on its own frame of the sheet and its own row
/// (the outer whole, the middle its orange and hotter, the inner its
/// heart), its height by the flicker, its tip leaning against the motion.
#[allow(clippy::too_many_arguments)]
fn flame_cards(
    b: &mut Build,
    base: Vec3,
    up: Vec3,
    lean: Vec3,
    size: f32,
    flick: f32,
    t: f32,
    fps: f32,
) {
    let up = up.normalize_or(Vec3::Y);
    let mut toward = -base;
    toward -= up * toward.dot(up);
    let toward = toward.normalize_or(Vec3::Z);
    let right = up.cross(toward).normalize_or(Vec3::X);
    for card in 0..3u32 {
        let c = card as f32;
        let k = [1.0, 0.8, 0.56][card as usize];
        let height = size * k * (1.0 + 1.6 * (flick - 1.0));
        let width = size * k * 0.68;
        let frame = (((t * fps).floor() as i64 + i64::from(card) * 3).rem_euclid(8)) as f32;
        let row = c;
        let at = |x: f32, y: f32| {
            base + right * (x * width + (c - 1.0) * 0.004 * size / 0.25)
                + up * (y * height)
                + lean * (y * y)
                + toward * (0.003 * c)
        };
        let uv = |x: f32, y: f32| {
            let ux = if card == 1 { 0.5 - x } else { x + 0.5 }.clamp(0.002, 0.998);
            let v = (1.0 - y).clamp(0.01, 0.99);
            [(frame + ux) / 8.0, (row + v) / 3.0]
        };
        b.quad(
            [at(-0.5, 0.0), at(0.5, 0.0), at(0.5, 1.0), at(-0.5, 1.0)],
            toward,
            [uv(-0.5, 0.0), uv(0.5, 0.0), uv(0.5, 1.0), uv(-0.5, 1.0)],
            [1.0; 4],
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The flicker swings about one by about the share asked, steps with the
    /// sheet's frame, and is steady at nought.
    #[test]
    fn the_flicker_is_about_its_share() {
        let (mut lo, mut hi) = (f32::MAX, f32::MIN);
        for i in 0..6_000 {
            let f = flicker(i as f32 / 60.0, 0.0, 0.15, 11.0);
            lo = lo.min(f);
            hi = hi.max(f);
        }
        assert!(lo > 0.8 && hi < 1.2, "{lo}..{hi}");
        assert!(hi - lo > 0.1, "it flickers: {lo}..{hi}");
        assert_eq!(flicker(3.3, 0.0, 0.0, 11.0), 1.0);
    }

    /// Only a torch, a lantern and the glowing mushroom have a held look.
    #[test]
    fn what_has_a_held_look() {
        assert_eq!(
            LightKind::of(Item::Block(Material::Torch)),
            Some(LightKind::Torch)
        );
        assert_eq!(
            LightKind::of(Item::Block(Material::LanternHanging)),
            Some(LightKind::Lantern)
        );
        assert_eq!(
            LightKind::of(Item::Good(Good::GLOWING_MUSHROOM)),
            Some(LightKind::Mushroom)
        );
        assert_eq!(LightKind::of(Item::Block(Material::Stone)), None);
        assert_eq!(LightKind::of(Item::Block(Material::Candle)), None);
    }

    /// Each hold turns the pickaxe's grip so its handle runs the hold's way
    /// and its fist is at the grip; the torch's flame then stands at the
    /// right of the view, chest-high, clear of the crosshair.
    #[test]
    fn the_torch_is_held_up_and_to_the_right() {
        let pick = models().get(Tool::Pickaxe);
        let hold = LightHold::default();
        let (rest, fist) = hold_rest(pick, hold.torch);
        assert_eq!(fist, Vec3::from(hold.torch.grip));
        let handle = (rest.rotation * Vec3::X).normalize();
        assert!(handle.dot(Vec3::from(hold.torch.up).normalize()) > 0.999);
        let fist_tool = rest.compute_affine().inverse().transform_point3(fist);
        let frame = item_frame(fist_tool, pick.scale, None);
        let flame = rest.transform_point(frame.transform_point(FLAME_AT));
        assert!(
            (flame - fist).length() > 0.25,
            "the flame stands over the fist"
        );
        assert!(
            flame.x > 0.1 && flame.y > -0.2 && flame.z < -0.3,
            "{flame:?}"
        );
        let off_axis = flame.truncate().length() / -flame.z;
        assert!(off_axis > 0.2, "clear of the crosshair: {off_axis}");
    }

    /// The flame's cards are three quads, the inner ones smaller, standing on
    /// the view's up from the base.
    #[test]
    fn the_flame_is_three_cards_on_the_views_up() {
        let mut b = Build::default();
        flame_cards(
            &mut b,
            Vec3::new(0.2, 0.0, -0.6),
            Vec3::Y,
            Vec3::ZERO,
            0.19,
            1.0,
            0.0,
            11.0,
        );
        assert_eq!(b.p.len(), 18);
        let top = b.p.iter().map(|p| p[1]).fold(f32::MIN, f32::max);
        assert!((top - 0.19).abs() < 1e-4, "{top}");
        assert!(
            b.uv.iter()
                .all(|uv| (0.0..=1.0).contains(&uv[0]) && (0.0..=1.0).contains(&uv[1]))
        );
    }

    #[test]
    fn the_defaults_validate() {
        LightHold::default().validate().unwrap();
        let bad = LightHold {
            torch_flicker: 0.9,
            ..LightHold::default()
        };
        assert!(bad.validate().is_err());
    }
}
