//! The instruments aboard, and the prompt that says a craft is in reach. Both
//! are read off the craft's own telemetry: the HUD computes nothing.
//!
//! The on-foot screen is kept bare on the owner's instruction, so this shows
//! only while it is a control: the prompt while a craft can be boarded, the
//! panel while one is being driven. Text is plain ASCII, as every string a
//! player reads is.

use super::{Aboard, Vehicle, view::VehicleView};
use crate::walking::{Walker, WalkingState};
use avian3d::prelude::Position;
use bevy::prelude::*;
use pbd_core::vehicle::{CraftState, SailState, Telemetry};
use std::fmt::Write;

#[derive(Component)]
pub struct VehicleText;

pub fn spawn(mut commands: Commands) {
    commands.spawn((
        Text::new(""),
        TextFont {
            font_size: 13.0,
            ..default()
        },
        TextColor(Color::srgb(0.92, 0.97, 0.95)),
        TextShadow::default(),
        Node {
            position_type: PositionType::Absolute,
            left: px(10),
            bottom: px(64),
            padding: UiRect::all(px(9)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.02, 0.035, 0.05, 0.88)),
        Visibility::Hidden,
        VehicleText,
    ));
}

pub fn show(
    aboard: Res<Aboard>,
    view: Res<VehicleView>,
    walking: Option<Res<WalkingState>>,
    frame: Res<crate::planet::PlanetRenderFrame>,
    vehicles: Query<&Vehicle>,
    walkers: Query<&Position, With<Walker>>,
    mut text: Query<(&mut Text, &mut Visibility), With<VehicleText>>,
) {
    let Ok((mut text, mut visibility)) = text.single_mut() else {
        return;
    };
    let line = match aboard.0.and_then(|e| vehicles.get(e).ok()) {
        Some(vehicle) => panel(vehicle, view.seat),
        None => walking
            .filter(|w| w.active)
            .and_then(|_| walkers.single().ok())
            .and_then(|walker| {
                let at = walker.0.as_dvec3() - frame.center;
                vehicles
                    .iter()
                    .map(|v| (v, v.craft.body.position.distance(at)))
                    .filter(|(v, d)| *d <= v.craft.seat().reach_m as f64)
                    .min_by(|a, b| a.1.total_cmp(&b.1))
                    .map(|(v, _)| format!("[G] board the {}", v.craft.kind.name()))
            })
            .unwrap_or_default(),
    };
    *visibility = if line.is_empty() {
        Visibility::Hidden
    } else {
        Visibility::Inherited
    };
    if text.0 != line {
        text.0 = line;
    }
}

/// The instruments and the keys for the craft aboard.
fn panel(vehicle: &Vehicle, seat: bool) -> String {
    let craft = &vehicle.craft;
    let mut s = String::new();
    let moored = match craft.mooring {
        Some(m) if m.anchored => "  ANCHORED",
        Some(_) => "  MOORED",
        None => "",
    };
    let _ = writeln!(s, "{}{moored}", craft.kind.name().to_uppercase());
    match (&craft.telemetry, &craft.state) {
        (Telemetry::Kestrel(t), CraftState::Kestrel(st)) => {
            let _ = writeln!(
                s,
                "air {:>5.1} m/s  ground {:>5.1} m/s  climb {:>+5.1} m/s  height {:>6.1} m",
                t.airspeed, t.ground_speed, t.vertical_speed, t.height
            );
            let _ = writeln!(
                s,
                "nacelles {:>3.0} deg  power {:>3.0}%  wing {:>3.0}%  aoa {:>+5.1} deg{}{}",
                t.nacelle.to_degrees(),
                t.throttle * 100.0,
                t.wing_share * 100.0,
                t.alpha.to_degrees(),
                if t.stall > 0.3 { "  WING STALL" } else { "" },
                if t.wet_loss > 0.02 { "  WET WING" } else { "" },
            );
            let _ = writeln!(
                s,
                "assist {}  brake {}  collective: {}",
                on(st.assist),
                on(st.brake),
                if t.climb_hold {
                    "CLIMB RATE"
                } else {
                    "POWER LEVER"
                }
            );
        }
        (Telemetry::Tern(t), CraftState::Tern(_)) => {
            let sail = match t.sail_state {
                SailState::Luffing => "luffing",
                SailState::Driving => "drawing",
                SailState::Stalled => "stalled",
            };
            let _ = writeln!(
                s,
                "water {:>4.1} m/s  ground {:>4.1} m/s  hull speed {:>3.0}%  heel {:>+4.0} deg",
                t.water_speed,
                t.speed,
                t.hull_speed * 100.0,
                t.heel.to_degrees()
            );
            let _ = writeln!(
                s,
                "apparent wind {:>4.1} m/s  {} {:>3.0} deg  windward VMG (ground) {:>+4.1} m/s",
                t.apparent.length(),
                side(t.apparent_angle),
                t.apparent_angle.to_degrees().abs(),
                t.upwind
            );
            let _ = writeln!(
                s,
                "sheet {:>3.0}%  sail {sail}  leeway {:>+4.1} deg  water aboard {:>4.0} kg",
                t.sheet * 100.0,
                t.leeway.to_degrees(),
                craft.bilge_kg
            );
            let hint = match t.sail_state {
                SailState::Luffing => "Bear away from headwind or trim the sheet in.",
                SailState::Stalled => "Ease the sheet to restore attached flow.",
                SailState::Driving => "Trim for windward VMG; hike against the heel.",
            };
            let _ = writeln!(s, "{hint}");
        }
        (Telemetry::Loon(t), CraftState::Loon(_)) => {
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
            } else {
                "paddle resting"
            };
            let _ = writeln!(
                s,
                "{phase} - alternate strokes to make way; rudder steers while moving."
            );
        }
        _ => {}
    }
    if craft.mooring.is_some() {
        let _ = writeln!(
            s,
            "{} to get underway.",
            if craft.mooring.is_some_and(|m| m.anchored) {
                "Weigh anchor"
            } else {
                "Cast off"
            }
        );
    }
    // The keys come off the one bindings table, never a copy of it here.
    let _ = writeln!(
        s,
        "{}",
        crate::controls::line(&craft.kind.name().to_uppercase())
    );
    let _ = write!(
        s,
        "{}   ({} view)",
        crate::controls::line("VEHICLES"),
        if seat { "seat" } else { "chase" }
    );
    s
}

