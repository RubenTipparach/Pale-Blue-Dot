//! A road cell as drawn and stood on (`townsfolk` design 10): its top, the
//! side faces down to its own ground, a raised bed or a bridge's deck, rails
//! and posts; and the floor the walker stands on, which is that same top.
//!
//! Each cell is a piece in its own tangent frame at its low side, as a
//! town's building is ([`BuildingSolids`]), so the walker meets it through
//! the same floors and solids it meets a town through. Its meshes are in
//! planet-centred metres, by texture, as a town's are ([`Meshes`]).

use super::{Kind, RoadCell, Surface, centre, corners};
use crate::settlement::pieces::{self, BuildingSolids, Frame, MeshBuf, Meshes, Solid};
use glam::{Vec2, Vec3};

/// How far a road top stands over the ground it lies on, m: enough that the
/// two never fight for a pixel, too little for a foot to feel.
pub const LIFT_M: f32 = 0.02;
/// A bridge deck's thickness, m.
pub const DECK_M: f32 = 0.2;
/// A bridge rail's height over the deck, m, and its thickness.
pub const RAIL_M: f32 = 1.0;
pub const RAIL_T: f32 = 0.1;
/// A post's side, m.
pub const POST_M: f32 = 0.16;

/// The texture a road surface is drawn in (`assets/textures/settlement`).
pub fn texture(surface: Surface) -> &'static str {
    match surface {
        Surface::Dirt => "roaddirt",
        Surface::Gravel => "gravel",
    }
}
/// The road bed's sides, and a slope's.
pub const SIDE: &str = "dirt";
/// A bridge's deck and its frame.
pub const DECK: &str = "plank";
pub const TIMBER: &str = "timber";

/// One road cell, cut.
#[derive(Clone, Debug)]
pub struct Piece {
    pub key: u32,
    pub meshes: Meshes,
    pub solids: BuildingSolids,
    pub surface: Surface,
}

/// The frame a cell is cut in: at its centre, `height_m` over the sea
/// radius, `y` up and `x` east (along the pole's turn, so neighbouring
/// cells' textures run the same way).
pub fn frame(key: u32, height_m: f32, radius_m: f32) -> Option<Frame> {
    let c = centre(key)?;
    let x = Vec3::Y
        .cross(c)
        .try_normalize()
        .unwrap_or_else(|| c.any_orthonormal_vector());
    let z = x.cross(c);
    Some(Frame {
        origin: c * (radius_m + height_m),
        x,
        y: c,
        z,
    })
}

fn tri(buf: &mut MeshBuf, p: [Vec3; 3], uv: [Vec2; 3], toward: Vec3) {
    let mut n = (p[1] - p[0]).cross(p[2] - p[0]);
    let (p, uv) = if n.dot(toward) < 0.0 {
        n = -n;
        ([p[0], p[2], p[1]], [uv[0], uv[2], uv[1]])
    } else {
        (p, uv)
    };
    let n = n.normalize_or_zero();
    for k in 0..3 {
        buf.positions.push(p[k].to_array());
        buf.normals.push(n.to_array());
        buf.uvs.push(uv[k].to_array());
    }
}

fn quad(buf: &mut MeshBuf, p: [Vec3; 4], uv: [Vec2; 4], toward: Vec3) {
    tri(buf, [p[0], p[1], p[2]], [uv[0], uv[1], uv[2]], toward);
    tri(buf, [p[0], p[2], p[3]], [uv[0], uv[2], uv[3]], toward);
}

/// A plan outline turned counter-clockwise, as a floor wants it.
fn ccw(mut outline: Vec<Vec2>) -> Vec<Vec2> {
    let n = outline.len();
    let area: f32 = (0..n)
        .map(|i| outline[i].perp_dot(outline[(i + 1) % n]))
        .sum();
    if area < 0.0 {
        outline.reverse();
    }
    outline
}

