//! The instruments aboard, read off the craft's own telemetry: the HUD
//! computes nothing.
//!
//! The on-foot screen is kept bare on the owner's instruction, so this shows
//! only while a craft is being driven. A craft in reach on foot is named by
//! the interact prompt (`crate::interact`), which knows whether F boards it
//! or does something nearer first. Text is plain ASCII, as every string a
//! player reads is.

use super::{Aboard, Vehicle, view::VehicleView};
use bevy::prelude::*;
use pbd_core::vehicle::{Kind, Telemetry};
use std::fmt::Write;

/// The instruments behind F3: plain text, a developer's.
#[derive(Component)]
pub struct VehicleText;

/// The boat panel: the stone panel a player reads aboard, and the column its
/// rows are rebuilt into when they change.
#[derive(Component)]
pub struct BoatPanel;
#[derive(Component)]
pub struct BoatPanelBody;

pub fn spawn(mut commands: Commands) {
    commands.spawn((
        Text::new(""),
        TextFont {
            font_size: 10.0,
            ..default()
        },
        TextColor(Color::srgb(0.92, 0.97, 0.95)),
        crate::ui_kit::text_shadow(),
        Node {
            position_type: PositionType::Absolute,
            left: px(5),
            top: px(130),
            padding: UiRect::all(px(5)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.02, 0.035, 0.05, 0.88)),
        Visibility::Hidden,
        VehicleText,
    ));
}

/// What the panel says: the title, a row per control (each key's label and
/// what it does), and a warning when there is one.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct Lines {
    pub title: String,
    pub rows: Vec<(Vec<&'static str>, String)>,
    pub warning: Option<String>,
}

impl Lines {
    /// As one string: what a test reads.
    #[cfg(test)]
    pub fn text(&self) -> String {
        let mut s = self.title.clone();
        for (keys, what) in &self.rows {
            let _ = write!(s, "\n{}  {what}", keys.join("/"));
        }
        if let Some(w) = &self.warning {
            let _ = write!(s, "\n{w}");
        }
        s
    }
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn show(
    mut commands: Commands,
    aboard: Res<Aboard>,
    view: Res<VehicleView>,
    vehicles: Query<&Vehicle>,
    mut text: Query<(&mut Text, &mut Visibility), (With<VehicleText>, Without<BoatPanel>)>,
    mut panels: Query<&mut Visibility, (With<BoatPanel>, Without<VehicleText>)>,
    bodies: Query<Entity, With<BoatPanelBody>>,
    kit: Option<Res<crate::ui_kit::UiKit>>,
    binds: Option<Res<crate::controls::KeyBinds>>,
    menu: Option<Res<crate::controls::MenuOpen>>,
    debug: Option<Res<crate::controls::DebugOverlay>>,
    mut shown: Local<Option<Lines>>,
) {
    let none = crate::controls::KeyBinds::default();
    let binds = binds.as_deref().unwrap_or(&none);
    let vehicle = aboard.0.and_then(|e| vehicles.get(e).ok());
    // A menu aboard (the pack, with a raft's hold) stands over the panel,
    // which would draw through it.
    let menu = menu.is_some_and(|m| m.0);
    let lines = vehicle.filter(|_| !menu).map(|v| panel(v, binds));
    // The instruments, behind F3.
    if let Ok((mut text, mut visibility)) = text.single_mut() {
        let line = vehicle
            .filter(|_| !menu && debug.is_some_and(|d| d.0))
            .map(|v| instruments(v, view.seat))
            .unwrap_or_default();
        let want = if line.is_empty() {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        };
        if *visibility != want {
            *visibility = want;
        }
        if text.0 != line {
            text.0 = line;
        }
    }
    let Some(kit) = kit else {
        return;
    };
    if panels.is_empty() {
        spawn_panel(&mut commands);
        *shown = None;
        return;
    }
    let want = if lines.is_some() {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
    for mut visibility in &mut panels {
        if *visibility != want {
            *visibility = want;
        }
    }
    if *shown == lines {
        return;
    }
    shown.clone_from(&lines);
    let Some(lines) = lines else {
        return;
    };
    for body in &bodies {
        commands.entity(body).despawn_related::<Children>();
        commands
            .entity(body)
            .with_children(|body| fill(body, &kit, &lines));
    }
}

/// The panel, empty and hidden, bottom left: no background of its own
/// (owner, 2026-11-16: "probably leave off the background here, and add drop
/// shadow or outline for text"), its words outlined so they read over water
/// and sand alike.
fn spawn_panel(commands: &mut Commands) {
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: px(8),
            bottom: px(40),
            flex_direction: FlexDirection::Column,
            row_gap: px(3),
            ..default()
        },
        Visibility::Hidden,
        BoatPanel,
        BoatPanelBody,
    ));
}

