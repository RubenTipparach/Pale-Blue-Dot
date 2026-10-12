//! Where the sun's shadow cascades stand (`sun-shadows` decision 2).
//!
//! Four cascades split the view between [`NEAR_M`] and the reach, each the
//! bounding sphere of its slice of the view frustum, drawn in an orthographic
//! box looking down the sun. Three rules keep a shadow still:
//!
//! - **A sphere, not a box.** Its radius depends on the slice and the lens
//!   only, so turning in place never changes a texel's size.
//! - **A radius rounded up to [`RADIUS_STEP_M`]**, so it changes only when the
//!   slice grows, as it does when the reach grows in flight.
//! - **A centre snapped to a whole texel** in the light's frame, so a walk of
//!   less than a texel moves nothing and a longer one moves the shadow in whole
//!   texels. The texel grid lies on the planet, not on the camera.
//!
//! The box's near plane is pulled toward the sun far enough to take in every
//! caster, which is anything below [`CASTER_SHELL_M`] over the radius. Built in
//! f64 in the planet's frame and cast to f32 at the upload, as the terrain's
//! `clip_from_body` is.

use glam::{DMat4, DVec3, DVec4};

/// Cascades per view.
pub const CASCADES: usize = 4;
/// Texels along a side of each cascade's map.
pub const TEXELS: u32 = 2048;
/// The practical split's weight toward the logarithmic split.
pub const LAMBDA: f64 = 0.95;
/// Where the first cascade starts, metres from the eye.
pub const NEAR_M: f64 = 0.5;
/// How far the cascades reach on foot, metres.
pub const FOOT_REACH_M: f64 = 900.0;
/// The most they reach in flight, metres.
pub const MAX_REACH_M: f64 = 6000.0;
/// The highest caster over the planet's radius, metres: the highest summit
/// with a tree on it.
pub const CASTER_SHELL_M: f64 = 320.0;
/// A cascade's radius is rounded up to a whole number of these, metres.
pub const RADIUS_STEP_M: f64 = 1.0;
/// The most a cascade's near plane is pulled toward the sun, metres.
pub const MAX_PULL_M: f64 = 6000.0;
/// How often each cascade is redrawn, in frames (decision 2, "Refresh").
pub const PERIOD: [u64; CASCADES] = [1, 1, 2, 4];

/// The camera, in the planet's frame.
#[derive(Clone, Copy, Debug)]
pub struct View {
    pub eye: DVec3,
    /// Unit vectors: where it looks, and its up.
    pub forward: DVec3,
    pub up: DVec3,
    /// The vertical field of view, radians, and width over height.
    pub fov_y: f64,
    pub aspect: f64,
}

/// One cascade as it was fitted.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Cascade {
    /// Planet frame to the cascade's clip space: x and y in -1..1 across the
    /// box, depth 0 at the near plane (toward the sun) to 1 at the far.
    pub clip_from_body: DMat4,
    /// The sphere the box is fitted round, in the planet's frame, after the
    /// snap.
    pub centre: DVec3,
    pub radius: f64,
    /// The slice of the view it serves, metres from the eye.
    pub from_m: f64,
    pub to_m: f64,
    /// A texel's width on the ground, metres.
    pub texel_m: f64,
    /// Metres from the near plane to the far, what one unit of depth is.
    pub depth_m: f64,
}

/// How far the cascades reach at a height over the ground: the horizon's
/// distance, never less than on foot nor more than [`MAX_REACH_M`].
pub fn reach(radius: f64, altitude: f64) -> f64 {
    let horizon = (2.0 * radius * altitude.max(0.0)).sqrt();
    horizon.clamp(FOOT_REACH_M, MAX_REACH_M)
}

/// The cascades' edges from `near` to `far`, the practical split: `LAMBDA` of
/// the logarithmic split and the rest of the uniform one.
pub fn splits(near: f64, far: f64) -> [f64; CASCADES + 1] {
    let mut edges = [near; CASCADES + 1];
    for (i, edge) in edges.iter_mut().enumerate().skip(1) {
        let s = i as f64 / CASCADES as f64;
        let log = near * (far / near).powf(s);
        let uniform = near + (far - near) * s;
        *edge = LAMBDA * log + (1.0 - LAMBDA) * uniform;
    }
    edges[CASCADES] = far;
    edges
}

