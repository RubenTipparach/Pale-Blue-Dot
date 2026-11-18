//! A repro record (`repro-capture`): where the player is, which way they
//! look, when and in what weather, and the blocks round them, written so an
//! agent can stand there again with the launch flags.
//!
//! The owner, 2026-11-16: "add a button for capturing location data so I can
//! help repro stuff (shift alt F or something)". The key and the files are
//! the desktop's (`desktop/repro.rs`); what a record holds, how it reads the
//! world and the command line it writes are here, so a test can hold the
//! command line to the game's own argument parser.

pub mod blocks;

use bevy::prelude::*;
use pbd_core::geo;

use crate::config::Reflections;
use crate::place::{self, Place};

/// How the player was moving.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Walking,
    Swimming,
    Flying,
    /// Aboard a craft, by its key (`loon`, `cog`, ...).
    Aboard(&'static str),
}

impl Mode {
    pub fn name(self) -> String {
        match self {
            Mode::Walking => "walking".into(),
            Mode::Swimming => "swimming".into(),
            Mode::Flying => "flying".into(),
            Mode::Aboard(kind) => format!("aboard the {kind}"),
        }
    }
}

/// Everything a repro record says.
#[derive(Clone, Debug, PartialEq)]
pub struct Record {
    /// The world's name; `None` for a world with no disk behind it.
    pub world: Option<String>,
    pub seed: u64,
    /// The generator the world is made by, and this build's newest.
    pub generator: u32,
    pub newest_generator: u32,
    /// How many edits the world holds: none of them is in the command's
    /// fresh world.
    pub edits: usize,
    /// The eye's place.
    pub place: Place,
    /// The eye over the feet on foot; over the terrain flying.
    pub eye_height_m: f32,
    /// As `--yaw` and `--pitch` take them, degrees.
    pub yaw_deg: f32,
    pub pitch_deg: f32,
    pub mode: Mode,
    /// In water, how deep over the feet, and whether over the eyes.
    pub in_water: bool,
    pub water_depth_m: f32,
    pub eyes_under: bool,
    /// The clock.
    pub hour: f32,
    pub day: u64,
    pub season: &'static str,
    /// The weather here, as shown.
    pub rain: f32,
    pub snowing: bool,
    pub wetness: f32,
    pub cover: f32,
    pub sheltered: bool,
    /// Surface wind, m/s, and the compass bearing it blows toward, degrees.
    pub wind_mps: f32,
    pub wind_toward_deg: f32,
    /// The settings that change the look.
    pub render_scale: u32,
    pub reflections: Reflections,
    pub rain_shadows: bool,
    /// The build's source revision, or "unknown".
    pub commit: &'static str,
    /// The blocks round the player, where the fine set is resident there.
    pub blocks: Option<blocks::BlockDump>,
}

/// The source revision this build came from (`build.rs`), or "unknown".
pub const COMMIT: &str = match option_env!("PBD_GIT_COMMIT") {
    Some(commit) => commit,
    None => "unknown",
};

/// Rain this strong or more is passed on as `--rain`.
const RAIN_FLAG_MIN: f32 = 0.05;

/// The `fast` profile's binary, which every play and capture build is.
pub fn binary() -> &'static str {
    if cfg!(windows) {
        r"target\fast\pbd-app.exe"
    } else {
        "target/fast/pbd-app"
    }
}

impl Record {
    /// The hour and day the command passes: the hour to four places, and a
    /// clock a hair before midnight rounded onto the next day, not back to
    /// the start of its own.
    pub fn clock_flags(&self) -> (f32, u64) {
        let hour = (self.hour * 10_000.0).round() / 10_000.0;
        if hour >= 24.0 {
            (0.0, self.day + 1)
        } else {
            (hour.max(0.0), self.day)
        }
    }

    /// The `--rain` that brings weather like this here, if any fell.
    pub fn rain_flag(&self) -> Option<f32> {
        (self.rain >= RAIN_FLAG_MIN).then(|| (self.rain * 100.0).round().clamp(0.0, 100.0) / 100.0)
    }

