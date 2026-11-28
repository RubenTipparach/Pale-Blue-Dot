//! The player's own character (`openspec/changes/player-character`): a
//! townsperson's [`Look`], chosen in the Options menu, kept per install and
//! sent to every host. One figure, one art with the townsfolk (decision 1).
//!
//! - [`Character`] is the look as the install's `character.ron` holds it,
//!   its parts by name so the file reads and survives a reordering.
//! - [`valid`] checks a look against M5's parts and palettes: only what the
//!   townsfolk's own code can draw.
//! - [`rolled`] is the look an install has before its player chooses one,
//!   from its key, the same every time.
//! - [`encode`] and [`decode`] are its bytes on the wire (`Hello`, design 24).

use super::face::Expression;
use super::look::{self, Beard, Build, Hair, Look, Nose, PEOPLES, SHOES};
use serde::{Deserialize, Serialize};

/// The faces a character may rest in (the mockup's resting faces).
pub const RESTING: [Expression; 4] = [
    Expression::Happy,
    Expression::Neutral,
    Expression::Thinking,
    Expression::Sleepy,
];

/// A look as the install's file holds it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Character {
    pub build: String,
    pub height_m: f32,
    pub head: f32,
    pub eye: f32,
    pub gap: f32,
    pub nose: String,
    pub hair: String,
    pub beard: String,
    pub ears: bool,
    pub skin: u32,
    pub hair_colour: u32,
    pub shirt: u32,
    pub trousers: u32,
    pub shoe: u32,
    pub hat: u32,
    pub resting: String,
}

const BUILDS: [(Build, &str); 4] = [
    (Build::Round, "round"),
    (Build::Pear, "pear"),
    (Build::Tall, "tall"),
    (Build::Stout, "stout"),
];
const NOSES: [(Nose, &str); 3] = [
    (Nose::None, "none"),
    (Nose::Button, "button"),
    (Nose::Long, "long"),
];
const HAIRS: [(Hair, &str); 5] = [
    (Hair::Bun, "bun"),
    (Hair::Crop, "crop"),
    (Hair::Braid, "braid"),
    (Hair::Bald, "bald"),
    (Hair::Hat, "hat"),
];
const BEARDS: [(Beard, &str); 3] = [
    (Beard::None, "none"),
    (Beard::Goatee, "goatee"),
    (Beard::FullFront, "full"),
];

