//! The detailed trees (`flora-v2` design 10): the approved M13 mockup's
//! parametric trunks, branches and leaf cards (`docs/mockups/foliage.html`,
//! `branchTree`, `TREE_BUILD`, `leafCard`), ported line for line, four
//! seeded variants a species, and packed with their baked impostors
//! ([`crate::tree_bake`]) into the one buffer the planet shader pulls them
//! from.
//!
//! A tree's species, variant, turn and size are decoration picked at draw
//! time from the tree's own id ([`pick`]): where a tree stands and whether a
//! cell can be chopped stay [`crate::flora::grows_tree`]'s, untouched.

use crate::flora::shader_hash;

/// The species the game's biomes grow, in the order of the near draws.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Species {
    Oak,
    Birch,
    Kapok,
    Palm,
    Acacia,
    Pine,
    Willow,
    Cactus,
}

pub const SPECIES: [Species; 8] = [
    Species::Oak,
    Species::Birch,
    Species::Kapok,
    Species::Palm,
    Species::Acacia,
    Species::Pine,
    Species::Willow,
    Species::Cactus,
];
/// Variants built per species, as the mockup builds (`VARS = 4`).
pub const VARIANTS: u32 = 4;

impl Species {
    pub fn index(self) -> u32 {
        SPECIES.iter().position(|s| *s == self).unwrap() as u32
    }
    /// The mockup's key, whose length seeds the variants.
    pub fn name(self) -> &'static str {
        match self {
            Species::Oak => "oak",
            Species::Birch => "birch",
            Species::Kapok => "kapok",
            Species::Palm => "palm",
            Species::Acacia => "acacia",
            Species::Pine => "pine",
            Species::Willow => "willow",
            Species::Cactus => "cactus",
        }
    }
}

/// Where a tree's texture comes from: a tile of the atlas (`planet_terrain`'s
/// sheet order, the tile's column and row), or one of the kit's own tiles
/// ([`KIT_TILES`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tex {
    Atlas { sheet: u32, col: u32, row: u32 },
    Kit(u32),
}

impl Tex {
    /// The shader's word: bit 31 for an atlas tile, then sheet, column, row.
    pub fn word(self) -> u32 {
        match self {
            Tex::Atlas { sheet, col, row } => 1 << 31 | sheet << 8 | col << 4 | row,
            Tex::Kit(k) => k,
        }
    }
}

/// The kit's own 32 x 32 tiles, the textures the trees wear that the atlas
/// does not hold, by their asset path under `assets/`.
pub const KIT_TILES: [&str; 4] = [
    "textures/settlement/leaves.png",
    "textures/settlement/needles.png",
    "textures/flora/bark_birch.png",
    "textures/flora/bark_palm.png",
];
const LEAVES: Tex = Tex::Kit(0);
const NEEDLES: Tex = Tex::Kit(1);
const BARK_BIRCH: Tex = Tex::Kit(2);
const BARK_PALM: Tex = Tex::Kit(3);
// Atlas sheets (`planet_terrain::TILESETS`, sorted names).
const DESERT: u32 = 3;
const FIELDS: u32 = 4;
const JUNGLE: u32 = 6;
const REDWOOD: u32 = 10;
const SWAMP: u32 = 12;
const fn bark_of(sheet: u32) -> Tex {
    Tex::Atlas {
        sheet,
        col: 2,
        row: 1,
    }
}
const fn leaves_of(sheet: u32) -> Tex {
    Tex::Atlas {
        sheet,
        col: 0,
        row: 2,
    }
}

/// A species' look (the mockup's `SPECIES`): its bark and leaf textures,
/// the pixel mask its cards are cut by (`maskOf`: 1 leaves on a twig, 2 a
/// needle spray, 4 a frond), whether it drops its leaves, and whether its
/// leaves wear the savanna's dry olive (`RAMPS.savannaLeaf`).
#[derive(Clone, Copy, Debug)]
pub struct Look {
    pub bark: Tex,
    pub leaf: Option<Tex>,
    pub mask: u32,
    pub deciduous: bool,
    pub olive: bool,
}

pub fn look(s: Species) -> Look {
    let l = |bark, leaf, mask, deciduous| Look {
        bark,
        leaf: Some(leaf),
        mask,
        deciduous,
        olive: false,
    };
    match s {
        Species::Oak => l(bark_of(FIELDS), leaves_of(FIELDS), 1, true),
        Species::Birch => l(BARK_BIRCH, LEAVES, 1, true),
        Species::Kapok => l(bark_of(JUNGLE), leaves_of(JUNGLE), 1, false),
        Species::Palm => l(BARK_PALM, leaves_of(JUNGLE), 4, false),
        Species::Acacia => Look {
            olive: true,
            ..l(bark_of(FIELDS), LEAVES, 1, false)
        },
        Species::Pine => l(bark_of(REDWOOD), NEEDLES, 2, false),
        Species::Willow => l(bark_of(SWAMP), leaves_of(SWAMP), 4, true),
        Species::Cactus => Look {
            bark: bark_of(DESERT),
            leaf: None,
            mask: 0,
            deciduous: false,
            olive: false,
        },
    }
}

/// Each biome's species and their shares (the mockup's `BIOMES[...].trees`
/// on the game's biomes, design 10.1), in `planet_gen::Biome` order: ocean,
/// beach, fields, desert, jungle, swamp, mountains, tundra. The desert's
/// cacti are scenery ([`grows_cactus`]); the rest stand on `grows_tree`'s
/// cells.
pub const BIOME_TREES: [&[(Species, f32)]; 8] = [
    &[],
    &[(Species::Acacia, 1.0)],
    &[(Species::Oak, 0.6), (Species::Birch, 0.4)],
    &[(Species::Cactus, 1.0)],
    &[(Species::Kapok, 0.55), (Species::Palm, 0.45)],
    &[(Species::Willow, 0.75), (Species::Birch, 0.25)],
    &[],
    &[(Species::Pine, 0.85), (Species::Birch, 0.15)],
];

/// The surface pass's `roll`: one decision about one cell, 0..1.
pub fn roll(id: u32, salt: u32) -> f32 {
    (shader_hash(id ^ salt.wrapping_mul(2_654_435_761)) & 0xff_ffff) as f32 / 16_777_216.0
}
pub const SALT_SPECIES: u32 = 0x59;
pub const SALT_VARIANT: u32 = 0x5a;
pub const SALT_TURN: u32 = 0x5b;
pub const SALT_SIZE: u32 = 0x5c;
pub const SALT_CACTUS: u32 = 0x5d;

