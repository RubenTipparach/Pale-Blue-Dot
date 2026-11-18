//! MEASUREMENT (`movement-smoothness`): `PBD_MOTION_TRACE=<path.csv>` writes
//! one row per rendered frame of where the things the player watches were
//! drawn: the camera, the walker's physics body, the fishing float and one
//! creature (the fish on the line, else the nearest one to the eye), beside
//! the frame's wall time, how many fixed physics steps ran in it and how far
//! the fixed clock had run past its last step. The creature's drawn forward
//! and up are written too (`movement-smoothness` design 8.3), with a key
//! naming which creature it is. It runs in `Last`, after
//! transforms propagate, so it reads what the frame draws. Changes nothing
//! drawn.
//!
//! `PBD_MOTION_TRACE_SECONDS=<s>` ends the run that long after `READY`, so a
//! real-time windowed run (vsync on or off) stops by itself.

use std::{fs::File, io::Write as _, path::PathBuf, time::Instant};

use avian3d::prelude::Position;
use bevy::prelude::*;

use crate::{
    SimulationClock,
    fish::{Fishery, draw::Float},
    walking::{Walker, WalkingCamera},
};

pub struct MotionTracePlugin;

impl Plugin for MotionTracePlugin {
    fn build(&self, app: &mut App) {
        let Some(path) = std::env::var_os("PBD_MOTION_TRACE").map(PathBuf::from) else {
            return;
        };
        let seconds = std::env::var("PBD_MOTION_TRACE_SECONDS")
            .ok()
            .and_then(|s| s.trim().parse::<f32>().ok())
            .filter(|s| s.is_finite() && *s > 0.0);
        let mut file = File::create(&path).expect("PBD_MOTION_TRACE: cannot create the file");
        writeln!(
            file,
            "frame,wall_s,dt_s,steps,overstep,\
             cam_x,cam_y,cam_z,cam_qx,cam_qy,cam_qz,cam_qw,\
             body_x,body_y,body_z,\
             float_x,float_y,float_z,float_shown,\
             fish_x,fish_y,fish_z,fish_held,phase,\
             hand_vx,hand_vy,hand_vz,hand_lx,hand_ly,hand_lz,\
             float_vx,float_vy,float_vz,fish_vx,fish_vy,fish_vz,\
             hand_rx,hand_ry,hand_rz,\
             origin_x,origin_y,origin_z,\
             line_vx,line_vy,line_vz,tip_vx,tip_vy,tip_vz,\
             fish_key,fish_fx,fish_fy,fish_fz,fish_ux,fish_uy,fish_uz,\
             fish_lx,fish_ly,fish_lz,mouth_x,mouth_y,mouth_z,lure_x,lure_y,lure_z,\
             mouth_lure_m,dip,timer,run,targeted"
        )
        .expect("PBD_MOTION_TRACE: cannot write");
        info!("MOTION_TRACE writing {}", path.display());
        app.insert_resource(MotionTrace {
            file,
            seconds,
            ready_at: None,
            frame: 0,
            last_ticks: None,
            start: Instant::now(),
            followed: None,
        })
        .add_systems(Last, trace.run_if(crate::loading::ready));
    }
}

