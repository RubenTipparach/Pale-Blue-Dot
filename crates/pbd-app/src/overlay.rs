//! The weather overlays in the app: which one is showing, and the overlay map
//! the renderer draws it from.
//!
//! What an overlay shows is `pbd_core::overlay`'s: a reading of the same
//! atmosphere state the clouds are drawn from. This module only resamples that
//! reading onto a cube map, the same way the cloud maps are made, and only
//! while an overlay is showing, so an overlay that is off costs nothing. The
//! drawing is the `overlay` pass in `water.wgsl`; the legend is the desktop's.

use crate::atmosphere::{Air, MAP_SIZE, cube_direction};
use crate::planet::weather_maps::WeatherMapsNow;
use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task, block_on, futures_lite::future};
use pbd_core::atmosphere::Atmosphere;
use pbd_core::overlay::{Overlay, Ramp};
use std::sync::Arc;

/// Which overlay the player asked for; `None` is the plain view.
#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct OverlayMode(pub Option<Overlay>);

/// The display colours of every ramp, five stops each, in `Ramp::ALL`'s
/// order: the legend's colour bar is drawn from this. The shader carries the
/// same table (`OVERLAY_RAMPS` in `water.wgsl`), because a shader cannot read
/// a Rust constant; `the_legend_and_the_shader_share_one_ramp_table` holds the
/// two together.
pub const RAMPS: [[[f32; 3]; 5]; 6] = [
    [
        [0.14, 0.20, 0.55],
        [0.10, 0.55, 0.75],
        [0.25, 0.75, 0.35],
        [0.95, 0.85, 0.25],
        [0.85, 0.20, 0.15],
    ],
    [
        [0.05, 0.10, 0.20],
        [0.25, 0.32, 0.45],
        [0.55, 0.60, 0.68],
        [0.80, 0.83, 0.88],
        [1.00, 1.00, 1.00],
    ],
    [
        [0.55, 0.25, 0.80],
        [0.80, 0.70, 0.95],
        [0.92, 0.92, 0.92],
        [0.40, 0.65, 0.95],
        [0.08, 0.20, 0.70],
    ],
    [
        [0.55, 0.40, 0.20],
        [0.80, 0.70, 0.45],
        [0.85, 0.88, 0.75],
        [0.40, 0.75, 0.70],
        [0.10, 0.45, 0.55],
    ],
    [
        [0.05, 0.03, 0.02],
        [0.45, 0.15, 0.03],
        [0.85, 0.40, 0.05],
        [0.98, 0.75, 0.25],
        [1.00, 0.97, 0.75],
    ],
    [
        [0.15, 0.25, 0.75],
        [0.45, 0.70, 0.95],
        [0.95, 0.95, 0.95],
        [0.98, 0.65, 0.30],
        [0.80, 0.12, 0.10],
    ],
];

/// A ramp's display colour at `t` in 0..1, interpolated between its stops as
/// the shader does.
pub fn ramp_colour(ramp: Ramp, t: f32) -> [f32; 3] {
    let stops = &RAMPS[ramp.index()];
    let x = t.clamp(0.0, 1.0) * (stops.len() - 1) as f32;
    let i = (x.floor() as usize).min(stops.len() - 2);
    let f = x - i as f32;
    std::array::from_fn(|c| stops[i][c] + (stops[i + 1][c] - stops[i][c]) * f)
}

/// The overlay map: one overlay's texel at every texel of the cube, faces and
/// rows as the weather maps lay them out.
pub fn overlay_texels(atmosphere: &Atmosphere, overlay: Overlay) -> Vec<[f32; 4]> {
    let mut texels = Vec::with_capacity(6 * MAP_SIZE * MAP_SIZE);
    for face in 0..6 {
        for row in 0..MAP_SIZE {
            for column in 0..MAP_SIZE {
                let direction = cube_direction(face, row, column, MAP_SIZE);
                texels.push(overlay.texel(&atmosphere.sample(direction)));
            }
        }
    }
    texels
}

/// The map being built off the main thread, and which state and overlay the
/// one on the GPU was built from.
/// A finished build: the state's generation, the overlay, and its texels.
type Built = (u64, Overlay, Vec<[f32; 4]>);

#[derive(Default)]
pub struct OverlayJob {
    task: Option<Task<Built>>,
    built: Option<(u64, Overlay)>,
}

fn show(now: &mut WeatherMapsNow, overlay: Overlay, texels: Vec<[f32; 4]>) {
    now.overlay = Some(Arc::new(texels));
    now.overlay_generation += 1;
    now.overlay_kind = Some(overlay);
}