/// Cacti per 256 desert sand cells (design 10.1): the mockup's desert `treeP`
/// 0.07 times its cactus share 0.8, about 14 in 256 of its cells on its
/// forest weighting; 8 here, where every sand cell is eligible.
pub const CACTUS_DENSITY: u32 = 8;

/// Whether a desert cell grows a cactus: scenery drawn by the tree draw, not
/// a tree (`grows_tree` stays false, so it cannot be chopped). `tree_id` and
/// `word` are the record's, as for `grows_tree`.
pub fn grows_cactus(tree_id: u32, word: u32) -> bool {
    if (word >> 24) & 1 == 1 {
        return false;
    }
    let material = word & 0xff;
    let biome = (word >> 8) & 0xff;
    biome == 3
        && (material == 1 || material == 4)
        && shader_hash(tree_id ^ SALT_CACTUS) & 0xff < CACTUS_DENSITY
}

/// What the draw makes of a tree: its species, variant (0..4), turn about
/// the up axis (radians) and size (the mockup's 0.85..1.2).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pick {
    pub species: Species,
    pub variant: u32,
    pub turn: f32,
    pub size: f32,
}

/// The tree a cell stands, from its id and biome: decoration, derived at
/// draw time (design 10.1). None where the biome grows no tree.
pub fn pick(tree_id: u32, biome: u32) -> Option<Pick> {
    let mix = BIOME_TREES.get(biome as usize)?;
    let total: f32 = mix.iter().map(|(_, w)| w).sum();
    if mix.is_empty() {
        return None;
    }
    let mut at = roll(tree_id, SALT_SPECIES) * total;
    let mut species = mix[mix.len() - 1].0;
    for (s, w) in mix.iter() {
        if at < *w {
            species = *s;
            break;
        }
        at -= w;
    }
    Some(Pick {
        species,
        variant: ((roll(tree_id, SALT_VARIANT) * VARIANTS as f32) as u32).min(VARIANTS - 1),
        turn: roll(tree_id, SALT_TURN) * std::f32::consts::TAU,
        size: 0.85 + roll(tree_id, SALT_SIZE) * 0.35,
    })
}

// ---- the mockup's builders ----

type V3 = [f64; 3];
fn add(a: V3, b: V3) -> V3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}
fn sub(a: V3, b: V3) -> V3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn scl(a: V3, k: f64) -> V3 {
    [a[0] * k, a[1] * k, a[2] * k]
}
fn cross(a: V3, b: V3) -> V3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
fn len(a: V3) -> f64 {
    (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt()
}
fn norm(a: V3) -> V3 {
    let l = len(a);
    let l = if l == 0.0 { 1.0 } else { l };
    scl(a, 1.0 / l)
}
fn lerp3(a: V3, b: V3, t: f64) -> V3 {
    add(a, scl(sub(b, a), t))
}
/// `rotAxis`: `v` turned by `ang` about the unit `axis` (three.js's
/// quaternion from an axis and an angle).
fn rot_axis(v: V3, axis: V3, ang: f64) -> V3 {
    let (s, c) = ang.sin_cos();
    let k = norm(axis);
    let kv = cross(k, v);
    let kd = k[0] * v[0] + k[1] * v[1] + k[2] * v[2];
    add(add(scl(v, c), scl(kv, s)), scl(k, kd * (1.0 - c)))
}

/// The mockup's `rngOf`: xorshift32, 0..1.
pub struct Rng(u32);
impl Rng {
    pub fn new(seed: u32) -> Self {
        Rng(if seed == 0 { 1 } else { seed })
    }
    pub fn float(&mut self) -> f64 {
        let mut s = self.0;
        s ^= s << 13;
        s ^= s >> 17;
        s ^= s << 5;
        self.0 = s;
        f64::from(s) / 4_294_967_296.0
    }
}

/// One vertex of a near tree, in the tree's own frame (y up, metres).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TreeVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
    /// The card's id 0..1 (zero on bark): its autumn colour and its turn to
    /// fall in winter.
    pub card: f32,
    /// The mockup's root-to-tip AO, 0..1.
    pub ao: f32,
    pub leaf: bool,
}

/// The mockup's `GB`, one for bark and one for leaves.
#[derive(Default)]
struct Gb {
    v: Vec<TreeVertex>,
    leaf: bool,
}
impl Gb {
    fn new(leaf: bool) -> Self {
        Gb {
            v: Vec::new(),
            leaf,
        }
    }
    #[allow(clippy::too_many_arguments)]
    fn tri(&mut self, p: [V3; 3], uv: [[f64; 2]; 3], n: Option<[V3; 3]>, card: f64, ao: [f64; 3]) {
        let n = n.unwrap_or_else(|| {
            let f = norm(cross(sub(p[1], p[0]), sub(p[2], p[0])));
            [f, f, f]
        });
        for i in 0..3 {
            self.v.push(TreeVertex {
                position: p[i].map(|x| x as f32),
                normal: n[i].map(|x| x as f32),
                uv: uv[i].map(|x| x as f32),
                card: card as f32,
                ao: ao[i] as f32,
                leaf: self.leaf,
            });
        }
    }
    fn quad(&mut self, q: [V3; 4], uv: [[f64; 2]; 4], n: V3, card: f64, ao: [f64; 4]) {
        let n = Some([n, n, n]);
        self.tri(
            [q[0], q[1], q[2]],
            [uv[0], uv[1], uv[2]],
            n,
            card,
            [ao[0], ao[1], ao[2]],
        );
        self.tri(
            [q[0], q[2], q[3]],
            [uv[0], uv[2], uv[3]],
            n,
            card,
            [ao[0], ao[2], ao[3]],
        );
    }
    /// A prism of `sides` from `p0` to `p1`, radius `r0` to `r1`.
    #[allow(clippy::too_many_arguments)]
    fn tube(&mut self, p0: V3, p1: V3, r0: f64, r1: f64, sides: u32, v_scale: f64) {
        let ax = norm(sub(p1, p0));
        let length = len(sub(p1, p0));
        let t1 = norm(if ax[1].abs() < 0.95 {
            cross(ax, [0.0, 1.0, 0.0])
        } else {
            cross(ax, [1.0, 0.0, 0.0])
        });
        let t2 = cross(ax, t1);
        let tau = std::f64::consts::PI * 2.0;
        for i in 0..sides {
            let a0 = f64::from(i) / f64::from(sides) * tau;
            let a1 = f64::from(i + 1) / f64::from(sides) * tau;
            let d0 = add(scl(t1, a0.cos()), scl(t2, a0.sin()));
            let d1 = add(scl(t1, a1.cos()), scl(t2, a1.sin()));
            let a = add(p0, scl(d0, r0));
            let b = add(p0, scl(d1, r0));
            let c = add(p1, scl(d1, r1));
            let d = add(p1, scl(d0, r1));
            let nm = norm(add(d0, d1));
            let (u0, u1, v1) = (
                f64::from(i) / f64::from(sides),
                f64::from(i + 1) / f64::from(sides),
                length / v_scale,
            );
            self.quad(
                [a, b, c, d],
                [[u0, 0.0], [u1, 0.0], [u1, v1], [u0, v1]],
                nm,
                0.0,
                [0.7, 0.7, 1.0, 1.0],
            );
        }
    }
}