/// Cut one road cell of `surface`: `road_neighbours` are the cells next to
/// it in its road (a bridge's rails run along every other side).
/// `repeat_m` gives a texture's repeat. `None` for a key no cell has.
pub fn cut(
    cell: &RoadCell,
    surface: Surface,
    road_neighbours: &[u32],
    radius_m: f32,
    repeat_m: &dyn Fn(&str) -> f32,
) -> Option<Piece> {
    let c = centre(cell.key)?;
    let ring = corners(cell.key);
    if ring.len() < 5 {
        return None;
    }
    // The top's lowest point: a half-height bed's half layer included.
    let low = cell.low_top_m();
    let fr = frame(cell.key, low, radius_m)?;
    let at = |d: Vec3, h: f32| d * (radius_m + h);
    let plan = |p: Vec3| {
        let l = fr.local(p);
        Vec2::new(l.x, l.z)
    };
    let bridge = matches!(cell.kind, Kind::Bridge { .. });
    let top_tex = if bridge { DECK } else { texture(surface) };
    let top_h = |d: Vec3| cell.top_at(d) + LIFT_M;
    let mut meshes = Meshes::new();
    // The top: a fan from the centre over the corners.
    {
        let rep = repeat_m(top_tex);
        let buf = meshes.entry(top_tex.to_string()).or_default();
        let mid = at(c, top_h(c));
        let uv = |p: Vec3| plan(p) / rep;
        for k in 0..ring.len() {
            let (a, b) = (ring[k], ring[(k + 1) % ring.len()]);
            let (pa, pb) = (at(a, top_h(a)), at(b, top_h(b)));
            tri(buf, [mid, pa, pb], [uv(mid), uv(pa), uv(pb)], c);
        }
    }
    // The sides: down to the cell's ground (a bed or a slope), or a deck's
    // underside on a bridge.
    let foot_h = if bridge {
        low - DECK_M
    } else {
        cell.ground_m() as f32
    };
    let side_tex = if bridge { TIMBER } else { SIDE };
    {
        let rep = repeat_m(side_tex);
        let buf = meshes.entry(side_tex.to_string()).or_default();
        for k in 0..ring.len() {
            let (a, b) = (ring[k], ring[(k + 1) % ring.len()]);
            let (ta, tb) = (top_h(a), top_h(b));
            if ta.max(tb) - foot_h < 0.05 {
                continue;
            }
            let out = ((a + b) * 0.5 - c).normalize_or_zero();
            let w = (a - b).length() * radius_m;
            quad(
                buf,
                [at(a, foot_h), at(b, foot_h), at(b, tb), at(a, ta)],
                [
                    Vec2::new(0.0, foot_h) / rep,
                    Vec2::new(w, foot_h) / rep,
                    Vec2::new(w, tb) / rep,
                    Vec2::new(0.0, ta) / rep,
                ],
                out,
            );
        }
        if bridge {
            // the deck's underside
            let mid = at(c, foot_h);
            for k in 0..ring.len() {
                let (a, b) = (ring[k], ring[(k + 1) % ring.len()]);
                let (pa, pb) = (at(a, foot_h), at(b, foot_h));
                tri(
                    buf,
                    [mid, pa, pb],
                    [plan(mid) / rep, plan(pa) / rep, plan(pb) / rep],
                    -c,
                );
            }
        }
    }
    let outline = ccw(ring.iter().map(|&d| plan(at(d, low))).collect());
    let reach_m = outline.iter().map(|p| p.length()).fold(0.0, f32::max) + 0.1;
    let mut solids = Vec::new();
    if bridge {
        let around = super::ring(cell.key);
        let rep = repeat_m(TIMBER);
        let buf = meshes.entry(TIMBER.to_string()).or_default();
        for k in 0..ring.len() {
            let (a, b) = (ring[k], ring[(k + 1) % ring.len()]);
            let side_mid = (a + b).normalize();
            // The neighbour across this side ([`super::corners`]).
            let across = around.get((k + 1) % ring.len());
            if across.is_some_and(|n| road_neighbours.contains(n)) {
                continue;
            }
            // A rail along the side, just inside it, and a post at its
            // first corner down to the bed.
            let (pa, pb) = (plan(at(a, low)), plan(at(b, low)));
            let inward = -plan(at(side_mid, low)).normalize_or_zero();
            let (ia, ib) = (pa + inward * RAIL_T, pb + inward * RAIL_T);
            solids.push(Solid {
                outline: ccw(vec![pa, pb, ib, ia]),
                y0: 0.0,
                y1: RAIL_M,
            });
            let up = |p: Vec2, h: f32| fr.world(Vec3::new(p.x, h, p.y));
            let uvp = |p: Vec2, h: f32| Vec2::new(p.x + p.y, h) / rep;
            let faces: [([Vec2; 2], Vec3); 3] = [
                ([pa, pb], (a + b) * 0.5 - c),
                ([ib, ia], c - (a + b) * 0.5),
                ([pa, ia], b - a),
            ];
            for ([p, q], toward) in faces {
                quad(
                    buf,
                    [up(p, 0.0), up(q, 0.0), up(q, RAIL_M), up(p, RAIL_M)],
                    [uvp(p, 0.0), uvp(q, 0.0), uvp(q, RAIL_M), uvp(p, RAIL_M)],
                    toward,
                );
            }
            quad(
                buf,
                [
                    up(pa, RAIL_M),
                    up(pb, RAIL_M),
                    up(ib, RAIL_M),
                    up(ia, RAIL_M),
                ],
                [uvp(pa, 0.0), uvp(pb, 0.0), uvp(ib, RAIL_T), uvp(ia, RAIL_T)],
                c,
            );
            // the post, from the bed to the deck's underside
            let bed = cell.base_m as f32 - low;
            let s = POST_M / 2.0;
            let p0 = pa + inward * (RAIL_T / 2.0);
            let along = (pb - pa).normalize_or_zero();
            let across2 = inward;
            let corners2 = [
                p0 - along * s - across2 * s,
                p0 + along * s - across2 * s,
                p0 + along * s + across2 * s,
                p0 - along * s + across2 * s,
            ];
            for i in 0..4 {
                let (p, q) = (corners2[i], corners2[(i + 1) % 4]);
                let mid2 = (p + q) * 0.5 - p0;
                let toward = fr.x * mid2.x + fr.z * mid2.y;
                quad(
                    buf,
                    [up(p, bed), up(q, bed), up(q, -DECK_M), up(p, -DECK_M)],
                    [uvp(p, bed), uvp(q, bed), uvp(q, 0.0), uvp(p, 0.0)],
                    toward,
                );
            }
        }
    }
    let surface_piece = match cell.slope() {
        Some((foot, head)) => {
            let (f, h) = (plan(at(foot, low)), plan(at(head, low)));
            let along = h - f;
            // Only the part that rises: the whole cell, or the part of it
            // a wide road's plane rises over (design 42m).
            let (s0, s1, rise) = cell.bed.ramp();
            pieces::Surface::Steps {
                outline: outline.clone(),
                foot: f + along * s0,
                dir: along.normalize_or_zero(),
                full: along.length() * (s1 - s0),
                base: LIFT_M,
                rise,
            }
        }
        None => pieces::Surface::Floor {
            outline: outline.clone(),
            top: LIFT_M,
            bottom: if bridge { -DECK_M } else { LIFT_M - 1.0 },
        },
    };
    Some(Piece {
        key: cell.key,
        meshes,
        solids: BuildingSolids {
            frame: fr,
            reach_m,
            solids,
            roof_plan: Vec::new(),
            surfaces: vec![surface_piece],
            doors: Vec::new(),
            rooms: Meshes::new(),
            lights: Vec::new(),
            top_m: 0.0,
            furniture: Vec::new(),
        },
        surface,
    })
}