/// The panel's rows: the title, then each control as its keys' stone
/// glyphs and what it does, then the warning.
fn fill(body: &mut ChildSpawnerCommands, kit: &crate::ui_kit::UiKit, lines: &Lines) {
    use crate::ui_kit::{INK, KitText, LABEL, TITLE};
    body.spawn((
        KitText::new(lines.title.clone(), TITLE).outline(INK),
        Node {
            margin: UiRect::bottom(px(2)),
            ..default()
        },
    ));
    for (keys, what) in &lines.rows {
        body.spawn(Node {
            column_gap: px(2),
            align_items: AlignItems::Center,
            ..default()
        })
        .with_children(|row| {
            for key in keys {
                crate::ui_kit::key_cap(row, kit, key, ());
            }
            row.spawn((
                KitText::new(what.clone(), LABEL).outline(INK),
                Node {
                    margin: UiRect::left(px(4)),
                    padding: UiRect::top(px(2)),
                    ..default()
                },
            ));
        });
    }
    if let Some(w) = &lines.warning {
        body.spawn(KitText::new(w.clone(), TITLE).outline(INK));
    }
}

/// The key a player has on a slot (`"vehicles:KeyT"`), as its label.
fn bound(name: &str, default: KeyCode, binds: &crate::controls::KeyBinds) -> &'static str {
    let key = crate::controls::slots()
        .find(|s| s.name() == name)
        .map_or(default, |s| binds.key(&s));
    crate::controls::key_label(key)
}

/// Water aboard, kg, over which the panel says to bail.
const BAIL_KG: f64 = 20.0;

/// What a player reads aboard (owner, 2026-11-16: "please just show simple
/// raft controls, make it user friendly"): the craft's name, whether it is at
/// anchor, its keys one to a line off the one bindings table, the anchor and
/// the camera, and a word to bail only when there is water to bail. The
/// instruments are a developer's and stand behind F3 with the frame graph
/// ([`instruments`]).
fn panel(vehicle: &Vehicle, binds: &crate::controls::KeyBinds) -> Lines {
    let craft = &vehicle.craft;
    let moored = match craft.mooring {
        Some(m) if m.anchored => " - at anchor",
        Some(_) => " - tied up",
        None => "",
    };
    let mut lines = Lines {
        title: format!("{}{moored}", craft.kind.name().to_uppercase()),
        ..default()
    };
    let bail = craft.bilge_kg > BAIL_KG;
    for (keys, what) in crate::controls::caps(&craft.kind.name().to_uppercase(), binds) {
        // The bail key only while there is water to bail.
        if what == "bail" && !bail {
            continue;
        }
        // Moving on W, A, S and D goes without saying (owner, 2026-11-16:
        // "we probably dont need WASD, its obvious enough"); a row the
        // player has moved to other keys is shown.
        if keys.iter().all(|k| ["W", "A", "S", "D"].contains(k)) {
            continue;
        }
        lines.rows.push((keys, what.to_string()));
    }
    let anchor = match craft.mooring {
        Some(m) if m.anchored => "raise the anchor",
        Some(_) => "cast off",
        None => "drop the anchor",
    };
    lines.rows.push((
        vec![bound("vehicles:KeyT", KeyCode::KeyT, binds)],
        anchor.into(),
    ));
    lines.rows.push((
        vec![crate::controls::key_label(crate::interact::key(binds))],
        "step off".into(),
    ));
    lines.rows.push((
        vec![bound("vehicles:KeyV", KeyCode::KeyV, binds)],
        "camera".into(),
    ));
    if bail {
        lines.warning = Some("Water aboard: bail".into());
    }
    lines
}

