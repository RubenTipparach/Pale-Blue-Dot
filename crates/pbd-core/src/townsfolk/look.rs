//! How a townsperson looks (`townsfolk` decisions 16 and 20): M5 version
//! 9's `genSpec`, ported number for number, its xorshift too, seeded by the
//! site and the person's index so a town's people are the same on every
//! visit and every machine.

use super::face::Expression;

/// The mockup's `rng`: a 32-bit xorshift, as JavaScript runs it, giving
/// numbers in `[0, 1)`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rng(u32);

impl Rng {
    pub fn new(seed: u32) -> Self {
        Self(if seed == 0 { 1 } else { seed })
    }

    pub fn roll(&mut self) -> f32 {
        let mut s = self.0;
        s ^= s << 13;
        s ^= s >> 17;
        s ^= s << 5;
        self.0 = s;
        (f64::from(s) / 4_294_967_296.0) as f32
    }

    /// The mockup's `pick`.
    pub fn pick<T: Copy>(&mut self, of: &[T]) -> T {
        let n = of.len();
        of[((self.roll() * n as f32) as usize) % n]
    }
}

/// A body's build, with the torso's profile (`BUILDS`): its width scale,
/// how much of the height under the head is legs, and `[t, r]` from the
/// hips (0) to the shoulders (1), `r` the circumradius over the width.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Build {
    Round,
    Pear,
    Tall,
    Stout,
}

impl Build {
    pub const ALL: [Build; 4] = [Build::Round, Build::Pear, Build::Tall, Build::Stout];

    pub fn width(self) -> f32 {
        match self {
            Build::Round | Build::Pear => 1.0,
            Build::Tall => 0.92,
            Build::Stout => 1.08,
        }
    }

    pub fn legs(self) -> f32 {
        match self {
            Build::Round => 0.40,
            Build::Pear => 0.39,
            Build::Tall => 0.46,
            Build::Stout => 0.36,
        }
    }

    pub fn profile(self) -> &'static [[f32; 2]] {
        match self {
            Build::Round => &[
                [0.0, 0.17],
                [0.12, 0.21],
                [0.45, 0.25],
                [0.78, 0.22],
                [0.95, 0.15],
                [1.0, 0.10],
            ],
            Build::Pear => &[
                [0.0, 0.21],
                [0.15, 0.25],
                [0.45, 0.24],
                [0.75, 0.18],
                [0.95, 0.13],
                [1.0, 0.09],
            ],
            Build::Tall => &[
                [0.0, 0.15],
                [0.15, 0.18],
                [0.5, 0.19],
                [0.8, 0.18],
                [0.95, 0.13],
                [1.0, 0.09],
            ],
            Build::Stout => &[
                [0.0, 0.20],
                [0.12, 0.25],
                [0.45, 0.27],
                [0.78, 0.25],
                [0.95, 0.17],
                [1.0, 0.11],
            ],
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Nose {
    None,
    Button,
    Long,
}

/// A beard: pieces on the face's front only (`townsfolk` decision 36).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Beard {
    None,
    /// A moustache over the mouth and a tuft under it.
    Goatee,
    /// The moustache, a jaw plate under the mouth and a cheek piece each
    /// side of it.
    FullFront,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hair {
    Bun,
    Crop,
    Braid,
    Bald,
    Hat,
}

/// Everything that tells one townsperson from another to look at.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Look {
    pub build: Build,
    /// Feet to the top of the head, metres.
    pub height_m: f32,
    /// The head block's scale.
    pub head: f32,
    /// The eyes' scale.
    pub eye: f32,
    /// Each eye's centre from the face's middle, metres.
    pub gap: f32,
    pub nose: Nose,
    pub hair: Hair,
    pub beard: Beard,
    pub ears: bool,
    /// Colours, `0xRRGGBB` sRGB.
    pub skin: u32,
    pub hair_colour: u32,
    pub shirt: u32,
    pub trousers: u32,
    pub shoe: u32,
    pub hat: u32,
    /// The face they wear when nothing is happening to them.
    pub resting: Expression,
}

/// A people's colours (M5's `PEOPLES`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct People {
    pub skin: &'static [u32],
    pub cloth: &'static [u32],
    pub hair: &'static [u32],
}