/// Keep the overlay map in step with the weather and the mode: rebuilt when
/// the atmosphere publishes a new state or the player picks another overlay,
/// and hidden the moment the map on the GPU is no longer the one asked for, so
/// a ramp is never drawn over another overlay's numbers. A capture builds it
/// in place, so its picture is a function of its flags.
pub fn fill_overlay(
    mode: Res<OverlayMode>,
    air: Option<Res<Air>>,
    mut now: ResMut<WeatherMapsNow>,
    mut job: Local<OverlayJob>,
) {
    let Some(air) = air else {
        return;
    };
    if let Some(task) = job.task.as_mut() {
        let Some((generation, overlay, texels)) = block_on(future::poll_once(task)) else {
            return;
        };
        job.task = None;
        job.built = Some((generation, overlay));
        if mode.0 == Some(overlay) {
            show(&mut now, overlay, texels);
        }
    }
    let Some(want) = mode.0 else {
        if now.overlay_kind.is_some() {
            now.overlay_kind = None;
        }
        job.built = None;
        return;
    };
    if now.overlay_kind.is_some_and(|shown| shown != want) {
        now.overlay_kind = None;
    }
    if job.built == Some((air.generation, want)) && now.overlay_kind == Some(want) {
        return;
    }
    let generation = air.generation;
    if air.in_place {
        show(&mut now, want, overlay_texels(&air.now, want));
        job.built = Some((generation, want));
        return;
    }
    let atmosphere = air.now.clone();
    job.task = Some(
        AsyncComputeTaskPool::get()
            .spawn(async move { (generation, want, overlay_texels(&atmosphere, want)) }),
    );
}

/// M steps to the next overlay, and past the last one back to the plain view.
pub fn cycle_overlay(keys: Res<ButtonInput<KeyCode>>, mut mode: ResMut<OverlayMode>) {
    if keys.just_pressed(KeyCode::KeyM) {
        mode.0 = Overlay::next(mode.0);
        match mode.0 {
            Some(overlay) => info!("overlay: {}", overlay.name()),
            None => info!("overlay: off"),
        }
    }
}

/// The overlay's lanes of the water pass's uniform: the ramp's row plus one
/// (zero when nothing is showing), the range, the opacity; then the streak
/// step, scroll and strength, and the flags (1 flows, 2 fades toward zero).
pub fn overlay_lanes(
    shown: Option<Overlay>,
    settings: &crate::config::WeatherSettings,
) -> (Vec4, Vec4) {
    let Some(overlay) = shown else {
        return (Vec4::ZERO, Vec4::ZERO);
    };
    let (low, high) = overlay.range();
    let flags = u32::from(overlay.flows()) | (u32::from(overlay.fades()) << 1);
    (
        Vec4::new(
            overlay.ramp().index() as f32 + 1.0,
            low,
            high,
            settings.overlay_opacity,
        ),
        Vec4::new(
            settings.overlay_streak_step_m,
            settings.overlay_streak_scroll,
            settings.overlay_streak_strength,
            flags as f32,
        ),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The legend's colour bar and the shader's ramps are one table written
    /// twice, because WGSL cannot read Rust: this reads the shipped shader's
    /// `OVERLAY_RAMPS` and fails on any stop that differs.
    #[test]
    fn the_legend_and_the_shader_share_one_ramp_table() {
        let shader = include_str!("../../../assets/shaders/water.wgsl");
        let start = shader
            .find("const OVERLAY_RAMPS")
            .expect("water.wgsl declares OVERLAY_RAMPS");
        // The table closes on a line of its own. Matched on the newline before
        // it rather than after, so a CRLF checkout (Windows, `autocrlf`) reads
        // the same table as an LF one.
        let body = &shader[start..start + shader[start..].find("\n);").unwrap()];
        let stops: Vec<[f32; 3]> = body
            .split("vec3<f32>(")
            .skip(1) // what comes before the first stop
            .map(|stop| {
                let inside = &stop[..stop.find(')').unwrap()];
                let v: Vec<f32> = inside
                    .split(',')
                    .map(|n| n.trim().parse().unwrap())
                    .collect();
                [v[0], v[1], v[2]]
            })
            .collect();
        let ours: Vec<[f32; 3]> = RAMPS.iter().flatten().copied().collect();
        assert_eq!(stops, ours);
        assert_eq!(RAMPS.len(), Ramp::ALL.len());
    }

    #[test]
    fn the_ramp_runs_from_its_first_stop_to_its_last() {
        for ramp in Ramp::ALL {
            let stops = &RAMPS[ramp.index()];
            let near =
                |a: [f32; 3], b: [f32; 3]| a.iter().zip(b).all(|(x, y)| (x - y).abs() < 1e-6);
            assert!(near(ramp_colour(ramp, 0.0), stops[0]));
            assert!(near(ramp_colour(ramp, 1.0), stops[4]));
            assert!(near(ramp_colour(ramp, -3.0), stops[0]));
        }
    }

    #[test]
    fn the_lanes_carry_the_overlay_and_off_is_zero() {
        let settings = crate::config::WeatherSettings::default();
        assert_eq!(overlay_lanes(None, &settings), (Vec4::ZERO, Vec4::ZERO));
        let (look, flow) = overlay_lanes(Some(Overlay::Wind), &settings);
        assert_eq!(look.x, Ramp::Speed.index() as f32 + 1.0);
        assert_eq!((look.y, look.z), Overlay::Wind.range());
        assert_eq!(flow.w, 1.0, "wind flows and does not fade");
        let (_, flow) = overlay_lanes(Some(Overlay::Rain), &settings);
        assert_eq!(flow.w, 2.0, "rain fades and does not flow");
    }
}
