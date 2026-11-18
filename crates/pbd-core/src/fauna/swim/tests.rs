use super::*;
use crate::fauna::{FaunaSettings, HOME_BODY, Water};
use crate::fishing::{Angler, Controls, Event};

/// The planet-centred origin: written in the planet's frame, which is the
/// local space of this origin (`world-frames`).
const PLANET: SimOrigin = SimOrigin::new(crate::world_frame::FrameId(1), glam::DVec3::ZERO);

const R: f32 = 4800.0;
const G: f32 = 25.0;
const FRAME: f32 = 1.0 / 60.0;

/// Dry for z < 0, shelving south at 0.4 m a metre to 8 m (the fishing
/// tests' shore).
struct Shore;

impl Water for Shore {
    fn surface(&self, _point: Vec3) -> f32 {
        R
    }
    fn bed(&self, point: Vec3) -> f32 {
        let z = (point.normalize_or(Vec3::Y) * R).z;
        R - (0.4 * z).clamp(-2.0, 8.0)
    }
}

fn at(x: f32, depth: f32, z: f32) -> Vec3 {
    Vec3::new(x, R, z).normalize() * (R - depth)
}

fn angler() -> Angler {
    let feet = at(0.0, -0.4, -1.0);
    let up = feet.normalize();
    Angler {
        tip: feet + up * 1.9,
        look: (Vec3::Z - up * Vec3::Z.dot(up) + up * 0.1).normalize(),
        feet,
    }
}

fn kind_of(id: &str) -> WaveKind {
    match id {
        "eel" | "serpent" => WaveKind::Eel,
        "ray" => WaveKind::Ray,
        _ => WaveKind::Fish,
    }
}

/// What the line's fish looked like on one frame.
#[derive(Clone, Copy, Debug)]
struct Seen {
    phase: Phase,
    timer: f32,
    run: f32,
    forward: Vec3,
    top: Vec3,
    origin: Vec3,
    mouth: Vec3,
    lure: Vec3,
}

/// The app's frame, in the core: the line steps every frame, the schools on
/// their own 30 Hz clock, the float is drawn eased, and the drawing turns
/// every fish and moves the line's.
struct Run {
    line: Line,
    schools: Vec<School>,
    roster: Vec<Species>,
    fauna: FaunaSettings,
    drawing: Drawing,
    float: DrawnFloat,
    step_s: f32,
    bite_s: f32,
    reeling: bool,
}

impl Run {
    fn new(seed: u64, id: &str, anchor: Vec3) -> Self {
        let fauna = FaunaSettings::default();
        let roster = fauna.roster(HOME_BODY).to_vec();
        let i = roster.iter().position(|s| s.id == id).unwrap();
        let school = School::spawn(&PLANET, i as u16, &roster[i], anchor, seed, &Shore);
        Self {
            line: Line::new(seed),
            schools: vec![school],
            roster,
            fauna,
            drawing: Drawing::default(),
            float: DrawnFloat::default(),
            step_s: 0.0,
            bite_s: 0.0,
            reeling: false,
        }
    }

    fn frame(&mut self, controls: Controls) -> (Vec<Event>, Option<Seen>) {
        let events = self.line.step(
            &PLANET,
            FRAME,
            controls,
            &angler(),
            &mut self.schools,
            &self.roster,
            &self.fauna.fishing,
            &self.fauna.flock,
            &Shore,
            G,
            1.0,
        );
        let step = 1.0 / self.fauna.spawn.step_hz;
        self.step_s += FRAME;
        while self.step_s >= step {
            self.step_s -= step;
            let splash = self.line.splash();
            for school in &mut self.schools {
                let species = &self.roster[school.species as usize];
                school.step(&PLANET, step, species, &self.fauna.flock, &Shore, splash);
            }
        }
        self.float.update(
            &PLANET,
            FRAME,
            self.line.float,
            self.line.phase,
            self.line.run,
        );
        let drawn = self.float.drawn(&PLANET, self.line.float);
        let lure = drawn - PLANET.up_at(drawn) * 0.35;
        let feet = angler().feet;
        let alpha = self.step_s / step;
        let roster = &self.roster;
        self.drawing.frame(
            &PLANET,
            &mut self.schools,
            roster,
            &self.fauna.behaviour,
            self.line.is_out().then_some(LineView {
                line: &self.line,
                lure,
                feet,
            }),
            alpha,
            FRAME,
            |school, i, species| {
                let length_m = school.length_cm(i, species) as f32 / 100.0;
                Some((
                    Build {
                        length_m,
                        mouth_m: 0.5 * length_m,
                        speeds: species.speed_mps,
                    },
                    kind_of(&species.id),
                ))
            },
            |p| {
                let q = PLANET.query_point(p);
                let r = PLANET.radius_at(p);
                let (top, bottom) = (
                    f64::from(Shore.surface(q) - 0.25),
                    f64::from(Shore.bed(q) + 0.25),
                );
                PLANET.with_radius(p, r.clamp(bottom, top.max(bottom)))
            },
        );
        let seen = self.drawing.line_fish.as_ref().and_then(|f| {
            let pose = self.drawing.poses.get(&f.key)?;
            let school = self.schools.iter().find(|s| s.seed == f.key.0)?;
            let i = school.ids.iter().position(|id| *id == f.key.1)?;
            let length_m =
                school.length_cm(i, &self.roster[school.species as usize]) as f32 / 100.0;
            let up = PLANET.up_at(pose.pos);
            let (forward, top, origin) = pose.frame(up, 0.5 * length_m);
            Some(Seen {
                phase: self.line.phase,
                timer: self.line.timer(),
                run: self.line.run,
                forward,
                top,
                origin,
                mouth: origin + forward * 0.5 * length_m,
                lure: lure + f.lure_offset,
            })
        });
        (events, seen)
    }

