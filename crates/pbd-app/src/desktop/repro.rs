//! The repro key (`repro-capture`): Shift+Alt+F saves where the player is,
//! as a screenshot, a text record and a JSON block dump in `repro/`, and
//! puts the record on the clipboard.
//!
//! The owner, 2026-11-16: "add a button for capturing location data so I can
//! help repro stuff (shift alt F or something)", and "It should auto save to
//! my clipboard".
//!
//! The chord is read off the PHYSICAL keyboard (the `KeyboardInput`
//! messages), because a rebind rewrites `ButtonInput` (`controls::remap_keys`),
//! and on the frame it fires F's press is taken out of `ButtonInput`, so the
//! same press never interacts. It is read with a panel open too; not while
//! the saves page's name field has the keyboard, where it is typing.

use bevy::input::ButtonState;
use bevy::input::keyboard::{KeyboardFocusLost, KeyboardInput};
use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, ScreenshotCaptured};
use pbd_app::repro::{self, Record, SystemClipboard};
use std::path::{Path, PathBuf};

use super::farm::FarmNote;
use super::menu::NameField;

/// Where the records go: beside `saves/`, not in it (every directory there
/// is listed as a world).
pub const DIR: &str = "repro";

/// How long the on-screen line stays, seconds.
const NOTE_S: f32 = 2.0;

/// The chord was pressed this frame: the exclusive system below saves.
#[derive(Resource, Default)]
pub struct Asked(bool);

/// The modifiers held on the physical keyboard.
#[derive(Default)]
pub struct Held {
    shift: bool,
    alt: bool,
}

/// Read the chord, and take its F away from every F reader.
pub fn read_key(
    mut typed: MessageReader<KeyboardInput>,
    mut lost: MessageReader<KeyboardFocusLost>,
    keys: Option<ResMut<ButtonInput<KeyCode>>>,
    name: Option<Res<NameField>>,
    mut held: Local<Held>,
    mut asked: ResMut<Asked>,
) {
    if lost.read().count() > 0 {
        *held = Held::default();
    }
    let mut chord = false;
    for event in typed.read() {
        let down = event.state == ButtonState::Pressed;
        match event.key_code {
            KeyCode::ShiftLeft | KeyCode::ShiftRight => held.shift = down,
            KeyCode::AltLeft | KeyCode::AltRight => held.alt = down,
            KeyCode::KeyF if down && !event.repeat && held.shift && held.alt => chord = true,
            _ => {}
        }
    }
    if !chord || name.is_some_and(|n| n.focused) {
        return;
    }
    if let Some(mut keys) = keys {
        keys.clear_just_pressed(KeyCode::KeyF);
        keys.release(KeyCode::KeyF);
        keys.clear_just_released(KeyCode::KeyF);
    }
    asked.0 = true;
}

/// The file stem for now: UTC date and time, and `-2`, `-3`... for a second
/// press in the same second.
fn stem(dir: &Path, unix_s: u64) -> String {
    let (y, mo, d, h, mi, s) = repro::utc(unix_s);
    let base = format!("{y:04}-{mo:02}-{d:02}-{h:02}{mi:02}{s:02}");
    let mut stem = base.clone();
    let mut n = 1;
    while dir.join(format!("{stem}.txt")).exists() || dir.join(format!("{stem}.png")).exists() {
        n += 1;
        stem = format!("{base}-{n}");
    }
    stem
}