/// `leafCard`: a card rooted on its branch at `root`, running out along
/// `dir` to its tip; the root edge half the tip's width.
#[allow(clippy::too_many_arguments)]
fn leaf_card(
    gb: &mut Gb,
    root: V3,
    dir: V3,
    length: f64,
    w: f64,
    rnd: &mut Rng,
    cid: f64,
    face: V3,
    n_out: Option<V3>,
) {
    let dir = norm(dir);
    let mut side = cross(face, dir);
    if len(side) < 1e-3 {
        side = cross([1.0, 0.0, 0.0], dir);
    }
    side = norm(rot_axis(norm(side), dir, (rnd.float() - 0.5) * 0.9));
    let root = add(root, scl(side, (rnd.float() - 0.5) * 0.04));
    let mut n = norm(cross(dir, side));
    if n[1] < 0.0 {
        n = scl(n, -1.0);
    }
    let nn = match n_out {
        Some(o) => norm(add(scl(o, 0.6), scl(n, 0.4))),
        None => n,
    };
    let tip = add(root, scl(dir, length));
    let a = add(root, scl(side, -w * 0.25));
    let b = add(root, scl(side, w * 0.25));
    let c = add(tip, scl(side, w / 2.0));
    let d = add(tip, scl(side, -w / 2.0));
    gb.quad(
        [a, b, c, d],
        [[0.25, 0.0], [0.75, 0.0], [1.0, 1.0], [0.0, 1.0]],
        nn,
        cid,
        [0.7, 0.7, 1.0, 1.0],
    );
}

/// `branchTree`'s parameters (the mockup's `P`).
struct P {
    trunk_h: f64,
    trunk_r: f64,
    crown: u32,
    len: f64,
    depth: u32,
    kids: u32,
    spread: f64,
    up: f64,
    shrink: f64,
    taper: f64,
    leaf_s: f64,
    cards: u32,
    fan: f64,
    droop: f64,
}

struct Twig {
    a: V3,
    b: V3,
    d: V3,
}

/// `branchTree`: recursing branches, every last-order branch carrying leaf
/// cards rooted along it. Returns the twigs and pushes the fruit anchors.
fn branch_tree(
    bark: &mut Gb,
    leaves: &mut Gb,
    rnd: &mut Rng,
    p: &P,
    anchors: &mut Vec<V3>,
) -> Vec<Twig> {
    let mut twigs = Vec::new();
    #[allow(clippy::too_many_arguments)]
    fn br(
        bark: &mut Gb,
        rnd: &mut Rng,
        p: &P,
        twigs: &mut Vec<Twig>,
        at: V3,
        dir: V3,
        length: f64,
        r: f64,
        depth: u32,
    ) {
        let end = add(at, scl(dir, length));
        let sides = if depth + 1 >= p.depth { 6 } else { 5 };
        bark.tube(at, end, r, r * p.taper, sides, 1.2);
        if depth <= 1 {
            twigs.push(Twig {
                a: at,
                b: end,
                d: dir,
            });
        }
        if depth == 0 {
            return;
        }
        for k in 0..p.kids {
            let t = if p.kids == 1 {
                1.0
            } else {
                0.55 + 0.45 * f64::from(k + 1) / f64::from(p.kids)
            };
            let start = add(at, scl(dir, length * t));
            let az = rnd.float() * std::f64::consts::PI * 2.0 + f64::from(k) * 2.1;
            let side = norm([az.cos(), 0.0, az.sin()]);
            let mut nd = norm(add(
                scl(dir, p.spread.cos()),
                scl(side, p.spread.sin() * (0.7 + rnd.float() * 0.6)),
            ));
            nd = norm(add(nd, [0.0, p.up, 0.0]));
            let next_len = length * p.shrink * (0.8 + rnd.float() * 0.4);
            br(
                bark,
                rnd,
                p,
                twigs,
                start,
                nd,
                next_len,
                r * p.taper * 0.85,
                depth - 1,
            );
        }
    }
    let top = [0.0, p.trunk_h, 0.0];
    bark.tube([0.0; 3], top, p.trunk_r, p.trunk_r * 0.75, 6, 1.2);
    for k in 0..p.crown {
        let az = f64::from(k) / f64::from(p.crown) * std::f64::consts::PI * 2.0 + rnd.float();
        let d = norm([
            az.cos() * p.spread.sin(),
            p.spread.cos() + p.up,
            az.sin() * p.spread.sin(),
        ]);
        let at = add(top, [0.0, -rnd.float() * p.trunk_h * 0.25, 0.0]);
        br(
            bark,
            rnd,
            p,
            &mut twigs,
            at,
            d,
            p.len,
            p.trunk_r * 0.6,
            p.depth,
        );
    }
    let centre = [0.0, p.trunk_h + p.len * 0.8, 0.0];
    for tw in &twigs {
        let cid = 0.05 + rnd.float() * 0.95;
        for c in 0..p.cards {
            let t = 0.3 + 0.7 * (f64::from(c) + 0.5) / f64::from(p.cards);
            let root = lerp3(tw.a, tw.b, t);
            let side = norm(cross(tw.d, [0.0, 1.0, 0.0]));
            let sign = if c % 2 == 1 { 1.0 } else { -1.0 };
            let fan = add(tw.d, scl(side, sign * p.fan * (0.6 + rnd.float() * 0.6)));
            let dir = norm(add(fan, [0.0, -p.droop + (rnd.float() - 0.5) * 0.3, 0.0]));
            let length = p.leaf_s * (0.8 + rnd.float() * 0.4);
            leaf_card(
                leaves,
                root,
                dir,
                length,
                p.leaf_s * 0.8,
                rnd,
                cid,
                [0.0, 1.0, 0.0],
                Some(norm(sub(root, centre))),
            );
        }
        leaf_card(
            leaves,
            tw.b,
            norm(add(tw.d, [0.0, 0.2, 0.0])),
            p.leaf_s,
            p.leaf_s * 0.8,
            rnd,
            cid,
            [0.0, 1.0, 0.0],
            Some(norm(sub(tw.b, centre))),
        );
    }
    for tw in &twigs {
        anchors.push(tw.b);
        anchors.push(lerp3(tw.a, tw.b, 0.6));
    }
    twigs
}