    /// The launch arguments that put a capture where this record was taken,
    /// with the PNG at `capture`. No `--world`: a capture with none runs in
    /// memory and never writes to a player's save.
    pub fn args(&self, capture: &str) -> Vec<String> {
        let mut args: Vec<String> = Vec::new();
        let flying = self.mode == Mode::Flying;
        args.push(if flying { "--fly" } else { "--walk" }.into());
        args.extend([
            "--at".into(),
            format!("{:.5}", self.place.lat),
            format!("{:.5}", self.place.lon),
        ]);
        if flying {
            args.extend([
                "--height".into(),
                format!("{:.2}", self.eye_height_m.max(0.0)),
            ]);
        }
        let (hour, day) = self.clock_flags();
        args.extend([
            "--yaw".into(),
            format!("{:.2}", self.yaw_deg),
            "--pitch".into(),
            format!("{:.2}", self.pitch_deg.clamp(-89.0, 89.0)),
            "--time".into(),
            format!("{hour:.4}"),
            "--day".into(),
            day.to_string(),
        ]);
        if let Some(rain) = self.rain_flag() {
            args.extend(["--rain".into(), format!("{rain:.2}")]);
        }
        if self.generator != self.newest_generator {
            args.extend(["--generator".into(), self.generator.to_string()]);
        }
        args.extend([
            "--render-scale".into(),
            self.render_scale.to_string(),
            "--reflections".into(),
            self.reflections.label().to_lowercase(),
            "--rain-shadows".into(),
            if self.rain_shadows { "on" } else { "off" }.into(),
            "--capture".into(),
            capture.into(),
        ]);
        args
    }