#[cfg(test)]
mod tests {
    use super::super::{Kind, RoadCell, nearest_key, neighbours};
    use super::*;

    const R: f32 = 4_800.0;

    fn cell(up: Option<u32>, kind: Kind) -> RoadCell {
        let key = nearest_key(glam::Vec3::new(0.3, 0.6, 0.7)).unwrap();
        RoadCell {
            key,
            base_m: 12,
            kind,
            raised: false,
            cut_fill: 0,
            bed: Default::default(),
            up,
            from: None,
        }
    }

    /// The floor a walker stands on is the top that is drawn: at the
    /// slope's foot the low side, at its head a layer up, and the drawn
    /// top's height at its own corners is the floor's there.
    #[test]
    fn a_sloped_cell_is_stood_on_where_it_is_drawn() {
        let base = cell(None, Kind::Ground);
        let up = neighbours(base.key)[0];
        let c = RoadCell {
            up: Some(up),
            ..base
        };
        let p = cut(&c, Surface::Gravel, &[], R, &|_| 2.0).unwrap();
        let (foot, head) = c.slope().unwrap();
        let stand = |d: Vec3| {
            let local = p.solids.frame.local(d * (R + 20.0));
            p.solids.stand_in(local, 50.0).0.unwrap()
        };
        let toward = |d: Vec3, t: f32| centre(c.key).unwrap().lerp(d, t).normalize();
        assert!((stand(toward(foot, 0.98)) - LIFT_M).abs() < 0.03);
        assert!((stand(toward(head, 0.98)) - (1.0 + LIFT_M)).abs() < 0.03);
        assert!(p.meshes.contains_key("gravel") && p.meshes.contains_key(SIDE));
        // drawn heights over the sea radius against the floor's
        let gravel = &p.meshes["gravel"];
        for v in &gravel.positions {
            let v = Vec3::from_array(*v);
            let d = centre(c.key).unwrap().lerp(v.normalize(), 0.98).normalize();
            let drawn = c.top_at(d) + LIFT_M;
            assert!((v.length() - R - (c.top_at(v.normalize()) + LIFT_M)).abs() < 1e-3);
            let floor = stand(d) + c.low_m() as f32;
            assert!((drawn - floor).abs() < 0.02, "drawn {drawn}, floor {floor}");
        }
    }

    #[test]
    fn a_bridge_has_a_deck_rails_where_the_road_does_not_go_and_posts() {
        let b = cell(None, Kind::Bridge { deck_m: 15 });
        let ns = neighbours(b.key);
        let p = cut(&b, Surface::Dirt, &[ns[0], ns[3]], R, &|_| 2.0).unwrap();
        assert!(p.meshes.contains_key(DECK) && p.meshes.contains_key(TIMBER));
        assert_eq!(
            p.solids.solids.len(),
            ns.len() - 2,
            "a rail a side but the two"
        );
        let local = p.solids.frame.local(centre(b.key).unwrap() * (R + 15.5));
        let (floor, _) = p.solids.stand_in(local, 1.0);
        assert!((floor.unwrap() - LIFT_M).abs() < 1e-3, "the deck");
    }
}
