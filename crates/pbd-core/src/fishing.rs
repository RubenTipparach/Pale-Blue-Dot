//! The fishing line: charge, cast, float, nibble, bite, hook, reel, catch.
//!
//! Every rule a catch depends on is here and nowhere else, so single player
//! and a future multiplayer land the same fish. The line works in planet-local
//! metres, like the schools (`fauna::school`), and is stepped with the
//! schools it fishes in: the fish that bites is a fish that was swimming
//! there, and the one caught is taken out of its school.
//!
//! The hook window is Tenebris's formula (`FishingSettings::hook_window`);
//! reeling against a tension meter is this game's (owner question 5 in the
//! proposal).

use crate::fauna::{FishingSettings, FlockSettings, Rng, School, Species, Water};
use glam::Vec3;

/// Where the line is.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Phase {
    /// Nothing out; a press starts a charge.
    Ready,
    /// The button is down, charging the cast.
    Charging,
    /// The float is in the air.
    Flying,
    /// The float is on the water, waiting for interest.
    Floating,
    /// A fish is at the lure, testing it. Hooking now spooks it.
    Nibble,
    /// The float is under: hook now.
    Bite,
    /// A fish is on the line.
    Hooked,
}

/// This step's controls.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Controls {
    /// The use button went down this step.
    pub pressed: bool,
    /// The use button is down.
    pub held: bool,
    /// The use button came up this step.
    pub released: bool,
    /// Reel the line in, empty.
    pub reel_in: bool,
}

/// Where the angler is, planet-local.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Angler {
    /// The rod tip: where a cast leaves from.
    pub tip: Vec3,
    /// Where the angler is looking, unit.
    pub look: Vec3,
    /// Where the angler stands.
    pub feet: Vec3,
}

/// What happened on a step, for the HUD, the sounds and the save.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Event {
    Cast {
        speed_mps: f32,
    },
    /// The float came down on dry ground, or flew too long.
    Dry,
    Splash,
    /// A fish of this species has left its school for the lure.
    Interest {
        species: u16,
    },
    Nibble {
        species: u16,
    },
    Bite {
        species: u16,
        window_s: f32,
    },
    /// Hooked during a nibble: the fish bolted.
    TooEarly {
        species: u16,
    },
    /// The window passed.
    Missed {
        species: u16,
    },
    Hooked {
        species: u16,
    },
    Snapped {
        species: u16,
    },
    Caught {
        species: u16,
        length_cm: u32,
    },
    /// Wound in empty, or the angler walked away from it.
    ReeledIn,
}

/// Which fish the line is working: a school and a fish in it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Target {
    pub school: usize,
    pub fish: usize,
}

/// One rod's line.
#[derive(Clone, Debug, PartialEq)]
pub struct Line {
    pub phase: Phase,
    /// How charged the cast is, 0..1.
    pub power: f32,
    /// Where the float is, planet-local.
    pub float: Vec3,
    velocity: Vec3,
    /// How far under the surface the float is pulled, m (positive is down).
    pub dip: f32,
    flight_s: f32,
    /// Seconds the splash still scares fish off.
    pub splash_s: f32,
    check_s: f32,
    timer: f32,
    nibbles: u8,
    pub target: Option<Target>,
    /// On the line: tension 0..1 (1 snaps it), the fish's stamina, how hard
    /// it is running now, and how much line is out, m.
    pub tension: f32,
    pub stamina: f32,
    pub run: f32,
    run_s: f32,
    pub line_m: f32,
    /// Which way the fish runs, along the surface at the angler, fixed at the
    /// hook; its swing across the line is measured from this, never from
    /// last step's, or it would compound round onto the beach.
    heading: Vec3,
    clock: f32,
    rng: Rng,
}

impl Line {
    pub fn new(seed: u64) -> Self {
        Self {
            phase: Phase::Ready,
            power: 0.0,
            float: Vec3::ZERO,
            velocity: Vec3::ZERO,
            dip: 0.0,
            flight_s: 0.0,
            splash_s: 0.0,
            check_s: 0.0,
            timer: 0.0,
            nibbles: 0,
            target: None,
            tension: 0.0,
            stamina: 1.0,
            run: 0.0,
            run_s: 0.0,
            line_m: 0.0,
            heading: Vec3::X,
            clock: 0.0,
            rng: Rng::new(seed),
        }
    }