/// One built variant: its vertices (bark first, then leaves), its fruit
/// anchors and the frame the bake fits it in.
#[derive(Clone, Debug)]
pub struct TreeModel {
    pub species: Species,
    pub variant: u32,
    pub vertices: Vec<TreeVertex>,
    /// How many of `vertices` are bark (they come first).
    pub bark: usize,
    /// How many are vines (they come last, design 12.4).
    pub vines: usize,
    pub anchors: Vec<[f32; 3]>,
    /// The mockup's `Hh` and `Wd`: the height and the width the impostor
    /// spans, metres.
    pub height: f32,
    pub width: f32,
    /// The bake's frame, from just under the base to the tilted top.
    pub lo: f32,
    pub hi: f32,
}

/// How far above the horizontal the bake looks down (`BAKE_TILT`).
pub const BAKE_TILT: f64 = 0.55;

/// Build variant `v` of a species, seeded as the mockup seeds it
/// (`rngOf(9000 + v * 31 + sp.length * 7)`). The mockup's own literals
/// (its 6.28 and 3.14) are kept, so a variant is the tree it draws.
#[allow(clippy::approx_constant)]
pub fn build(species: Species, v: u32) -> TreeModel {
    let mut rnd = Rng::new(9000 + v * 31 + species.name().len() as u32 * 7);
    let rnd = &mut rnd;
    let mut b = Gb::new(false);
    let mut l = Gb::new(true);
    let mut anchors: Vec<V3> = Vec::new();
    let up: V3 = [0.0, 1.0, 0.0];
    match species {
        Species::Oak => {
            let p = P {
                trunk_h: 2.4,
                trunk_r: 0.24,
                crown: 4,
                len: 1.7,
                depth: 2,
                kids: 3,
                spread: 0.7,
                up: 0.25,
                shrink: 0.72,
                taper: 0.7,
                leaf_s: 1.1,
                cards: 3,
                fan: 0.9,
                droop: 0.15,
            };
            branch_tree(&mut b, &mut l, rnd, &p, &mut anchors);
        }
        Species::Birch => {
            let p = P {
                trunk_h: 4.6,
                trunk_r: 0.13,
                crown: 4,
                len: 1.3,
                depth: 2,
                kids: 2,
                spread: 0.45,
                up: 0.5,
                shrink: 0.75,
                taper: 0.7,
                leaf_s: 0.85,
                cards: 3,
                fan: 0.8,
                droop: 0.35,
            };
            branch_tree(&mut b, &mut l, rnd, &p, &mut anchors);
        }
        Species::Willow => {
            let p = P {
                trunk_h: 2.8,
                trunk_r: 0.3,
                crown: 5,
                len: 1.8,
                depth: 1,
                kids: 3,
                spread: 0.9,
                up: 0.35,
                shrink: 0.75,
                taper: 0.7,
                leaf_s: 0.9,
                cards: 2,
                fan: 0.7,
                droop: 0.3,
            };
            let twigs = branch_tree(&mut b, &mut l, rnd, &p, &mut anchors);
            for tw in &twigs {
                // strands hang from the branch tip: root up on the branch, tip down
                let out = norm([tw.d[0], 0.0, tw.d[2]]);
                let length = 2.4 + rnd.float() * 1.6;
                let cid = 0.1 + rnd.float() * 0.9;
                leaf_card(
                    &mut l,
                    tw.b,
                    norm([out[0] * 0.25, -1.0, out[2] * 0.25]),
                    length,
                    0.9,
                    rnd,
                    cid,
                    out,
                    Some(out),
                );
            }
        }
        Species::Acacia => {
            b.tube([0.0; 3], [0.0, 2.0, 0.0], 0.2, 0.16, 6, 1.0);
            let n = 3 + (rnd.float() * 2.0) as u32;
            for k in 0..n {
                let az = f64::from(k) / f64::from(n) * 6.28 + rnd.float();
                let d = norm([az.cos() * 0.8, 1.0, az.sin() * 0.8]);
                let e = add([0.0, 1.9, 0.0], scl(d, 2.4));
                b.tube([0.0, 1.9, 0.0], e, 0.13, 0.07, 5, 1.0);
                for j in 0..2 {
                    let az2 = az + (f64::from(j) - 0.5) * 0.9;
                    let e2 = add(e, [az2.cos() * 1.2, 0.35, az2.sin() * 1.2]);
                    b.tube(e, e2, 0.07, 0.04, 5, 1.0);
                    anchors.push(lerp3(e, e2, 0.55));
                    anchors.push(lerp3(e, e2, 0.85));
                    let tw = norm(sub(e2, e));
                    let cid = 0.1 + rnd.float() * 0.9;
                    for c in 0..5 {
                        let root = lerp3(e, e2, 0.4 + f64::from(c) * 0.15);
                        let a = (f64::from(c) - 2.0) * 0.5 + (rnd.float() - 0.5) * 0.3;
                        let dir = norm(add(rot_axis(tw, up, a), [0.0, 0.05, 0.0]));
                        leaf_card(&mut l, root, dir, 1.3, 1.0, rnd, cid, up, None);
                    }
                }
            }
        }
        Species::Pine => {
            let h = 8.5 + rnd.float() * 1.5;
            b.tube([0.0; 3], [0.0, h, 0.0], 0.2, 0.03, 6, 1.2);
            let mut y = 1.2;
            while y < h - 0.3 {
                let t = (y - 1.2) / (h - 1.2);
                let l_br = (1.0 - t) * 2.3 + 0.35;
                let n = 6;
                for k in 0..n {
                    let az = f64::from(k) / f64::from(n) * 6.28 + y * 1.7;
                    let d = norm([az.cos(), -0.22, az.sin()]);
                    let base = [0.0, y, 0.0];
                    let e = add(base, scl(d, l_br));
                    b.tube(base, e, 0.045, 0.015, 4, 1.0);
                    let cid = 0.1 + rnd.float() * 0.9;
                    let side = norm(cross(d, up));
                    let n2 = ((l_br / 0.32 + 0.5).floor() as u32).max(2);
                    for c in 0..n2 {
                        let tr = (f64::from(c) + 0.3) / f64::from(n2);
                        let root = add(base, scl(d, l_br * tr));
                        let s2 = if c % 2 == 1 { 1.0 } else { -1.0 };
                        let length = (l_br * (1.0 - tr) + 0.3).min(0.75);
                        let dir = norm(add(add(d, scl(side, s2 * 0.55)), [0.0, -0.3, 0.0]));
                        let face = norm(add(up, scl(side, s2 * 1.1)));
                        leaf_card(&mut l, root, dir, length, 0.55, rnd, cid, face, None);
                    }
                    leaf_card(
                        &mut l,
                        add(base, scl(d, l_br * 0.92)),
                        norm(add(d, [0.0, -0.25, 0.0])),
                        0.4,
                        0.36,
                        rnd,
                        cid,
                        norm(add(up, scl(side, 0.8))),
                        None,
                    );
                    if t > 0.25 && t < 0.8 {
                        anchors.push(add(base, scl(d, l_br * (0.55 + 0.3 * rnd.float()))));
                    }
                }
                y += 0.42 + rnd.float() * 0.15;
            }
            for k in 0..3 {
                let a = f64::from(k) * 2.1;
                leaf_card(
                    &mut l,
                    [0.0, h - 0.7, 0.0],
                    norm([a.cos() * 0.3, 1.0, a.sin() * 0.3]),
                    0.8,
                    0.4,
                    rnd,
                    0.5,
                    [(a + 1.57).cos(), 0.0, (a + 1.57).sin()],
                    None,
                );
            }
        }
        Species::Kapok => {
            let h = 7.0 + rnd.float();
            b.tube([0.0; 3], [0.0, h, 0.0], 0.36, 0.24, 6, 1.4);
            for k in 0..4 {
                let az = f64::from(k) * 1.57 + rnd.float() * 0.4;
                b.tube(
                    [az.cos() * 0.9, 0.0, az.sin() * 0.9],
                    [0.0, 1.8, 0.0],
                    0.12,
                    0.05,
                    4,
                    1.0,
                );
            }
            for k in 0..6 {
                let az = f64::from(k) / 6.0 * 6.28 + rnd.float() * 0.5;
                let ex = az.cos() * (2.4 + rnd.float());
                let ey = h + 0.6 + rnd.float() * 0.6;
                let ez = az.sin() * (2.4 + rnd.float());
                let e = [ex, ey, ez];
                let s = [0.0, h - 0.4, 0.0];
                b.tube(s, e, 0.16, 0.07, 5, 1.0);
                anchors.push(lerp3(s, e, 0.55));
                anchors.push(lerp3(s, e, 0.8));
                let d = norm(sub(e, s));
                let side = norm(cross(d, up));
                let cid = 0.1 + rnd.float() * 0.9;
                for c in 0..6 {
                    let root = lerp3(s, e, 0.45 + f64::from(c) * 0.1);
                    let sign = if c % 2 == 1 { 1.0 } else { -1.0 };
                    let fan = add(d, scl(side, sign * (0.5 + rnd.float() * 0.6)));
                    let dir = norm(add(fan, [0.0, 0.15 - rnd.float() * 0.3, 0.0]));
                    leaf_card(
                        &mut l,
                        root,
                        dir,
                        1.6,
                        1.3,
                        rnd,
                        cid,
                        up,
                        Some(norm(sub(root, [0.0, h, 0.0]))),
                    );
                }
            }
        }
        Species::Palm => {
            let h = 6.0 + rnd.float() * 1.5;
            let lean = (rnd.float() - 0.5) * 1.6;
            let mut prev = [0.0; 3];
            for i in 1..=6 {
                let t = f64::from(i) / 6.0;
                let p = [lean * t * t, h * t, 0.0];
                b.tube(prev, p, 0.19 - t * 0.05, 0.19 - t * 0.06, 6, 0.6);
                prev = p;
            }
            let top = prev;
            for k in 0..6 {
                let a = f64::from(k) * 1.05;
                anchors.push(add(top, [a.cos() * 0.22, -0.05, a.sin() * 0.22]));
            }
            for k in 0..9 {
                let az = f64::from(k) / 9.0 * 6.28 + rnd.float() * 0.3;
                let d = [az.cos(), 0.0, az.sin()];
                let side = [-d[2], 0.0, d[0]];
                let cid = 0.1 + rnd.float() * 0.9;
                for s in 0..4 {
                    let t0 = f64::from(s) / 4.0;
                    let t1 = f64::from(s + 1) / 4.0;
                    let ll = 2.8;
                    let y0 = 0.5 * t0 - 2.0 * t0 * t0;
                    let y1 = 0.5 * t1 - 2.0 * t1 * t1;
                    let w0 = 0.15 + 0.45 * (std::f64::consts::PI * t0.max(0.15)).sin();
                    let w1 = 0.15 + 0.45 * (std::f64::consts::PI * t1).sin();
                    let a = add(top, [d[0] * ll * t0, y0, d[2] * ll * t0]);
                    let c = add(top, [d[0] * ll * t1, y1, d[2] * ll * t1]);
                    let nn = norm([d[0] * 0.3, 1.0, d[2] * 0.3]);
                    l.quad(
                        [
                            sub(a, scl(side, w0)),
                            add(a, scl(side, w0)),
                            add(c, scl(side, w1)),
                            sub(c, scl(side, w1)),
                        ],
                        [[0.0, t0], [1.0, t0], [1.0, t1], [0.0, t1]],
                        nn,
                        cid,
                        [0.8, 0.8, 1.0, 1.0],
                    );
                }
            }
        }
        Species::Cactus => {
            let h = 3.0 + rnd.float() * 1.5;
            b.tube([0.0; 3], [0.0, h, 0.0], 0.26, 0.24, 8, 1.0);
            b.tube([0.0, h, 0.0], [0.0, h + 0.2, 0.0], 0.24, 0.08, 8, 1.0);
            let n = 1 + (rnd.float() * 2.5) as u32;
            for k in 0..n {
                let az = f64::from(k) * 3.14 + rnd.float();
                let d = [az.cos(), 0.0, az.sin()];
                let y = 1.1 + rnd.float() * 1.2;
                let e = add([0.0, y, 0.0], scl(d, 0.7));
                let top = add(e, [0.0, 0.9 + rnd.float() * 0.8, 0.0]);
                b.tube([0.0, y, 0.0], e, 0.15, 0.15, 8, 1.0);
                b.tube(add(e, [0.0, -0.1, 0.0]), top, 0.15, 0.13, 8, 1.0);
                b.tube(top, add(top, [0.0, 0.12, 0.0]), 0.13, 0.05, 8, 1.0);
            }
        }
    }
    // Vines (`flora-v2` design 12.4): M13 hangs them from the jungle's and
    // the swamp's trees. A strand of narrow leaf cards falls from some of the
    // model's anchors, on a stream of its own so the tree it hangs from is the
    // one it always was. No strand is longer than the lowest anchor is high.
    let vine_chance = match species {
        Species::Kapok => 0.35,
        Species::Palm => 0.2,
        Species::Willow => 0.25,
        _ => 0.0,
    };
    let leaves = l.v.len();
    if vine_chance > 0.0 && !anchors.is_empty() {
        let mut vr = Rng::new(0x7a1e + v * 131 + species.index() * 17);
        let floor = anchors.iter().map(|a| a[1]).fold(f64::MAX, f64::min);
        let reach = (floor * 0.8).max(0.0);
        for (k, a) in anchors.clone().iter().enumerate() {
            if vr.float() >= vine_chance {
                continue;
            }
            let cards = 3 + (vr.float() * 4.0) as u32;
            let piece = (reach / f64::from(cards)).min(0.5);
            if piece < 0.1 {
                continue;
            }
            let az = vr.float() * 6.28;
            let face = [az.cos(), 0.0, az.sin()];
            let mut at = *a;
            for c in 0..cards {
                let sway = (vr.float() - 0.5) * 0.12;
                let dir = norm([sway, -1.0, sway * 0.5]);
                let cid = 900.0 + k as f64 + f64::from(c) * 0.01;
                leaf_card(&mut l, at, dir, piece, 0.16, &mut vr, cid, face, None);
                at = add(at, scl(dir, piece * 0.95));
            }
        }
    }
    let vines = l.v.len() - leaves;
    let bark = b.v.len();
    let mut vertices = b.v;
    vertices.extend(l.v);
    // The mockup's box: Hh = max y * 1.02, Wd = twice the widest reach * 1.04.
    let (mut top, mut reach) = (0.0f32, 0.0f32);
    for v in &vertices {
        top = top.max(v.position[1]);
        reach = reach.max(v.position[0].abs()).max(v.position[2].abs());
    }
    let height = top * 1.02;
    let width = 2.0 * reach * 1.04;
    let tilt = BAKE_TILT as f32;
    TreeModel {
        species,
        variant: v,
        vertices,
        bark,
        vines,
        anchors: anchors.into_iter().map(|a| a.map(|x| x as f32)).collect(),
        height,
        width,
        lo: -width / 2.0 * tilt.sin(),
        hi: height * tilt.cos() + width / 2.0 * tilt.sin(),
    }
}

