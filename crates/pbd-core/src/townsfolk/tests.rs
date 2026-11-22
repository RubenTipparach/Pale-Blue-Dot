use super::face::{Chosen, Expression, Face, Mood, Mouth, Seen};
use super::figure::{self, FACE_TRIANGLES, Figure, HD, HH, HW};
use super::look::{self, Beard, Build, Hair, Look, Nose, Rng};
use super::*;
use crate::settlement::chart::Patch;
use crate::settlement::record;
use crate::settlement::sea::{cell_at, centre};
use crate::topology::dual_sphere;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::OnceLock;

const RADIUS_M: f32 = 300.0;

fn template(name: &str) -> Template {
    let path = format!(
        "{}/../../assets/settlements/v1/{name}.json",
        env!("CARGO_MANIFEST_DIR")
    );
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("{path}: {e}"))
}

/// A patch of the level-7 sphere round a cell well away from the pentagons
/// (the settlement tests' own).
fn patch() -> &'static (Patch, usize) {
    static PATCH: OnceLock<(Patch, usize)> = OnceLock::new();
    PATCH.get_or_init(|| {
        let cells = dual_sphere(7);
        let anchor = Vec3::new(0.3, 0.5, 0.8).normalize();
        let centre = cells
            .iter()
            .enumerate()
            .max_by(|a, b| {
                a.1.direction
                    .dot(anchor)
                    .total_cmp(&b.1.direction.dot(anchor))
            })
            .map(|(i, _)| i)
            .unwrap();
        let reach = (0.4f32).cos();
        let kept: Vec<usize> = (0..cells.len())
            .filter(|&i| cells[i].direction.dot(cells[centre].direction) > reach)
            .collect();
        let mut index = vec![usize::MAX; cells.len()];
        for (k, &i) in kept.iter().enumerate() {
            index[i] = k;
        }
        let local: Vec<_> = kept
            .iter()
            .map(|&i| {
                let mut c = cells[i].clone();
                c.neighbors = c.neighbors.iter().map(|&n| index[n]).collect();
                c
            })
            .collect();
        let keys = kept.iter().map(|&i| i as u32).collect();
        (Patch { cells: local, keys }, index[centre])
    })
}

fn laid(t: &Template, site: u32) -> (record::Town, crate::settlement::chart::Chart) {
    let (patch, at) = patch();
    let (_, east) = crate::geo::north_east(patch.cells[*at].direction);
    let d0 = patch.side_toward(*at, east);
    let town = record::lay(t, site, patch, *at, d0, |_| 12.4).unwrap_or_else(|e| panic!("{e}"));
    let chart = record::chart_of(&town, patch).unwrap_or_else(|e| panic!("{e}"));
    (town, chart)
}

fn people(name: &str, site: u32) -> Vec<Person> {
    let t = template(name);
    let (town, chart) = laid(&t, site);
    people_of(&town, &t, &chart, &patch().0)
}

const LAND: [&str; 5] = ["village", "town", "desert", "tundra", "jungle"];

/// Task 1.1: the walled town's people, as the mockup has them.
#[test]
fn the_walled_town_has_its_nineteen_with_a_smith_and_a_gate_guard() {
    let folk = people("town", 3);
    assert_eq!(
        folk.len(),
        19,
        "{:?}",
        folk.iter().map(|p| &p.role).collect::<Vec<_>>()
    );
    let smith = folk.iter().find(|p| p.role == "smith").expect("a smith");
    assert_eq!(smith.work(), Some(Work::Hammer));
    let guard = folk
        .iter()
        .find(|p| p.role == "guard")
        .expect("a gate guard");
    assert!(guard.spear);
    assert!(
        folk.iter()
            .filter(|p| p.role == "wall_guard")
            .all(|p| p.pace().is_some())
    );
}

/// Every land template's people are all laid; the others hold theirs too.
#[test]
fn every_template_has_its_people_and_every_one_is_laid() {
    let counts: BTreeMap<&str, usize> = [
        ("village", 5),
        ("town", 19),
        ("coast", 12),
        ("desert", 6),
        ("tundra", 4),
        ("jungle", 3),
    ]
    .into();
    for (name, n) in &counts {
        assert_eq!(template(name).people.len(), *n, "{name}");
    }
    for name in LAND {
        assert_eq!(people(name, 5).len(), counts[name], "{name}");
    }
}