    /// Whether the float is out, in the air or on the water.
    pub fn is_out(&self) -> bool {
        !matches!(self.phase, Phase::Ready | Phase::Charging)
    }

    /// The splash the schools should scatter from this step.
    pub fn splash(&self) -> Option<Vec3> {
        (self.splash_s > 0.0).then_some(self.float)
    }

    /// The lure: a hand's width under the float, where a fish comes to it.
    pub fn lure(&self) -> Vec3 {
        self.float - self.float.normalize_or(Vec3::Y) * 0.35
    }

    /// Take the line in, empty, freeing any fish it was working.
    pub fn wind_in(&mut self, schools: &mut [School], flock: &FlockSettings) {
        self.release(schools, flock);
        self.phase = Phase::Ready;
        self.power = 0.0;
        self.dip = 0.0;
        self.splash_s = 0.0;
        self.tension = 0.0;
    }

    fn release(&mut self, schools: &mut [School], flock: &FlockSettings) {
        if let Some(t) = self.target.take()
            && let Some(school) = schools.get_mut(t.school)
        {
            school.frighten(t.fish, flock.spook_s);
            school.lure = None;
            school.held = None;
        }
    }

    /// One step. `schools` are the schools in the water around the angler,
    /// `roster` the body's species, `gravity` the planet's pull at the
    /// surface, m/s^2, and `bite_factor` the weather's
    /// (`FishingSettings::bite_factor`).
    #[allow(clippy::too_many_arguments)]
    pub fn step(
        &mut self,
        dt: f32,
        controls: Controls,
        angler: &Angler,
        schools: &mut [School],
        roster: &[Species],
        s: &FishingSettings,
        flock: &FlockSettings,
        water: &impl Water,
        gravity: f32,
        bite_factor: f32,
    ) -> Vec<Event> {
        let mut events = Vec::new();
        if dt.is_nan() || dt <= 0.0 {
            return events;
        }
        self.clock += dt;
        self.splash_s = (self.splash_s - dt).max(0.0);
        // A target whose school has gone (despawned out of range) is no
        // target: the line goes back to waiting rather than steering nothing.
        if let Some(t) = self.target
            && schools
                .get(t.school)
                .is_none_or(|school| t.fish >= school.len())
        {
            self.target = None;
            if matches!(self.phase, Phase::Nibble | Phase::Bite | Phase::Hooked) {
                self.phase = Phase::Floating;
                self.check_s = s.scent_every_s;
            }
        }
        for school in schools.iter_mut() {
            school.lure = None;
            school.held = None;
        }
        if controls.reel_in
            && matches!(
                self.phase,
                Phase::Flying | Phase::Floating | Phase::Nibble | Phase::Bite
            )
        {
            self.wind_in(schools, flock);
            events.push(Event::ReeledIn);
            return events;
        }
        if self.is_out() && self.float.distance(angler.feet) > s.line_max_m {
            self.wind_in(schools, flock);
            events.push(Event::ReeledIn);
            return events;
        }
        match self.phase {
            Phase::Ready => {
                if controls.pressed {
                    self.phase = Phase::Charging;
                    self.power = 0.0;
                }
            }
            Phase::Charging => {
                self.power = (self.power + dt / s.charge_s).min(1.0);
                if controls.released || !controls.held {
                    let speed = lerp(s.speed_mps, self.power);
                    let up = angler.tip.normalize_or(Vec3::Y);
                    self.float = angler.tip;
                    self.velocity =
                        angler.look.normalize_or(up) * speed + up * lerp(s.lift_mps, self.power);
                    self.flight_s = 0.0;
                    self.dip = 0.0;
                    self.phase = Phase::Flying;
                    events.push(Event::Cast { speed_mps: speed });
                }
            }
            Phase::Flying => {
                self.flight_s += dt;
                let up = self.float.normalize_or(Vec3::Y);
                self.velocity -= up * gravity * dt;
                self.float += self.velocity * dt;
                let r = self.float.length();
                let (surface, bed) = (water.surface(self.float), water.bed(self.float));
                if bed >= surface - 0.05 && r <= bed {
                    self.phase = Phase::Ready;
                    events.push(Event::Dry);
                } else if r <= surface {
                    self.float = self.float.normalize_or(up) * surface;
                    self.phase = Phase::Floating;
                    self.splash_s = s.splash_s;
                    self.check_s = s.scent_every_s;
                    events.push(Event::Splash);
                } else if self.flight_s > s.flight_max_s {
                    self.phase = Phase::Ready;
                    events.push(Event::Dry);
                }
            }
            Phase::Floating => {
                self.check_s -= dt;
                if self.check_s <= 0.0 && self.splash_s <= 0.0 {
                    self.check_s = s.scent_every_s;
                    if self.target.is_none() {
                        self.look_for_interest(schools, roster, s, water, bite_factor, &mut events);
                    }
                }
                if let Some(t) = self.target {
                    let lure = self.lure();
                    let school = &mut schools[t.school];
                    school.lure = Some((t.fish, lure));
                    if school.positions[t.fish].distance(lure) < s.nibble_m {
                        self.phase = Phase::Nibble;
                        self.nibbles =
                            self.rng.range(s.nibbles.0 as f32, s.nibbles.1 as f32 + 1.0) as u8;
                        self.nibbles = self.nibbles.clamp(s.nibbles.0.max(1), s.nibbles.1.max(1));
                        self.timer = self.rng.range(s.nibble_s.0, s.nibble_s.1);
                        events.push(Event::Nibble {
                            species: school.species,
                        });
                    }
                }
            }
            Phase::Nibble => {
                let Some(t) = self.target else {
                    self.phase = Phase::Floating;
                    return events;
                };
                let species = schools[t.school].species;
                schools[t.school].held = Some((t.fish, self.lure()));
                if controls.pressed {
                    self.release(schools, flock);
                    self.phase = Phase::Floating;
                    self.check_s = 2.0;
                    events.push(Event::TooEarly { species });
                } else {
                    self.timer -= dt;
                    if self.timer <= 0.0 {
                        self.nibbles = self.nibbles.saturating_sub(1);
                        if self.nibbles == 0 {
                            let strength = roster.get(species as usize).map_or(1, |sp| sp.strength);
                            self.timer = s.hook_window(strength);
                            self.phase = Phase::Bite;
                            events.push(Event::Bite {
                                species,
                                window_s: self.timer,
                            });
                        } else {
                            self.timer = self.rng.range(s.nibble_s.0, s.nibble_s.1);
                        }
                    }
                }
            }
            Phase::Bite => {
                let Some(t) = self.target else {
                    self.phase = Phase::Floating;
                    return events;
                };
                let species = schools[t.school].species;
                schools[t.school].held = Some((t.fish, self.lure()));
                if controls.pressed {
                    let pull = roster.get(species as usize).map_or(0.5, |sp| sp.pull);
                    self.phase = Phase::Hooked;
                    self.tension = 0.2;
                    self.stamina = 1.0;
                    self.run = pull;
                    self.run_s = self.rng.range(0.8, 1.6);
                    let out = tangent(self.float - angler.feet, angler.feet);
                    self.line_m = out.length();
                    self.heading =
                        out.normalize_or(tangent(angler.look, angler.feet).normalize_or(Vec3::X));
                    events.push(Event::Hooked { species });
                } else {
                    self.timer -= dt;
                    if self.timer <= 0.0 {
                        self.release(schools, flock);
                        self.phase = Phase::Floating;
                        self.check_s = 1.5;
                        events.push(Event::Missed { species });
                    }
                }
            }
            Phase::Hooked => {
                let Some(t) = self.target else {
                    self.phase = Phase::Ready;
                    return events;
                };
                let species = schools[t.school].species;
                let pull = roster.get(species as usize).map_or(0.5, |sp| sp.pull);
                self.run_s -= dt;
                if self.run_s <= 0.0 {
                    self.run = if self.run > 0.0 { 0.0 } else { pull };
                    self.run_s = if self.run > 0.0 {
                        self.rng.range(0.8, 2.0)
                    } else {
                        self.rng.range(1.2, 3.0)
                    };
                }
                if controls.held {
                    self.line_m -= s.reel_mps * dt * (1.0 - 0.6 * (self.run / 1.5).min(1.0));
                    self.tension +=
                        (s.tension_base + s.tension_pull * self.run * self.stamina) * dt;
                } else {
                    self.tension -= s.tension_fall * dt;
                    self.line_m += self.run * 1.2 * self.stamina * dt;
                }
                if self.run > 0.0 && self.tension > 0.25 {
                    self.stamina = (self.stamina - s.stamina_drain * dt).max(s.stamina_floor);
                }
                self.tension = self.tension.max(0.0);
                self.line_m = self.line_m.clamp(0.0, s.line_max_m);
                // The fish runs away from the angler and across the line, so
                // the float weaves while it runs.
                let up = angler.feet.normalize_or(Vec3::Y);
                let swing = (self.clock * 1.7).sin() * 0.12 * self.run;
                let away = rotate_about(
                    tangent(self.heading, angler.feet).normalize_or(self.heading),
                    up,
                    swing,
                );
                let point = angler.feet + away * self.line_m;
                let dir = point.normalize_or(up);
                self.float = dir * (water.surface(dir * point.length()) - self.dip);
                schools[t.school].held = Some((t.fish, self.lure()));
                if self.tension >= 1.0 {
                    self.release(schools, flock);
                    self.phase = Phase::Ready;
                    self.tension = 0.0;
                    events.push(Event::Snapped { species });
                } else if self.line_m <= s.land_line_m || water.depth(self.float) < s.land_depth_m {
                    let length_cm = roster
                        .get(species as usize)
                        .map_or(1, |sp| schools[t.school].length_cm(t.fish, sp));
                    schools[t.school].remove(t.fish);
                    self.target = None;
                    self.phase = Phase::Ready;
                    self.tension = 0.0;
                    events.push(Event::Caught { species, length_cm });
                }
            }
        }
        // The float rides the sea wherever it is on it.
        if matches!(self.phase, Phase::Floating | Phase::Nibble | Phase::Bite) {
            let want = match self.phase {
                Phase::Bite => s.bite_dip_m,
                Phase::Nibble if self.timer < 0.12 => s.nibble_dip_m,
                _ => 0.0,
            };
            self.dip += (want - self.dip) * (dt * 14.0).min(1.0);
            let up = self.float.normalize_or(Vec3::Y);
            self.float = up * (water.surface(self.float) - self.dip);
        }
        events
    }

