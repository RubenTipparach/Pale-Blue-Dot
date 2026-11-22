//! Logs on the water (`log-drives`, design "As built"): a log thrown into a
//! river is carried by the water's own flow and comes to rest where the
//! water stops flowing or is too shallow to float it.
//!
//! The rule is here and pure: given what the water says at the log (its
//! surface, its current, its depth), [`step`] moves it. The app asks the
//! same water a craft asks (`vehicles::fresh_water`), so a log and a raft in
//! one river drift the same way. A log is world state the player made, so
//! it is a stored record ([`LOG_RECORD`]) with a body-frame f64 position
//! (`world-frames` 10.4).

use glam::DVec3;
use serde::{Deserialize, Serialize};

/// The record kind a log is kept as, and its schema.
pub const LOG_RECORD: &str = "log";
pub const LOG_SCHEMA: u32 = 1;
/// How fast a log's velocity eases toward the water's, per second.
pub const DRAG_PER_S: f64 = 1.2;
/// Under this current a log is in still water, m/s.
pub const REST_MPS: f64 = 0.15;
/// How long it must sit in still water to come to rest, s.
pub const REST_S: f64 = 1.5;
/// Over this current a resting log floats off again, m/s.
pub const WAKE_MPS: f64 = 0.3;
/// The log's draught: shallower water grounds it, m.
pub const FLOAT_DEPTH_M: f32 = 0.3;
/// How far its middle rides under the surface, m: a log floats half out.
pub const RIDE_M: f64 = 0.0;
/// How often a moving log is written, s.
pub const SAVE_EVERY_S: f64 = 5.0;
/// A log's length and radius, m.
pub const LENGTH_M: f32 = 2.4;
pub const RADIUS_M: f32 = 0.18;
/// How fast its long axis turns to the flow, per second.
pub const TURN_PER_S: f64 = 0.8;

/// Where a log is in its life.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub enum LogState {
    Floating,
    Resting,
    /// Picked up: into the slots, or a raft's hold. Kept so the journal
    /// says where it went; never built again.
    Taken,
}

/// A log: where it is (body frame, metres, f64), its long axis, and its
/// state. Its velocity and the still-water clock are not stored: a loaded
/// log starts from rest in the water it is in.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
pub struct LogBody {
    pub position: [f64; 3],
    pub heading: [f64; 3],
    pub state: LogState,
}

/// What the water is at a log.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Water {
    /// Its surface's radius, m.
    pub surface: f64,
    /// Its flow along the surface, m/s.
    pub current: DVec3,
    /// How deep it is under the log, m.
    pub depth: f32,
}

/// A log in the world, with what a step keeps between frames.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Log {
    pub id: u64,
    pub body: LogBody,
    pub velocity: DVec3,
    /// How long it has been in still water, s.
    pub still_s: f64,
    /// Since it was last written, s.
    pub unsaved_s: f64,
}

/// What one step did that the save must hear of.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stepped {
    /// Nothing to write.
    Quiet,
    /// It came to rest, or floated off again: write it now.
    Changed,
    /// It moved for [`SAVE_EVERY_S`] since it was last written.
    Due,
}

impl Log {
    pub fn new(id: u64, body: LogBody) -> Self {
        Self {
            id,
            body,
            velocity: DVec3::ZERO,
            still_s: 0.0,
            unsaved_s: 0.0,
        }
    }

    pub fn position(&self) -> DVec3 {
        DVec3::from_array(self.body.position)
    }

    pub fn heading(&self) -> DVec3 {
        DVec3::from_array(self.body.heading)
    }
}

/// A log made at `position` (body frame), floating, its axis across `up`.
pub fn made(position: DVec3, heading: DVec3) -> LogBody {
    let up = position.normalize_or(DVec3::Y);
    let along = (heading - up * heading.dot(up)).normalize_or(up.any_orthonormal_vector());
    LogBody {
        position: position.to_array(),
        heading: along.to_array(),
        state: LogState::Floating,
    }
}

/// Whether water can float a log.
fn floats(water: Option<Water>) -> Option<Water> {
    water.filter(|w| w.depth >= FLOAT_DEPTH_M)
}

/// One step of `dt` seconds, in `water` (none: on dry ground or off the
/// known world, where it lies still).
pub fn step(log: &mut Log, water: Option<Water>, dt: f64) -> Stepped {
    if log.body.state == LogState::Taken || dt <= 0.0 {
        return Stepped::Quiet;
    }
    let position = log.position();
    let up = position.normalize_or(DVec3::Y);
    match (log.body.state, floats(water)) {
        // Grounded or on the bank: it lies where it is.
        (LogState::Floating, None) => {
            log.velocity = DVec3::ZERO;
            log.still_s = 0.0;
            log.body.state = LogState::Resting;
            log.unsaved_s = 0.0;
            Stepped::Changed
        }
        (LogState::Resting, None) => Stepped::Quiet,
        (LogState::Resting, Some(w)) => {
            if w.current.length() > WAKE_MPS {
                log.body.state = LogState::Floating;
                log.still_s = 0.0;
                log.unsaved_s = 0.0;
                Stepped::Changed
            } else {
                Stepped::Quiet
            }
        }
        (LogState::Floating, Some(w)) => {
            let current = w.current - up * w.current.dot(up);
            let ease = 1.0 - (-DRAG_PER_S * dt).exp();
            log.velocity += (current - log.velocity) * ease;
            let moved = position + log.velocity * dt;
            let up2 = moved.normalize_or(up);
            let at = up2 * (w.surface - RIDE_M);
            // Its long axis turns toward the flow (either end first).
            let heading = log.heading();
            let flow = current.normalize_or_zero();
            let toward = if flow.dot(heading) < 0.0 { -flow } else { flow };
            let turned = heading + (toward - heading) * (1.0 - (-TURN_PER_S * dt).exp());
            let turned = (turned - up2 * turned.dot(up2)).normalize_or(heading);
            log.body.position = at.to_array();
            log.body.heading = turned.to_array();
            log.unsaved_s += dt;
            if current.length() < REST_MPS {
                log.still_s += dt;
            } else {
                log.still_s = 0.0;
            }
            if log.still_s >= REST_S {
                log.velocity = DVec3::ZERO;
                log.body.state = LogState::Resting;
                log.unsaved_s = 0.0;
                return Stepped::Changed;
            }
            if log.unsaved_s >= SAVE_EVERY_S {
                log.unsaved_s = 0.0;
                return Stepped::Due;
            }
            Stepped::Quiet
        }
        (LogState::Taken, _) => Stepped::Quiet,
    }
}