/// Task 1.2: the same record gives the same people twice, and two towns
/// of one template are peopled apart.
#[test]
fn the_same_record_gives_the_same_people() {
    for name in LAND {
        assert_eq!(people(name, 11), people(name, 11), "{name}");
        let (a, b) = (people(name, 11), people(name, 12));
        assert!(a.iter().zip(&b).any(|(a, b)| a.look != b.look), "{name}");
    }
}

/// Task 1.2: no post stands in a wall and no path crosses one: along each
/// laid path every step is in the same building as the post (or none) and
/// under no masonry higher than the walker's feet, unless it walks on the
/// wall's top. The mockup's own village lane walker went through its hall;
/// the game's stops short of it.
#[test]
fn no_post_or_path_crosses_a_wall() {
    for name in ["village", "town", "coast", "desert", "tundra", "jungle"] {
        let t = template(name);
        let cell_m = t.grid.cell_m;
        let building: BTreeMap<(i32, i32), usize> = t
            .buildings
            .iter()
            .enumerate()
            .flat_map(|(i, b)| b.cells.iter().map(move |c| ((c[0], c[1]), i)))
            .collect();
        let masonry: BTreeMap<(i32, i32), (i32, i32)> = t
            .masonry
            .iter()
            .map(|m| ((m.c, m.r), (m.from, m.to)))
            .collect();
        let walled = |x: f32, z: f32, feet: f32| {
            masonry
                .get(&cell_at(x, z, cell_m))
                .is_some_and(|&(from, to)| {
                    (from as f32) < feet + SOLID_HEIGHT_M && (to as f32) > feet + 0.05
                })
        };
        let inside = |x: f32, z: f32| building.get(&cell_at(x, z, cell_m)).copied();
        let walls = Walls::of(&t);
        for p in &t.people {
            let [x, feet, z] = p.at;
            assert!(
                !walled(x, z, feet),
                "{name}: the {} stands in a wall",
                p.role
            );
            let Some(ends) = &p.path else { continue };
            let home = inside(x, z);
            let [a, b] = walls.clip(p.at, [ends[0], ends[1]]);
            for [ex, ez] in [a, b] {
                let len = ((ex - x).powi(2) + (ez - z).powi(2)).sqrt();
                let n = (len / 0.05).ceil().max(1.0) as usize;
                for k in 0..=n {
                    let u = k as f32 / n as f32;
                    let (px, pz) = (x + (ex - x) * u, z + (ez - z) * u);
                    assert_eq!(
                        inside(px, pz),
                        home,
                        "{name}: the {}'s path crosses a wall",
                        p.role
                    );
                    assert!(
                        !walled(px, pz, feet),
                        "{name}: the {}'s path crosses a wall",
                        p.role
                    );
                }
            }
            // Most of every path is kept.
            let kept = ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)).sqrt();
            let whole =
                ((ends[0][0] - ends[1][0]).powi(2) + (ends[0][1] - ends[1][1]).powi(2)).sqrt();
            assert!(
                kept > whole * 0.5,
                "{name}: the {} keeps {kept} of {whole} m",
                p.role
            );
        }
    }
}

/// A person stands on their town's ground: the mockup's height over its
/// datum on the town's terrace, so the smith is on the terrace and a wall
/// guard on the wall walk.
#[test]
fn people_stand_at_their_heights_over_the_terrace() {
    let t = template("town");
    let (town, chart) = laid(&t, 3);
    let folk = people_of(&town, &t, &chart, &patch().0);
    let cells: BTreeMap<(i32, i32), usize> = town
        .cells
        .iter()
        .enumerate()
        .map(|(i, c)| ((c.0, c.1), i))
        .collect();
    let smith = folk.iter().find(|p| p.role == "smith").unwrap();
    let def = &t.people[smith.id.index as usize];
    let cell = cell_at(def.at[0], def.at[2], t.grid.cell_m);
    assert_eq!(
        smith.altitude_m,
        town.terrace_of(cells[&cell]),
        "the smith on his terrace"
    );
    let wall = folk.iter().find(|p| p.role == "wall_guard").unwrap();
    assert!(
        wall.altitude_m >= town.terrace as f32 + 7.0,
        "a wall guard up on the walk"
    );
    // And their posts are where the mockup puts them, within a cell.
    let (x0, z0) = centre(cell.0, cell.1, t.grid.cell_m);
    let d = ((def.at[0] - x0).powi(2) + (def.at[2] - z0).powi(2)).sqrt();
    assert!(d < t.grid.cell_m, "{d}");
}