/// The instruments, behind F3 (`DebugOverlay`): what the panel showed before
/// it was made simple, kept for whoever tunes a hull.
fn instruments(vehicle: &Vehicle, seat: bool) -> String {
    let craft = &vehicle.craft;
    let mut s = String::new();
    if let Telemetry::Loon(t) = &craft.telemetry {
        let _ = writeln!(
            s,
            "water {:>4.1} m/s  ground {:>4.1} m/s  water slip {:>+4.1} m/s  ground drift {:>+4.1} m/s",
            t.water_speed, t.speed, t.water_drift, t.drift
        );
        let _ = writeln!(
            s,
            "strokes {:>3.0}/min  freeboard {:>4.2} m  heel {:>+4.0} deg  water aboard {:>4.0} kg",
            t.strokes_per_minute,
            t.freeboard,
            t.heel.to_degrees(),
            craft.bilge_kg
        );
        let phase = if t.rudder_at.is_some() {
            "stern rudder"
        } else if t.stroke.is_some_and(|stroke| stroke.phase < 1.0) {
            "power stroke"
        } else if t.stroke.is_some() {
            "recovering"
        } else if craft.kind == Kind::Tube {
            "hands resting"
        } else {
            "paddle resting"
        };
        let _ = writeln!(
            s,
            "{phase} - alternate strokes to make way; rudder steers while moving."
        );
    }
    let _ = write!(s, "({} view)", if seat { "seat" } else { "chase" });
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use pbd_core::vehicle::{Craft, Hulls, Kind, LoonTelemetry, Mooring, spec::VehicleSpecs};
    use pbd_core::{DQuat, DVec3};
    use std::sync::Arc;

    fn vehicle(kind: Kind) -> Vehicle {
        let specs = Arc::new(VehicleSpecs::default());
        Vehicle::new(Craft::new(
            kind,
            1,
            specs.clone(),
            Hulls::new(&specs),
            DVec3::Y * 4800.0,
            DQuat::IDENTITY,
        ))
    }

    /// The panel is the name and plain keys; the bail word only with water
    /// aboard; the instruments only behind F3.
    #[test]
    fn the_panel_is_simple_and_the_instruments_are_behind_f3() {
        let mut v = vehicle(Kind::Raft);
        v.craft.telemetry = Telemetry::Loon(LoonTelemetry {
            water_speed: 1.0,
            speed: 2.0,
            ..default()
        });
        let text = panel(&v, &Default::default()).text();
        for want in [
            "RAFT",
            "Q/E  steer with the pole",
            "drop the anchor",
            "step off",
            "camera",
        ] {
            assert!(text.contains(want), "missing {want}: {text}");
        }
        for not in ["W/S", "A/D", "m/s", "freeboard", "slip", "bail", "deg"] {
            assert!(!text.contains(not), "{not} in the panel: {text}");
        }
        v.craft.bilge_kg = 80.0;
        v.craft.mooring = Some(Mooring {
            at: DVec3::ZERO,
            length: 10.0,
            anchored: true,
        });
        let text = panel(&v, &Default::default()).text();
        assert!(
            text.contains("at anchor") && text.contains("raise the anchor"),
            "{text}"
        );
        let mut loon = vehicle(Kind::Loon);
        assert!(!panel(&loon, &Default::default()).text().contains("bail"));
        loon.craft.bilge_kg = 80.0;
        let wet = panel(&loon, &Default::default()).text();
        assert!(
            wet.contains("Water aboard: bail") && wet.contains("B  bail"),
            "{wet}"
        );
        let deep = instruments(&v, false);
        assert!(
            deep.contains("water  1.0") && deep.contains("freeboard"),
            "{deep}"
        );
    }
}