pub const FIELDS: People = People {
    skin: &[0xe2ac80, 0xc99a74, 0x9c6b4a, 0xf0c49e, 0x7a5038],
    cloth: &[0x5c9a46, 0x7a5a3a, 0xb0884a, 0x4a6e9a, 0xa04a3a, 0xd8c8a0],
    hair: &[0x3b2a1e, 0x8a5a2a, 0xd8b060, 0x1e1a18, 0xa04a2a],
};
pub const DESERT: People = People {
    skin: &[0xb07a52, 0x8a5a3c, 0xc89064, 0x6a4430],
    cloth: &[0xe0b050, 0xc0603a, 0xf0e0c0, 0x3a8aa0, 0x9a3a2a],
    hair: &[0x1e1a18, 0x2a201a, 0x3a2a20],
};
pub const TUNDRA: People = People {
    skin: &[0xe8c0a0, 0xc8a080, 0xa88060],
    cloth: &[0x8aa0b8, 0xe8e4dc, 0x6a4a3a, 0xc04040, 0x5a6a50],
    hair: &[0xe0d8c8, 0x3a2a20, 0x6a5040],
};
pub const COAST: People = People {
    skin: &[0xd8a880, 0xb08060, 0xe8c0a0, 0x8a5a40],
    cloth: &[0xf0c830, 0x2a4a7a, 0xe8e0d0, 0x6a8a9a, 0xa04a3a],
    hair: &[0x8a6a4a, 0xd0b080, 0x2a201a],
};

/// Every people a town can be of.
pub const PEOPLES: [People; 4] = [FIELDS, DESERT, TUNDRA, COAST];

/// The shoes anyone wears.
pub const SHOES: [u32; 3] = [0x4a3428, 0x3a2e26, 0x6a4a30];

/// A template's people, by its scene (decision 20): the fields folk where
/// M5 has none of its own (the jungle).
pub fn people_of_scene(scene: &str) -> People {
    match scene {
        "coast" => COAST,
        "desert" => DESERT,
        "tundra" => TUNDRA,
        _ => FIELDS,
    }
}

/// The head block's half width and the eye white's radius, for the gap's
/// limits (the figure's `HW`, `HC`, `WR`).
const HW: f32 = 0.5;
const HC: f32 = 0.035;
const WR: f32 = 0.08;

/// The mockup's `genSpec(seed, i)`, the shirt the template's tunic.
pub fn generate(people: People, seed: u32, index: u32, tunic: u32) -> Look {
    let mut rn = Rng::new(
        seed.wrapping_mul(9973)
            .wrapping_add(index.wrapping_mul(131))
            .wrapping_add(7),
    );
    let build = rn.pick(&Build::ALL);
    let height_m = match build {
        Build::Tall => 1.6 + rn.roll() * 0.1,
        Build::Stout => 1.45 + rn.roll() * 0.08,
        _ => 1.48 + rn.roll() * 0.16,
    };
    let eye = 0.88 + rn.roll() * 0.2;
    let lo = WR * eye + 0.008;
    let hi = HW / 2.0 - HC - 0.008 - WR * eye * 1.15;
    let gap = (0.093 + rn.roll() * 0.014).max(lo).min(hi.max(lo));
    let hair = rn.pick(&[
        Hair::Bun,
        Hair::Crop,
        Hair::Braid,
        Hair::Bald,
        Hair::Hat,
        Hair::Crop,
    ]);
    // The mockup rolls a shirt here; the template's tunic is worn instead,
    // and the roll is kept so the rest of the sequence is the mockup's.
    let _rolled_shirt = rn.pick(people.cloth);
    let head = 0.9 + rn.roll() * 0.12;
    let nose = rn.pick(&[Nose::None, Nose::Button, Nose::Button, Nose::Long]);
    let bearded = !matches!(hair, Hair::Bun | Hair::Braid) && rn.roll() < 0.35;
    let ears = rn.roll() < 0.6;
    let skin = rn.pick(people.skin);
    let hair_colour = rn.pick(people.hair);
    let shirt = tunic;
    let others: Vec<u32> = people
        .cloth
        .iter()
        .copied()
        .filter(|c| *c != shirt)
        .collect();
    let trousers = rn.pick(&others);
    let shoe = rn.pick(&SHOES);
    let hat = rn.pick(people.cloth);
    // The mockup's crowd shows a mood each; in a town the resting face is
    // a calm one (decision 22).
    let resting = match rn.pick(&[
        "happy",
        "neutral",
        "happy",
        "thinking",
        "sleepy",
        "delighted",
        "surprised",
        "talking",
    ]) {
        "neutral" | "surprised" | "sleepy" => Expression::Neutral,
        "thinking" => Expression::Thinking,
        _ => Expression::Happy,
    };
    // The beard's style, appended after every other roll so nobody else's
    // look moves (decision 36): half goatees, half full front beards.
    let style = rn.roll();
    let beard = match (bearded, style < 0.5) {
        (false, _) => Beard::None,
        (true, true) => Beard::Goatee,
        (true, false) => Beard::FullFront,
    };
    Look {
        build,
        height_m,
        head,
        eye,
        gap,
        nose,
        hair,
        beard,
        ears,
        skin,
        hair_colour,
        shirt,
        trousers,
        shoe,
        hat,
        resting,
    }
}