/// A walker goes back and forth along their path at their pace, facing
/// the way they go.
#[test]
fn a_walker_walks_its_path_back_and_forth() {
    let folk = people("village", 4);
    let p = folk.iter().find(|p| p.pace().is_some()).unwrap();
    let Activity::Path {
        points,
        speed,
        length_m,
        start_s,
    } = &p.activity
    else {
        unreachable!()
    };
    let r = f64::from(RADIUS_M);
    let half = f64::from(*length_m / *speed);
    let t0 = -f64::from(*start_s);
    let a = p.at(t0, r);
    let b = p.at(t0 + half, r);
    let a_end = points[0].as_dvec3().normalize() * (r + f64::from(p.altitude_m));
    let b_end = points[points.len() - 1].as_dvec3().normalize() * (r + f64::from(p.altitude_m));
    assert!(a.feet.distance(a_end) < 0.01, "{}", a.feet.distance(a_end));
    assert!(b.feet.distance(b_end) < 0.05, "{}", b.feet.distance(b_end));
    let mid = p.at(t0 + half * 0.5, r);
    assert!(mid.walking);
    assert!(
        mid.forward.dot(b_end - a_end) > 0.0,
        "facing the far end on the way"
    );
    let back = p.at(t0 + half * 1.5, r);
    assert!(
        back.forward.dot(b_end - a_end) < 0.0,
        "facing home on the way back"
    );
    // At its pace: a second along is the pace's metres.
    let one = p.at(t0 + half * 0.5 + 1.0, r);
    let moved = one.feet.distance(mid.feet) as f32;
    assert!((moved - speed).abs() < 0.1, "{moved} against {speed}");
}

/// The mockup's xorshift and `genSpec`, number for number.
#[test]
fn the_generator_is_the_mockups() {
    let mut rn = Rng::new(7 * 9973 + 3 * 131 + 7);
    for want in [0.419_510_8, 0.594_311_9, 0.294_543_9] {
        assert!((rn.roll() - want).abs() < 1e-6);
    }
    let l = look::generate(look::FIELDS, 7, 3, 0xa04a3a);
    assert_eq!(l.build, Build::Pear);
    assert!((l.height_m - 1.575_09).abs() < 1e-4);
    assert!((l.head - 0.903_33).abs() < 1e-4);
    assert!((l.eye - 0.938_91).abs() < 1e-4);
    assert!((l.gap - 0.095_475).abs() < 1e-5);
    assert_eq!(
        (l.nose, l.hair, l.beard, l.ears),
        (Nose::Button, Hair::Braid, Beard::None, true)
    );
    assert_eq!(
        (l.skin, l.hair_colour, l.trousers),
        (0x9c6b4a, 0x3b2a1e, 0x7a5a3a)
    );
    let l = look::generate(look::FIELDS, 7, 0, 0x7a5a3a);
    assert_eq!(
        (l.hair, l.beard, l.ears, l.trousers),
        (Hair::Hat, Beard::FullFront, false, 0xd8c8a0)
    );
}