// ---- the kit buffer (design 10.2): what `planet_surface.wgsl` reads ----

/// Words per species header, and where the variants' headers start.
pub const SPECIES_WORDS: u32 = 16;
pub const VARIANT_BASE: u32 = SPECIES.len() as u32 * SPECIES_WORDS;
pub const VARIANT_WORDS: u32 = 16;
/// Where the kit's own tiles start, and each one's words (32 x 32 RGBA8).
pub const TILE_BASE: u32 = VARIANT_BASE + SPECIES.len() as u32 * VARIANTS * VARIANT_WORDS;
pub const TILE_WORDS: u32 = 32 * 32;
/// Words per vertex.
pub const VERTEX_WORDS: u32 = 6;

/// A texel source for the bake and the means: an atlas (RGB, 512 x 512,
/// row-major from the top) and the kit's tiles (RGBA, 32 x 32 each).
pub struct Textures {
    pub atlas: Vec<u8>,
    pub atlas_size: u32,
    pub tiles: Vec<Vec<u8>>,
}

impl Textures {
    /// The sRGB texel of `tex` at texel (x, y), y from the tile's top row.
    pub fn texel(&self, tex: Tex, x: u32, y: u32) -> [u8; 3] {
        let (x, y) = (x.min(31), y.min(31));
        match tex {
            Tex::Atlas { sheet, col, row } => {
                let k = self.atlas_size / 512;
                let px = ((sheet % 4) * 128 + col * 32 + x) * k;
                let py = ((sheet / 4) * 128 + row * 32 + y) * k;
                let i = ((py * self.atlas_size + px) * 3) as usize;
                [self.atlas[i], self.atlas[i + 1], self.atlas[i + 2]]
            }
            Tex::Kit(t) => {
                let i = ((y * 32 + x) * 4) as usize;
                let tile = &self.tiles[t as usize];
                [tile[i], tile[i + 1], tile[i + 2]]
            }
        }
    }
    /// The mockup's `atlasTile`: the texel a uv falls in, v counted up from
    /// the tile's bottom (`fract(1 - v)`), nearest.
    pub fn at_uv(&self, tex: Tex, u: f32, v: f32) -> [u8; 3] {
        let x = (u.rem_euclid(1.0) * 32.0).floor() as u32;
        let y = ((1.0 - v).rem_euclid(1.0) * 32.0).floor() as u32;
        self.texel(tex, x, y)
    }
    /// The mean LINEAR colour of a tile (`meanOf`: every texel, 2.2 gamma).
    pub fn mean(&self, tex: Tex) -> [f32; 3] {
        let mut m = [0.0f64; 3];
        for y in 0..32 {
            for x in 0..32 {
                let t = self.texel(tex, x, y);
                for c in 0..3 {
                    m[c] += (f64::from(t[c]) / 255.0).powf(2.2);
                }
            }
        }
        m.map(|c| (c / 1024.0) as f32)
    }
}