/// The smallest sphere round the slice `near..far` of a frustum, as the
/// distance of its centre along the view and its radius. `k2` is the squared
/// tangent of the half-angle to the frustum's corner.
fn slice_sphere(near: f64, far: f64, k2: f64) -> (f64, f64) {
    if k2 >= (far - near) / (far + near) {
        // The far face's corners and the far centre bound it.
        return (far, far * k2.sqrt());
    }
    let centre = 0.5 * (far + near) * (1.0 + k2);
    let radius = 0.5
        * ((far - near).powi(2)
            + 2.0 * (far * far + near * near) * k2
            + (far + near).powi(2) * k2 * k2)
            .sqrt();
    (centre, radius)
}

/// The light's frame: `x` and `y` across the box, `z` toward the sun. Fixed on
/// the planet's axes rather than the camera's, so the texel grid holds still.
pub fn light_axes(sun: DVec3) -> (DVec3, DVec3, DVec3) {
    let z = sun.normalize();
    let reference = if z.y.abs() > 0.95 { DVec3::X } else { DVec3::Y };
    let x = reference.cross(z).normalize();
    let y = z.cross(x);
    (x, y, z)
}

/// Where the ray from `from` toward the sun leaves the casters' shell, as a
/// distance along it; zero when it starts outside.
fn exit_shell(from: DVec3, z: DVec3, shell: f64) -> f64 {
    let b = from.dot(z);
    let c = from.length_squared() - shell * shell;
    if c >= 0.0 {
        return 0.0;
    }
    -b + (b * b - c).sqrt()
}

/// Fit the four cascades to a view, with the sun toward `sun`, on a planet of
/// `radius` metres centred at the origin.
pub fn fit(view: &View, sun: DVec3, radius: f64) -> [Cascade; CASCADES] {
    let altitude = view.eye.length() - radius;
    let edges = splits(NEAR_M, reach(radius, altitude));
    let tan = (0.5 * view.fov_y).tan();
    let k2 = tan * tan * (1.0 + view.aspect * view.aspect);
    let (x, y, z) = light_axes(sun);
    let shell = radius + CASTER_SHELL_M;
    std::array::from_fn(|i| {
        let (along, r) = slice_sphere(edges[i], edges[i + 1], k2);
        let r = (r / RADIUS_STEP_M).ceil() * RADIUS_STEP_M;
        let texel = 2.0 * r / f64::from(TEXELS);
        let raw = view.eye + view.forward * along;
        let snap = |v: f64| (v / texel).round() * texel;
        let centre = x * snap(raw.dot(x)) + y * snap(raw.dot(y)) + z * raw.dot(z);
        // The near plane: as far toward the sun as the shell reaches from any
        // point of the sphere's shadow, so whatever stands between the slice
        // and the sun is in the box.
        let down = centre.normalize();
        let mut pull = r;
        for q in [
            centre,
            centre - down * r,
            centre + x * r,
            centre - x * r,
            centre + y * r,
            centre - y * r,
        ] {
            pull = pull.max((q - centre).dot(z) + exit_shell(q, z, shell));
        }
        let pull = pull.min(MAX_PULL_M.max(r));
        let depth = pull + r;
        let rows = [
            DVec4::new(x.x / r, x.y / r, x.z / r, -centre.dot(x) / r),
            DVec4::new(y.x / r, y.y / r, y.z / r, -centre.dot(y) / r),
            DVec4::new(
                -z.x / depth,
                -z.y / depth,
                -z.z / depth,
                (pull + centre.dot(z)) / depth,
            ),
            DVec4::W,
        ];
        Cascade {
            clip_from_body: DMat4::from_cols(rows[0], rows[1], rows[2], rows[3]).transpose(),
            centre,
            radius: r,
            from_m: edges[i],
            to_m: edges[i + 1],
            texel_m: texel,
            depth_m: depth,
        }
    })
}