/// A beard is one shaped piece on the face's front only (decisions 36 and
/// 36a): every point of it in front of the head but for the 5 mm it sinks,
/// on the front or its front bevels and never the sides, nothing in the
/// mouth's opening, its beard cells one connected piece, and the same every
/// time it is built; and the styles come half and half, a third of the town
/// bearded or so, with every other roll the same.
#[test]
fn a_beard_is_only_in_front_and_off_the_mouth() {
    let hero = |beard| Look {
        build: Build::Round,
        height_m: 1.55,
        head: 0.98,
        eye: 1.08,
        gap: 0.1,
        nose: Nose::Button,
        hair: Hair::Crop,
        beard,
        ears: false,
        skin: 0xe2ac80,
        hair_colour: 0x8a5a2a,
        shirt: 0x5c9a46,
        trousers: 0x7a5a3a,
        shoe: 0x4a3428,
        hat: 0xb0884a,
        resting: Expression::Happy,
    };
    let bare = Figure::of(&hero(Beard::None)).head.positions.len();
    let [lo_y, hi_y, half_x] = figure::MOUTH_CLEAR;
    for b in [Beard::Goatee, Beard::FullFront] {
        let f = Figure::of(&hero(b));
        assert_eq!(
            f,
            Figure::of(&hero(b)),
            "{b:?} is built the same every time"
        );
        let added = &f.head.positions[bare..];
        assert!(!added.is_empty(), "{b:?} adds a shell");
        for p in added {
            // In front of the head's flat front and its front bevels (moved
            // in by the bottom bevel at the chin), less the sink.
            let inset = (figure::HC - p[1]).clamp(0.0, figure::HC);
            let sink = figure::BEARD_SINK * std::f32::consts::SQRT_2 + 1e-5;
            assert!(
                p[2] >= figure::HA - figure::HC - inset - sink
                    && (p[2] >= figure::HA - inset - sink
                        || p[2] + p[0].abs()
                            >= figure::HA + HW / 2.0 - figure::HC - 2.0 * inset - sink),
                "{b:?} behind the front at {p:?}"
            );
            assert!(
                p[0].abs() <= HW / 2.0 + 0.03,
                "{b:?} past the sides at {p:?}"
            );
            assert!(
                !(p[0].abs() < half_x && p[1] > lo_y && p[1] < hi_y),
                "{b:?} in the mouth at {p:?}"
            );
        }
        let cells = figure::beard_cells(b);
        for &(x0, x1, y0, y1, _) in &cells {
            let over_mouth = y0 < hi_y && y1 > lo_y && x0 < half_x && x1 > -half_x;
            assert!(
                !over_mouth,
                "{b:?} cell {x0}..{x1} x {y0}..{y1} covers the mouth"
            );
        }
        // One piece: the beard cells, joined edge to edge, are one region.
        let beard: Vec<_> = cells.iter().filter(|c| !c.4).collect();
        let touch = |a: &&(f32, f32, f32, f32, bool), b: &&(f32, f32, f32, f32, bool)| {
            let side = (a.1 == b.0 || b.1 == a.0) && a.2 == b.2;
            let over = (a.3 == b.2 || b.3 == a.2) && a.0 == b.0;
            side || over
        };
        let mut seen = vec![false; beard.len()];
        let mut stack = vec![0];
        seen[0] = true;
        while let Some(k) = stack.pop() {
            for m in 0..beard.len() {
                if !seen[m] && touch(&beard[k], &beard[m]) {
                    seen[m] = true;
                    stack.push(m);
                }
            }
        }
        assert!(seen.iter().all(|&s| s), "{b:?} is in more than one piece");
        // It hangs below the chin, and its triangles stay few.
        assert!(
            added.iter().any(|p| p[1] < -0.05),
            "{b:?} hangs below the chin"
        );
        let tris = (f.head.positions.len() - bare) / 3;
        assert!((150..=450).contains(&tris), "{b:?}: {tris} triangles");
    }
    let (mut goatee, mut full, mut none) = (0i32, 0i32, 0);
    for i in 0..2000 {
        match look::generate(look::FIELDS, 11, i, 0xa04a3a).beard {
            Beard::None => none += 1,
            Beard::Goatee => goatee += 1,
            Beard::FullFront => full += 1,
        }
    }
    assert!(
        (400..560).contains(&(goatee + full)),
        "{goatee} + {full} bearded of 2000"
    );
    assert!((goatee - full).abs() < 120, "{goatee} goatees, {full} full");
    assert!(none > 1000);
}