    /// Every school near the float gets a chance to send one fish: the ones
    /// within its reach turn toward the lure, and one within its sense range
    /// may commit a fish to it.
    fn look_for_interest(
        &mut self,
        schools: &mut [School],
        roster: &[Species],
        s: &FishingSettings,
        water: &impl Water,
        bite_factor: f32,
        events: &mut Vec<Event>,
    ) {
        let lure = self.lure();
        for (index, school) in schools.iter_mut().enumerate() {
            let Some(species) = roster.get(school.species as usize) else {
                continue;
            };
            let d = tangent(school.centre() - lure, lure).length();
            if d > s.scent_reach * species.sense_m {
                continue;
            }
            if d > 2.5 {
                school.scent(species, lure, water, 3.0);
            }
            if d <= species.sense_m
                && self.rng.unit() < species.bite * bite_factor * s.bite_chance
                && let Some(fish) = school.nearest_calm(lure)
            {
                self.target = Some(Target {
                    school: index,
                    fish,
                });
                events.push(Event::Interest {
                    species: school.species,
                });
                return;
            }
        }
    }
}

fn lerp(range: (f32, f32), t: f32) -> f32 {
    range.0 + (range.1 - range.0) * t.clamp(0.0, 1.0)
}

/// The part of `v` along the surface at `at`.
fn tangent(v: Vec3, at: Vec3) -> Vec3 {
    let up = at.normalize_or(Vec3::Y);
    v - up * v.dot(up)
}

fn rotate_about(v: Vec3, axis: Vec3, angle: f32) -> Vec3 {
    glam::Quat::from_axis_angle(axis.normalize_or(Vec3::Y), angle) * v
}

#[cfg(test)]
mod tests;