/// Save the record the chord asked for: the record is taken now, the files
/// are written when the screenshot of this frame arrives.
pub fn save(world: &mut World) {
    if !std::mem::take(&mut world.resource_mut::<Asked>().0) {
        return;
    }
    let started = std::time::Instant::now();
    let Some(record) = Record::gather(world) else {
        warn!("REPRO: no camera over the planet yet; nothing saved");
        return;
    };
    let gathered_ms = started.elapsed().as_secs_f64() * 1e3;
    let dir = PathBuf::from(DIR);
    if let Err(error) = std::fs::create_dir_all(&dir) {
        warn!("REPRO: could not make {}: {error}", dir.display());
        note(world, "Location not saved");
        return;
    }
    let unix_s = pbd_app::saves::now_unix_s();
    let stem = stem(&dir, unix_s);
    let (y, mo, d, h, mi, s) = repro::utc(unix_s);
    let saved = format!("{y:04}-{mo:02}-{d:02} {h:02}:{mi:02}:{s:02} UTC");
    let png = dir.join(format!("{stem}.png"));
    let txt = dir.join(format!("{stem}.txt"));
    let json = dir.join(format!("{stem}.blocks.json"));
    let again = format!("{DIR}/{stem}-again.png");
    info!(
        "REPRO gathered in {gathered_ms:.2} ms ({} cells)",
        record.blocks.as_ref().map_or(0, |b| b.cells.len())
    );
    world.spawn(Screenshot::primary_window()).observe(
        move |event: On<ScreenshotCaptured>, time: Res<Time>, mut farm_note: ResMut<FarmNote>| {
            let shot = event
                .image
                .clone()
                .try_into_dynamic()
                .map_err(|error| format!("{error:?}"))
                .and_then(|image| image.to_rgb8().save(&png).map_err(|e| e.to_string()));
            if let Err(error) = &shot {
                warn!("REPRO: the screenshot was not saved: {error}");
            }
            let text = repro::write_record(
                &record,
                &mut SystemClipboard,
                &saved,
                &png.display().to_string(),
                &again,
            );
            let wrote = std::fs::write(&txt, &text);
            if let Some(blocks) = &record.blocks {
                match serde_json::to_string_pretty(blocks) {
                    Ok(body) => {
                        if let Err(error) = std::fs::write(&json, body) {
                            warn!("REPRO: {} not written: {error}", json.display());
                        }
                    }
                    Err(error) => warn!("REPRO: the block dump did not serialise: {error}"),
                }
            }
            match wrote {
                Ok(()) => {
                    info!("REPRO {} {}", txt.display(), record.command(&again));
                    farm_note.say_for(time.elapsed_secs(), "Location copied", NOTE_S);
                }
                Err(error) => {
                    warn!("REPRO: {} not written: {error}", txt.display());
                    farm_note.say_for(time.elapsed_secs(), "Location not saved", NOTE_S);
                }
            }
        },
    );
}

fn note(world: &mut World, text: &str) {
    let now = world.resource::<Time>().elapsed_secs();
    world.resource_mut::<FarmNote>().say_for(now, text, NOTE_S);
}

/// `PBD_PRESS_REPRO=<frame>`: press Shift+Alt+F on that frame through the
/// keyboard's own messages, as a hand would. A capture instrument: a
/// headless run has no hand on the keys.
pub fn scripted_press(
    state: Res<super::CaptureState>,
    windows: Query<Entity, With<bevy::window::PrimaryWindow>>,
    mut keys: MessageWriter<KeyboardInput>,
    mut at: Local<Option<Option<u32>>>,
) {
    let Some(frame) = *at.get_or_insert_with(|| {
        std::env::var("PBD_PRESS_REPRO")
            .ok()
            .and_then(|f| f.trim().parse().ok())
    }) else {
        return;
    };
    let window = windows.iter().next().unwrap_or(Entity::PLACEHOLDER);
    let send = |keys: &mut MessageWriter<KeyboardInput>, key: KeyCode, down: bool| {
        keys.write(KeyboardInput {
            key_code: key,
            logical_key: bevy::input::keyboard::Key::Unidentified(
                bevy::input::keyboard::NativeKey::Unidentified,
            ),
            state: if down {
                ButtonState::Pressed
            } else {
                ButtonState::Released
            },
            text: None,
            repeat: false,
            window,
        });
    };
    if state.frame == frame {
        info!("PBD_PRESS_REPRO: Shift+Alt+F at frame {frame}");
        for key in [KeyCode::ShiftLeft, KeyCode::AltLeft, KeyCode::KeyF] {
            send(&mut keys, key, true);
        }
    } else if state.frame == frame + 2 {
        for key in [KeyCode::KeyF, KeyCode::AltLeft, KeyCode::ShiftLeft] {
            send(&mut keys, key, false);
        }
    }
}