fn id_of<T: PartialEq + Copy>(table: &[(T, &'static str)], v: T) -> &'static str {
    table.iter().find(|(t, _)| *t == v).map_or("", |(_, id)| id)
}
fn of_id<T: Copy>(table: &[(T, &'static str)], id: &str) -> Option<T> {
    table.iter().find(|(_, i)| *i == id).map(|(t, _)| *t)
}
fn index_of<T: PartialEq + Copy>(table: &[(T, &'static str)], v: T) -> u8 {
    table.iter().position(|(t, _)| *t == v).unwrap_or(0) as u8
}

impl Character {
    pub fn of(look: &Look) -> Self {
        Self {
            build: id_of(&BUILDS, look.build).into(),
            height_m: look.height_m,
            head: look.head,
            eye: look.eye,
            gap: look.gap,
            nose: id_of(&NOSES, look.nose).into(),
            hair: id_of(&HAIRS, look.hair).into(),
            beard: id_of(&BEARDS, look.beard).into(),
            ears: look.ears,
            skin: look.skin,
            hair_colour: look.hair_colour,
            shirt: look.shirt,
            trousers: look.trousers,
            shoe: look.shoe,
            hat: look.hat,
            resting: look.resting.id().into(),
        }
    }

    /// The look, if every part is one this build has and it is [`valid`].
    pub fn look(&self) -> Option<Look> {
        let look = Look {
            build: of_id(&BUILDS, &self.build)?,
            height_m: self.height_m,
            head: self.head,
            eye: self.eye,
            gap: self.gap,
            nose: of_id(&NOSES, &self.nose)?,
            hair: of_id(&HAIRS, &self.hair)?,
            beard: of_id(&BEARDS, &self.beard)?,
            ears: self.ears,
            skin: self.skin,
            hair_colour: self.hair_colour,
            shirt: self.shirt,
            trousers: self.trousers,
            shoe: self.shoe,
            hat: self.hat,
            resting: Expression::of_id(&self.resting)?,
        };
        valid(&look).then_some(look)
    }

    pub fn to_ron(&self) -> String {
        ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default()).unwrap_or_default()
    }

    pub fn from_ron(text: &str) -> Option<Self> {
        ron::from_str(text).ok()
    }
}

/// The heights a build may stand at, m: [`look::generate`]'s range.
pub fn height_range(build: Build) -> (f32, f32) {
    match build {
        Build::Tall => (1.6, 1.7),
        Build::Stout => (1.45, 1.53),
        _ => (1.48, 1.64),
    }
}

fn any_skin(c: u32) -> bool {
    PEOPLES.iter().any(|p| p.skin.contains(&c))
}
fn any_hair(c: u32) -> bool {
    PEOPLES.iter().any(|p| p.hair.contains(&c))
}
fn any_cloth(c: u32) -> bool {
    PEOPLES.iter().any(|p| p.cloth.contains(&c))
}

/// Whether the townsfolk's own code can draw `look`: every part one of its
/// own, every measure in [`look::generate`]'s ranges, every colour from M5's
/// palettes, the resting face one of [`RESTING`].
pub fn valid(look: &Look) -> bool {
    let (lo, hi) = height_range(look.build);
    let within = |v: f32, lo: f32, hi: f32| v.is_finite() && v >= lo - 1e-4 && v <= hi + 1e-4;
    let gap_lo = super::figure::WR * look.eye + 0.008;
    let gap_hi =
        super::figure::HW / 2.0 - super::figure::HC - 0.008 - super::figure::WR * look.eye * 1.15;
    within(look.height_m, lo, hi)
        && within(look.head, 0.9, 1.02)
        && within(look.eye, 0.88, 1.08)
        && within(look.gap, gap_lo, gap_hi.max(gap_lo))
        && any_skin(look.skin)
        && any_hair(look.hair_colour)
        && any_cloth(look.shirt)
        && any_cloth(look.trousers)
        && any_cloth(look.hat)
        && SHOES.contains(&look.shoe)
        && RESTING.contains(&look.resting)
        // Long hair and a beard are not drawn together (look::generate).
        && !(matches!(look.hair, Hair::Bun | Hair::Braid) && look.beard != Beard::None)
}

/// The look an install has before its player chooses one: rolled from its
/// key by [`look::generate`], from the people and tunic the key picks, its
/// resting face one of [`RESTING`]. The same key rolls the same look.
pub fn rolled(key: &[u8; 16]) -> Look {
    let seed = u32::from_le_bytes([key[0], key[1], key[2], key[3]]);
    let people = PEOPLES[usize::from(key[4]) % PEOPLES.len()];
    let tunic = people.cloth[usize::from(key[5]) % people.cloth.len()];
    let mut look = look::generate(people, seed, 0, tunic);
    if !RESTING.contains(&look.resting) {
        look.resting = Expression::Happy;
    }
    look
}

/// A look's bytes on the wire: its parts by index, its measures and colours.
pub fn encode(look: &Look, out: &mut Vec<u8>) {
    out.push(index_of(&BUILDS, look.build));
    out.push(index_of(&NOSES, look.nose));
    out.push(index_of(&HAIRS, look.hair));
    out.push(index_of(&BEARDS, look.beard));
    out.push(u8::from(look.ears));
    out.push(
        Expression::ALL
            .iter()
            .position(|e| *e == look.resting)
            .unwrap_or(0) as u8,
    );
    for v in [look.height_m, look.head, look.eye, look.gap] {
        out.extend_from_slice(&v.to_le_bytes());
    }
    for c in [
        look.skin,
        look.hair_colour,
        look.shirt,
        look.trousers,
        look.shoe,
        look.hat,
    ] {
        out.extend_from_slice(&c.to_le_bytes());
    }
}

/// The bytes [`encode`] writes: 6 + 4 * 4 + 6 * 4.
pub const ENCODED_LEN: usize = 6 + 16 + 24;

/// A look from [`encode`]'s bytes, if they are whole and the look [`valid`].
pub fn decode(bytes: &[u8]) -> Option<Look> {
    if bytes.len() != ENCODED_LEN {
        return None;
    }
    let f = |i: usize| f32::from_le_bytes(bytes[i..i + 4].try_into().unwrap());
    let c = |i: usize| u32::from_le_bytes(bytes[i..i + 4].try_into().unwrap());
    let look = Look {
        build: BUILDS.get(usize::from(bytes[0]))?.0,
        nose: NOSES.get(usize::from(bytes[1]))?.0,
        hair: HAIRS.get(usize::from(bytes[2]))?.0,
        beard: BEARDS.get(usize::from(bytes[3]))?.0,
        ears: match bytes[4] {
            0 => false,
            1 => true,
            _ => return None,
        },
        resting: *Expression::ALL.get(usize::from(bytes[5]))?,
        height_m: f(6),
        head: f(10),
        eye: f(14),
        gap: f(18),
        skin: c(22),
        hair_colour: c(26),
        shirt: c(30),
        trousers: c(34),
        shoe: c(38),
        hat: c(42),
    };
    valid(&look).then_some(look)
}

/// A row of the Character page (`player-character` design 7), in its order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Field {
    Build,
    Height,
    Skin,
    Hair,
    HairColour,
    Beard,
    Face,
    Nose,
    Ears,
    Shirt,
    Trousers,
    Shoes,
    HatColour,
}

impl Field {
    pub const ALL: [Field; 13] = [
        Field::Build,
        Field::Height,
        Field::Skin,
        Field::Hair,
        Field::HairColour,
        Field::Beard,
        Field::Face,
        Field::Nose,
        Field::Ears,
        Field::Shirt,
        Field::Trousers,
        Field::Shoes,
        Field::HatColour,
    ];

    /// Its id: its label is `character.<id>` in the locale file.
    pub fn id(self) -> &'static str {
        match self {
            Field::Build => "build",
            Field::Height => "height",
            Field::Skin => "skin",
            Field::Hair => "hair",
            Field::HairColour => "hair_colour",
            Field::Beard => "beard",
            Field::Face => "face",
            Field::Nose => "nose",
            Field::Ears => "ears",
            Field::Shirt => "shirt",
            Field::Trousers => "trousers",
            Field::Shoes => "shoes",
            Field::HatColour => "hat_colour",
        }
    }

    /// Whether its value is a colour, shown as a swatch.
    pub fn is_colour(self) -> bool {
        matches!(
            self,
            Field::Skin
                | Field::HairColour
                | Field::Shirt
                | Field::Trousers
                | Field::Shoes
                | Field::HatColour
        )
    }
}

/// The step between heights on the page, m.
pub const HEIGHT_STEP: f32 = 0.02;

fn every(pick: impl Fn(&look::People) -> &'static [u32]) -> Vec<u32> {
    let mut out: Vec<u32> = Vec::new();
    for p in &PEOPLES {
        for &c in pick(p) {
            if !out.contains(&c) {
                out.push(c);
            }
        }
    }
    out
}

/// Every colour a field may take, in the peoples' order, each once.
pub fn colours(field: Field) -> Vec<u32> {
    match field {
        Field::Skin => every(|p| p.skin),
        Field::HairColour => every(|p| p.hair),
        Field::Shirt | Field::Trousers | Field::HatColour => every(|p| p.cloth),
        Field::Shoes => SHOES.to_vec(),
        _ => Vec::new(),
    }
}

/// A field's value as its id (a part or a face), for the page's label:
/// `character.<field>.<id>` in the locale file. `None` for a colour or the
/// height, which are shown as themselves.
pub fn value_id(look: &Look, field: Field) -> Option<&'static str> {
    Some(match field {
        Field::Build => id_of(&BUILDS, look.build),
        Field::Hair => id_of(&HAIRS, look.hair),
        Field::Beard => id_of(&BEARDS, look.beard),
        Field::Nose => id_of(&NOSES, look.nose),
        Field::Face => look.resting.id(),
        Field::Ears => {
            if look.ears {
                "yes"
            } else {
                "no"
            }
        }
        _ => return None,
    })
}

/// A field's colour, for its swatch.
pub fn colour(look: &Look, field: Field) -> Option<u32> {
    Some(match field {
        Field::Skin => look.skin,
        Field::HairColour => look.hair_colour,
        Field::Shirt => look.shirt,
        Field::Trousers => look.trousers,
        Field::Shoes => look.shoe,
        Field::HatColour => look.hat,
        _ => return None,
    })
}

fn cycle<T: PartialEq + Copy>(list: &[T], now: T, by: i32) -> T {
    let n = list.len() as i32;
    let i = list.iter().position(|v| *v == now).unwrap_or(0) as i32;
    list[(i + by).rem_euclid(n) as usize]
}

/// `look` with `field` stepped `by` (1 the next, -1 the last), to the next
/// value the townsfolk can draw: a part that cannot go with the rest (long
/// hair with a beard) is stepped past, and a new build keeps the height in
/// its own range. A [`valid`] look stays valid.
pub fn step(look: &Look, field: Field, by: i32) -> Look {
    let by = by.signum();
    let mut next = *look;
    if field == Field::Height {
        let (lo, hi) = height_range(look.build);
        let h = (look.height_m + by as f32 * HEIGHT_STEP).clamp(lo, hi);
        next.height_m = (h * 100.0).round() / 100.0;
        next.height_m = next.height_m.clamp(lo, hi);
        return next;
    }
    // At most once round the whole list, so a field with no other value
    // that can be drawn comes back where it was.
    for _ in 0..32 {
        match field {
            Field::Build => {
                let builds = BUILDS.map(|(b, _)| b);
                next.build = cycle(&builds, next.build, by);
                let (lo, hi) = height_range(next.build);
                next.height_m = next.height_m.clamp(lo, hi);
            }
            Field::Hair => next.hair = cycle(&HAIRS.map(|(h, _)| h), next.hair, by),
            Field::Beard => next.beard = cycle(&BEARDS.map(|(b, _)| b), next.beard, by),
            Field::Nose => next.nose = cycle(&NOSES.map(|(n, _)| n), next.nose, by),
            Field::Face => next.resting = cycle(&RESTING, next.resting, by),
            Field::Ears => next.ears = !next.ears,
            Field::Height => unreachable!(),
            colour => {
                let list = colours(colour);
                let slot = match colour {
                    Field::Skin => &mut next.skin,
                    Field::HairColour => &mut next.hair_colour,
                    Field::Shirt => &mut next.shirt,
                    Field::Trousers => &mut next.trousers,
                    Field::Shoes => &mut next.shoe,
                    _ => &mut next.hat,
                };
                *slot = cycle(&list, *slot, by);
            }
        }
        if valid(&next) || next == *look {
            return next;
        }
    }
    *look
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `player-character` 2.1: every key's rolled look is valid and round
    /// trips through the file and the wire; the same key rolls the same.
    #[test]
    fn a_rolled_look_is_valid_and_round_trips() {
        for n in 0..200u8 {
            let mut key = [0u8; 16];
            key[0] = n;
            key[3] = n.wrapping_mul(7);
            key[4] = n / 3;
            key[5] = n / 5;
            let look = rolled(&key);
            assert!(valid(&look), "{look:?}");
            assert_eq!(rolled(&key), look);
            let file = Character::from_ron(&Character::of(&look).to_ron()).unwrap();
            assert_eq!(file.look(), Some(look));
            let mut bytes = Vec::new();
            encode(&look, &mut bytes);
            assert_eq!(bytes.len(), ENCODED_LEN);
            assert_eq!(decode(&bytes), Some(look));
        }
    }

    /// `player-character` 2.1: a look with a part, measure or colour the
    /// townsfolk cannot draw is refused, from the file and the wire.
    #[test]
    fn a_look_the_townsfolk_cannot_draw_is_refused() {
        let look = rolled(&[1; 16]);
        let mut tall = look;
        tall.height_m = 2.5;
        assert!(!valid(&tall));
        let mut pink = look;
        pink.shirt = 0xff00ff;
        assert!(!valid(&pink));
        let mut cross = look;
        cross.resting = Expression::Cross;
        assert!(!valid(&cross));
        let mut file = Character::of(&look);
        file.hair = "mohawk".into();
        assert_eq!(file.look(), None);
        let mut bytes = Vec::new();
        encode(&tall, &mut bytes);
        assert_eq!(decode(&bytes), None);
        bytes.pop();
        assert_eq!(decode(&bytes), None);
    }

    /// `player-character` 3.1: every row of the page stepped all the way
    /// round, either way, from many looks: every look on the way is valid,
    /// and a field with more than one value moves.
    #[test]
    fn every_row_steps_through_looks_the_townsfolk_can_draw() {
        for n in 0..40u8 {
            let mut start = rolled(&[n, n / 2, 3, n, n, n / 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
            for field in Field::ALL {
                for by in [1, -1] {
                    let mut look = start;
                    for _ in 0..40 {
                        let next = step(&look, field, by);
                        assert!(valid(&next), "{field:?} {by}: {next:?}");
                        look = next;
                    }
                }
                start = step(&start, field, 1);
            }
        }
        let look = rolled(&[9; 16]);
        assert_ne!(step(&look, Field::Skin, 1).skin, look.skin);
        assert_ne!(step(&look, Field::Build, 1).build, look.build);
        // Long hair with a beard is stepped past.
        let mut bearded = look;
        bearded.hair = Hair::Crop;
        bearded.beard = Beard::Goatee;
        assert!(valid(&bearded));
        let hair = step(&bearded, Field::Hair, -1);
        assert!(!matches!(hair.hair, Hair::Bun | Hair::Braid), "{hair:?}");
        // The height stops at its build's ends.
        let (lo, hi) = height_range(look.build);
        let mut tall = look;
        for _ in 0..20 {
            tall = step(&tall, Field::Height, 1);
        }
        assert!((tall.height_m - hi).abs() < 0.011, "{}", tall.height_m);
        let mut short = look;
        for _ in 0..20 {
            short = step(&short, Field::Height, -1);
        }
        assert!((short.height_m - lo).abs() < 0.011, "{}", short.height_m);
    }
}