/// The savanna's dry olive (`RAMPS.savannaLeaf`): linear luminance 0.07 to
/// 0.23 through three sRGB colours, as linear colour.
pub fn olive(c: [f32; 3]) -> [f32; 3] {
    let l = 0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2];
    let t = ((l - 0.07) / (0.23 - 0.07)).clamp(0.0, 1.0);
    let lin = |h: u32| {
        [(h >> 16) & 0xff, (h >> 8) & 0xff, h & 0xff].map(|v| (v as f32 / 255.0).powf(2.2))
    };
    let (a, b, d) = (lin(0x3e3a1a), lin(0x7a7432), lin(0xb7a95a));
    let mix = |x: [f32; 3], y: [f32; 3], k: f32| [0, 1, 2].map(|i| x[i] + (y[i] - x[i]) * k);
    if t < 0.5 {
        mix(a, b, t * 2.0)
    } else {
        mix(b, d, t * 2.0 - 1.0)
    }
}
/// The olive ramp's own colours, added to the snap palette for an olive
/// crown (Sweaft has no dry olive).
pub const OLIVE_COLOURS: [u32; 3] = [0x3e3a1a, 0x7a7432, 0xb7a95a];

/// The whole kit: every variant built, baked, and packed.
pub struct Kit {
    pub models: Vec<TreeModel>,
    pub words: Vec<u32>,
    /// Per species, the near draw's vertex count before the fruit: its
    /// largest variant's.
    pub vertex_max: [u32; 8],
}