/// The figure is the Faces tab's: its block head, its height, and about
/// its triangles (the hero's 678, face included).
#[test]
fn the_figure_is_the_faces_tabs() {
    let hero = Look {
        build: Build::Round,
        height_m: 1.55,
        head: 0.98,
        eye: 1.08,
        gap: 0.1,
        nose: Nose::Button,
        hair: Hair::Crop,
        beard: Beard::None,
        ears: true,
        skin: 0xe2ac80,
        hair_colour: 0x8a5a2a,
        shirt: 0x5c9a46,
        trousers: 0x7a5a3a,
        shoe: 0x4a3428,
        hat: 0xb0884a,
        resting: Expression::Happy,
    };
    let f = Figure::of(&hero);
    assert!((f.height_m() - 1.545).abs() < 0.01, "{}", f.height_m());
    let tris = f.triangles() + FACE_TRIANGLES;
    assert!((660..=700).contains(&tris), "{tris} triangles");
    // The head block alone: 0.5 wide, 0.6 tall, 0.45 deep.
    let block = figure::block(HW, HH, HD, figure::HC, 0.0, Default::default(), 0);
    let (mut lo, mut hi) = (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN));
    for p in &block.positions {
        lo = lo.min(Vec3::from(*p));
        hi = hi.max(Vec3::from(*p));
    }
    let size = hi - lo;
    assert!(
        (size.x - HW).abs() < 1e-4 && (size.y - HH).abs() < 1e-4 && (size.z - HD).abs() < 1e-4,
        "{size}"
    );
}

/// Every colour any person of any template wears is in the swatches the
/// game's atlas holds.
#[test]
fn every_colour_a_person_wears_is_a_swatch() {
    let names = ["village", "town", "coast", "desert", "tundra", "jungle"];
    let templates: Vec<Template> = names.iter().map(|n| template(n)).collect();
    let tunics = templates
        .iter()
        .flat_map(|t| t.people.iter().map(|p| p.tunic));
    let swatches: BTreeSet<u32> = look::swatches(tunics).into_iter().collect();
    for t in &templates {
        for (i, def) in t.people.iter().enumerate() {
            for seed in [1, 99, 12345] {
                let l = look::generate(look::people_of_scene(&t.scene), seed, i as u32, def.tunic);
                for c in Figure::of(&l).colours() {
                    assert!(
                        swatches.contains(&c),
                        "{}: {c:06x} is not a swatch",
                        t.scene
                    );
                }
            }
        }
    }
}

/// Decision 22: meeting the player surprises, then smiles, then rests;
/// night is sleepy; a held expression wins; the smith works with his brows
/// down.
#[test]
fn the_mood_follows_what_happens() {
    let mut m = Mood::new(Expression::Neutral);
    let far = Seen::default();
    let near = Seen {
        player_m: Some(3.0),
        ..far
    };
    assert_eq!(m.step(0.1, far).expression, Expression::Neutral);
    assert_eq!(m.step(0.1, near).expression, Expression::Surprised);
    let mut seen = Vec::new();
    for _ in 0..80 {
        seen.push(m.step(0.1, near).expression);
    }
    assert!(
        seen[..3].iter().all(|e| *e == Expression::Surprised),
        "{seen:?}"
    );
    assert!(
        seen[10..50].iter().all(|e| *e == Expression::Happy),
        "{seen:?}"
    );
    assert_eq!(*seen.last().unwrap(), Expression::Neutral);
    // Gone past 8 m and back: surprised again.
    m.step(
        0.1,
        Seen {
            player_m: Some(9.0),
            ..far
        },
    );
    assert_eq!(m.step(0.1, near).expression, Expression::Surprised);
    let night = Seen { night: true, ..far };
    let mut m = Mood::new(Expression::Happy);
    assert_eq!(m.step(0.1, night).expression, Expression::Sleepy);
    m.hold(Expression::Delighted, 1.0);
    assert_eq!(m.step(0.5, night).expression, Expression::Delighted);
    assert_eq!(m.step(0.6, night).expression, Expression::Sleepy);
    let mut smith = Mood::new(Expression::Neutral);
    let at_work = smith.step(
        0.1,
        Seen {
            hammering: true,
            ..far
        },
    );
    assert_eq!(
        at_work,
        Chosen {
            expression: Expression::Neutral,
            focus: true
        }
    );
}

