//! The impostor bake (`flora-v2` design 10.3): the mockup's `bakeImpostors`
//! on the CPU. Each variant's near model is drawn orthographically from 8
//! bearings, tilted [`crate::tree_kit::BAKE_TILT`] down, and once from
//! straight above for the crown cap (`crossed-tree-cards` design 3), into a
//! strip of 48 x 72 px views at three times that size; every pixel is snapped to
//! Sweaft-64 in OKLab and each 3 x 3 block folded to one pixel. A small
//! depth-tested rasteriser does the drawing, so the bake is deterministic
//! and nothing is read back from the GPU.

use crate::tree_kit::{Look, OLIVE_COLOURS, Tex, Textures, TreeModel, TreeVertex, olive};

pub const VIEW_W: usize = 48;
pub const VIEW_H: usize = 72;
/// The bearings baked from the side, 45 degrees apart.
pub const VIEWS: usize = 8;
/// The view from straight above, after the side views: the model's width
/// across both of its axes, x along the view's 48 px and -z up its 72.
pub const TOP_VIEW: usize = VIEWS;
/// Views in a strip: the side bearings and the top.
pub const STRIP_VIEWS: usize = VIEWS + 1;
/// The strip's width and height, pixels.
pub const STRIP_W: usize = VIEW_W * STRIP_VIEWS;
const S3: usize = 3;

/// The Sweaft-64 palette (`foliage.html`, `SWEAFT`).
pub const SWEAFT: [u32; 64] = [
    0x010e1a, 0x011c34, 0x1f3b52, 0x3e586e, 0x5d778d, 0x92a3b2, 0xbfcad7, 0xfffefe, 0xa09f91,
    0x434137, 0x2c2c25, 0x161612, 0x010000, 0x1c020d, 0x4a051e, 0x841c1d, 0xd4462b, 0xfe6420,
    0xfb902f, 0xfaaf5c, 0x003227, 0x05582d, 0x297537, 0x559642, 0x9bb675, 0x0c2f44, 0x204f63,
    0x53998b, 0x84e3b5, 0xc1f1d8, 0x191954, 0x15578a, 0x1096c3, 0x74effe, 0xd2f8ff, 0x230c44,
    0x4f1d83, 0x7d2dbd, 0x9e3ba2, 0xc34988, 0xdd576d, 0x2b1f35, 0x3e3551, 0x69608b, 0x170916,
    0x2f1525, 0x522b2f, 0x7d4636, 0xa8633e, 0xe69d69, 0xeab798, 0x41322c, 0x725244, 0xab8465,
    0xcfaa81, 0x561c27, 0x733126, 0x8d4526, 0xc56d25, 0x2e0d06, 0x681712, 0xa61e1f, 0xd12d41,
    0xfa3a64,
];

/// sRGB bytes to OKLab (the mockup's `oklab`).
pub fn oklab(r: u8, g: u8, b: u8) -> [f64; 3] {
    let f = |c: u8| {
        let c = f64::from(c) / 255.0;
        if c <= 0.04045 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    };
    let (r, g, b) = (f(r), f(g), f(b));
    let l = (0.412_221_470_8 * r + 0.536_332_536_3 * g + 0.051_445_992_9 * b).cbrt();
    let m = (0.211_903_498_2 * r + 0.680_699_545_1 * g + 0.107_396_956_6 * b).cbrt();
    let s = (0.088_302_461_9 * r + 0.281_718_837_6 * g + 0.629_978_700_5 * b).cbrt();
    [
        0.210_454_255_3 * l + 0.793_617_785 * m - 0.004_072_046_8 * s,
        1.977_998_495_1 * l - 2.428_592_205 * m + 0.450_593_709_9 * s,
        0.025_904_037_1 * l + 0.782_771_766_2 * m - 0.808_675_766 * s,
    ]
}

fn bytes(c: u32) -> [u8; 3] {
    [(c >> 16) as u8, (c >> 8) as u8, c as u8]
}

