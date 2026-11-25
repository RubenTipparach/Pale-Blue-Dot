//! A townsperson's walk to a seat or a bed and back (`furniture` task 4.3,
//! design 11a): from where they stand to just outside the building's
//! doorway, through it, and on to the place; and back out the same way to
//! their post. Planet points in f64; drawn only, nothing reads it.

use glam::DVec3;

/// How fast a townsperson walks to a seat or bed, m/s.
pub const WALK_MPS: f64 = 1.2;
/// How far outside the doorway's middle the walk passes, m.
pub const DOOR_OUT_M: f64 = 0.9;
/// How far inside it, m.
pub const DOOR_IN_M: f64 = 0.7;
/// A place higher than this over the doorway's floor is on an upper
/// storey: the walk ends inside the door and the person steps to it, as
/// they do today, until stairs are walked.
pub const UPPER_M: f64 = 0.5;

/// A doorway as a walk reads it: its middle on its floor, and out of the
/// building, a unit tangent.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Doorway {
    pub middle: DVec3,
    pub out: DVec3,
}

impl Doorway {
    pub fn outside(&self) -> DVec3 {
        self.middle + self.out * DOOR_OUT_M
    }

    pub fn inside(&self) -> DVec3 {
        self.middle - self.out * DOOR_IN_M
    }

    /// Whether `place` is on an upper storey: higher than [`UPPER_M`] over
    /// the doorway's floor, along the planet's up.
    pub fn is_upstairs(&self, place: DVec3) -> bool {
        place.length() - self.middle.length() > UPPER_M
    }
}

/// A walk along straight legs, planet points.
#[derive(Clone, Debug, PartialEq)]
pub struct Route {
    points: Vec<DVec3>,
    /// The distance to each point from the first.
    along: Vec<f64>,
}

/// Where a walk is: feet, the way it goes (a tangent there), and whether it
/// has arrived.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Step {
    pub feet: DVec3,
    pub forward: DVec3,
    pub arrived: bool,
}

impl Route {
    /// A walk through `points`, a point the same as the one before it
    /// dropped. At least one point.
    pub fn new(points: Vec<DVec3>) -> Self {
        let mut kept: Vec<DVec3> = Vec::with_capacity(points.len());
        for p in points {
            if kept.last().is_none_or(|q| (*q - p).length() > 1e-6) {
                kept.push(p);
            }
        }
        let mut along = vec![0.0; kept.len()];
        for k in 1..kept.len() {
            along[k] = along[k - 1] + (kept[k] - kept[k - 1]).length();
        }
        Self {
            points: kept,
            along,
        }
    }

    /// In: from `from` to outside `door`, through it, to `place` (or, for a
    /// place upstairs, to inside the door).
    pub fn in_through(from: DVec3, door: Doorway, place: DVec3) -> Self {
        let mut points = vec![from, door.outside(), door.inside()];
        if !door.is_upstairs(place) {
            points.push(place);
        }
        Self::new(points)
    }

    /// Out: from `place` (or inside the door, from upstairs) through `door`
    /// to `post`.
    pub fn out_through(place: DVec3, door: Doorway, post: DVec3) -> Self {
        let mut points = Vec::new();
        if !door.is_upstairs(place) {
            points.push(place);
        }
        points.extend([door.inside(), door.outside(), post]);
        Self::new(points)
    }

    pub fn length(&self) -> f64 {
        self.along.last().copied().unwrap_or(0.0)
    }

    pub fn points(&self) -> &[DVec3] {
        &self.points
    }

    /// Where the walk is `seconds` after it set out at [`WALK_MPS`].
    pub fn at(&self, seconds: f64) -> Step {
        let d = (seconds.max(0.0) * WALK_MPS).min(self.length());
        let last = self.points.len() - 1;
        let k = self
            .along
            .partition_point(|a| *a <= d)
            .clamp(1, last.max(1));
        if last == 0 {
            return Step {
                feet: self.points[0],
                forward: DVec3::ZERO,
                arrived: true,
            };
        }
        let (a, b) = (self.points[k - 1], self.points[k]);
        let leg = (self.along[k] - self.along[k - 1]).max(1e-9);
        let t = ((d - self.along[k - 1]) / leg).clamp(0.0, 1.0);
        let feet = a.lerp(b, t);
        let up = feet.normalize_or(DVec3::Y);
        let dir = b - a;
        let forward = (dir - up * dir.dot(up)).normalize_or_zero();
        Step {
            feet,
            forward,
            arrived: d >= self.length() - 1e-9,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const R: f64 = 300.0;

    fn at(x: f64, z: f64) -> DVec3 {
        DVec3::new(x, R, z)
    }

    /// The walk in goes outside the door, then inside it, then to the seat,
    /// at the walking pace, and arrives; the walk out is the same in
    /// reverse to the post.
    #[test]
    fn a_walk_goes_through_the_door_and_arrives() {
        let door = Doorway {
            middle: at(0.0, 0.0),
            out: DVec3::Z,
        };
        let post = at(0.0, 10.0);
        let seat = at(1.5, -2.0);
        let route = Route::in_through(post, door, seat);
        assert_eq!(route.points(), &[post, door.outside(), door.inside(), seat]);
        let legs = (post - door.outside()).length()
            + (door.outside() - door.inside()).length()
            + (door.inside() - seat).length();
        assert!((route.length() - legs).abs() < 1e-9);
        let start = route.at(0.0);
        assert_eq!(start.feet, post);
        assert!(!start.arrived);
        assert!(start.forward.dot(-DVec3::Z) > 0.99, "toward the door");
        let mid = route.at((10.0 - DOOR_OUT_M) / WALK_MPS);
        assert!((mid.feet - door.outside()).length() < 1e-6);
        let end = route.at(route.length() / WALK_MPS + 1.0);
        assert!(end.arrived && end.feet == seat);
        let back = Route::out_through(seat, door, post);
        assert_eq!(back.points(), &[seat, door.inside(), door.outside(), post]);
        assert!((back.length() - route.length()).abs() < 1e-9);
    }

    /// A bed upstairs: the walk ends inside the door (the person steps up to
    /// it, as before), and starts there on the way out.
    #[test]
    fn a_bed_upstairs_ends_the_walk_inside_the_door() {
        let door = Doorway {
            middle: at(0.0, 0.0),
            out: DVec3::X,
        };
        let bed = DVec3::new(-2.0, R + 3.0, 0.0);
        assert!(door.is_upstairs(bed));
        let route = Route::in_through(at(8.0, 0.0), door, bed);
        assert_eq!(route.points().last(), Some(&door.inside()));
        let back = Route::out_through(bed, door, at(8.0, 0.0));
        assert_eq!(back.points().first(), Some(&door.inside()));
        let one = Route::new(vec![at(1.0, 1.0), at(1.0, 1.0)]);
        assert!(one.at(5.0).arrived && one.length() == 0.0);
    }
}