    /// The command line, ready to paste.
    pub fn command(&self, capture: &str) -> String {
        std::iter::once(binary().to_string())
            .chain(self.args(capture))
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// The record as text, which is also what goes on the clipboard: the
    /// command first, then `key: value` lines, then the block table, then
    /// the screenshot's path.
    pub fn text(&self, saved: &str, screenshot: &str, capture: &str) -> String {
        let p = &self.place;
        let ns = if p.map_lat < 0.0 { 'S' } else { 'N' };
        let ew = if p.lon < 0.0 { 'W' } else { 'E' };
        let water = if self.in_water {
            format!(
                "in water, {:.2} m deep over the feet{}",
                self.water_depth_m,
                if self.eyes_under { ", eyes under" } else { "" }
            )
        } else {
            "dry".into()
        };
        let world = match &self.world {
            Some(name) => name.clone(),
            None => "(unsaved)".into(),
        };
        let weather_flag = match self.rain_flag() {
            Some(rain) => format!(
                "--rain {rain:.2} (approximate: forces a storm of that strength here; \
                 the simulated weather is not seeded by any flag)"
            ),
            None => "none (dry here; the simulated weather is not seeded by any flag)".into(),
        };
        let lines = [
            ("command", self.command(capture)),
            ("saved", saved.to_string()),
            ("world", world),
            ("seed", self.seed.to_string()),
            (
                "generator",
                format!(
                    "{} (this build's newest: {})",
                    self.generator, self.newest_generator
                ),
            ),
            (
                "edits",
                format!(
                    "{} (in the world, not in the command's fresh world)",
                    self.edits
                ),
            ),
            ("mode", self.mode.name()),
            (
                "at",
                format!(
                    "{:.5} {:.5} (as --at takes it; the map reads {:.3}{ns} {:.3}{ew})",
                    p.lat,
                    p.lon,
                    p.map_lat.abs(),
                    p.lon.abs()
                ),
            ),
            ("elevation_m", format!("{:.2}", p.elevation_m)),
            ("above_ground_m", format!("{:.2}", p.above_ground_m)),
            ("eye_height_m", format!("{:.2}", self.eye_height_m)),
            ("yaw_deg", format!("{:.2}", self.yaw_deg)),
            ("pitch_deg", format!("{:.2}", self.pitch_deg)),
            ("water", water),
            (
                "clock",
                format!("hour {:.4}, day {}, {}", self.hour, self.day, self.season),
            ),
            (
                "weather",
                format!(
                    "{} {:.2}, wetness {:.2}, cover {:.2}, wind {:.1} m/s toward {:.0} deg{}",
                    if self.snowing { "snow" } else { "rain" },
                    self.rain,
                    self.wetness,
                    self.cover,
                    self.wind_mps,
                    self.wind_toward_deg,
                    if self.sheltered { ", sheltered" } else { "" }
                ),
            ),
            ("weather_flag", weather_flag),
            (
                "settings",
                format!(
                    "render scale {}, reflections {}, rain shadow {}",
                    self.render_scale,
                    self.reflections.label().to_lowercase(),
                    if self.rain_shadows { "on" } else { "off" }
                ),
            ),
            ("commit", self.commit.to_string()),
        ];
        let mut out = String::new();
        for (key, value) in lines {
            out.push_str(&format!("{key}: {value}\n"));
        }
        match &self.blocks {
            Some(blocks) => out.push_str(&blocks.table()),
            None => out.push_str("blocks: none resident here\n"),
        }
        out.push_str(&format!("screenshot: {screenshot}\n"));
        out
    }

    /// The record of the world as it is now, from the active 3D camera.
    /// `None` before a camera draws over the planet.
    pub fn gather(world: &mut World) -> Option<Self> {
        let frame = *world.get_resource::<crate::planet::PlanetRenderFrame>()?;
        let eye_transform = {
            let mut cameras = world.query_filtered::<(&Camera, &GlobalTransform), With<Camera3d>>();
            *cameras.iter(world).find(|(c, _)| c.is_active)?.1
        };
        let eye = (eye_transform.translation().as_dvec3() - frame.center).as_vec3();
        let up = eye.normalize_or(Vec3::Y);
        let forward = eye_transform.forward().as_vec3();
        let place = Place::of(eye);
        let on_foot = world
            .get_resource::<crate::walking::WalkingReadout>()
            .is_some_and(|w| w.active);
        let aboard = world
            .get_resource::<crate::vehicles::Aboard>()
            .and_then(|a| a.0)
            .and_then(|e| world.get::<crate::vehicles::Vehicle>(e))
            .map(|v| v.craft.kind.key());
        let water = if on_foot {
            crate::walking::walker_water(world).unwrap_or_default()
        } else {
            crate::walking::WaterState::default()
        };
        let wade = world
            .get_resource::<crate::walking::WalkingConfig>()
            .map_or(0.0, |c| c.water_wade_depth);
        let mode = match (aboard, on_foot) {
            (Some(kind), _) => Mode::Aboard(kind),
            (None, true) if water.body && water.depth > wade => Mode::Swimming,
            (None, true) => Mode::Walking,
            (None, false) => Mode::Flying,
        };
        let walker_feet = crate::walking::walker_feet(world);
        let feet = match (mode, walker_feet) {
            (Mode::Walking | Mode::Swimming, Some(feet)) => Some(feet),
            _ => None,
        };
        let eye_height_m = match feet {
            Some(feet) => eye.length() - feet.length(),
            None => eye.length() - crate::planet::terrain_radius(up),
        };
        let clock = world
            .get_resource::<crate::sky::Sun>()
            .map(|s| s.clock)
            .unwrap_or_default();
        let weather = world
            .get_resource::<crate::weather::Weather>()
            .copied()
            .unwrap_or_default();
        let wind = world
            .get_resource::<crate::atmosphere::Air>()
            .map_or(Vec3::ZERO, |air| air.now.sample(up).wind);
        let settings = world
            .get_resource::<crate::settings::PlayerSettings>()
            .copied()
            .unwrap_or_default();
        let save = world.get_resource::<crate::saves::WorldSave>();
        let (name, seed, generator, edits) = match save {
            Some(save) => (
                save.slot().map(|s| s.file.name.clone()),
                save.identity.seed,
                save.identity.generator,
                save.edits.len(),
            ),
            None => (
                None,
                pbd_core::planet_gen::TerrainConfig::TENEBRIS.seed,
                crate::planet::generator_version(),
                0,
            ),
        };
        let feet = feet.unwrap_or(eye - up * crate::walking::EYE_HEIGHT);
        let blocks = blocks_here(world, feet, eye, forward);
        Some(Self {
            world: name,
            seed,
            generator,
            newest_generator: pbd_core::terrain::GENERATOR_VERSION,
            edits,
            place,
            eye_height_m,
            yaw_deg: place::yaw_of(up, forward).to_degrees(),
            pitch_deg: place::pitch_of(up, forward).to_degrees(),
            mode,
            in_water: water.body,
            water_depth_m: water.depth,
            eyes_under: water.eyes,
            hour: clock.hour(),
            day: clock.day(),
            season: clock.season(),
            rain: weather.rain,
            snowing: weather.snowing,
            wetness: weather.wetness,
            cover: weather.cover,
            sheltered: weather.sheltered,
            wind_mps: wind.length(),
            wind_toward_deg: if wind.length_squared() > 1e-8 {
                geo::compass_heading(up, wind)
                    .to_degrees()
                    .rem_euclid(360.0)
            } else {
                0.0
            },
            render_scale: settings.render_scale,
            reflections: settings.reflections,
            rain_shadows: settings.rain_shadows,
            commit: COMMIT,
            blocks,
        })
    }
}

impl Record {
    /// A known record, for the tests here and the desktop's round trip.
    pub fn example() -> Self {
        let direction = geo::direction(geo::LatLon {
            lat: 28.25f32.to_radians(),
            lon: 0.75f32.to_radians(),
        });
        let ground = crate::planet::PLANET_RADIUS + crate::planet::surface_height(direction);
        Self {
            world: Some("Caves".into()),
            seed: 7,
            generator: 9,
            newest_generator: 10,
            edits: 12,
            place: Place::of(direction * (ground + 1.6)),
            eye_height_m: 1.6,
            yaw_deg: 132.4,
            pitch_deg: -8.0,
            mode: Mode::Walking,
            in_water: false,
            water_depth_m: 0.0,
            eyes_under: false,
            hour: 14.5,
            day: 3,
            season: "southern summer",
            rain: 0.42,
            snowing: false,
            wetness: 0.3,
            cover: 0.8,
            sheltered: false,
            wind_mps: 6.2,
            wind_toward_deg: 135.0,
            render_scale: 85,
            reflections: Reflections::Medium,
            rain_shadows: false,
            commit: "abc123",
            blocks: None,
        }
    }
}

/// The block dump round the player, from the resident fine set; `None`
/// where there is none.
fn blocks_here(world: &World, feet: Vec3, eye: Vec3, forward: Vec3) -> Option<blocks::BlockDump> {
    let fine = world.get_resource::<crate::planet::PlanetFine>()?;
    let contact = world
        .get_resource::<crate::planet::PlanetContact>()
        .filter(|c| c.serves(&fine.set));
    let save = world.get_resource::<crate::saves::WorldSave>()?;
    let axe_logs = world
        .get_resource::<crate::config::GatherConfig>()
        .map_or(4, |g| g.0.axe_logs);
    blocks::dump(
        &fine.set,
        contact,
        &save.edits,
        &save.records,
        axe_logs,
        feet,
        eye,
        forward,
    )
}

/// Where a record's text goes besides its file: the clipboard, or a fake in
/// a test.
pub trait Clipboard {
    fn copy(&mut self, text: &str);
}

/// The platform's own clipboard command (`pbcopy`, `clip`, `wl-copy` or
/// `xclip`), run on a thread of its own so a slow one never holds a frame.
/// No crate: the commands are on every machine this is played on. A failure
/// is a warning; the files are written regardless.
pub struct SystemClipboard;

impl Clipboard for SystemClipboard {
    fn copy(&mut self, text: &str) {
        use std::io::Write;
        use std::process::{Command, Stdio};
        let tries: &[(&str, &[&str])] = if cfg!(target_os = "macos") {
            &[("pbcopy", &[])]
        } else if cfg!(windows) {
            &[("clip", &[])]
        } else {
            &[("wl-copy", &[]), ("xclip", &["-selection", "clipboard"])]
        };
        let text = text.to_string();
        std::thread::spawn(move || {
            for (program, args) in tries {
                let Ok(mut child) = Command::new(program)
                    .args(*args)
                    .stdin(Stdio::piped())
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .spawn()
                else {
                    continue;
                };
                let written = child
                    .stdin
                    .take()
                    .is_some_and(|mut stdin| stdin.write_all(text.as_bytes()).is_ok());
                if child.wait().is_ok_and(|s| s.success()) && written {
                    return;
                }
            }
            warn!("repro: the record was not copied (no clipboard command worked)");
        });
    }
}

/// A date and time from Unix seconds, UTC: `(year, month, day, hour,
/// minute, second)`. Howard Hinnant's days-to-civil, so no date crate.
pub fn utc(unix_s: u64) -> (i64, u32, u32, u32, u32, u32) {
    let days = (unix_s / 86_400) as i64;
    let rest = unix_s % 86_400;
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let month = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let year = yoe + era * 400 + i64::from(month <= 2);
    (
        year,
        month,
        day,
        (rest / 3600) as u32,
        (rest / 60 % 60) as u32,
        (rest % 60) as u32,
    )
}

/// The record's text, handed to the clipboard as it is: what the desktop
/// writes to the `.txt` file.
pub fn write_record(
    record: &Record,
    clipboard: &mut impl Clipboard,
    saved: &str,
    screenshot: &str,
    capture: &str,
) -> String {
    let text = record.text(saved, screenshot, capture);
    clipboard.copy(&text);
    text
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    #[test]
    fn a_record_says_everything() {
        let text = Record::example().text(
            "2026-11-16 14:22:33 UTC",
            "repro/a.png",
            "repro/a-again.png",
        );
        assert!(text.starts_with("command: "), "the command first:\n{text}");
        assert!(
            text.trim_end().ends_with("screenshot: repro/a.png"),
            "the screenshot last:\n{text}"
        );
        for key in [
            "saved:",
            "world: Caves",
            "seed: 7",
            "generator: 9",
            "edits: 12",
            "mode: walking",
            "at: 28.25000 0.75000",
            "map reads 28.250S 0.750E",
            "elevation_m:",
            "above_ground_m:",
            "eye_height_m: 1.60",
            "yaw_deg: 132.40",
            "pitch_deg: -8.00",
            "water: dry",
            "clock: hour 14.5000, day 3, southern summer",
            "weather: rain 0.42, wetness 0.30, cover 0.80, wind 6.2 m/s toward 135 deg",
            "weather_flag: --rain 0.42",
            "settings: render scale 85, reflections medium, rain shadow off",
            "commit: abc123",
            "blocks:",
        ] {
            assert!(text.contains(key), "no '{key}' in\n{text}");
        }
        let command = text.lines().next().unwrap();
        for flag in [
            "--walk",
            "--at 28.25000 0.75000",
            "--yaw 132.40",
            "--pitch -8.00",
            "--time 14.5000",
            "--day 3",
            "--rain 0.42",
            "--generator 9",
            "--render-scale 85",
            "--reflections medium",
            "--rain-shadows off",
            "--capture repro/a-again.png",
        ] {
            assert!(command.contains(flag), "no '{flag}' in {command}");
        }
        assert!(!command.contains("--world"), "a repro never opens a save");
    }

    #[test]
    fn a_clock_a_hair_before_midnight_goes_onto_the_next_day() {
        let mut record = Record::example();
        record.hour = 23.999_99;
        assert_eq!(record.clock_flags(), (0.0, 4));
        record.hour = 6.25;
        assert_eq!(record.clock_flags(), (6.25, 3));
    }

    #[test]
    fn utc_reads_a_known_day() {
        // 2026-11-16 14:22:33 UTC.
        assert_eq!(utc(1_794_838_953), (2026, 11, 16, 14, 22, 33));
        assert_eq!(utc(0), (1970, 1, 1, 0, 0, 0));
    }

    /// The dump is what the game's own lookups read: the same cells as a
    /// walk of the neighbour table out to three rings, and in each the
    /// tier's column's material at every layer, an edit flagged as one.
    #[test]
    fn the_block_dump_reads_what_the_tier_holds() {
        use crate::planet::PlanetContact;
        use pbd_core::edits::{Edit, Edits};
        use pbd_core::terrain::Material;
        use std::sync::Arc;
        let spawn = crate::flight_view::FlightViewConfig::default().spawn_direction;
        let mut contact = PlanetContact::test_planet(5);
        let anchor = contact.find_land_near(spawn);
        let first = crate::planet::lod::generate_fine(
            anchor,
            &crate::config::ColumnSettings::default(),
            &Edits::new(),
        );
        // The player's cell, its surface, and a dig one ring out.
        let here = first
            .finest_records()
            .iter()
            .position(|r| Vec3::from_slice(&r.direction_height[..3]).dot(anchor) > 0.999_999)
            .or_else(|| {
                (0..first.finest_records().len()).max_by(|a, b| {
                    let d = |i: usize| {
                        Vec3::from_slice(&first.finest_records()[i].direction_height[..3])
                            .dot(anchor)
                    };
                    d(*a).total_cmp(&d(*b))
                })
            })
            .unwrap();
        let top = first.columns.column(here).unwrap().surface().unwrap();
        let next = first.finest_neighbors[here][0] as usize;
        let next_key = first.finest_records()[next].key();
        let next_top = first.columns.column(next).unwrap().surface().unwrap();
        let mut edits = Edits::new();
        edits.set(Edit {
            cell: next_key,
            layer: next_top as u16,
            material: Material::Air,
        });
        let set = Arc::new(crate::planet::lod::generate_fine(
            anchor,
            &crate::config::ColumnSettings::default(),
            &edits,
        ));
        contact.set_fine(&set);
        let direction = Vec3::from_slice(&set.finest_records()[here].direction_height[..3]);
        let feet = direction
            * (crate::planet::PLANET_RADIUS + pbd_core::column::layer_altitude(top) + 1.0);
        let eye = feet + direction * crate::walking::EYE_HEIGHT;
        let forward = Vec3::Y.cross(direction).normalize();
        let dump = blocks::dump(
            &set,
            Some(&contact),
            &edits,
            &pbd_core::records::Records::new(),
            4,
            feet,
            eye,
            forward,
        )
        .expect("a dump on the tier");
        let rings = blocks::rings_round(&set, here, blocks::RINGS);
        assert_eq!(dump.cells.len(), rings.len());
        assert!(dump.cells.len() >= 30, "three rings: {}", dump.cells.len());
        assert_eq!(dump.player_cell, set.finest_records()[here].key());
        assert_eq!(dump.feet_layer, top + 1);
        assert_eq!(dump.from_layer, top + 1 - blocks::LAYERS_PAST);
        assert_eq!(dump.to_layer, dump.head_layer + blocks::LAYERS_PAST);
        for (cell, (record, ring)) in dump.cells.iter().zip(&rings) {
            assert_eq!(cell.key, set.finest_records()[*record].key());
            assert_eq!(cell.ring, *ring);
            let column = set
                .columns
                .column(*record)
                .expect("resident near the anchor");
            assert_eq!(cell.layers.len(), dump.to_layer - dump.from_layer + 1);
            for layer in &cell.layers {
                assert_eq!(
                    layer.material,
                    format!("{:?}", column.material(layer.layer)),
                    "cell {} layer {}",
                    cell.key,
                    layer.layer
                );
                let edited = cell.key == next_key && layer.layer == next_top;
                assert_eq!(
                    layer.edit, edited,
                    "cell {} layer {}",
                    cell.key, layer.layer
                );
            }
        }
        let dug = dump.cells.iter().find(|c| c.key == next_key).unwrap();
        let hole = dug.layers.iter().find(|l| l.layer == next_top).unwrap();
        assert_eq!(hole.material, "Air", "the dig is in the dump");
        // The JSON carries the same.
        let json = serde_json::to_string(&dump).unwrap();
        assert!(json.contains(&format!("\"key\":{next_key}")));
    }

    /// What the clipboard is handed is the record file's text, byte for byte.
    #[test]
    fn the_clipboard_gets_the_record_as_written() {
        struct Fake(Vec<String>);
        impl Clipboard for Fake {
            fn copy(&mut self, text: &str) {
                self.0.push(text.to_string());
            }
        }
        let mut fake = Fake(Vec::new());
        let written = write_record(
            &Record::example(),
            &mut fake,
            "2026-11-16 14:22:33 UTC",
            "repro/a.png",
            "repro/a-again.png",
        );
        assert_eq!(fake.0, vec![written]);
    }
}