struct Palette {
    rgb: Vec<[u8; 3]>,
    lab: Vec<[f64; 3]>,
}
impl Palette {
    fn new(olive: bool, own: &[[u8; 3]]) -> Self {
        let mut rgb: Vec<[u8; 3]> = SWEAFT.iter().map(|c| bytes(*c)).collect();
        if olive {
            rgb.extend(OLIVE_COLOURS.iter().map(|c| bytes(*c)));
        }
        rgb.extend_from_slice(own);
        let lab = rgb.iter().map(|c| oklab(c[0], c[1], c[2])).collect();
        Palette { rgb, lab }
    }
    /// Snapped hue first: a dark green must stay green.
    fn snap(&self, c: [u8; 3]) -> i16 {
        let q = oklab(c[0], c[1], c[2]);
        let mut best = 0;
        let mut bd = f64::MAX;
        for (i, p) in self.lab.iter().enumerate() {
            let d = (p[0] - q[0]).powi(2) + 8.0 * ((p[1] - q[1]).powi(2) + (p[2] - q[2]).powi(2));
            if d < bd {
                bd = d;
                best = i;
            }
        }
        best as i16
    }
}

/// One pass's picture: per pixel of the 3x strip, the snapped colour (or
/// -1 where nothing was drawn).
struct Target {
    colour: Vec<[u8; 3]>,
    depth: Vec<f32>,
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// The bake's shading (`FRAG`, `uBake`): the albedo pulled to the mean,
/// a fixed key light, and on the cards half-metre bands.
fn shade(v: &ShadeIn, look: &Look, tex: &Textures, means: &([f32; 3], [f32; 3])) -> [u8; 3] {
    let source: Tex = if v.leaf {
        look.leaf.unwrap()
    } else {
        look.bark
    };
    let t = tex.at_uv(source, v.uv[0], v.uv[1]);
    let mut alb = t.map(|c| (f32::from(c) / 255.0).powf(2.2));
    let mut mean = if v.leaf { means.1 } else { means.0 };
    if v.leaf && look.olive {
        alb = olive(alb);
        mean = olive(mean);
    }
    // A baked crown is seen at 48 px: its leaf tile's speckle would be
    // salt-and-pepper, so it keeps less (0.2) than a near card (0.45).
    let detail = if v.leaf { 0.2 } else { 0.7 };
    for c in 0..3 {
        alb[c] = mean[c] + (alb[c] - mean[c]) * detail;
    }
    let key = if v.leaf {
        1.15
    } else {
        let l = [-0.5f32, 0.8, 0.4];
        let k = (l[0] * l[0] + l[1] * l[1] + l[2] * l[2]).sqrt();
        0.85 + 0.3 * dot(v.normal, l.map(|x| x / k)).max(0.0)
    };
    let layer = if v.leaf {
        let band = (v.y / 0.5).rem_euclid(1.0);
        if band < 0.3 {
            0.85
        } else if band > 0.65 {
            1.08
        } else {
            1.0
        }
    } else {
        1.0
    };
    alb.map(|c| ((c * key * layer).clamp(0.0, 1.0).powf(1.0 / 2.2) * 255.0).round() as u8)
}

struct ShadeIn {
    uv: [f32; 2],
    normal: [f32; 3],
    y: f32,
    leaf: bool,
}

/// The crown's own three tones: the leaf tile's texels as the bake shades
/// them, sorted by luminance and cut in thirds, each third's mean. Added to
/// the snap palette as the mockup adds a tree's ramp colours (its acacia's
/// olive, "Sweaft has no dry olive"): Sweaft has no olive leaf green either,
/// and a fields crown snapped to Sweaft alone came out a darker, cooler
/// green than the near crown beside it.
fn leaf_ramp(look: &Look, tex: &Textures, means: &([f32; 3], [f32; 3])) -> Vec<[u8; 3]> {
    if look.leaf.is_none() {
        return Vec::new();
    }
    let mut shaded: Vec<[u8; 3]> = Vec::with_capacity(1024);
    for y in 0..32 {
        for x in 0..32 {
            let input = ShadeIn {
                uv: [(x as f32 + 0.5) / 32.0, 1.0 - (y as f32 + 0.5) / 32.0],
                normal: [0.0, 1.0, 0.0],
                y: 0.25,
                leaf: true,
            };
            shaded.push(shade(&input, look, tex, means));
        }
    }
    let lum = |c: &[u8; 3]| {
        0.2126 * f64::from(c[0]) + 0.7152 * f64::from(c[1]) + 0.0722 * f64::from(c[2])
    };
    shaded.sort_by(|a, b| lum(a).total_cmp(&lum(b)));
    shaded
        .chunks(shaded.len().div_ceil(3))
        .map(|third| {
            let mut m = [0.0f64; 3];
            for c in third {
                for i in 0..3 {
                    m[i] += (f64::from(c[i]) / 255.0).powf(2.2);
                }
            }
            m.map(|v| ((v / third.len() as f64).powf(1.0 / 2.2) * 255.0).round() as u8)
        })
        .collect()
}

/// Draw `verts` from bearing `view` (or from above, [`TOP_VIEW`]) into the
/// 3x strip.
fn draw(
    target: &mut Target,
    verts: &[TreeVertex],
    m: &TreeModel,
    view: usize,
    look: &Look,
    tex: &Textures,
    means: &([f32; 3], [f32; 3]),
) {
    let top = view == TOP_VIEW;
    let th = view as f64 * std::f64::consts::TAU / VIEWS as f64;
    let tilt = crate::tree_kit::BAKE_TILT;
    let (z, x) = if top {
        ([0.0, 1.0, 0.0], [1.0, 0.0, 0.0])
    } else {
        (
            [th.sin() * tilt.cos(), tilt.sin(), th.cos() * tilt.cos()].map(|x| x as f32),
            [th.cos(), 0.0, -th.sin()].map(|x| x as f32),
        )
    };
    let y = [
        z[1] * x[2] - z[2] * x[1],
        z[2] * x[0] - z[0] * x[2],
        z[0] * x[1] - z[1] * x[0],
    ];
    let (w3, h3) = ((VIEW_W * S3) as f32, (VIEW_H * S3) as f32);
    let x0 = view * VIEW_W * S3;
    let project = |p: [f32; 3]| {
        let up = if top {
            (dot(p, y) + m.width / 2.0) / m.width
        } else {
            (dot(p, y) - m.lo) / (m.hi - m.lo)
        };
        [
            (dot(p, x) + m.width / 2.0) / m.width * w3,
            up * h3,
            dot(p, z),
        ]
    };
    for tri in verts.chunks(3) {
        let s = [
            project(tri[0].position),
            project(tri[1].position),
            project(tri[2].position),
        ];
        let area =
            (s[1][0] - s[0][0]) * (s[2][1] - s[0][1]) - (s[2][0] - s[0][0]) * (s[1][1] - s[0][1]);
        if area.abs() < 1e-9 || (!tri[0].leaf && area < 0.0) {
            // Bark is single-sided, as the mockup's (`single: true`); cards
            // are seen from both sides.
            continue;
        }
        let lo_x = s
            .iter()
            .map(|p| p[0])
            .fold(f32::MAX, f32::min)
            .floor()
            .max(0.0) as usize;
        let hi_x = (s.iter().map(|p| p[0]).fold(f32::MIN, f32::max).ceil() as isize)
            .clamp(0, w3 as isize) as usize;
        let lo_y = s
            .iter()
            .map(|p| p[1])
            .fold(f32::MAX, f32::min)
            .floor()
            .max(0.0) as usize;
        let hi_y = (s.iter().map(|p| p[1]).fold(f32::MIN, f32::max).ceil() as isize)
            .clamp(0, h3 as isize) as usize;
        for py in lo_y..hi_y {
            for px in lo_x..hi_x {
                let (cx, cy) = (px as f32 + 0.5, py as f32 + 0.5);
                let e = |a: [f32; 3], b: [f32; 3]| {
                    (b[0] - a[0]) * (cy - a[1]) - (b[1] - a[1]) * (cx - a[0])
                };
                let w0 = e(s[1], s[2]) / area;
                let w1 = e(s[2], s[0]) / area;
                let w2 = e(s[0], s[1]) / area;
                if w0 < 0.0 || w1 < 0.0 || w2 < 0.0 {
                    continue;
                }
                let d = w0 * s[0][2] + w1 * s[1][2] + w2 * s[2][2];
                let i = py * STRIP_W * S3 + x0 + px;
                if d <= target.depth[i] {
                    continue;
                }
                let lerp2 = |k: usize| w0 * tri[0].uv[k] + w1 * tri[1].uv[k] + w2 * tri[2].uv[k];
                let input = ShadeIn {
                    uv: [lerp2(0), lerp2(1)],
                    normal: tri[0].normal,
                    y: w0 * tri[0].position[1] + w1 * tri[1].position[1] + w2 * tri[2].position[1],
                    leaf: tri[0].leaf,
                };
                target.depth[i] = d;
                target.colour[i] = shade(&input, look, tex, means);
            }
        }
    }
}

fn snapped(
    verts: &[TreeVertex],
    m: &TreeModel,
    look: &Look,
    tex: &Textures,
    means: &([f32; 3], [f32; 3]),
    pal: &Palette,
) -> Vec<i16> {
    let n = STRIP_W * S3 * VIEW_H * S3;
    let mut t = Target {
        colour: vec![[0; 3]; n],
        depth: vec![f32::MIN; n],
    };
    for view in 0..STRIP_VIEWS {
        draw(&mut t, verts, m, view, look, tex, means);
    }
    // A bake holds a few dozen distinct colours: snap each once.
    let mut seen = std::collections::HashMap::new();
    (0..n)
        .map(|i| {
            if t.depth[i] == f32::MIN {
                -1
            } else {
                *seen
                    .entry(t.colour[i])
                    .or_insert_with(|| pal.snap(t.colour[i]))
            }
        })
        .collect()
}

/// Alpha of a baked pixel that came from the crown (leaves), and of one from
/// the bark: both opaque, told apart so the shader lays the season and the
/// crown's colour match on the leaves alone.
pub const CROWN_ALPHA: u32 = 255;
pub const BARK_ALPHA: u32 = 254;

/// Bake one variant: leafy, or `bare` (bark only, a deciduous tree in
/// winter). Returns the strip's RGBA8 words, `STRIP_W` x `VIEW_H`, from the
/// bottom row up; alpha is 0, [`BARK_ALPHA`] or [`CROWN_ALPHA`].
pub fn bake(m: &TreeModel, look: &Look, tex: &Textures, bare: bool) -> Vec<u32> {
    let means = (
        tex.mean(look.bark),
        look.leaf.map_or([0.0; 3], |l| tex.mean(l)),
    );
    let pal = Palette::new(look.olive, &leaf_ramp(look, tex, &means));
    let (bark, leaves) = m.vertices.split_at(m.bark);
    // A tree with leaves: the crown and the bark are two passes, the bark
    // laid over the crown only where it fills most of a block.
    let split = !leaves.is_empty();
    let crown = if !split {
        snapped(bark, m, look, tex, &means, &pal)
    } else if bare {
        vec![-1; STRIP_W * S3 * VIEW_H * S3]
    } else {
        snapped(leaves, m, look, tex, &means, &pal)
    };
    let bark_pass = split.then(|| snapped(bark, m, look, tex, &means, &pal));
    let w3 = STRIP_W * S3;
    let block = |snap: &[i16], x: usize, y: usize| {
        let mut ids = Vec::with_capacity(9);
        for j in 0..S3 {
            for i in 0..S3 {
                let c = snap[(S3 * y + j) * w3 + S3 * x + i];
                if c >= 0 {
                    ids.push(c);
                }
            }
        }
        ids
    };
    let mode = |ids: &[i16]| {
        let mut best = ids[0];
        let mut bn = 0;
        for &c in ids {
            let n = ids.iter().filter(|d| **d == c).count();
            if n > bn || (n == bn && pal.lab[c as usize][1] < pal.lab[best as usize][1]) {
                bn = n;
                best = c;
            }
        }
        best
    };
    let mut out = vec![0u32; STRIP_W * VIEW_H];
    for y in 0..VIEW_H {
        for x in 0..STRIP_W {
            let ids = block(&crown, x, y);
            let mut best = -1;
            // A tree with no leaves draws its whole self in the crown pass.
            let mut alpha = if split { CROWN_ALPHA } else { BARK_ALPHA };
            if ids.len() >= 6 {
                best = mode(&ids);
            }
            if let Some(bp) = &bark_pass {
                let b = block(bp, x, y);
                // From above the crown hides the trunk and the boughs under
                // it: the bark shows only where no leaf is.
                let over = x / VIEW_W != TOP_VIEW && b.len() >= 6;
                if over || (best < 0 && b.len() >= 5) {
                    best = mode(&b);
                    alpha = BARK_ALPHA;
                }
            }
            if best < 0 {
                continue;
            }
            let c = pal.rgb[best as usize];
            out[y * STRIP_W + x] =
                u32::from(c[0]) | u32::from(c[1]) << 8 | u32::from(c[2]) << 16 | alpha << 24;
        }
    }
    out
}

/// The mean linear colour of a bake's crown pixels in its side views: what
/// the shader scales a baked crown by to meet the near crown's own mean
/// (design 10.3). Zero where the bake has no crown. The top view is left
/// out, so adding it moved no tree's colour.
pub fn crown_mean(bake: &[u32]) -> [f32; 3] {
    let mut m = [0.0f64; 3];
    let mut n = 0.0;
    let side = bake
        .iter()
        .enumerate()
        .filter(|(i, _)| i % STRIP_W < VIEWS * VIEW_W)
        .map(|(_, p)| p);
    for p in side.filter(|p| *p >> 24 == CROWN_ALPHA) {
        for (c, v) in m.iter_mut().enumerate() {
            *v += (f64::from((p >> (8 * c)) & 0xff) / 255.0).powf(2.2);
        }
        n += 1.0;
    }
    if n == 0.0 {
        return [0.0; 3];
    }
    m.map(|v| (v / n) as f32)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tree_kit::{Species, build, look};

    /// A stand-in texture set: every tile a two-tone check of greens and
    /// browns, so the bake has texels to read.
    fn textures() -> Textures {
        let mut atlas = vec![0u8; 512 * 512 * 3];
        for (i, p) in atlas.chunks_mut(3).enumerate() {
            let (x, y) = (i % 512, i / 512);
            let on = (x + y) % 2 == 0;
            p.copy_from_slice(if on { &[60, 110, 40] } else { &[90, 60, 30] });
        }
        let tile = (0..1024)
            .flat_map(|i| {
                if i % 3 == 0 {
                    [70, 120, 50, 255]
                } else {
                    [40, 80, 30, 255]
                }
            })
            .collect::<Vec<u8>>();
        Textures {
            atlas,
            atlas_size: 512,
            tiles: vec![tile; 4],
        }
    }

    #[test]
    fn a_bake_is_palette_pixels_with_hard_alpha() {
        let tex = textures();
        let mut pal: Vec<u32> = SWEAFT.iter().chain(OLIVE_COLOURS.iter()).copied().collect();
        for s in [Species::Oak, Species::Pine, Species::Acacia] {
            let lk = look(s);
            let means = (tex.mean(lk.bark), lk.leaf.map_or([0.0; 3], |l| tex.mean(l)));
            pal.extend(
                leaf_ramp(&lk, &tex, &means)
                    .iter()
                    .map(|c| u32::from(c[0]) << 16 | u32::from(c[1]) << 8 | u32::from(c[2])),
            );
        }
        for s in [
            Species::Oak,
            Species::Pine,
            Species::Acacia,
            Species::Cactus,
        ] {
            let m = build(s, 1);
            let lk = look(s);
            let a = bake(&m, &lk, &tex, false);
            assert_eq!(a, bake(&m, &lk, &tex, false), "deterministic");
            assert_eq!(a.len(), STRIP_W * VIEW_H);
            for p in &a {
                let alpha = p >> 24;
                assert!(alpha == 0 || alpha == BARK_ALPHA || alpha == CROWN_ALPHA);
                if alpha != 0 {
                    let rgb = (p & 0xff) << 16 | (p >> 8 & 0xff) << 8 | (p >> 16 & 0xff);
                    assert!(pal.contains(&rgb), "{rgb:06x}");
                }
            }
            for view in 0..STRIP_VIEWS {
                let filled = (0..VIEW_H)
                    .flat_map(|y| (0..VIEW_W).map(move |x| (x, y)))
                    .filter(|(x, y)| a[y * STRIP_W + view * VIEW_W + x] >> 24 != 0)
                    .count();
                assert!(filled > 150, "{s:?} view {view}: {filled} px");
            }
        }
        // The view from above is mostly crown for every leafy species.
        for s in [Species::Oak, Species::Pine, Species::Acacia] {
            let a = bake(&build(s, 0), &look(s), &tex, false);
            let top = |alpha: u32| {
                (0..VIEW_H)
                    .flat_map(|y| (0..VIEW_W).map(move |x| y * STRIP_W + TOP_VIEW * VIEW_W + x))
                    .filter(|i| a[*i] >> 24 == alpha)
                    .count()
            };
            let (crown, bark) = (top(CROWN_ALPHA), top(BARK_ALPHA));
            assert!(
                crown > 4 * bark,
                "{s:?} from above: {crown} crown, {bark} bark"
            );
        }
        // Bare is the bark alone: fewer pixels than the leafy crown.
        let m = build(Species::Oak, 0);
        let count = |v: &[u32]| v.iter().filter(|p| *p >> 24 != 0).count();
        assert!(
            count(&bake(&m, &look(Species::Oak), &tex, true)) * 2
                < count(&bake(&m, &look(Species::Oak), &tex, false))
        );
    }
}