/// MEASUREMENT (`loading-screen` 5.3c): `PBD_KEY_SCRIPT=<frame>:<key>,...`
/// presses each key (Bevy's name, `KeyM`, `Escape`) on that capture frame
/// and lets it go two frames later, through the keyboard's own messages, so
/// a real-time run can open the pack, the map and the menu after `READY` and
/// its log show whether doing so compiles a pipeline late. Nothing reads it
/// in the game.
pub fn scripted_keys(
    state: Res<super::CaptureState>,
    windows: Query<Entity, With<bevy::window::PrimaryWindow>>,
    mut keys: MessageWriter<KeyboardInput>,
    script: Local<std::sync::OnceLock<Vec<(u32, KeyCode, Option<char>)>>>,
) {
    let script = script.get_or_init(|| {
        std::env::var("PBD_KEY_SCRIPT")
            .unwrap_or_default()
            .split(',')
            .filter_map(|step| {
                let (frame, key) = step.trim().split_once(':')?;
                let code = match key.trim() {
                    "Escape" => KeyCode::Escape,
                    name => pbd_app::controls::key_named(name)?,
                };
                // A letter key (`KeyH`) is typed as itself, so a script can
                // type into a text box (the chat's shot, `multiplayer`
                // design 31).
                let letter = match key.trim().as_bytes() {
                    [b'K', b'e', b'y', c] if c.is_ascii_alphabetic() => {
                        Some(c.to_ascii_lowercase() as char)
                    }
                    _ => None,
                };
                Some((frame.trim().parse().ok()?, code, letter))
            })
            .collect()
    });
    let window = windows.iter().next().unwrap_or(Entity::PLACEHOLDER);
    for &(frame, key, letter) in script {
        let down = state.frame == frame;
        if !down && state.frame != frame + 2 {
            continue;
        }
        if down {
            info!("PBD_KEY_SCRIPT: {key:?} at frame {frame}");
        }
        keys.write(KeyboardInput {
            key_code: key,
            logical_key: match letter {
                Some(l) => bevy::input::keyboard::Key::Character(l.to_string().into()),
                None if key == KeyCode::Space => bevy::input::keyboard::Key::Space,
                None => bevy::input::keyboard::Key::Unidentified(
                    bevy::input::keyboard::NativeKey::Unidentified,
                ),
            },
            state: if down {
                ButtonState::Pressed
            } else {
                ButtonState::Released
            },
            text: None,
            repeat: false,
            window,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pbd_core::geo;

    /// The record's command line, parsed by the game's own argument parser,
    /// is the same place, look and clock, on foot and in the air; and the
    /// heading a start takes from the parsed yaw is the way the record faced.
    #[test]
    fn a_record_command_parses_back_to_the_same_view() {
        for mode in [repro::Mode::Walking, repro::Mode::Flying] {
            let mut record = Record::example();
            record.mode = mode;
            record.eye_height_m = 37.5;
            let args = record.args("repro/x-again.png");
            let launch = super::super::Launch::parse(&args);
            let (lat, lon) = launch.at.expect("--at");
            assert!((lat - record.place.lat).abs() < 1e-4, "{lat}");
            assert!((lon - record.place.lon).abs() < 1e-4, "{lon}");
            assert!((launch.yaw.unwrap() - record.yaw_deg).abs() < 0.006);
            assert!((launch.pitch.unwrap() - record.pitch_deg).abs() < 0.006);
            assert!((launch.time.unwrap() - record.hour).abs() < 1e-4);
            assert_eq!(u64::from(launch.day.unwrap()), record.day);
            assert_eq!(launch.walk, mode == repro::Mode::Walking);
            assert_eq!(launch.fly, mode == repro::Mode::Flying);
            if mode == repro::Mode::Flying {
                assert!((launch.height.unwrap() - 37.5).abs() < 0.006);
            }
            assert!(launch.world.is_none(), "a repro opens no save");
            assert_eq!(launch.generator, Some(record.generator));
            assert_eq!(launch.settings.render_scale, Some(record.render_scale));
            assert_eq!(launch.settings.reflections, Some(record.reflections));
            assert_eq!(launch.settings.rain_shadows, Some(record.rain_shadows));
            assert_eq!(launch.settings.antialiasing, Some(record.antialiasing));
            assert_eq!(
                launch.capture.as_deref(),
                Some(Path::new("repro/x-again.png"))
            );
            let up = geo::direction(geo::LatLon {
                lat: lat.to_radians(),
                lon: lon.to_radians(),
            });
            let heading = pbd_app::place::heading_at_yaw(up, launch.yaw.unwrap().to_radians());
            let back = pbd_app::place::yaw_of(up, heading).to_degrees();
            assert!((back - record.yaw_deg).abs() < 0.01, "{back}");
        }
    }

    #[test]
    fn a_second_press_in_the_same_second_gets_its_own_name() {
        let dir = std::env::temp_dir().join(format!("pbd-repro-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let first = stem(&dir, 1_794_838_953);
        assert_eq!(first, "2026-11-16-142233");
        std::fs::write(dir.join(format!("{first}.txt")), "").unwrap();
        assert_eq!(stem(&dir, 1_794_838_953), "2026-11-16-142233-2");
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