    /// The angler's hands: strike 0.3 s into a bite, reel in a band of
    /// tension (as `--fish-play` and M37's angler do), else nothing.
    fn hands(&mut self) -> Controls {
        match self.line.phase {
            Phase::Bite => {
                self.bite_s += FRAME;
                let strike = self.bite_s >= 0.3;
                Controls {
                    pressed: strike,
                    held: strike,
                    ..Default::default()
                }
            }
            Phase::Hooked => {
                self.bite_s = 0.0;
                if self.reeling && self.line.tension > 0.75 {
                    self.reeling = false;
                } else if !self.reeling && self.line.tension < 0.45 {
                    self.reeling = true;
                }
                Controls {
                    held: self.reeling,
                    ..Default::default()
                }
            }
            _ => {
                self.bite_s = 0.0;
                Controls::default()
            }
        }
    }

    fn cast(&mut self) {
        self.frame(Controls {
            pressed: true,
            held: true,
            ..Default::default()
        });
        for _ in 0..36 {
            self.frame(Controls {
                held: true,
                ..Default::default()
            });
        }
        self.frame(Controls {
            released: true,
            ..Default::default()
        });
    }

    /// Play until `until` says stop or `seconds` pass, the angler's hands
    /// on the controls; every frame the line's fish was drawn.
    fn play(&mut self, seconds: f32, until: impl Fn(&Self, &[Event]) -> bool) -> Vec<Seen> {
        let mut seen = Vec::new();
        for _ in 0..(seconds / FRAME) as usize {
            let controls = self.hands();
            let (events, s) = self.frame(controls);
            seen.extend(s);
            if until(self, &events) {
                break;
            }
        }
        seen
    }
}

fn degrees(a: Vec3, b: Vec3) -> f32 {
    a.angle_between(b).to_degrees()
}

/// No frame turns, rolls or moves the line's fish further than a fish
/// can swim round: forward and up each within 20 deg of the frame before,
/// and its middle within 10 cm (6 m/s at 60 frames a second).
fn assert_smooth(label: &str, seen: &[Seen]) {
    for (i, w) in seen.windows(2).enumerate() {
        let (a, b) = (w[0], w[1]);
        let turn = degrees(a.forward, b.forward);
        let roll = degrees(a.top, b.top);
        let moved = a.origin.distance(b.origin);
        assert!(
            turn <= 20.0 && roll <= 20.0 && moved <= 0.1,
            "{label}, frame {i} ({:?} -> {:?}): turned {turn:.1} deg, its up {roll:.1} deg, \
             moved {:.1} cm",
            a.phase,
            b.phase,
            moved * 100.0
        );
    }
}

/// The nibbles of a run: each nibble's frames, cut where its timer jumps
/// back up.
fn nibbles(seen: &[Seen]) -> Vec<Vec<Seen>> {
    let mut out: Vec<Vec<Seen>> = Vec::new();
    let mut last = f32::INFINITY;
    for s in seen.iter().filter(|s| s.phase == Phase::Nibble) {
        if s.timer > last + 1e-3 || out.is_empty() {
            out.push(Vec::new());
        }
        last = s.timer;
        out.last_mut().unwrap().push(*s);
    }
    out
}