fn f(x: f32) -> u32 {
    x.to_bits()
}

/// How many words past an impostor strip's first its relief strip starts
/// (`tree-impostor-depth`): one strip's pixels.
pub const IMPOSTOR_RELIEF: u32 = (crate::tree_bake::STRIP_W * crate::tree_bake::VIEW_H) as u32;

/// Build every species' variants, bake their impostors and pack the kit.
pub fn build_kit(tex: &Textures) -> Kit {
    let mut models = Vec::new();
    for s in SPECIES {
        for v in 0..VARIANTS {
            models.push(build(s, v));
        }
    }
    let mut words = vec![0u32; TILE_BASE as usize];
    // The kit's own tiles.
    for tile in &tex.tiles {
        for p in tile.chunks(4) {
            words.push(
                u32::from(p[0])
                    | u32::from(p[1]) << 8
                    | u32::from(p[2]) << 16
                    | u32::from(p[3]) << 24,
            );
        }
    }
    let mut vertex_max = [0u32; 8];
    for (si, s) in SPECIES.iter().enumerate() {
        let lk = look(*s);
        let h = si * SPECIES_WORDS as usize;
        words[h] = lk.bark.word();
        words[h + 1] = lk.leaf.map_or(u32::MAX, Tex::word);
        words[h + 2] = lk.mask;
        words[h + 3] =
            u32::from(lk.deciduous) | u32::from(lk.olive) << 1 | u32::from(lk.leaf.is_some()) << 2;
        let bm = tex.mean(lk.bark);
        let mut lm = lk.leaf.map_or([0.0; 3], |l| tex.mean(l));
        if lk.olive {
            lm = olive(lm);
        }
        for c in 0..3 {
            words[h + 4 + c] = f(bm[c]);
            words[h + 8 + c] = f(lm[c]);
        }
    }
    for (mi, m) in models.iter().enumerate() {
        let si = m.species.index() as usize;
        vertex_max[si] = vertex_max[si].max(m.vertices.len() as u32);
        let h = (VARIANT_BASE + mi as u32 * VARIANT_WORDS) as usize;
        words[h] = words.len() as u32;
        words[h + 1] = m.vertices.len() as u32;
        for v in &m.vertices {
            words.extend(v.position.map(f));
            let sn = |x: f32| ((x.clamp(-1.0, 1.0) * 127.0).round() as i32 as u8) as u32;
            words.push(sn(v.normal[0]) | sn(v.normal[1]) << 8 | sn(v.normal[2]) << 16);
            let fx = |x: f32| ((x * 4096.0).round().clamp(0.0, 65535.0)) as u32;
            words.push(fx(v.uv[0]) | fx(v.uv[1]) << 16);
            let b = |x: f32| (x.clamp(0.0, 1.0) * 255.0).round() as u32;
            words.push(b(v.card) | b(v.ao) << 8 | u32::from(v.leaf) << 16);
        }
        words[h + 2] = words.len() as u32;
        words[h + 3] = m.anchors.len() as u32;
        for a in &m.anchors {
            words.extend(a.map(f));
        }
        words[h + 4] = f(m.height);
        words[h + 5] = f(m.width);
        words[h + 6] = f(m.lo);
        words[h + 7] = f(m.hi);
    }
    for (si, max) in vertex_max.iter().enumerate() {
        words[si * SPECIES_WORDS as usize + 12] = *max;
    }
    // The impostors: leafy, and for a deciduous species bare as well.
    for (mi, m) in models.iter().enumerate() {
        let lk = look(m.species);
        let h = (VARIANT_BASE + mi as u32 * VARIANT_WORDS) as usize;
        words[h + 8] = words.len() as u32;
        // Each strip is followed by its relief strip (`tree-impostor-depth`),
        // [`IMPOSTOR_RELIEF`] words on: the shader reads a pixel's normal and
        // AO at the colour's index plus that.
        let (leafy, leafy_relief) = crate::tree_bake::bake_with_relief(m, &lk, tex, false);
        // The baked crown's mean, which the shader meets to the near
        // crown's (words 10..12).
        for (c, v) in crate::tree_bake::crown_mean(&leafy).iter().enumerate() {
            words[h + 10 + c] = f(*v);
        }
        words.extend(leafy);
        words.extend(leafy_relief);
        if lk.deciduous {
            words[h + 9] = words.len() as u32;
            let (bare, bare_relief) = crate::tree_bake::bake_with_relief(m, &lk, tex, true);
            words.extend(bare);
            words.extend(bare_relief);
        } else {
            words[h + 9] = words[h + 8];
        }
    }
    Kit {
        models,
        words,
        vertex_max,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_generator_is_the_mockups_xorshift() {
        // rngOf(1): s = 1 -> 270369 after one step (1 ^ 1<<13 = 8193; ^ >>17 = 8193; ^ <<5 = 270369).
        let mut r = Rng::new(1);
        assert_eq!((r.float() * 4_294_967_296.0) as u64, 270_369);
    }

    #[test]
    fn variants_are_deterministic_and_bounded() {
        for s in SPECIES {
            for v in 0..VARIANTS {
                let a = build(s, v);
                let b = build(s, v);
                assert_eq!(a.vertices, b.vertices);
                assert!(a.vertices.len().is_multiple_of(3) && !a.vertices.is_empty());
                assert!(
                    a.vertices.len() < 1 << 16,
                    "{s:?} {v}: the near draw's vertex range"
                );
                assert!(a.height > 1.0 && a.height < 15.0, "{s:?} {}", a.height);
                assert!(a.width > 0.3 && a.width < 14.0, "{s:?} {}", a.width);
                for p in &a.vertices {
                    assert!(p.position.iter().all(|x| x.is_finite()));
                    assert!(p.uv[0] >= 0.0 && p.uv[1] >= 0.0 && p.uv[0] < 16.0 && p.uv[1] < 16.0);
                }
                // Within 15 m of the base: the cull's tree bound.
                let reach = a
                    .vertices
                    .iter()
                    .map(|p| {
                        (p.position[0].powi(2) + p.position[1].powi(2) + p.position[2].powi(2))
                            .sqrt()
                    })
                    .fold(0.0, f32::max);
                assert!(reach * 1.2 < 15.0, "{s:?} {v} reaches {reach}");
                if s != Species::Cactus && s != Species::Palm {
                    assert!(!a.anchors.is_empty());
                }
            }
        }
    }

    #[test]
    fn every_leaf_card_is_rooted_on_wood() {
        // A card's root edge midpoint (uv v = 0) lies on a branch: within
        // reach of one of the bark's long edges, "on wood, never floating".
        for s in [
            Species::Oak,
            Species::Birch,
            Species::Pine,
            Species::Kapok,
            Species::Acacia,
            Species::Willow,
        ] {
            let m = build(s, 0);
            let edges: Vec<_> = m.vertices[..m.bark]
                .chunks(6)
                .map(|q| (q[0].position, q[2].position))
                .collect();
            let seg = |p: [f32; 3], a: [f32; 3], b: [f32; 3]| {
                let ab = [0, 1, 2].map(|i| b[i] - a[i]);
                let ap = [0, 1, 2].map(|i| p[i] - a[i]);
                let t = ((ap[0] * ab[0] + ap[1] * ab[1] + ap[2] * ab[2])
                    / (ab[0] * ab[0] + ab[1] * ab[1] + ab[2] * ab[2]).max(1e-9))
                .clamp(0.0, 1.0);
                (0..3)
                    .map(|i| (ap[i] - ab[i] * t).powi(2))
                    .sum::<f32>()
                    .sqrt()
            };
            // The crown's cards; a vine hangs from the card above it (its own
            // test), so it is left out here.
            for tri in m.vertices[m.bark..m.vertices.len() - m.vines].chunks(6) {
                let root = [0, 1, 2].map(|i| (tri[0].position[i] + tri[1].position[i]) / 2.0);
                let near = edges
                    .iter()
                    .map(|(a, b)| seg(root, *a, *b))
                    .fold(f32::MAX, f32::min);
                assert!(near < 0.6, "{s:?}: a card {near} m from any bark");
            }
        }
    }

    #[test]
    fn the_mix_sums_to_one_and_picks_are_stable() {
        for mix in BIOME_TREES {
            if !mix.is_empty() {
                assert!((mix.iter().map(|(_, w)| w).sum::<f32>() - 1.0).abs() < 1e-6);
            }
        }
        let mut seen = std::collections::HashMap::new();
        for id in 0..20_000u32 {
            let p = pick(id, 2).unwrap();
            assert_eq!(Some(p), pick(id, 2));
            assert!(p.variant < VARIANTS && (0.85..=1.2).contains(&p.size));
            *seen.entry(p.species).or_insert(0) += 1;
        }
        let oak = f64::from(seen[&Species::Oak]) / 20_000.0;
        assert!((oak - 0.6).abs() < 0.02, "{oak}");
        assert!(pick(1, 0).is_none() && pick(1, 6).is_none());
    }

    #[test]
    fn a_cactus_is_never_a_tree() {
        for id in 0..10_000u32 {
            for word in [1 | 3 << 8, 4 | 3 << 8] {
                if grows_cactus(id, word) {
                    for table in [crate::flora::TreeTable::V4, crate::flora::TreeTable::V10] {
                        assert!(!crate::flora::grows_tree(&table, id, word));
                    }
                }
            }
            assert!(!grows_cactus(id, 2 | 2 << 8));
        }
        let n = (0..10_000u32)
            .filter(|id| grows_cactus(*id, 4 | 3 << 8))
            .count();
        assert!(n > 150 && n < 500, "{n}");
    }

    /// Vines (design 12.4): the kapok, the palm and the willow carry some;
    /// every strand hangs from one of the model's anchors and falls no lower
    /// than the ground; the other species carry none.
    #[test]
    fn vines_hang_from_anchors_and_never_reach_the_ground() {
        let mut strands = 0;
        for species in SPECIES {
            for v in 0..VARIANTS {
                let m = build(species, v);
                let vines = &m.vertices[m.vertices.len() - m.vines..];
                if !matches!(species, Species::Kapok | Species::Palm | Species::Willow) {
                    assert_eq!(m.vines, 0, "{species:?}");
                    continue;
                }
                assert_eq!(m.vines % 6, 0, "whole cards");
                for card in vines.chunks(6) {
                    assert!(card.iter().all(|p| p.leaf && p.position[1] > 0.0));
                }
                // Each strand's first card starts within a hand of an anchor.
                let mut card = 0;
                while card < vines.len() {
                    let top = vines[card].position;
                    let near = m
                        .anchors
                        .iter()
                        .any(|a| (0..3).map(|i| (a[i] - top[i]).powi(2)).sum::<f32>().sqrt() < 0.2);
                    if near {
                        strands += 1;
                    }
                    card += 6;
                }
            }
        }
        assert!(
            strands >= 8,
            "{strands} strands across the jungle and swamp trees"
        );
    }

    #[test]
    fn report_the_vertex_counts() {
        for s in SPECIES {
            let counts: Vec<_> = (0..VARIANTS).map(|v| build(s, v).vertices.len()).collect();
            println!("{:>7}: {:?}", s.name(), counts);
        }
    }
}
