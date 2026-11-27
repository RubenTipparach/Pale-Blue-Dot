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

/// A bridge's rail card (design 42p): posts at its ends, a hand rail, a
/// mid rail and a bottom rail, clear between, mapped once across a side.
pub const RAIL_CARD: &str = "bridgerail";
/// A bridge's leg card: a leg at each end and cross braces, clear between,
/// repeating a metre down from the deck.
pub const TRESTLE_CARD: &str = "bridgetrestle";

/// A card: the quad `p` (its foot first, then its top) as two single-sided
/// faces back to back, one toward `out` and one away, `u` across it from 0
/// to 1 and `v` from `v[k]` (0 at its top edge).
fn card(buf: &mut MeshBuf, p: [Vec3; 4], v: [f32; 4], out: Vec3) {
    let uv = [
        Vec2::new(0.0, v[0]),
        Vec2::new(1.0, v[1]),
        Vec2::new(1.0, v[2]),
        Vec2::new(0.0, v[3]),
    ];
    quad(buf, p, uv, out);
    quad(buf, p, uv, -out);
}

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
        // The rails and the legs are cards (design 42p, the owner's survey
        // P7, 2026-11-26: "use a texture with alpha cutout instead of adding
        // geom"): flat, in a cut-out texture, each two single-sided faces back
        // to back, one facing out over the water and one in over the deck, so
        // it reads from both and only the one facing the eye is drawn.
        let around = super::ring(cell.key);
        for k in 0..ring.len() {
            let (a, b) = (ring[k], ring[(k + 1) % ring.len()]);
            let side_mid = (a + b).normalize();
            // The neighbour across this side ([`super::corners`]).
            let across = around.get((k + 1) % ring.len());
            if across.is_some_and(|n| road_neighbours.contains(n)) {
                continue;
            }
            // The rail's solid along the side, just inside it, as it was:
            // what stops the walker.
            let (pa, pb) = (plan(at(a, low)), plan(at(b, low)));
            let inward = -plan(at(side_mid, low)).normalize_or_zero();
            let (ia, ib) = (pa + inward * RAIL_T, pb + inward * RAIL_T);
            solids.push(Solid {
                outline: ccw(vec![pa, pb, ib, ia]),
                y0: 0.0,
                y1: RAIL_M,
            });
            // The cards stand on the rail's middle line.
            let (ca, cb) = (pa + inward * (RAIL_T / 2.0), pb + inward * (RAIL_T / 2.0));
            let up = |p: Vec2, h: f32| fr.world(Vec3::new(p.x, h, p.y));
            let out = (a + b) * 0.5 - c;
            // The rail: from the deck to the hand rail, mapped once.
            card(
                meshes.entry(RAIL_CARD.to_string()).or_default(),
                [up(ca, 0.0), up(cb, 0.0), up(cb, RAIL_M), up(ca, RAIL_M)],
                [1.0, 1.0, 0.0, 0.0],
                out,
            );
            // The legs, the land part: from the deck's underside down to the
            // bed or the bank, a repeat a metre from the deck down.
            let bed = cell.base_m as f32 - low;
            if -DECK_M - bed > 0.05 {
                let depth = -DECK_M - bed;
                card(
                    meshes.entry(TRESTLE_CARD.to_string()).or_default(),
                    [up(ca, bed), up(cb, bed), up(cb, -DECK_M), up(ca, -DECK_M)],
                    [depth, depth, 0.0, 0.0],
                    out,
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
    fn a_bridge_has_a_deck_and_rail_and_leg_cards_where_the_road_does_not_go() {
        let b = cell(None, Kind::Bridge { deck_m: 15 });
        let ns = neighbours(b.key);
        let p = cut(&b, Surface::Dirt, &[ns[0], ns[3]], R, &|_| 2.0).unwrap();
        assert!(p.meshes.contains_key(DECK) && p.meshes.contains_key(TIMBER));
        assert_eq!(
            p.solids.solids.len(),
            ns.len() - 2,
            "a rail a side but the two"
        );
        // The rails and legs are cards: no timber box over the deck, and on
        // each railed side a rail card and a leg card, each two triangles a
        // face and two faces back to back.
        let railed = ns.len() - 2;
        let rails = &p.meshes[RAIL_CARD];
        let legs = &p.meshes[TRESTLE_CARD];
        assert_eq!(rails.positions.len(), railed * 2 * 6, "two quads a rail");
        assert_eq!(legs.positions.len(), railed * 2 * 6, "two quads a leg card");
        let radius_of = |p: &[f32; 3]| Vec3::from_array(*p).length() - R;
        let deck_top = 15.0 + LIFT_M + 1e-3;
        assert!(
            p.meshes[TIMBER]
                .positions
                .iter()
                .all(|v| radius_of(v) <= deck_top),
            "no timber stands over the deck"
        );
        for buf in [rails, legs] {
            for face in buf.normals.chunks(12) {
                let (front, back) = (Vec3::from_array(face[0]), Vec3::from_array(face[6]));
                assert!(front.dot(back) < -0.99, "back to back: {front} {back}");
            }
            assert!(buf.uvs.iter().all(|uv| (0.0..=1.0).contains(&uv[0])));
        }
        let top = rails
            .positions
            .iter()
            .map(radius_of)
            .fold(f32::MIN, f32::max);
        assert!(
            (top - (15.0 + LIFT_M + RAIL_M)).abs() < 0.05,
            "the hand rail at {top}"
        );
        let foot = legs
            .positions
            .iter()
            .map(radius_of)
            .fold(f32::MAX, f32::min);
        assert!((foot - 12.0).abs() < 0.05, "the legs to the bed at {foot}");
        let local = p.solids.frame.local(centre(b.key).unwrap() * (R + 15.5));
        let (floor, _) = p.solids.stand_in(local, 1.0);
        assert!((floor.unwrap() - LIFT_M).abs() < 1e-3, "the deck");
    }
}
