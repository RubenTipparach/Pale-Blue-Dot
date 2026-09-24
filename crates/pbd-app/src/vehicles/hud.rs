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
            ..default()
        },
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
                if t.stall > 0.3 { "  STALL" } else { "" },
                if t.wet_loss > 0.02 { "  WET WING" } else { "" },
            );
            let _ = writeln!(s, "assist {}  brake {}", on(st.assist), on(st.brake));
        }
        (Telemetry::Tern(t), CraftState::Tern(_)) => {
            let sail = match t.sail_state {
                SailState::Luffing => "luffing",
                SailState::Driving => "drawing",
                SailState::Stalled => "stalled",
            };
            let _ = writeln!(
                s,
                "speed {:>4.1} m/s  hull {:>4.1}  wind {:>4.1} m/s at {:>4.0} deg  heel {:>+4.0} deg",
                t.speed,
                t.hull_speed,
                t.true_wind.length(),
                t.true_angle.to_degrees().abs(),
                t.heel.to_degrees()
            );
            let _ = writeln!(
                s,
                "sheet {:>3.0}%  sail {sail}  leeway {:>+4.1} deg  water aboard {:>4.0} kg",
                t.sheet * 100.0,
                t.leeway.to_degrees(),
                craft.bilge_kg
            );
        }
        (Telemetry::Loon(t), CraftState::Loon(_)) => {
            let _ = writeln!(
                s,
                "speed {:>4.1} m/s  drift {:>+4.1} m/s  strokes {:>3.0}/min  freeboard {:>4.2} m",
                t.speed, t.drift, t.strokes_per_minute, t.freeboard
            );
            let _ = writeln!(
                s,
                "heel {:>+4.0} deg  water aboard {:>4.0} kg",
                t.heel.to_degrees(),
                craft.bilge_kg
            );
        }
        _ => {}
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