#[derive(Resource)]
struct MotionTrace {
    file: File,
    seconds: Option<f32>,
    ready_at: Option<Instant>,
    frame: u64,
    last_ticks: Option<u64>,
    start: Instant,
    /// The creature followed, by its school's seed and its own id, so the
    /// trace stays on one fish rather than whichever is nearest.
    followed: Option<(u64, u32)>,
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn trace(
    mut trace: ResMut<MotionTrace>,
    real: Res<Time<Real>>,
    fixed: Res<Time<Fixed>>,
    clock: Res<SimulationClock>,
    frame: Res<crate::planet::PlanetRenderFrame>,
    fishery: Option<Res<Fishery>>,
    drawn_float: Option<Res<crate::fish::draw::DrawnFloat>>,
    headings: Option<Res<crate::fish::draw::DrawnHeadings>>,
    lines: Query<(&GlobalTransform, &InheritedVisibility), With<crate::fish::draw::LineMesh>>,
    fauna: Option<Res<crate::config::FaunaConfig>>,
    cameras: Query<&GlobalTransform, With<WalkingCamera>>,
    walkers: Query<&Position, With<Walker>>,
    floats: Query<(&GlobalTransform, &InheritedVisibility), With<Float>>,
    in_hand: Query<(
        &GlobalTransform,
        &Transform,
        &InheritedVisibility,
        &crate::held::Held,
    )>,
    mut exit: MessageWriter<AppExit>,
) {
    let now = Instant::now();
    let ready_at = *trace.ready_at.get_or_insert(now);
    let steps = trace
        .last_ticks
        .map_or(0, |t| clock.ticks.saturating_sub(t));
    trace.last_ticks = Some(clock.ticks);
    trace.frame += 1;
    let Ok(camera) = cameras.single() else {
        return;
    };
    let (_, rotation, eye) = camera.to_scale_rotation_translation();
    // Positions are written planet-local (f64 from render space, so a
    // rebase of the render origin is not a jump in the trace); the camera
    // space columns are what the GPU gets (`movement-smoothness` design 6).
    let local = |p: Vec3| (p.as_dvec3() - frame.center).as_vec3();
    let body = walkers.single().map_or(Vec3::NAN, |p| {
        (p.0.as_dvec3() - frame.physics_center()).as_vec3()
    });
    let (float, shown) = floats
        .iter()
        .next()
        .map_or((Vec3::NAN, false), |(t, v)| (t.translation(), v.get()));
    let step_hz = fauna.as_deref().map_or(30.0, |c| c.0.spawn.step_hz);
    let followed = trace.followed;
    let mut now_followed = followed;
    let o = crate::sim_origin::from_render_frame(&frame);
    let (fish, held, phase, fish_local) =
        fishery
            .as_deref()
            .map_or((Vec3::NAN, false, String::new(), Vec3::NAN), |f| {
                // The fishery is local (`world-frames`).
                let local_eye = crate::sim_origin::local_from_render(&frame, eye);
                let find = |key: (u64, u32)| {
                    f.schools.iter().enumerate().find_map(|(s, school)| {
                        (school.seed == key.0)
                            .then(|| {
                                school
                                    .ids
                                    .iter()
                                    .position(|id| *id == key.1)
                                    .map(|i| (s, i))
                            })
                            .flatten()
                    })
                };
                let pick = f.line.target.map(|t| (t.school, t.fish));
                let pick = pick.or_else(|| followed.and_then(find)).or_else(|| {
                    f.schools
                        .iter()
                        .enumerate()
                        .flat_map(|(s, school)| {
                            school
                                .positions
                                .iter()
                                .enumerate()
                                .map(move |(i, p)| (s, i, p.distance(local_eye)))
                        })
                        .min_by(|a, b| a.2.total_cmp(&b.2))
                        .map(|(s, i, _)| (s, i))
                });
                now_followed = pick
                    .and_then(|(s, i)| Some((f.schools.get(s)?.seed, *f.schools[s].ids.get(i)?)));
                let drawn = pick.and_then(|(s, i)| {
                    crate::fish::draw::creature_drawn(
                        &o,
                        f,
                        &drawn_float.as_deref().copied().unwrap_or_default(),
                        s,
                        i,
                        step_hz,
                    )
                    .map(|p| {
                        // Planet-local for the columns, from f64.
                        let held = f.schools[s].held.is_some_and(|(h, _)| h == i);
                        ((o.to_world(p).pos).as_vec3(), held, p)
                    })
                });
                let (p, held, local) = drawn.unwrap_or((Vec3::NAN, false, Vec3::NAN));
                (p, held, format!("{:?}", f.line.phase), local)
            });
    // Camera space as the GPU gets it: an f32 view matrix (the inverse of
    // the camera's world matrix) times an f32 world position, so the
    // rounding of planet-scale coordinates shows in what is drawn.
    let view = camera.to_matrix().inverse();
    let (hand_view, hand_local) = in_hand
        .iter()
        .max_by_key(|(_, _, v, _)| v.get())
        .map_or((Vec3::NAN, Vec3::NAN), |(g, t, _, _)| {
            (view.transform_point3(g.translation()), t.translation)
        });
    // The A/B: the same tool drawn with the render origin rebased to the
    // camera, snapped to a 16 m grid (a floating origin that moves in 16 m
    // jumps). Its world point is rebuilt in f64 from the camera and the
    // pose, the origin subtracted, and only then cast to f32 (`CLAUDE.md`:
    // "Subtract the origin before casting"), and drawn through an f32 view
    // matrix as the GPU would.
    let origin = (eye.as_dvec3() / 16.0).round() * 16.0;
    let hand_rebased = if hand_local.is_nan() {
        Vec3::NAN
    } else {
        let world = eye.as_dvec3() + rotation.as_dquat() * hand_local.as_dvec3();
        let eye_rebased = (eye.as_dvec3() - origin).as_vec3();
        let view_rebased = Mat4::from_rotation_translation(rotation, eye_rebased).inverse();
        view_rebased.transform_point3((world - origin).as_vec3())
    };
    // The line's start as drawn, and the tip of the rod as drawn (its pose
    // under the camera, swayed), in camera space (`movement-smoothness`
    // 7.3): the gap between them is the line leaving the rod off its tip.
    let line_view = lines
        .iter()
        .find(|(_, v)| v.get())
        .map_or(Vec3::NAN, |(g, _)| view.transform_point3(g.translation()));
    let tip_view = in_hand
        .iter()
        .find(|(_, _, v, h)| v.get() && h.tool() == pbd_core::inventory::Tool::Rod)
        .map_or(Vec3::NAN, |(g, _, _, h)| {
            view.transform_point3(g.transform_point(h.in_tool(crate::held::rod_tip())))
        });
    let float_view = if shown {
        view.transform_point3(float)
    } else {
        Vec3::NAN
    };
    trace.followed = now_followed;
    // The followed creature's drawn forward and up, as its mesh was built.
    let pose = now_followed.and_then(|key| headings.as_deref()?.map.get(&key).copied());
    let (fwd, top, mouth) = pose.map_or((Vec3::NAN, Vec3::NAN, Vec3::NAN), |h| {
        (h.forward, h.up, h.mouth)
    });
    // `fish-behaviour`: the lure as drawn (under the drawn float, moved by
    // what the fish does to it), the drawn mouth's distance from it, and the
    // line's dip, timer and run, local.
    let (lure, dip, timer, run) =
        fishery
            .as_deref()
            .map_or((Vec3::NAN, f32::NAN, f32::NAN, f32::NAN), |f| {
                let float = drawn_float.as_deref().copied().unwrap_or_default();
                let lure = if f.line.is_out() {
                    crate::fish::draw::drawn_lure(&o, f, &float)
                } else {
                    Vec3::NAN
                };
                (lure, f.line.dip, f.line.timer(), f.line.run)
            });
    let key = now_followed.map_or(String::new(), |(seed, id)| format!("{seed:x}-{id}"));
    let fish_view = view.transform_point3(crate::render_origin::render_from_physics(
        &frame, fish_local,
    ));
    let eye_local = local(eye);
    let float_local = local(float);
    let wall = now.duration_since(trace.start).as_secs_f64();
    let row = format!(
        "{},{:.6},{:.6},{},{:.4},{:.5},{:.5},{:.5},{:.6},{:.6},{:.6},{:.6},{:.5},{:.5},{:.5},{:.5},{:.5},{:.5},{},{:.5},{:.5},{:.5},{},{},{:.7},{:.7},{:.7},{:.7},{:.7},{:.7},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.7},{:.7},{:.7},{:.1},{:.1},{:.1},{:.7},{:.7},{:.7},{:.7},{:.7},{:.7},{},{:.5},{:.5},{:.5},{:.5},{:.5},{:.5},{:.5},{:.5},{:.5},{:.5},{:.5},{:.5},{:.5},{:.5},{:.5},{:.5},{:.5},{:.4},{:.4},{}",
        trace.frame,
        wall,
        real.delta_secs_f64(),
        steps,
        fixed.overstep_fraction(),
        eye_local.x,
        eye_local.y,
        eye_local.z,
        rotation.x,
        rotation.y,
        rotation.z,
        rotation.w,
        body.x,
        body.y,
        body.z,
        float_local.x,
        float_local.y,
        float_local.z,
        u8::from(shown),
        fish.x,
        fish.y,
        fish.z,
        u8::from(held),
        phase,
        hand_view.x,
        hand_view.y,
        hand_view.z,
        hand_local.x,
        hand_local.y,
        hand_local.z,
        float_view.x,
        float_view.y,
        float_view.z,
        fish_view.x,
        fish_view.y,
        fish_view.z,
        hand_rebased.x,
        hand_rebased.y,
        hand_rebased.z,
        frame.origin.x,
        frame.origin.y,
        frame.origin.z,
        line_view.x,
        line_view.y,
        line_view.z,
        tip_view.x,
        tip_view.y,
        tip_view.z,
        key,
        fwd.x,
        fwd.y,
        fwd.z,
        top.x,
        top.y,
        top.z,
        fish_local.x,
        fish_local.y,
        fish_local.z,
        mouth.x,
        mouth.y,
        mouth.z,
        lure.x,
        lure.y,
        lure.z,
        mouth.distance(lure),
        dip,
        timer,
        run,
        u8::from(fishery.as_deref().is_some_and(|f| f.line.target.is_some())),
    );
    let _ = writeln!(trace.file, "{row}");
    if let Some(seconds) = trace.seconds
        && now.duration_since(ready_at).as_secs_f32() >= seconds
    {
        let _ = trace.file.flush();
        info!("MOTION_TRACE_DONE {} frames", trace.frame);
        exit.write(AppExit::Success);
    }
}