#[test]
fn a_fish_pecks_in_time_with_the_float_takes_the_lure_and_fights_without_a_snap() {
    let mut checked = 0;
    for (id, seed) in [("perch", 41), ("minnow", 10), ("silverfin", 11)] {
        let mut run = Run::new(seed, id, at(0.0, 1.0, 8.0));
        run.cast();
        // Out to the end of the bite, then six seconds of the fight.
        let mut seen = run.play(90.0, |r, _| r.line.phase == Phase::Hooked);
        if run.line.phase != Phase::Hooked {
            continue;
        }
        seen.extend(run.play(12.0, |_, events| {
            events
                .iter()
                .any(|e| matches!(e, Event::Caught { .. } | Event::Snapped { .. }))
        }));
        assert!(!seen.is_empty(), "{id}: the line's fish was never drawn");
        assert_smooth(id, &seen);
        // Each nibble: the mouth meets the lure while the float dips, and is
        // off it before.
        let all = nibbles(&seen);
        assert!(!all.is_empty(), "{id}: no nibble drawn");
        for (n, nibble) in all.iter().enumerate() {
            let dip = nibble
                .iter()
                .filter(|s| s.timer < crate::fishing::NIBBLE_DIP_S)
                .map(|s| s.mouth.distance(s.lure))
                .fold(f32::INFINITY, f32::min);
            assert!(
                dip < 0.025,
                "{id}, nibble {n}: the mouth came no nearer than {:.1} cm in the dip",
                dip * 100.0
            );
            // A nibble long enough to back off in is off the lure before its
            // dart.
            if nibble.first().is_some_and(|s| s.timer > 0.7) {
                let off = nibble
                    .iter()
                    .filter(|s| s.timer > 0.4)
                    .map(|s| s.mouth.distance(s.lure))
                    .fold(0.0, f32::max);
                assert!(
                    off > 0.1,
                    "{id}, nibble {n}: never more than {:.1} cm off the lure",
                    off * 100.0
                );
            }
        }
        // On the line: the mouth stays on the drawn lure.
        for s in seen.iter().filter(|s| s.phase == Phase::Hooked) {
            assert!(
                s.mouth.distance(s.lure) < 0.01,
                "{id}: hooked, the mouth is {:.1} cm off the lure",
                s.mouth.distance(s.lure) * 100.0
            );
        }
        // The fight changed between running and resting while it was drawn.
        let switches = seen
            .windows(2)
            .filter(|w| w[0].phase == Phase::Hooked && (w[0].run > 0.0) != (w[1].run > 0.0))
            .count();
        assert!(switches >= 2, "{id}: only {switches} run/rest switches");
        checked += 1;
    }
    assert!(checked >= 2, "only {checked} fish hooked");
}

#[test]
fn a_fish_struck_too_early_bolts_and_rejoins_its_school_where_it_was_drawn() {
    let mut run = Run::new(30, "minnow", at(0.0, 1.0, 8.0));
    run.cast();
    let mut seen = run.play(90.0, |r, _| r.line.phase == Phase::Nibble);
    assert_eq!(run.line.phase, Phase::Nibble);
    seen.extend(run.play(0.3, |_, _| false));
    let key = run.drawing.line_fish.as_ref().unwrap().key;
    let (events, s) = run.frame(Controls {
        pressed: true,
        held: true,
        ..Default::default()
    });
    assert!(matches!(events.first(), Some(Event::TooEarly { .. })));
    seen.extend(s);
    // The bolt, then the fish back in its school's hands.
    let mut last = None;
    for _ in 0..(2.0 / FRAME) as usize {
        let (_, s) = run.frame(Controls::default());
        seen.extend(s);
        let pose = run.drawing.poses.get(&key).unwrap();
        let (forward, top, origin) = pose.frame(PLANET.up_at(pose.pos), 0.07);
        if let Some((f, t, o)) = last {
            assert!(
                degrees(f, forward) <= 20.0 && degrees(t, top) <= 20.0,
                "the bolt turned in one frame"
            );
            let o: Vec3 = o;
            assert!(
                o.distance(origin) <= 0.1,
                "the bolt jumped {:.2} m",
                o.distance(origin)
            );
        }
        last = Some((forward, top, origin));
    }
    assert!(run.drawing.line_fish.is_none(), "the bolt never ended");
    assert_eq!(
        run.schools[0].owned, None,
        "the school was not given it back"
    );
    assert_smooth("bolt", &seen);
}