/// Which cascades are redrawn on frame `frame`: the near two every frame, the
/// third every other and the fourth every fourth, staggered so they do not
/// fall on the same frame.
pub fn due(frame: u64) -> [bool; CASCADES] {
    std::array::from_fn(|i| (frame + i as u64).is_multiple_of(PERIOD[i]))
}

/// The cascades as they were last drawn. A cascade not redrawn keeps the
/// matrices it was drawn with, so what the receivers sample always matches
/// what is in its map.
#[derive(Clone, Copy, Debug, Default)]
pub struct Drawn {
    pub cascades: Option<[Cascade; CASCADES]>,
    /// Which were redrawn on the last update.
    pub redraw: [bool; CASCADES],
    pub frame: u64,
}

impl Drawn {
    /// Take this frame's fit for the cascades that are due, or for all of
    /// them after a jump in time (or on the first frame).
    pub fn update(&mut self, fitted: [Cascade; CASCADES], jumped: bool) {
        let due = due(self.frame);
        match &mut self.cascades {
            Some(drawn) if !jumped => {
                for i in 0..CASCADES {
                    if due[i] {
                        drawn[i] = fitted[i];
                    }
                }
                self.redraw = due;
            }
            _ => {
                self.cascades = Some(fitted);
                self.redraw = [true; CASCADES];
            }
        }
        self.frame += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const R: f64 = 4800.0;

    fn walker(at: DVec3, forward: DVec3) -> View {
        let up = at.normalize();
        let forward = (forward - up * forward.dot(up)).normalize();
        View {
            eye: at,
            forward,
            up,
            fov_y: 60f64.to_radians(),
            aspect: 1.6,
        }
    }

    fn clip(c: &Cascade, p: DVec3) -> DVec3 {
        let v = c.clip_from_body * p.extend(1.0);
        v.truncate() / v.w
    }

    fn inside(c: &Cascade, p: DVec3) -> bool {
        let q = clip(c, p);
        q.x.abs() <= 1.0 && q.y.abs() <= 1.0 && (0.0..=1.0).contains(&q.z)
    }

    fn sun() -> DVec3 {
        DVec3::new(0.3, 0.5, 0.8).normalize()
    }

    fn ground_eye() -> DVec3 {
        DVec3::new(0.2, 0.9, 0.4).normalize() * (R + 1.6)
    }

    /// On foot the edges are the design's: near 14, 43, 165 and 900 m.
    #[test]
    fn the_edges_on_foot_are_the_designs() {
        let e = splits(NEAR_M, reach(R, 1.6));
        let want = [0.5, 14.4, 42.6, 164.9, 900.0];
        for (got, want) in e.iter().zip(want) {
            assert!((got - want).abs() < 0.5, "{e:?}");
        }
        // In flight the reach is the horizon's, up to the cap.
        assert!((reach(R, 1000.0) - (2.0 * R * 1000.0).sqrt()).abs() < 1e-6);
        assert_eq!(reach(R, 10_000.0), MAX_REACH_M);
    }

    /// Every cascade covers its slice of the view: each corner of the slice is
    /// inside its box.
    #[test]
    fn each_cascade_covers_its_slice_of_the_view() {
        let v = walker(ground_eye(), DVec3::new(1.0, 0.1, -0.3));
        let cascades = fit(&v, sun(), R);
        let right = v.forward.cross(v.up).normalize();
        let tan = (0.5 * v.fov_y).tan();
        for c in &cascades {
            for d in [c.from_m, c.to_m] {
                for (sx, sy) in [(-1.0, -1.0), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)] {
                    let p = v.eye
                        + v.forward * d
                        + right * (sx * tan * v.aspect * d)
                        + v.up * (sy * tan * d);
                    assert!(inside(c, p), "slice {}..{} misses {p}", c.from_m, c.to_m);
                }
            }
        }
    }