fn on(flag: bool) -> &'static str {
    if flag { "on" } else { "off" }
}

fn side(angle: f64) -> &'static str {
    if angle.abs() < 0.5_f64.to_radians() {
        "AHEAD"
    } else if angle > 0.0 {
        "STBD"
    } else {
        "PORT"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pbd_core::vehicle::{
        Craft, Hulls, KestrelTelemetry, Kind, LoonTelemetry, Mooring, TernTelemetry,
        spec::VehicleSpecs,
    };
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

    #[test]
    fn sailing_panel_labels_water_ground_wind_side_vmg_and_action() {
        let mut v = vehicle(Kind::Tern);
        for (angle, side) in [
            (60_f64.to_radians(), "STBD"),
            (-60_f64.to_radians(), "PORT"),
        ] {
            v.craft.telemetry = Telemetry::Tern(TernTelemetry {
                water_speed: 2.0,
                speed: 3.0,
                apparent: DVec3::X * 8.0,
                apparent_angle: angle,
                upwind: 1.2,
                hull_speed: 0.75,
                sail_state: SailState::Stalled,
                ..default()
            });
            let text = panel(&v, false);
            for value in [
                "water  2.0 m/s",
                "ground  3.0 m/s",
                "hull speed  75%",
                "apparent wind  8.0 m/s",
                side,
                "60 deg",
                "VMG (ground) +1.2 m/s",
                "Ease the sheet",
            ] {
                assert!(text.contains(value), "missing {value}: {text}");
            }
        }
        v.craft.mooring = Some(Mooring {
            at: DVec3::ZERO,
            length: 10.0,
            anchored: true,
        });
        assert!(panel(&v, false).contains("Weigh anchor to get underway"));
    }

    #[test]
    fn flight_and_paddle_panels_explain_the_active_control_state() {
        let mut v = vehicle(Kind::Kestrel);
        for (hold, label) in [(true, "CLIMB RATE"), (false, "POWER LEVER")] {
            v.craft.telemetry = Telemetry::Kestrel(KestrelTelemetry {
                climb_hold: hold,
                ..default()
            });
            assert!(panel(&v, false).contains(label));
        }
        let mut v = vehicle(Kind::Loon);
        v.craft.telemetry = Telemetry::Loon(LoonTelemetry {
            rudder_at: Some(DVec3::ZERO),
            water_speed: 1.0,
            speed: 2.0,
            ..default()
        });
        let text = panel(&v, false);
        for value in [
            "stern rudder",
            "water  1.0",
            "ground  2.0",
            "water slip",
            "freeboard",
            "water aboard",
        ] {
            assert!(text.contains(value), "missing {value}: {text}");
        }
    }
}