#[test]
fn a_free_fish_turns_no_faster_than_its_limit_and_banks_into_turns() {
    let mut run = Run::new(5, "perch", at(0.0, 2.0, 12.0));
    let s = run.fauna.behaviour.pose;
    let mut last: HashMap<(u64, u32), Vec3> = HashMap::new();
    let mut banked = 0.0f32;
    for _ in 0..(30.0 / FRAME) as usize {
        run.frame(Controls::default());
        for (key, pose) in &run.drawing.poses {
            if let Some(was) = last.get(key) {
                let turn = degrees(*was, pose.forward);
                // Yaw at the turn limit and pitch at 0.6 of it, together.
                assert!(
                    turn <= s.turn_deg_s * 1.17 * FRAME + 1e-3,
                    "turned {turn:.2} deg in a frame"
                );
                let up = PLANET.up_at(pose.pos);
                let pitch = pose.forward.dot(up).asin().to_degrees().abs();
                assert!(pitch <= s.pitch_max_deg + 0.01, "pitched {pitch:.1} deg");
            }
            banked = banked.max(pose.roll.abs());
            last.insert(*key, pose.forward);
        }
    }
    assert!(banked > 2f32.to_radians(), "never banked: {banked}");
}

#[test]
fn the_body_wave_slope_is_the_derivative_of_its_offset() {
    for w in [WaveProfile::FISH, WaveProfile::EEL] {
        for k in 0..40 {
            let wave = Wave {
                phase: k as f32 * 0.37,
                amp: 0.09,
                bend: 0.05 * ((k % 5) as f32 - 2.0),
            };
            for j in 1..99 {
                let u = j as f32 / 100.0;
                let h = 1e-3;
                let numeric = (lateral(u + h, wave, &w).0 - lateral(u - h, wave, &w).0) / (2.0 * h);
                let (_, slope) = lateral(u, wave, &w);
                assert!(
                    (numeric - slope).abs() < 2e-3 * (1.0 + slope.abs()),
                    "u {u}: slope {slope}, numeric {numeric}"
                );
            }
        }
    }
}

#[test]
fn the_body_wave_runs_from_the_head_to_the_tail() {
    let w = WaveProfile::FISH;
    // The crest nearest the middle of the body, as the phase grows: it moves
    // toward the tail (u grows).
    let crest = |phase: f32| {
        let wave = Wave {
            phase,
            amp: 0.1,
            bend: 0.0,
        };
        (0..=1000)
            .map(|j| j as f32 / 1000.0)
            .filter(|u| (0.4..0.9).contains(u))
            .max_by(|a, b| lateral(*a, wave, &w).0.total_cmp(&lateral(*b, wave, &w).0))
            .unwrap()
    };
    let (a, b) = (crest(1.0), crest(1.3));
    assert!(b > a, "the crest moved from {a} to {b}: toward the head");
    // The head recoils against the tail.
    let wave = Wave {
        phase: 0.6,
        amp: 0.1,
        bend: 0.0,
    };
    let nose = lateral(0.0, wave, &w).0;
    let tail = lateral(1.0, wave, &w).0;
    assert!(nose * tail < 0.0, "nose {nose} and tail {tail} on one side");
}

#[test]
fn no_amplitude_and_no_bend_leave_the_body_as_laid() {
    for w in [WaveProfile::FISH, WaveProfile::EEL] {
        for j in 0..=20 {
            let (f, df) = lateral(
                j as f32 / 20.0,
                Wave {
                    phase: 1.7,
                    amp: 0.0,
                    bend: 0.0,
                },
                &w,
            );
            assert_eq!((f, df), (0.0, 0.0));
        }
    }
}

#[test]
fn a_turning_fish_bends_into_its_turn_and_a_gliding_one_straightens() {
    let s = BehaviourSettings::default();
    let w = s.fish_wave;
    let up = Vec3::Y;
    let mut pose = Pose::new(Vec3::ZERO, Vec3::X * 0.8, up, 0.0, &w);
    // Turning left at 90 deg/s, cruising.
    for _ in 0..120 {
        pose.yaw_rate = 90f32.to_radians();
        pose.speed = 0.8;
        pose.settle(FRAME, &s.pose, &w, 0.38, (0.8, 2.4));
    }
    // A left turn bows the body to the left: its bend is negative (toward
    // -side) and its roll banks into the turn.
    assert!(pose.wave.bend < -0.05, "bend {}", pose.wave.bend);
    assert!(pose.roll < -10f32.to_radians(), "roll {}", pose.roll);
    // Slowing hard: a glide, the tail's amplitude falls to the glide's.
    pose.yaw_rate = 0.0;
    let mut speed = 2.0;
    for _ in 0..60 {
        speed -= 1.0 * FRAME;
        pose.speed = speed;
        pose.settle(FRAME, &s.pose, &w, 0.38, (0.8, 2.4));
    }
    assert!(pose.wave.amp < 0.02, "gliding amplitude {}", pose.wave.amp);
}