    /// The box moves only in whole texels across the light, so a walk never
    /// crawls a shadow's edge: a step shorter than a texel moves it by none or
    /// one. A turn in place keeps every cascade's size and its texel grid.
    #[test]
    fn the_box_moves_in_whole_texels_and_a_turn_keeps_the_grid() {
        let eye = ground_eye();
        let a = fit(&walker(eye, DVec3::X), sun(), R);
        let (x, y, _) = light_axes(sun());
        let whole = |a: &Cascade, b: &Cascade| -> f64 {
            let mut most: f64 = 0.0;
            for axis in [x, y] {
                let texels = (b.centre - a.centre).dot(axis) / a.texel_m;
                assert!((texels - texels.round()).abs() < 1e-6, "{texels} texels");
                most = most.max(texels.round().abs());
            }
            most
        };
        let way = DVec3::new(0.3, 0.1, 0.5).normalize();
        for step in [0.001, 0.004, 0.03, 1.0, 17.0] {
            let b = fit(&walker(eye + way * step, DVec3::X), sun(), R);
            for (a, b) in a.iter().zip(&b) {
                assert_eq!(a.radius, b.radius);
                let moved = whole(a, b);
                if step < a.texel_m {
                    assert!(moved <= 1.0, "{step} m moved {moved} texels");
                }
            }
        }
        let turned = fit(&walker(eye, DVec3::new(-0.4, 0.2, 1.0)), sun(), R);
        for (a, t) in a.iter().zip(&turned) {
            assert_eq!(a.radius, t.radius, "a turn resized a cascade");
            whole(a, t);
        }
    }

    /// A caster on the shell, between the far slice and the sun, is inside the
    /// box's depth: a ridge's shadow reaches a valley under a low sun.
    #[test]
    fn a_caster_between_the_slice_and_the_sun_is_in_the_box() {
        let eye = ground_eye();
        // A low sun, eight degrees up, off to the side.
        let up = eye.normalize();
        let east = DVec3::Y.cross(up).normalize();
        let low = (up * 8f64.to_radians().sin() + east * 8f64.to_radians().cos()).normalize();
        let cascades = fit(&walker(eye, east), low, R);
        for c in &cascades {
            // Walk from the sphere's centre toward the sun until the shell,
            // and take the point just under it.
            let t = exit_shell(c.centre, low, R + CASTER_SHELL_M);
            let caster = c.centre + low * (t - 1.0);
            assert!(
                caster.length() < R + CASTER_SHELL_M,
                "the caster is on the shell"
            );
            let q = clip(c, caster);
            assert!(
                (0.0..=1.0).contains(&q.z),
                "caster depth {} for {:?}",
                q.z,
                c.to_m
            );
            // And its shadow on the slice's centre lands on the same texel.
            let r = clip(c, c.centre);
            assert!((q.truncate() - r.truncate()).length() < 1e-6);
        }
    }

    /// The schedule: the near two every frame, the third every other and the
    /// fourth every fourth; a jump in time redraws all four; one not redrawn
    /// keeps what it was drawn with.
    #[test]
    fn the_far_cascades_refresh_less_often_and_a_jump_redraws_all() {
        let counts = (0..8).fold([0; CASCADES], |mut n, f| {
            for (i, d) in due(f).iter().enumerate() {
                n[i] += usize::from(*d);
            }
            n
        });
        assert_eq!(counts, [8, 8, 4, 2]);
        let eye = ground_eye();
        let first = fit(&walker(eye, DVec3::X), sun(), R);
        let mut drawn = Drawn::default();
        drawn.update(first, false);
        assert_eq!(drawn.redraw, [true; CASCADES], "the first frame draws all");
        // Walk 30 m: every cascade's fit changes.
        let moved = fit(&walker(eye + DVec3::Z * 30.0, DVec3::X), sun(), R);
        drawn.update(moved, false);
        let due1 = due(1);
        for i in 0..CASCADES {
            let kept = drawn.cascades.unwrap()[i];
            assert_eq!(kept == moved[i], due1[i], "cascade {i}");
            assert_eq!(kept == first[i], !due1[i], "cascade {i}");
        }
        drawn.update(first, true);
        assert_eq!(drawn.redraw, [true; CASCADES], "a jump redraws all");
        assert_eq!(drawn.cascades.unwrap(), first);
    }
}
