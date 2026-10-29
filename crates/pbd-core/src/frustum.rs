//! A view frustum's planes and a conservative box test (`puff-frustum-cull`).
//!
//! The planes are read off a clip matrix by Gribb-Hartmann rows. A box is
//! outside only when it lies wholly on the outer side of one plane, so a box
//! that touches or crosses the frustum is never reported outside; a few boxes
//! near a frustum corner that miss it are reported inside, which is the safe
//! side.

use glam::{Mat4, Vec3, Vec4};

/// The six planes of a frustum, each `(n, d)` with `n.p + d >= 0` inside.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Planes {
    pub planes: [Vec4; 6],
}

impl Planes {
    /// The planes of `clip_from_space`, for a projection with depth in
    /// `0..=w` (wgpu's clip space; Bevy's reverse-Z included). An infinite
    /// reverse-Z projection's far plane is `z >= 0`, which every point in
    /// front of the eye passes, so it costs nothing and is kept for finite
    /// projections.
    pub fn from_clip(clip_from_space: Mat4) -> Self {
        let r0 = clip_from_space.row(0);
        let r1 = clip_from_space.row(1);
        let r2 = clip_from_space.row(2);
        let r3 = clip_from_space.row(3);
        Self {
            planes: [r3 + r0, r3 - r0, r3 + r1, r3 - r1, r2, r3 - r2],
        }
    }

    /// Whether the axis-aligned box `centre +- half` touches or crosses the
    /// frustum. False only when the box's corner furthest along some plane's
    /// normal is outside that plane. A non-finite box is kept.
    pub fn touches_box(&self, centre: Vec3, half: Vec3) -> bool {
        if !centre.is_finite() || !half.is_finite() {
            return true;
        }
        let half = half.abs();
        self.planes.iter().all(|p| {
            let n = p.truncate();
            n.dot(centre) + p.w + n.abs().dot(half) >= 0.0
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Bevy's camera: a reverse-Z infinite projection, looking down -Z.
    fn camera(eye: Vec3, toward: Vec3) -> Planes {
        let projection = Mat4::perspective_infinite_reverse_rh(1.0, 1.6, 0.1);
        let view = Mat4::look_at_rh(eye, toward, Vec3::Y);
        Planes::from_clip(projection * view)
    }

    #[test]
    fn a_box_ahead_is_kept_and_one_behind_is_not() {
        let f = camera(Vec3::ZERO, -Vec3::Z);
        assert!(f.touches_box(Vec3::new(0.0, 0.0, -50.0), Vec3::splat(1.0)));
        assert!(f.touches_box(Vec3::new(0.0, 0.0, -5.0e6), Vec3::splat(1.0)));
        assert!(!f.touches_box(Vec3::new(0.0, 0.0, 50.0), Vec3::splat(1.0)));
        // Off to each side, wholly outside.
        for side in [Vec3::X, -Vec3::X, Vec3::Y, -Vec3::Y] {
            assert!(!f.touches_box(side * 200.0 - Vec3::Z * 10.0, Vec3::splat(1.0)));
        }
    }

    #[test]
    fn a_box_across_a_plane_is_kept() {
        let f = camera(Vec3::ZERO, -Vec3::Z);
        // Half fov is 0.5 rad vertically: at the box's far face, 105 m out,
        // the top edge is ~57.4 m up. A box whose floor is under that is
        // across the plane; one whose floor is over it misses.
        let edge = 105.0 * (0.5f32).tan();
        assert!(f.touches_box(Vec3::new(0.0, edge + 4.0, -100.0), Vec3::splat(5.0)));
        assert!(!f.touches_box(Vec3::new(0.0, edge + 6.0, -100.0), Vec3::splat(5.0)));
        // Across the near plane, and round the eye itself.
        assert!(f.touches_box(Vec3::new(0.0, 0.0, 0.5), Vec3::splat(1.0)));
        assert!(f.touches_box(Vec3::ZERO, Vec3::splat(30.0)));
    }

    #[test]
    fn a_box_is_never_culled_while_a_point_of_it_is_inside() {
        // Sample points in each box: if any is inside the frustum (by the
        // clip matrix itself), the box must be kept.
        let projection = Mat4::perspective_infinite_reverse_rh(1.0, 1.6, 0.1);
        let view = Mat4::look_at_rh(
            Vec3::new(3.0, 2.0, 1.0),
            Vec3::new(40.0, -5.0, -60.0),
            Vec3::Y,
        );
        let clip = projection * view;
        let f = Planes::from_clip(clip);
        let inside = |p: Vec3| {
            let c = clip * p.extend(1.0);
            c.w > 0.0 && c.x.abs() <= c.w && c.y.abs() <= c.w && c.z >= 0.0 && c.z <= c.w
        };
        let mut seed = 1u32;
        let mut next = || {
            seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            (seed >> 8) as f32 / (1u32 << 24) as f32
        };
        let mut checked = 0;
        for _ in 0..4_000 {
            let centre = Vec3::new(next() - 0.5, next() - 0.5, next() - 0.5) * 400.0;
            let half = Vec3::splat(1.0 + next() * 40.0);
            let mut any = false;
            for i in 0..5 {
                for j in 0..5 {
                    for k in 0..5 {
                        let t = Vec3::new(i as f32, j as f32, k as f32) / 2.0 - Vec3::ONE;
                        any |= inside(centre + t * half);
                    }
                }
            }
            if any {
                checked += 1;
                assert!(
                    f.touches_box(centre, half),
                    "culled a box with a point inside at {centre}"
                );
            }
        }
        assert!(
            checked > 100,
            "too few boxes in view to mean anything: {checked}"
        );
    }

    #[test]
    fn bad_numbers_cull_nothing() {
        let f = camera(Vec3::ZERO, -Vec3::Z);
        assert!(f.touches_box(Vec3::splat(f32::NAN), Vec3::ONE));
        assert!(f.touches_box(Vec3::Z * 50.0, Vec3::splat(f32::INFINITY)));
    }
}