/// The face's layers: it blinks every few seconds, looks at the player
/// when told to, and a delighted face draws its arcs; the same seed gives
/// the same face.
#[test]
fn the_face_blinks_looks_and_is_its_own() {
    let calm = Chosen {
        expression: Expression::Happy,
        focus: false,
    };
    let mut a = Face::new(5, Expression::Happy);
    let mut b = Face::new(5, Expression::Happy);
    let (mut shut, mut frames) = (0, Vec::new());
    for _ in 0..600 {
        let p = a.step(1.0 / 60.0, calm, None, false);
        frames.push(p);
        assert_eq!(b.step(1.0 / 60.0, calm, None, false), p);
        if p.lid > 0.95 {
            shut += 1;
        }
    }
    assert!(shut >= 2, "blinked {shut} frames in 10 s");
    // Looking up and to the left at the player: the head turns, the pupils
    // go the rest of the way.
    let mut last = frames[0];
    for _ in 0..120 {
        last = a.step(1.0 / 60.0, calm, Some([0.5, 0.3]), false);
    }
    assert!(
        last.head_yaw > 0.2 && last.pupil[0] > 0.1 && last.pupil[1] > 0.1,
        "{last:?}"
    );
    let d = a.step(
        0.016,
        Chosen {
            expression: Expression::Delighted,
            focus: false,
        },
        None,
        false,
    );
    assert!(d.arcs && d.mouth == Mouth::Grin);
    // Talking cycles the mouth's open shapes.
    let mut mouths = BTreeSet::new();
    for _ in 0..120 {
        mouths.insert(a.step(1.0 / 60.0, calm, None, true).mouth.cell());
    }
    assert!(mouths.len() >= 3, "{mouths:?}");
}

/// Every mouth cell of the atlas is drawn.
#[test]
fn every_mouth_cell_is_drawn() {
    let atlas = face::mouth_atlas();
    for m in Mouth::ALL {
        let c = m.cell();
        let (ox, oy) = ((c % 4) * face::CELL_W, (c / 4) * face::CELL_H);
        let n = (0..face::CELL_H)
            .flat_map(|y| (0..face::CELL_W).map(move |x| (x, y)))
            .filter(|(x, y)| atlas[(oy + y) * face::ATLAS_W + ox + x][3] > 0)
            .count();
        assert!(n >= 10, "{m:?} has {n} pixels");
    }
}

/// Decision 25: a body walking into a person is held off by their circle,
/// and not over their head.
#[test]
fn a_person_is_a_solid_circle() {
    let feet = DVec3::new(0.0, 300.0, 0.0);
    let walker = |x: f64, up: f64| DVec3::new(x, 300.0 + up, 0.0);
    assert!(overlaps(walker(0.5, 0.9), 0.9, 0.3, feet).is_some());
    let out = overlaps(walker(0.5, 0.9), 0.9, 0.3, feet).unwrap();
    assert!(out.x > 0.99, "{out}");
    assert!(overlaps(walker(0.6, 0.9), 0.9, 0.3, feet).is_none());
    assert!(
        overlaps(walker(0.2, 2.9), 0.9, 0.3, feet).is_none(),
        "over their head"
    );
}

/// The walk and the work loops move what the mockup moves.
#[test]
fn the_body_walks_and_works() {
    let mut b = motion::Body::new(3);
    let mut swing = 0.0f32;
    for _ in 0..60 {
        let p = b.step(1.0 / 60.0, motion::Doing::Walking(1.0), 0.0);
        swing = swing.max(p.legs[0].abs());
        assert!(
            (p.legs[0] + p.legs[1]).abs() < 1e-6,
            "the legs swing against each other"
        );
    }
    assert!(swing > 0.4, "{swing}");
    let mut smith = motion::Body::new(3);
    let (mut lo, mut hi) = (f32::MAX, f32::MIN);
    for _ in 0..120 {
        let p = smith.step(1.0 / 60.0, motion::Doing::Hammering, 0.0);
        lo = lo.min(p.arms[1][0]);
        hi = hi.max(p.arms[1][0]);
        assert_eq!(p.legs, [0.0, 0.0]);
    }
    assert!(
        lo < -1.1 && hi > 0.1,
        "the hammer rises and falls: {lo} to {hi}"
    );
    let mut hop = motion::Body::new(3);
    hop.play(face::Act::Hop);
    let top = (0..50)
        .map(|_| hop.step(1.0 / 60.0, motion::Doing::Standing, 0.0).hop)
        .fold(0.0f32, f32::max);
    assert!(top > 0.25, "{top}");
}
