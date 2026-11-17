//! MEASUREMENT (`movement-smoothness` design 9.1): `PBD_STEP_TRACE=<path.csv>`
//! writes one row per fixed physics step, after the step, of the walker's
//! physics `Position` and `LinearVelocity` as the simulation stores them, at
//! full f32 precision, beside the physics frame's origin and the walker's
//! planet-local point rebuilt from them in f64. It is how the rounding of
//! the stored position is seen step by step: `tools/step_graph.py` compares
//! each step's move with `velocity * dt`. Changes nothing simulated.

use std::{fs::File, io::Write as _, path::PathBuf};

use avian3d::prelude::{LinearVelocity, Position};
use bevy::prelude::*;

use crate::{SimulationClock, walking::Walker};

pub struct StepTracePlugin;

impl Plugin for StepTracePlugin {
    fn build(&self, app: &mut App) {
        let Some(path) = std::env::var_os("PBD_STEP_TRACE").map(PathBuf::from) else {
            return;
        };
        let mut file = File::create(&path).expect("PBD_STEP_TRACE: cannot create the file");
        writeln!(
            file,
            "tick,dt_s,x,y,z,vx,vy,vz,origin_x,origin_y,origin_z,world_x,world_y,world_z"
        )
        .expect("PBD_STEP_TRACE: cannot write");
        info!("STEP_TRACE writing {}", path.display());
        app.insert_resource(StepTrace(file))
            .add_systems(FixedLast, trace);
    }
}

#[derive(Resource)]
struct StepTrace(File);

fn trace(
    mut out: ResMut<StepTrace>,
    clock: Res<SimulationClock>,
    time: Res<Time<Fixed>>,
    space: crate::SimSpace,
    walkers: Query<(&Position, &LinearVelocity), With<Walker>>,
) {
    let Ok((position, velocity)) = walkers.single() else {
        return;
    };
    let origin = space.origin();
    let world = origin.to_world(position.0);
    // `{:?}` on an f32 prints the shortest digits that read back to the same
    // f32: the stored value exactly.
    let _ = writeln!(
        out.0,
        "{},{:.9},{:?},{:?},{:?},{:?},{:?},{:?},{:.9},{:.9},{:.9},{:.9},{:.9},{:.9}",
        clock.ticks,
        time.delta_secs_f64(),
        position.0.x,
        position.0.y,
        position.0.z,
        velocity.0.x,
        velocity.0.y,
        velocity.0.z,
        origin.origin.x,
        origin.origin.y,
        origin.origin.z,
        world.pos.x,
        world.pos.y,
        world.pos.z,
    );
}
