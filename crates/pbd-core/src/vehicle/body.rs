//! One rigid body, integrated here rather than by Avian.
//!
//! A hull's buoyancy is stiff (an empty canoe heaves at about 52 rad/s), and a
//! force held constant across Avian's four substeps is a 60 Hz force. So a
//! craft steps itself at the substep rate, re-evaluating every force each
//! substep, and the app places the Avian body where this one is. Positions are
//! body-local (the planet's centre is the origin) and double precision; the
//! frame's axes are the planet's.

use glam::{DQuat, DVec3};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RigidBody {
    pub mass: f64,
    /// Principal moments about the body's own axes: right (x), up (y) and
    /// aft (z), kg m^2.
    pub inertia: DVec3,
    /// The centre of mass, body-local, m.
    pub position: DVec3,
    pub velocity: DVec3,
    /// Body axes to the planet's frame. Forward is the body's -z.
    pub orientation: DQuat,
    /// Angular velocity in the planet's frame, rad/s.
    pub angular_velocity: DVec3,
    force: DVec3,
    torque: DVec3,
}

impl RigidBody {
    pub fn new(mass: f64, inertia: DVec3) -> Self {
        Self {
            mass,
            inertia,
            position: DVec3::ZERO,
            velocity: DVec3::ZERO,
            orientation: DQuat::IDENTITY,
            angular_velocity: DVec3::ZERO,
            force: DVec3::ZERO,
            torque: DVec3::ZERO,
        }
    }

    /// A point given relative to the centre of mass in body axes, in the
    /// planet's frame.
    pub fn point(&self, local: DVec3) -> DVec3 {
        self.position + self.orientation * local
    }

    /// A body-axis direction in the planet's frame.
    pub fn axis(&self, local: DVec3) -> DVec3 {
        self.orientation * local
    }

    /// A planet-frame vector in body axes.
    pub fn local(&self, world: DVec3) -> DVec3 {
        self.orientation.inverse() * world
    }

    /// The velocity of a point of the body, planet frame.
    pub fn velocity_at(&self, world: DVec3) -> DVec3 {
        self.velocity + self.angular_velocity.cross(world - self.position)
    }

    /// Apply a force at a point, both in the planet's frame.
    pub fn push(&mut self, force: DVec3, at: DVec3) {
        self.force += force;
        self.torque += (at - self.position).cross(force);
    }

    /// Apply a force through the centre of mass.
    pub fn push_centre(&mut self, force: DVec3) {
        self.force += force;
    }

    /// Apply a torque, planet frame.
    pub fn twist(&mut self, torque: DVec3) {
        self.torque += torque;
    }

    /// The forces gathered so far this substep.
    pub fn gathered(&self) -> (DVec3, DVec3) {
        (self.force, self.torque)
    }

    /// Semi-implicit Euler over the gathered forces, then clear them.
    pub fn integrate(&mut self, dt: f64) {
        self.velocity += self.force / self.mass * dt;
        self.position += self.velocity * dt;
        let inverse = self.orientation.inverse();
        let mut w = inverse * self.angular_velocity;
        let t = inverse * self.torque;
        let i = self.inertia;
        let gyro = w.cross(i * w);
        w += (t - gyro) / i * dt;
        self.angular_velocity = self.orientation * w;
        let spin = DQuat::from_xyzw(
            self.angular_velocity.x,
            self.angular_velocity.y,
            self.angular_velocity.z,
            0.0,
        ) * self.orientation;
        let q = self.orientation;
        self.orientation = DQuat::from_xyzw(
            q.x + 0.5 * dt * spin.x,
            q.y + 0.5 * dt * spin.y,
            q.z + 0.5 * dt * spin.z,
            q.w + 0.5 * dt * spin.w,
        )
        .normalize();
        self.force = DVec3::ZERO;
        self.torque = DVec3::ZERO;
    }

    pub fn is_finite(&self) -> bool {
        self.position.is_finite()
            && self.velocity.is_finite()
            && self.orientation.is_finite()
            && self.angular_velocity.is_finite()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_push_through_the_centre_only_moves_it() {
        let mut b = RigidBody::new(10.0, DVec3::ONE);
        for _ in 0..240 {
            b.push(DVec3::X * 10.0, b.position);
            b.integrate(1.0 / 240.0);
        }
        assert!((b.velocity.x - 1.0).abs() < 1e-9);
        assert!(b.angular_velocity.length() < 1e-12);
    }

    #[test]
    fn a_push_off_centre_turns_it_the_right_way() {
        let mut b = RigidBody::new(10.0, DVec3::splat(2.0));
        // A push forward (-z) at the right side (+x) turns the nose left:
        // a positive turn about +y.
        b.push(DVec3::NEG_Z, DVec3::X);
        b.integrate(0.01);
        assert!(b.angular_velocity.y > 0.0);
    }

    #[test]
    fn a_spinning_body_keeps_a_unit_quaternion_and_its_energy() {
        let mut b = RigidBody::new(1.0, DVec3::new(1.0, 2.0, 3.0));
        b.angular_velocity = DVec3::new(0.0, 2.0, 0.01);
        let energy = |b: &RigidBody| {
            let w = b.local(b.angular_velocity);
            0.5 * (b.inertia * w * w).element_sum()
        };
        let e0 = energy(&b);
        for _ in 0..2400 {
            b.integrate(1.0 / 240.0);
        }
        assert!((b.orientation.length() - 1.0).abs() < 1e-9);
        assert!((energy(&b) - e0).abs() < e0 * 0.02);
    }
}