/// `shade(hex, k)`: each sRGB channel times `k`, as the mockup's
/// `new THREE.Color(hex)` holds it.
pub fn shade(hex: u32, k: f32) -> u32 {
    let ch = |s: u32| {
        let v = ((hex >> s) & 0xff) as f32 / 255.0 * k;
        ((v * 255.0).round().clamp(0.0, 255.0) as u32) << s
    };
    ch(16) | ch(8) | ch(0)
}

/// `mixHex(a, b, t)`: sRGB channels blended.
pub fn mix(a: u32, b: u32, t: f32) -> u32 {
    let ch = |s: u32| {
        let x = ((a >> s) & 0xff) as f32;
        let y = ((b >> s) & 0xff) as f32;
        ((x + (y - x) * t).round().clamp(0.0, 255.0) as u32) << s
    };
    ch(16) | ch(8) | ch(0)
}

/// The shades the figure takes of its colours (`friend`): skin's 0.84,
/// shirt's 0.74, trousers' 0.72, the brows' 0.62, the hair's 0.78.
pub const SHADES: [f32; 5] = [0.84, 0.78, 0.74, 0.72, 0.62];

/// The blush's pink, and the blush levels drawn (the expressions' 0.35,
/// 0.7 and 1).
pub const BLUSH_PINK: u32 = 0xf27a7a;
pub const BLUSH_LEVELS: [f32; 3] = [0.35, 0.7, 1.0];

/// The blush's colour on `skin` at `level`.
pub fn blush(skin: u32, level: f32) -> u32 {
    mix(skin, BLUSH_PINK, 0.55 * level)
}

/// Every colour a person of these peoples, in these tunics, can wear: each
/// base colour and its shades, the blushes, and the face's own (what the
/// game's swatch atlas holds, decision 21).
pub fn swatches(tunics: impl IntoIterator<Item = u32>) -> Vec<u32> {
    let mut base: Vec<u32> = PEOPLES
        .iter()
        .flat_map(|p| p.skin.iter().chain(p.cloth).chain(p.hair).copied())
        .chain(SHOES)
        .chain(tunics)
        .collect();
    base.sort_unstable();
    base.dedup();
    let mut out: Vec<u32> = base
        .iter()
        .flat_map(|&c| std::iter::once(c).chain(SHADES.iter().map(move |&k| shade(c, k))))
        .collect();
    for p in PEOPLES {
        for &s in p.skin {
            out.extend(BLUSH_LEVELS.iter().map(|&b| blush(s, b)));
        }
    }
    out.extend(super::figure::FACE_COLOURS);
    out.sort_unstable();
    out.dedup();
    out
}