/// The record that keeps a log as it is now.
pub fn record(log: &Log) -> crate::records::Record {
    crate::records::Record::of(LOG_RECORD, log.id, LOG_SCHEMA, &log.body)
}

/// Every log the records hold that has not been taken, by id.
pub fn from_records(records: &crate::records::Records) -> Vec<Log> {
    records
        .of_kind(LOG_RECORD)
        .filter(|r| r.schema == LOG_SCHEMA)
        .filter_map(|r| Some(Log::new(r.id, r.read::<LogBody>()?)))
        .filter(|l| l.body.state != LogState::Taken)
        .collect()
}

/// The id a new log takes: one past every log the records name.
pub fn next_id(records: &crate::records::Records) -> u64 {
    records
        .of_kind(LOG_RECORD)
        .map(|r| r.id + 1)
        .max()
        .unwrap_or(1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::records::{Author, Record, Records};

    const R: f64 = 4800.0;

    /// A river along +X at the top of the sphere: flowing `speed` m/s while
    /// x < `ends_at`, still past it, and dry past `bank`.
    fn river(p: DVec3, speed: f64, ends_at: f64, bank: f64) -> Option<Water> {
        if p.x > bank {
            return None;
        }
        Some(Water {
            surface: R + 2.0,
            current: if p.x < ends_at {
                DVec3::X * speed
            } else {
                DVec3::ZERO
            },
            depth: 1.5,
        })
    }

    fn start() -> Log {
        Log::new(1, made(DVec3::new(0.0, R + 2.0, 0.0), DVec3::Z))
    }

    fn run(log: &mut Log, water: impl Fn(DVec3) -> Option<Water>, seconds: f64) -> Vec<Stepped> {
        let dt = 1.0 / 60.0;
        let mut out = Vec::new();
        for _ in 0..(seconds / dt) as usize {
            let w = water(log.position());
            out.push(step(log, w, dt));
        }
        out
    }

    /// A log dropped upstream drifts down with the current and stops in the
    /// still water where the river ends.
    #[test]
    fn a_log_drifts_down_the_river_and_rests_where_it_stops_flowing() {
        let mut log = start();
        let said = run(&mut log, |p| river(p, 2.0, 40.0, 1000.0), 60.0);
        assert_eq!(log.body.state, LogState::Resting);
        let x = log.position().x;
        assert!((40.0..55.0).contains(&x), "came to rest at {x}");
        assert!(said.contains(&Stepped::Due), "written while it moved");
        assert!(said.contains(&Stepped::Changed), "written when it stopped");
        // Its axis has turned along the flow.
        assert!(log.heading().x.abs() > 0.9, "{:?}", log.heading());
        // It rides the surface.
        let r = log.position().length();
        assert!((r - (R + 2.0 - RIDE_M)).abs() < 0.05, "{r}");
    }

    /// Water shallower than its draught grounds it, and a bank stops it.
    #[test]
    fn a_log_grounds_in_the_shallows_and_on_a_bank() {
        let mut log = start();
        run(
            &mut log,
            |p| {
                river(p, 2.0, 1e9, 1e9).map(|w| Water {
                    depth: if p.x > 10.0 { 0.1 } else { w.depth },
                    ..w
                })
            },
            30.0,
        );
        assert_eq!(log.body.state, LogState::Resting);
        assert!((10.0..12.0).contains(&log.position().x));
        let mut log = start();
        run(&mut log, |p| river(p, 2.0, 1e9, 8.0), 30.0);
        assert_eq!(log.body.state, LogState::Resting);
        assert!((8.0..9.0).contains(&log.position().x));
    }

    /// A resting log floats off when the water moves again.
    #[test]
    fn a_resting_log_floats_again_when_the_water_moves() {
        let mut log = start();
        run(&mut log, |p| river(p, 0.0, 0.0, 1e9), 5.0);
        assert_eq!(log.body.state, LogState::Resting);
        let said = run(&mut log, |p| river(p, 1.0, 1e9, 1e9), 1.0);
        assert_eq!(said.first(), Some(&Stepped::Changed));
        assert_eq!(log.body.state, LogState::Floating);
    }

    /// A log's record round trips; a taken log is not built again, and a new
    /// log takes the next id.
    #[test]
    fn a_log_is_kept_until_it_is_taken() {
        let mut log = start();
        run(&mut log, |p| river(p, 2.0, 1e9, 1e9), 3.0);
        let mut records = Records::new();
        let line = record(&log).line(&Author::Player(0));
        records.put(Record::parse_line(&line).unwrap().1);
        let back = from_records(&records);
        assert_eq!(back.len(), 1);
        assert_eq!(back[0].body, log.body);
        assert_eq!(next_id(&records), 2);
        log.body.state = LogState::Taken;
        records.put(record(&log));
        assert!(from_records(&records).is_empty());
        assert_eq!(next_id(&records), 2);
    }
}
