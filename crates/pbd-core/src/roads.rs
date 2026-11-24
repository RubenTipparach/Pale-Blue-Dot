//! Roads (`townsfolk` design 10 and 11): road cells that slope instead of
//! step, in dirt or gravel, and the network the world lays between towns.
//!
//! **A road cell is an ordinary cell with a road top.** Its ground is the
//! generator's and is not edited: the road is a piece laid on it, as a
//! town's floors and street steps are. Where the next cell of the road
//! stands a layer higher, the top is a plane rising one layer from the
//! middle of the edge the road comes in by to the middle of the edge it
//! climbs out of ([`RoadCell::up`], [`RoadCell::from`]); straight through
//! that is one layer over the cell's 2.833 m. Never more than a layer a
//! cell, so a road has no step. A cell lower than both its neighbours in
//! the road (a dip one cell wide) is raised a layer on a bed of packed
//! earth, so it needs no two slopes.
//!
//! **The network is planned once** from the settlements and the seed
//! ([`plan`]): each settlement to its nearest neighbours within
//! [`RoadsConfig::link_m`], and every town of the largest kind to its
//! nearest such town. Each link is routed over the finest cells by ground
//! cost ([`route`]): a climb of more than a layer between cells is never
//! taken, shallow water is forded, deep water is bridged in timber for at
//! most [`RoadsConfig::max_bridge_cells`], the sea never. Cells the player
//! has changed are not crossed, so a road goes round the player's work.
//! The plan is stored as records the world authored ([`ROAD_RECORD`], one a
//! road, and [`ROAD_NET_RECORD`] saying the plan was made), so a world gains
//! its roads once, on the first open of a build that knows them, and keeps
//! them. Roads are not generation: no generator version moves for them.
//!
//! The lattice here (`neighbours`, `nearest_key`) walks the finest cells
//! by their exact keys ([`crate::cell_key`]), on whichever icosahedron face
//! each address lies, so it needs no tier and no patch.

use crate::cell_key::{self, Address};
use crate::records::{Record, Records};
use glam::Vec3;
use serde::{Deserialize, Serialize};
use std::cmp::Reverse;
use std::collections::{BTreeMap, BTreeSet, BinaryHeap, HashMap};

/// One road between two settlements, as the world laid it.
pub const ROAD_RECORD: &str = "road";
/// The plan's own record (id 0): that the network was planned, and by
/// which version of the planner. Its presence is what stops a world being
/// planned twice.
pub const ROAD_NET_RECORD: &str = "road-net";
/// A road cell the player built (`townsfolk` task 8.3), by its cell key.
pub const ROAD_CELL_RECORD: &str = "road-cell";
/// The schema of every road record kind.
pub const RECORD_SCHEMA: u32 = 1;
/// A `road` record with a cut or filled cell (`townsfolk` design 32): each
/// cell carries its cut or fill. A road with none is written in
/// [`RECORD_SCHEMA`], as every road before it was.
pub const CUT_FILL_SCHEMA: u32 = 2;
/// The planner's version, stored in the net record: 1 climbed a layer a
/// cell at most over the ground as it is; 2 may also cut and fill
/// (`townsfolk` design 32); 3 joins each settlement to at most one other,
/// so no network joins three (design 43). A world keeps the roads it was
/// planned with.
pub const NET_VERSION: u32 = 3;
/// The level the roads are laid on: the finest, the one the player edits.
pub const LEVEL: u8 = cell_key::LAST_LEVEL;

/// What a road is made of.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Surface {
    /// Packed earth: built with a shovel; the world's roads between villages.
    Dirt,
    /// Gravel: built with gravel; the world's roads between the large towns.
    /// Walked faster ([`RoadsConfig::gravel_speed`]).
    Gravel,
}

/// The roads' tunables (`assets/config/roads.ron`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RoadsConfig {
    /// How far a settlement looks for the neighbours it is joined to, m.
    pub link_m: f32,
    /// How many of its nearest neighbours within `link_m` a settlement is
    /// joined to.
    pub neighbours: usize,
    /// How far a town of the largest kind looks for its nearest such town, m.
    pub large_link_m: f32,
    /// Walking speed on a dirt road, times the ground's.
    pub dirt_speed: f32,
    /// Walking speed on a gravel road, times the ground's.
    pub gravel_speed: f32,
    /// The extra cost of a cell the road climbs or drops a layer into, as
    /// metres of level road it is worth.
    pub slope_cost_m: f32,
    /// The cost of a bridged cell, times its length.
    pub bridge_cost: f32,
    /// The cost of a forded cell, times its length.
    pub ford_cost: f32,
    /// Water at most this deep over its bed is forded, m.
    pub ford_depth_m: f32,
    /// The longest bridge, in cells.
    pub max_bridge_cells: u32,
    /// How many cells one road's search may look at before it gives up.
    pub max_expanded: usize,
    /// How far the search trusts its estimate: 1 finds the cheapest road
    /// and looks at the most cells; more finds a road a little dearer and
    /// looks at far fewer (weighted A*).
    pub search_weight: f32,
    /// Roads version 2 (`townsfolk` design 32): how many layers a road cell
    /// may stand over its ground on a bed of packed earth (fill), and under
    /// it in a cutting the world digs (cut). Zero for none, as version 1.
    #[serde(default)]
    pub max_fill_layers: u32,
    #[serde(default)]
    pub max_cut_layers: u32,
    /// What a layer of fill and of cut on a cell costs the search, as
    /// metres of level road it is worth.
    #[serde(default)]
    pub fill_cost_m: f32,
    #[serde(default)]
    pub cut_cost_m: f32,
    /// Lamps along the roads near a town (`townsfolk` design 34): within
    /// `lamp_town_m` of a settlement's anchor, every `lamp_every`th road
    /// cell has a lantern post beside it. Nought `lamp_every` for none.
    #[serde(default)]
    pub lamp_town_m: f32,
    #[serde(default)]
    pub lamp_every: u32,
    /// Roads version 3 (`townsfolk` design 43): the most settlements one
    /// network of the world's roads joins; nought for no limit (versions 1
    /// and 2: each to its nearest and the walled towns to each other). Only
    /// 2 is a rule today: each settlement joined to at most one other.
    #[serde(default)]
    pub max_network: u32,
    /// A survey stake's reach (design 39): the most cells between two
    /// stakes, and how many cells its search may look at.
    #[serde(default = "default_stake_cells")]
    pub max_stake_cells: u32,
    #[serde(default = "default_stake_expanded")]
    pub stake_expanded: usize,
    /// Seconds between two cells paved with the interact key held (design
    /// 40): the shovel's stroke.
    #[serde(default = "default_pave_beat")]
    pub pave_beat_s: f32,
    /// How fast a wheel runs on each surface, times its road speed
    /// (`land-transport` design 9). Only wheels read it; the walker keeps
    /// `dirt_speed` and `gravel_speed`.
    #[serde(default)]
    pub wheel: WheelFactors,
}

/// What a wheel stands on (`land-transport` design 9).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WheelGround {
    /// A gravel road cell.
    Gravel,
    /// A dirt road cell.
    Dirt,
    /// A forded road cell, under its river's water.
    Ford,
    /// A road's timber bridge deck.
    Bridge,
    /// A town's own lanes and streets (its footprint).
    Town,
    /// Anything else: grass, sand, snow, scree.
    Off,
}

/// A wheel's speed on each surface, times its road speed.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WheelFactors {
    /// A gravel road, a bridge deck or a town's lanes.
    pub gravel: f32,
    pub dirt: f32,
    pub ford: f32,
    /// Off every road (the owner: "like 1/4 speed").
    pub off_road: f32,
}

impl Default for WheelFactors {
    fn default() -> Self {
        Self {
            gravel: 1.0,
            dirt: 0.6,
            ford: 0.5,
            off_road: 0.25,
        }
    }
}

impl WheelFactors {
    /// The factor of one surface.
    pub fn of(&self, ground: WheelGround) -> f32 {
        match ground {
            WheelGround::Gravel | WheelGround::Bridge | WheelGround::Town => self.gravel,
            WheelGround::Dirt => self.dirt,
            WheelGround::Ford => self.ford,
            WheelGround::Off => self.off_road,
        }
    }
}

/// The factor a wheeled vehicle runs at with its wheels on `under`: the
/// lowest of them, so a cart half off the road is slowed (design 9). No
/// wheel down reads as off the road.
pub fn wheel_factor(factors: &WheelFactors, under: &[WheelGround]) -> f32 {
    under
        .iter()
        .map(|&g| factors.of(g))
        .reduce(f32::min)
        .unwrap_or(factors.off_road)
}

/// What a wheel on a road cell stands on: its kind and its surface.
pub fn wheel_ground(kind: Kind, surface: Surface) -> WheelGround {
    match (kind, surface) {
        (Kind::Bridge { .. }, _) => WheelGround::Bridge,
        (Kind::Ford, _) => WheelGround::Ford,
        (Kind::Ground, Surface::Gravel) => WheelGround::Gravel,
        (Kind::Ground, Surface::Dirt) => WheelGround::Dirt,
    }
}

fn default_stake_cells() -> u32 {
    24
}

fn default_stake_expanded() -> usize {
    4_000
}

fn default_pave_beat() -> f32 {
    0.45
}

impl RoadsConfig {
    pub fn validate(&self) -> Result<(), String> {
        let positive = |name: &str, v: f32| {
            (v.is_finite() && v > 0.0)
                .then_some(())
                .ok_or(format!("{name} must be positive"))
        };
        positive("link_m", self.link_m)?;
        positive("large_link_m", self.large_link_m)?;
        positive("dirt_speed", self.dirt_speed)?;
        positive("gravel_speed", self.gravel_speed)?;
        positive("bridge_cost", self.bridge_cost)?;
        positive("ford_cost", self.ford_cost)?;
        if !(self.slope_cost_m.is_finite() && self.slope_cost_m >= 0.0) {
            return Err("slope_cost_m must be zero or more".into());
        }
        if !(self.ford_depth_m.is_finite() && self.ford_depth_m >= 0.0) {
            return Err("ford_depth_m must be zero or more".into());
        }
        if !(self.search_weight.is_finite() && self.search_weight >= 1.0) {
            return Err("search_weight must be at least 1".into());
        }
        if self.neighbours == 0 || self.max_expanded == 0 {
            return Err("neighbours and max_expanded must be at least one".into());
        }
        for (name, v) in [
            ("fill_cost_m", self.fill_cost_m),
            ("cut_cost_m", self.cut_cost_m),
        ] {
            if !(v.is_finite() && v >= 0.0) {
                return Err(format!("{name} must be zero or more"));
            }
        }
        let w = &self.wheel;
        for (name, v) in [
            ("wheel.gravel", w.gravel),
            ("wheel.dirt", w.dirt),
            ("wheel.ford", w.ford),
            ("wheel.off_road", w.off_road),
        ] {
            if !(v.is_finite() && v > 0.0 && v <= 1.0) {
                return Err(format!("{name} must be over 0 and at most 1"));
            }
        }
        if self.max_fill_layers > 4 || self.max_cut_layers > 4 {
            return Err("max_fill_layers and max_cut_layers are at most 4".into());
        }
        if !matches!(self.max_network, 0 | 2) {
            return Err("max_network is 0 (no limit) or 2".into());
        }
        if self.max_stake_cells == 0 || self.stake_expanded == 0 {
            return Err("max_stake_cells and stake_expanded must be at least one".into());
        }
        positive("pave_beat_s", self.pave_beat_s)?;
        Ok(())
    }

    /// The planner as version `version` planned (`NET_VERSION`): version 1
    /// neither cuts nor fills. Kept so a version once shipped plans as it
    /// did.
    pub fn for_version(&self, version: u32) -> Self {
        let mut cfg = self.clone();
        if version < 2 {
            cfg.max_fill_layers = 0;
            cfg.max_cut_layers = 0;
        }
        if version < 3 {
            cfg.max_network = 0;
        }
        cfg
    }

    /// How much faster than the ground a road of `surface` is walked.
    pub fn speed(&self, surface: Surface) -> f32 {
        match surface {
            Surface::Dirt => self.dirt_speed,
            Surface::Gravel => self.gravel_speed,
        }
    }
}

impl Default for RoadsConfig {
    /// The values `assets/config/roads.ron` ships, for tests.
    fn default() -> Self {
        Self {
            link_m: 2_000.0,
            neighbours: 2,
            large_link_m: 6_000.0,
            dirt_speed: 1.0,
            gravel_speed: 1.15,
            slope_cost_m: 2.0,
            bridge_cost: 4.0,
            ford_cost: 2.0,
            ford_depth_m: 1.0,
            max_bridge_cells: 10,
            max_expanded: 100_000,
            search_weight: 1.5,
            max_fill_layers: 2,
            max_cut_layers: 2,
            fill_cost_m: 20.0,
            cut_cost_m: 25.0,
            lamp_town_m: 90.0,
            lamp_every: 5,
            max_network: 2,
            max_stake_cells: 24,
            stake_expanded: 4_000,
            pave_beat_s: 0.45,
            wheel: WheelFactors::default(),
        }
    }
}

// ---- The lattice -----------------------------------------------------------

/// The six steps between lattice neighbours on a face, in `(i, j)`.
const STEPS: [(i32, i32); 6] = [(1, 0), (-1, 0), (0, 1), (0, -1), (1, -1), (-1, 1)];

/// The keys of the cells across a cell's sides, in key order: six, or five
/// at the twelve pentagons. A point on a face's edge or corner is reached
/// on every face it lies on, so the cells across a face's edge are found
/// with the rest. Empty for a key no cell has.
pub fn neighbours(key: u32) -> Vec<u32> {
    let Some(a) = cell_key::unpack(key) else {
        return Vec::new();
    };
    let mut out = BTreeSet::new();
    for at in cell_key::addresses(a) {
        let side = at.side() as i64;
        for (di, dj) in STEPS {
            let (i, j) = (at.i as i64 + di as i64, at.j as i64 + dj as i64);
            if i < 0 || j < 0 || i + j > side {
                continue;
            }
            let n = Address {
                face: at.face,
                level: at.level,
                i: i as u32,
                j: j as u32,
            };
            if let Some(k) = cell_key::key(n)
                && k != key
            {
                out.insert(k);
            }
        }
    }
    out.into_iter().collect()
}

/// Where a cell lies: a unit vector. `None` for a key no cell has.
pub fn centre(key: u32) -> Option<Vec3> {
    cell_key::key_direction(key)
}

/// The finest cell whose centre is nearest `direction`: the face it falls
/// on, the lattice point its gnomonic weights round to, then a walk to the
/// neighbour nearer it until none is.
pub fn nearest_key(direction: Vec3) -> Option<u32> {
    let d = direction.try_normalize()?;
    let (vertices, faces) = crate::topology::icosahedron();
    let side = (1u32 << LEVEL) as f32;
    let mut best: Option<(f32, Address)> = None;
    for (f, tri) in faces.iter().enumerate() {
        let (a, b, c) = (vertices[tri[0]], vertices[tri[1]], vertices[tri[2]]);
        let n = (b - a).cross(c - a);
        let t = a.dot(n) / d.dot(n);
        if !t.is_finite() || t <= 0.0 {
            continue;
        }
        let p = d * t;
        // Barycentric weights of p in the face.
        let (v0, v1, v2) = (b - a, c - a, p - a);
        let (d00, d01, d11, d20, d21) =
            (v0.dot(v0), v0.dot(v1), v1.dot(v1), v2.dot(v0), v2.dot(v1));
        let den = d00 * d11 - d01 * d01;
        let wb = (d11 * d20 - d01 * d21) / den;
        let wc = (d00 * d21 - d01 * d20) / den;
        let wa = 1.0 - wb - wc;
        let worst = wa.min(wb).min(wc);
        if best.is_none_or(|(w, _)| worst > w) {
            let i = (wb * side).round().clamp(0.0, side) as u32;
            let j = ((wc * side).round().clamp(0.0, side) as u32).min(side as u32 - i);
            best = Some((
                worst,
                Address {
                    face: f as u8,
                    level: LEVEL,
                    i,
                    j,
                },
            ));
        }
    }
    let mut key = cell_key::key(best?.1)?;
    // Compared in f64: neighbouring centres differ from a direction by a
    // few ulps of an f32 dot.
    let far = |k: u32| centre(k).map(|c| (c.as_dvec3() - d.as_dvec3()).length_squared());
    let mut dist = far(key)?;
    loop {
        let next = neighbours(key)
            .into_iter()
            .filter_map(|n| Some((n, far(n)?)))
            .min_by(|a, b| a.1.total_cmp(&b.1).then(a.0.cmp(&b.0)));
        match next {
            Some((n, nd)) if nd < dist => {
                key = n;
                dist = nd;
            }
            _ => return Some(key),
        }
    }
}

/// A cell's neighbours in turn round it, counter-clockwise seen from
/// outside.
pub fn ring(key: u32) -> Vec<u32> {
    let Some(c) = centre(key) else {
        return Vec::new();
    };
    let tangent = c.any_orthonormal_vector();
    let bitangent = c.cross(tangent);
    let mut ring: Vec<(f32, u32)> = neighbours(key)
        .into_iter()
        .filter_map(|n| {
            let d = centre(n)?;
            Some((d.dot(bitangent).atan2(d.dot(tangent)), n))
        })
        .collect();
    ring.sort_by(|a, b| a.0.total_cmp(&b.0));
    ring.into_iter().map(|(_, n)| n).collect()
}

/// A cell's corners, counter-clockwise about its centre seen from outside:
/// each the normalised sum of the centre and two neighbours in turn, as
/// the dual's corners are ([`crate::topology::dual_from_triangles`]). The
/// side from corner `k` to corner `k + 1` is shared with `ring(key)[k + 1]`.
pub fn corners(key: u32) -> Vec<Vec3> {
    let Some(c) = centre(key) else {
        return Vec::new();
    };
    let ring: Vec<Vec3> = ring(key).into_iter().filter_map(centre).collect();
    let k = ring.len();
    (0..k)
        .map(|s| (c + ring[s] + ring[(s + 1) % k]).normalize())
        .collect()
}

/// The middle of the side two neighbouring cells share: halfway between
/// their centres, on the sphere.
pub fn edge_middle(a: u32, b: u32) -> Option<Vec3> {
    Some((centre(a)? + centre(b)?).normalize())
}

// ---- The ground a road is laid on ------------------------------------------

/// The water over a cell, as a road meets it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Water {
    Dry,
    /// A river's water, `depth_m` over its bed, with its surface at
    /// `surface_m` (metres over the sea radius).
    River {
        depth_m: i32,
        surface_m: i32,
    },
    /// The sea, which no road crosses.
    Sea,
}

/// What a road needs to know of one cell.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Ground {
    /// The top of its ground, metres over the sea radius, on a layer.
    pub top_m: i32,
    pub water: Water,
    /// In a town's footprint: crossed, never laid (the town's lanes are the
    /// road there).
    pub town: bool,
    /// Changed by the player: never crossed (design 11).
    pub player: bool,
}

/// What a road does on one of its cells.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Kind {
    /// A road top on the cell's ground.
    Ground,
    /// A road top on a river's bed, under its water.
    Ford,
    /// A timber deck at `deck_m`, metres over the sea radius.
    Bridge { deck_m: i32 },
}

/// A wide road's bed (`townsfolk` design 42m): the road top read off the
/// road's one plane at the cell's own place, so lanes meet with no seam.
/// Its top stands `lift_mm` over its whole layer and rises `rise_mm` between
/// `from` and `to` thousandths of the way from its foot to its head, level
/// either side. All nought is the plain cell: a whole-cell rise of a layer
/// on a whole layer, every road of the world's and every one-wide road of
/// the player's.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Bed {
    #[serde(default)]
    pub lift_mm: u16,
    #[serde(default)]
    pub from: u16,
    #[serde(default)]
    pub to: u16,
    #[serde(default)]
    pub rise_mm: u16,
}

impl Bed {
    /// The plain cell's.
    pub fn is_plain(&self) -> bool {
        *self == Bed::default()
    }

    /// How far the top stands over its whole layer, m.
    pub fn lift_m(&self) -> f32 {
        f32::from(self.lift_mm) / 1000.0
    }

    /// Where the rise starts and ends (shares of the way from foot to
    /// head) and how far it rises, m.
    pub fn ramp(&self) -> (f32, f32, f32) {
        if self.is_plain() {
            return (0.0, 1.0, 1.0);
        }
        let (a, b) = (f32::from(self.from) / 1000.0, f32::from(self.to) / 1000.0);
        (a, b.max(a + 1e-3), f32::from(self.rise_mm) / 1000.0)
    }

    /// How far the top has risen at `t` (0 at the foot, 1 at the head), m.
    pub fn risen(&self, t: f32) -> f32 {
        let (a, b, rise) = self.ramp();
        ((t - a) / (b - a)).clamp(0.0, 1.0) * rise
    }
}

/// One cell of a road, as laid.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RoadCell {
    pub key: u32,
    /// The top of the cell's ground when the road was planned, metres over
    /// the sea radius: where the road top stands (or a raised bed's foot).
    pub base_m: i32,
    pub kind: Kind,
    /// Raised a layer on a bed of packed earth: the road top is `base_m + 1`.
    pub raised: bool,
    /// Roads version 2 (`townsfolk` design 32): layers of fill under the
    /// road top (more than nought), or of cut into its ground (less), the
    /// world having dug them. Nought on every version 1 road.
    pub cut_fill: i32,
    /// A wide road's bed (`townsfolk` design 42m): plain on every road of
    /// the world's and every one-wide road of the player's.
    pub bed: Bed,
    /// The neighbour the top rises a layer toward, where it slopes.
    pub up: Option<u32>,
    /// The neighbour whose side the slope starts from, at `low_m`; the side
    /// opposite `up` where the road ends on this cell.
    pub from: Option<u32>,
}

impl RoadCell {
    /// The road top's low side, metres over the sea radius: the deck on a
    /// bridge, the bed's top on a raised cell, the ground otherwise.
    pub fn low_m(&self) -> i32 {
        match self.kind {
            Kind::Bridge { deck_m } => deck_m,
            _ => self.base_m + self.cut_fill + i32::from(self.raised),
        }
    }

    /// The top of the ground under the road, metres over the sea radius:
    /// `base_m`, or the floor of its cutting where it is cut.
    pub fn ground_m(&self) -> i32 {
        self.base_m + self.cut_fill.min(0)
    }

    /// The slope's foot and head on the sphere: the middle of the side it
    /// starts from and of the side it climbs out of. `None` where it is flat.
    pub fn slope(&self) -> Option<(Vec3, Vec3)> {
        let up = self.up?;
        let head = edge_middle(self.key, up)?;
        let foot = match self.from {
            Some(from) => edge_middle(self.key, from)?,
            // The side opposite `up`: the centre mirrored past the head.
            None => (centre(self.key)? * 2.0 - head).normalize(),
        };
        Some((foot, head))
    }

    /// The road top's height at `direction` in the cell, metres over the
    /// sea radius: on the plane from the foot (low) to the head (a layer
    /// up), measured along the line between them and held level past each.
    pub fn top_at(&self, direction: Vec3) -> f32 {
        let low = self.low_top_m();
        let Some((foot, head)) = self.slope() else {
            return low;
        };
        let along = head - foot;
        let t = (direction.normalize() - foot).dot(along) / along.length_squared().max(1e-20);
        low + self.bed.risen(t)
    }

    /// The road top's lowest point, metres over the sea radius: `low_m`,
    /// and its bed's lift on a wide road's bed (design 42m).
    pub fn low_top_m(&self) -> f32 {
        self.low_m() as f32 + self.bed.lift_m()
    }
}

/// A road, as the world laid it.
#[derive(Clone, Debug, PartialEq)]
pub struct Road {
    /// The settlements it joins, by site id, the lower first.
    pub from_site: u32,
    pub to_site: u32,
    pub surface: Surface,
    /// Its cells from `from_site` toward `to_site`, without the towns' own.
    pub cells: Vec<RoadCell>,
}

/// A settlement as the planner sees it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Settlement {
    pub site: u32,
    pub direction: Vec3,
    /// Of the largest kind (a walled town): joined to its nearest such town,
    /// in gravel.
    pub large: bool,
}

/// Which settlements are joined (design 11): each to its `neighbours`
/// nearest within `link_m`, and each large town to its nearest large town
/// within `large_link_m`. Pairs by site id, the lower first, in order.
pub fn links(settlements: &[Settlement], cfg: &RoadsConfig, radius_m: f32) -> Vec<(u32, u32)> {
    if cfg.max_network == 2 {
        return pairs(settlements, cfg, radius_m);
    }
    let mut pairs = BTreeSet::new();
    let arc =
        |a: &Settlement, b: &Settlement| crate::sites::arc_m(a.direction, b.direction, radius_m);
    let pair = |a: u32, b: u32| (a.min(b), a.max(b));
    for a in settlements {
        let mut near: Vec<(f32, u32)> = settlements
            .iter()
            .filter(|b| b.site != a.site)
            .map(|b| (arc(a, b), b.site))
            .filter(|(m, _)| *m <= cfg.link_m)
            .collect();
        near.sort_by(|x, y| x.0.total_cmp(&y.0).then(x.1.cmp(&y.1)));
        for (_, b) in near.into_iter().take(cfg.neighbours) {
            pairs.insert(pair(a.site, b));
        }
        if a.large
            && let Some((_, b)) = settlements
                .iter()
                .filter(|b| b.large && b.site != a.site)
                .map(|b| (arc(a, b), b.site))
                .filter(|(m, _)| *m <= cfg.large_link_m)
                .min_by(|x, y| x.0.total_cmp(&y.0).then(x.1.cmp(&y.1)))
        {
            pairs.insert(pair(a.site, b));
        }
    }
    pairs.into_iter().collect()
}

/// Roads version 3's links (`townsfolk` design 43): every pair within
/// `link_m`, nearest first (then by site ids), taken where neither town is
/// joined yet. Each settlement is in one pair at most.
fn pairs(settlements: &[Settlement], cfg: &RoadsConfig, radius_m: f32) -> Vec<(u32, u32)> {
    let mut all: Vec<(f32, u32, u32)> = Vec::new();
    for (i, a) in settlements.iter().enumerate() {
        for b in &settlements[i + 1..] {
            let m = crate::sites::arc_m(a.direction, b.direction, radius_m);
            if m <= cfg.link_m && a.site != b.site {
                all.push((m, a.site.min(b.site), a.site.max(b.site)));
            }
        }
    }
    all.sort_by(|x, y| x.0.total_cmp(&y.0).then((x.1, x.2).cmp(&(y.1, y.2))));
    let mut joined = BTreeSet::new();
    let mut out = Vec::new();
    for (_, a, b) in all {
        if joined.contains(&a) || joined.contains(&b) {
            continue;
        }
        joined.insert(a);
        joined.insert(b);
        out.push((a, b));
    }
    out.sort();
    out
}

/// The road's surface for a link (design 10, recommendation taken): gravel
/// between two large towns, dirt otherwise.
pub fn surface_of(a: &Settlement, b: &Settlement) -> Surface {
    if a.large && b.large {
        Surface::Gravel
    } else {
        Surface::Dirt
    }
}

// ---- The route ---------------------------------------------------------------

/// A search state: a cell, and on a bridge how long it has run and the
/// deck it carries.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct State {
    key: u32,
    run: u32,
    deck: i32,
    /// On the ground: the road's height over the cell's ground, layers
    /// (fill above nought, cut below; version 2).
    lift: i32,
}

/// One step of a route: a cell and what the road does there, before
/// profiling.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Step {
    pub key: u32,
    pub ground: Ground,
    /// The deck, where this cell is bridged.
    pub deck: Option<i32>,
    /// The road's height over the ground where it is cut or filled, layers.
    pub lift: i32,
}

/// What a road asks of the ground: a cell's [`Ground`] by key, from any
/// thread ([`plan`] routes its links side by side).
pub type GroundFn<'a> = dyn Fn(u32) -> Option<Ground> + Sync + 'a;

/// The cheapest way for a road from cell `from` to cell `to` over the
/// ground `ground` answers (memoised here: each cell is asked once, and its
/// centre and neighbours found once), by A* with the great-circle distance
/// as its estimate. `None` where there is none within the search's budget.
pub fn route(
    from: u32,
    to: u32,
    ground: &GroundFn,
    cfg: &RoadsConfig,
    radius_m: f32,
) -> Option<Vec<Step>> {
    let goal = centre(to)?;
    let mut cache: HashMap<u32, Option<Ground>> = HashMap::new();
    let ask = |key: u32, cache: &mut HashMap<u32, Option<Ground>>| -> Option<Ground> {
        *cache.entry(key).or_insert_with(|| ground(key))
    };
    let mut centres: HashMap<u32, Option<Vec3>> = HashMap::new();
    let mut centre = |key: u32| *centres.entry(key).or_insert_with(|| centre(key));
    let mut rings: HashMap<u32, Vec<u32>> = HashMap::new();
    let arc = |a: Vec3, b: Vec3| crate::sites::arc_m(a, b, radius_m);
    // Costs in millimetres, whole, so the order is total and the same on
    // every machine.
    let mm = |m: f32| (m * 1000.0).round().max(0.0) as u64;
    let start_ground = ask(from, &mut cache)?;
    if start_ground.player || start_ground.water != Water::Dry {
        return None;
    }
    let start = State {
        key: from,
        run: 0,
        deck: 0,
        lift: 0,
    };
    let mut best: HashMap<State, u64> = HashMap::from([(start, 0)]);
    let mut came: HashMap<State, State> = HashMap::new();
    let mut open = BinaryHeap::new();
    let estimate = |c: Vec3| mm(arc(c, goal) * cfg.search_weight);
    open.push(Reverse((estimate(centre(from)?), 0u64, start)));
    let mut expanded = 0usize;
    while let Some(Reverse((_, g, at))) = open.pop() {
        if best.get(&at).is_some_and(|&b| b < g) {
            continue;
        }
        if at.key == to && at.run == 0 && at.lift == 0 {
            let mut path = vec![at];
            let mut cur = at;
            while let Some(&prev) = came.get(&cur) {
                path.push(prev);
                cur = prev;
            }
            path.reverse();
            return Some(
                path.into_iter()
                    .map(|s| Step {
                        key: s.key,
                        ground: cache[&s.key].expect("a cell on the route was answered"),
                        deck: (s.run > 0).then_some(s.deck),
                        lift: if s.run > 0 { 0 } else { s.lift },
                    })
                    .collect(),
            );
        }
        expanded += 1;
        if expanded > cfg.max_expanded {
            return None;
        }
        let here = ask(at.key, &mut cache)?;
        let here_c = centre(at.key)?;
        // The height a road stands at here: a bridge's deck, else the ground.
        let here_h = if at.run > 0 {
            at.deck
        } else {
            here.top_m + at.lift
        };
        let around = rings
            .entry(at.key)
            .or_insert_with(|| neighbours(at.key))
            .clone();
        for n in around {
            let Some(there) = ask(n, &mut cache) else {
                continue;
            };
            // The one cell rule (design 42l): never the player's work or the
            // sea; the world crosses towns and bridges deep water.
            if matches!(
                check_cell(&there, false),
                Some(Clash::InTheWay | Clash::OpenWater)
            ) {
                continue;
            }
            let Some(there_c) = centre(n) else { continue };
            let len = arc(here_c, there_c);
            let mut next: Vec<(State, u64)> = Vec::new();
            let bridge = planner::bridged(
                there.water,
                at.run,
                at.deck,
                here_h,
                cfg.ford_depth_m,
                cfg.max_bridge_cells,
            );
            match (there.water, bridge) {
                (Water::Sea, _) => continue,
                // Bridged (design 10a, 42m: the one rule): the deck is the
                // bank's height, carried on, and must stand over the water.
                (_, Some(b)) => {
                    let Some((deck, run)) = b else { continue };
                    let s = State {
                        key: n,
                        run,
                        deck,
                        lift: 0,
                    };
                    next.push((s, mm(len * cfg.bridge_cost)));
                }
                (Water::River { .. } | Water::Dry, None) => {
                    // On the ground (a ford's ground is its bed): at most a
                    // layer from where the road stands now. On dry ground
                    // outside a town the road may stand over its ground on
                    // fill or under it in a cutting (version 2).
                    // Only where the ground itself cannot be taken (more
                    // than a layer from the road's height here), so a road
                    // cuts and fills only where it must, and a ramp goes on
                    // cell by cell only while the ground still is.
                    let free = there.water == Water::Dry
                        && !there.town
                        && (there.top_m - here_h).abs() > 1;
                    let (lo, hi) = if free {
                        (-(cfg.max_cut_layers as i32), cfg.max_fill_layers as i32)
                    } else {
                        (0, 0)
                    };
                    let step = planner::MAX_STEP_LAYERS;
                    for h in here_h - step..=here_h + step {
                        if check_step(here_h, h).is_some() {
                            continue;
                        }
                        let lift = h - there.top_m;
                        if lift < lo || lift > hi {
                            continue;
                        }
                        let mut cost = len;
                        if matches!(there.water, Water::River { .. }) {
                            cost *= cfg.ford_cost;
                        }
                        if h != here_h {
                            cost += cfg.slope_cost_m;
                        }
                        cost += cfg.fill_cost_m * lift.max(0) as f32
                            + cfg.cut_cost_m * (-lift).max(0) as f32;
                        let s = State {
                            key: n,
                            run: 0,
                            deck: 0,
                            lift,
                        };
                        next.push((s, mm(cost)));
                    }
                }
            }
            for (s, step) in next {
                let g2 = g + step;
                if best.get(&s).is_some_and(|&b| b <= g2) {
                    continue;
                }
                best.insert(s, g2);
                came.insert(s, at);
                open.push(Reverse((g2 + estimate(there_c), g2, s)));
            }
        }
    }
    None
}

/// A route made a road (design 10): each cell's road top, raised a layer
/// where it is lower than both its neighbours in the road, sloped up toward
/// the neighbour a layer higher. The towns' cells are left out. `None` if
/// any two neighbours in the road stand more than a layer apart.
pub fn profile(steps: &[Step]) -> Option<Vec<RoadCell>> {
    let n = steps.len();
    let mut h: Vec<i32> = steps
        .iter()
        .map(|s| s.deck.unwrap_or(s.ground.top_m + s.lift))
        .collect();
    let fixed = |k: usize| steps[k].deck.is_some() || steps[k].ground.town || steps[k].lift != 0;
    let mut raised = vec![false; n];
    // A dip one cell wide is raised to its neighbours, until there is none.
    for _ in 0..4 {
        let mut changed = false;
        for k in 1..n.saturating_sub(1) {
            if !fixed(k) && !raised[k] && h[k] < h[k - 1] && h[k] < h[k + 1] {
                h[k] += 1;
                raised[k] = true;
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    if h.windows(2).any(|w| check_step(w[0], w[1]).is_some()) {
        return None;
    }
    let mut out = Vec::new();
    for (k, s) in steps.iter().enumerate() {
        if s.ground.town {
            continue;
        }
        let kind = match (s.deck, s.ground.water) {
            (Some(deck_m), _) => Kind::Bridge { deck_m },
            (None, Water::River { .. }) => Kind::Ford,
            _ => Kind::Ground,
        };
        let (mut up, mut from) = (None, None);
        if s.deck.is_none() {
            for (j, o) in [(k + 1, k.wrapping_sub(1)), (k.wrapping_sub(1), k + 1)] {
                if j < n && h[j] == h[k] + 1 {
                    up = Some(steps[j].key);
                    from = (o < n).then(|| steps[o].key);
                    break;
                }
            }
        }
        out.push(RoadCell {
            key: s.key,
            base_m: s.ground.top_m,
            kind,
            raised: raised[k],
            cut_fill: if s.deck.is_some() { 0 } else { s.lift },
            bed: Default::default(),
            up,
            from,
        });
    }
    Some(out)
}

/// Plan the network (design 11): every link routed and profiled, side by
/// side on up to `threads` threads, and the roads given back in link order
/// whatever finished first. A settlement's road starts from the cell at its
/// anchor. A link with no route is left out.
pub fn plan(
    settlements: &[Settlement],
    ground: &GroundFn,
    cfg: &RoadsConfig,
    radius_m: f32,
    threads: usize,
) -> Vec<Road> {
    let by_site: BTreeMap<u32, &Settlement> = settlements.iter().map(|s| (s.site, s)).collect();
    let links = links(settlements, cfg, radius_m);
    let one = |(a, b): (u32, u32)| -> Option<Road> {
        let (sa, sb) = (by_site[&a], by_site[&b]);
        let steps = route(
            nearest_key(sa.direction)?,
            nearest_key(sb.direction)?,
            ground,
            cfg,
            radius_m,
        )?;
        // Version 3: a road through a third town's footprint would join it.
        if cfg.max_network == 2 && crosses_a_third(&steps, a, b, settlements, radius_m) {
            return None;
        }
        Some(Road {
            from_site: a,
            to_site: b,
            surface: surface_of(sa, sb),
            cells: profile(&steps)?,
        })
    };
    let threads = threads.clamp(1, links.len().max(1));
    let mut done: Vec<(usize, Road)> = std::thread::scope(|scope| {
        let workers: Vec<_> = (0..threads)
            .map(|t| {
                let links = &links;
                let one = &one;
                scope.spawn(move || {
                    links
                        .iter()
                        .enumerate()
                        .skip(t)
                        .step_by(threads)
                        .filter_map(|(k, &link)| one(link).map(|r| (k, r)))
                        .collect::<Vec<_>>()
                })
            })
            .collect();
        workers
            .into_iter()
            .flat_map(|w| w.join().expect("a road's search does not panic"))
            .collect()
    });
    done.sort_by_key(|(k, _)| *k);
    let roads: Vec<Road> = done.into_iter().map(|(_, r)| r).collect();
    if cfg.max_network == 2 {
        return apart(roads);
    }
    roads
}

/// Whether a route enters the footprint of a settlement other than its two
/// ends (version 3): a footprint cell whose nearest settlement is neither.
fn crosses_a_third(
    steps: &[Step],
    a: u32,
    b: u32,
    settlements: &[Settlement],
    radius_m: f32,
) -> bool {
    steps.iter().filter(|s| s.ground.town).any(|s| {
        let Some(c) = centre(s.key) else {
            return false;
        };
        settlements
            .iter()
            .min_by(|x, y| {
                crate::sites::arc_m(c, x.direction, radius_m)
                    .total_cmp(&crate::sites::arc_m(c, y.direction, radius_m))
                    .then(x.site.cmp(&y.site))
            })
            .is_some_and(|n| n.site != a && n.site != b)
    })
}

/// Version 3: the roads in link order, each left out where it shares or
/// touches a cell of one kept before it, so no two pairs meet on the ground.
fn apart(roads: Vec<Road>) -> Vec<Road> {
    let mut near: BTreeSet<u32> = BTreeSet::new();
    let mut kept = Vec::new();
    for road in roads {
        if road.cells.iter().any(|c| near.contains(&c.key)) {
            continue;
        }
        for c in &road.cells {
            near.insert(c.key);
            near.extend(neighbours(c.key));
        }
        kept.push(road);
    }
    kept
}

/// The networks a set of roads makes: settlements joined by a road, or by
/// roads whose cells share or touch, as sets of site ids. A settlement with
/// no road is in none.
pub fn networks(roads: &[Road]) -> Vec<BTreeSet<u32>> {
    fn find(p: &mut [usize], i: usize) -> usize {
        let mut i = i;
        while p[i] != i {
            p[i] = p[p[i]];
            i = p[i];
        }
        i
    }
    let mut parent: Vec<usize> = (0..roads.len()).collect();
    let mut owner: BTreeMap<u32, usize> = BTreeMap::new();
    let mut by_site: BTreeMap<u32, usize> = BTreeMap::new();
    for (i, r) in roads.iter().enumerate() {
        let mut others = Vec::new();
        for site in [r.from_site, r.to_site] {
            if let Some(&j) = by_site.get(&site) {
                others.push(j);
            }
            by_site.insert(site, i);
        }
        for c in &r.cells {
            for k in std::iter::once(c.key).chain(neighbours(c.key)) {
                if let Some(&j) = owner.get(&k) {
                    others.push(j);
                }
            }
        }
        for j in others {
            let (x, y) = (find(&mut parent, i), find(&mut parent, j));
            parent[x] = y;
        }
        for c in &r.cells {
            owner.insert(c.key, i);
        }
    }
    let mut sets: BTreeMap<usize, BTreeSet<u32>> = BTreeMap::new();
    for (i, r) in roads.iter().enumerate() {
        let root = find(&mut parent, i);
        let set = sets.entry(root).or_default();
        set.insert(r.from_site);
        set.insert(r.to_site);
    }
    sets.into_values().collect()
}

// ---- Clearance ---------------------------------------------------------------

/// The road cells standing in the world, by where they are: what the
/// foliage asks so no tree grows on a road (design 10a), as a town's
/// footprint is asked (`settlement::ground::cleared`). Installed when a
/// world's roads are read, before its planet is built.
#[derive(Clone, Debug, Default)]
pub struct Cleared {
    /// Road cell centres by the 4 m box their point at the sea radius
    /// falls in: a keyed lookup, never iterated.
    buckets: HashMap<(i32, i32, i32), Vec<Vec3>>,
    radius_m: f32,
    /// The lamps beside the roads ([`lamps`]): each one's cell centre and
    /// the altitude of the air layer it stands in, by the same boxes.
    lamps: HashMap<(i32, i32, i32), Vec<(Vec3, f32)>>,
}

/// The side of a clearance box, m.
const CLEAR_BOX_M: f32 = 4.0;
/// How near a road cell's centre a direction must be to be on it, m: under
/// a cell's inradius (1.42 m at the gold standard).
const CLEAR_REACH_M: f32 = 1.4;

impl Cleared {
    pub fn new(keys: impl IntoIterator<Item = u32>, radius_m: f32) -> Self {
        let mut buckets: HashMap<(i32, i32, i32), Vec<Vec3>> = HashMap::new();
        for key in keys {
            if let Some(c) = centre(key) {
                buckets
                    .entry(Self::bucket(c, radius_m))
                    .or_default()
                    .push(c);
            }
        }
        Self {
            buckets,
            radius_m,
            lamps: HashMap::new(),
        }
    }

    /// The same, with lamps: `(cell key, the top of its ground)`, each in
    /// the first air layer over its ground.
    pub fn with_lamps(mut self, lamps: impl IntoIterator<Item = (u32, i32)>) -> Self {
        for (key, top_m) in lamps {
            if let Some(c) = centre(key) {
                self.lamps
                    .entry(Self::bucket(c, self.radius_m))
                    .or_default()
                    .push((c, top_m as f32));
            }
        }
        self
    }

    /// The altitude of the lamp in the column at a direction, metres over
    /// the sea radius: its layer's foot.
    pub fn lamp(&self, direction: Vec3) -> Option<f32> {
        let d = direction.normalize_or_zero();
        let (x, y, z) = Self::bucket(d, self.radius_m);
        let reach = CLEAR_REACH_M / self.radius_m;
        for i in -1..=1 {
            for j in -1..=1 {
                for k in -1..=1 {
                    if let Some(at) = self.lamps.get(&(x + i, y + j, z + k)).and_then(|ls| {
                        ls.iter()
                            .find(|(c, _)| (*c - d).length() < reach)
                            .map(|l| l.1)
                    }) {
                        return Some(at);
                    }
                }
            }
        }
        None
    }

    fn bucket(d: Vec3, radius_m: f32) -> (i32, i32, i32) {
        let p = (d * radius_m / CLEAR_BOX_M).floor();
        (p.x as i32, p.y as i32, p.z as i32)
    }

    /// Whether a direction lies on a road cell.
    pub fn on_road(&self, direction: Vec3) -> bool {
        let d = direction.normalize_or_zero();
        let (x, y, z) = Self::bucket(d, self.radius_m);
        let reach = CLEAR_REACH_M / self.radius_m;
        (-1..=1).any(|i| {
            (-1..=1).any(|j| {
                (-1..=1).any(|k| {
                    self.buckets
                        .get(&(x + i, y + j, z + k))
                        .is_some_and(|cs| cs.iter().any(|c| (*c - d).length() < reach))
                })
            })
        })
    }
}

static CLEARED: std::sync::RwLock<Option<std::sync::Arc<Cleared>>> = std::sync::RwLock::new(None);
static CLEARED_GENERATION: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

thread_local! {
    /// This thread's copy of the installed clearance and its generation, so
    /// the build threads do not share a lock's count (as the towns' ground).
    static CLEARED_LOCAL: std::cell::RefCell<(u64, Option<std::sync::Arc<Cleared>>)> =
        const { std::cell::RefCell::new((u64::MAX, None)) };
}

/// Install the world's road cells, or none.
pub fn install_cleared(cleared: Option<Cleared>) {
    let _held = crate::settlement::ground::installing();
    *CLEARED.write().unwrap_or_else(|e| e.into_inner()) = cleared.map(std::sync::Arc::new);
    CLEARED_GENERATION.fetch_add(1, std::sync::atomic::Ordering::AcqRel);
}

/// The altitude of a lamp beside a road of the installed world in the
/// column at a direction (`townsfolk` design 34), or `None`.
pub fn lamp(direction: Vec3) -> Option<f32> {
    let generation = CLEARED_GENERATION.load(std::sync::atomic::Ordering::Acquire);
    CLEARED_LOCAL.with(|local| {
        let mut local = local.borrow_mut();
        if local.0 != generation {
            *local = (
                generation,
                CLEARED.read().unwrap_or_else(|e| e.into_inner()).clone(),
            );
        }
        local.1.as_ref().and_then(|c| c.lamp(direction))
    })
}

/// Where the lamps beside the roads stand (`townsfolk` design 34): along
/// each road from each end, within `lamp_town_m` of that end's settlement,
/// every `lamp_every`th cell past the town's own has a lamp on the cell
/// beside it nearest its left (the way out of the town), the first that is
/// dry, no road, no town's, holds no player's edit and stands within a
/// layer of the road's ground. `(key, top of its ground)`, in road order;
/// the same roads give the same lamps.
pub fn lamps(
    roads: &[Road],
    settlements: &[Settlement],
    cfg: &RoadsConfig,
    radius_m: f32,
    ground: &GroundFn,
) -> Vec<(u32, i32)> {
    if cfg.lamp_every == 0 {
        return Vec::new();
    }
    let on_road: BTreeSet<u32> = roads.iter().flat_map(|r| &r.cells).map(|c| c.key).collect();
    let mut taken = BTreeSet::new();
    let mut out = Vec::new();
    for road in roads {
        for (site, forward) in [(road.from_site, true), (road.to_site, false)] {
            let Some(town) = settlements.iter().find(|s| s.site == site) else {
                continue;
            };
            let cells: Vec<&RoadCell> = if forward {
                road.cells.iter().collect()
            } else {
                road.cells.iter().rev().collect()
            };
            for (k, w) in cells.windows(2).enumerate() {
                let (cell, next) = (w[0], w[1]);
                let (Some(c), Some(n)) = (centre(cell.key), centre(next.key)) else {
                    continue;
                };
                if crate::sites::arc_m(c, town.direction, radius_m) > cfg.lamp_town_m {
                    break;
                }
                if (k as u32 + 1) % cfg.lamp_every != 0 {
                    continue;
                }
                let left = c.cross(n - c).normalize_or_zero();
                let mut beside: Vec<(f32, u32)> = neighbours(cell.key)
                    .into_iter()
                    .filter(|m| !on_road.contains(m))
                    .filter_map(|m| Some(((centre(m)? - c).dot(left), m)))
                    .collect();
                beside.sort_by(|a, b| b.0.total_cmp(&a.0).then(a.1.cmp(&b.1)));
                let Some((key, top)) = beside.into_iter().find_map(|(_, m)| {
                    let g = ground(m)?;
                    (g.water == Water::Dry
                        && !g.town
                        && !g.player
                        && (g.top_m - cell.ground_m()).abs() <= 1
                        && !taken.contains(&m))
                    .then_some((m, g.top_m))
                }) else {
                    continue;
                };
                taken.insert(key);
                out.push((key, top));
            }
        }
    }
    out
}

/// Whether a direction lies on a road cell of the installed world.
pub fn cleared(direction: Vec3) -> bool {
    let generation = CLEARED_GENERATION.load(std::sync::atomic::Ordering::Acquire);
    CLEARED_LOCAL.with(|local| {
        let mut local = local.borrow_mut();
        if local.0 != generation {
            *local = (
                generation,
                CLEARED.read().unwrap_or_else(|e| e.into_inner()).clone(),
            );
        }
        local.1.as_ref().is_some_and(|c| c.on_road(direction))
    })
}

// ---- Records -----------------------------------------------------------------

/// A road record's body: the cells compact, `(key, base_m, kind, deck_m,
/// raised, up, from)` with `kind` 0 ground, 1 ford, 2 bridge and 0 for no
/// `up` or `from` (a finest cell's key carries its level in bits 24 and 25,
/// so none is 0).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct RoadBody {
    from: u32,
    to: u32,
    surface: Surface,
    cells: Vec<(u32, i32, u8, i32, bool, u32, u32)>,
}

/// A schema 2 road record's body: [`RoadBody`]'s cells with each one's cut
/// or fill last.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct RoadBody2 {
    from: u32,
    to: u32,
    surface: Surface,
    cells: Vec<(u32, i32, u8, i32, bool, u32, u32, i32)>,
}

/// The net record's body.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NetBody {
    pub version: u32,
    pub roads: u32,
}

/// A road's id: its two sites, the lower in the high half.
pub fn road_id(from_site: u32, to_site: u32) -> u64 {
    (u64::from(from_site.min(to_site)) << 32) | u64::from(from_site.max(to_site))
}

/// A road as its record: schema 1, or [`CUT_FILL_SCHEMA`] where any cell is
/// cut or filled.
pub fn to_record(road: &Road) -> Record {
    let cells: Vec<(u32, i32, u8, i32, bool, u32, u32, i32)> = road
        .cells
        .iter()
        .map(|c| {
            let (kind, deck) = match c.kind {
                Kind::Ground => (0, 0),
                Kind::Ford => (1, 0),
                Kind::Bridge { deck_m } => (2, deck_m),
            };
            (
                c.key,
                c.base_m,
                kind,
                deck,
                c.raised,
                c.up.unwrap_or(0),
                c.from.unwrap_or(0),
                c.cut_fill,
            )
        })
        .collect();
    let id = road_id(road.from_site, road.to_site);
    if cells.iter().any(|c| c.7 != 0) {
        return Record::of(
            ROAD_RECORD,
            id,
            CUT_FILL_SCHEMA,
            &RoadBody2 {
                from: road.from_site,
                to: road.to_site,
                surface: road.surface,
                cells,
            },
        );
    }
    Record::of(
        ROAD_RECORD,
        id,
        RECORD_SCHEMA,
        &RoadBody {
            from: road.from_site,
            to: road.to_site,
            surface: road.surface,
            cells: cells
                .into_iter()
                .map(|(a, b, c, d, e, f, g, _)| (a, b, c, d, e, f, g))
                .collect(),
        },
    )
}

/// A road from its record, schema 1 or 2, or `None` for one this build
/// cannot read.
pub fn from_record(record: &Record) -> Option<Road> {
    if record.kind != ROAD_RECORD {
        return None;
    }
    let body: RoadBody2 = match record.schema {
        RECORD_SCHEMA => {
            let b: RoadBody = record.read()?;
            RoadBody2 {
                from: b.from,
                to: b.to,
                surface: b.surface,
                cells: b
                    .cells
                    .into_iter()
                    .map(|(a, b, c, d, e, f, g)| (a, b, c, d, e, f, g, 0))
                    .collect(),
            }
        }
        CUT_FILL_SCHEMA => record.read()?,
        _ => return None,
    };
    let cells = body
        .cells
        .into_iter()
        .map(|(key, base_m, kind, deck, raised, up, from, cut_fill)| {
            Some(RoadCell {
                key,
                base_m,
                kind: match kind {
                    0 => Kind::Ground,
                    1 => Kind::Ford,
                    2 => Kind::Bridge { deck_m: deck },
                    _ => return None,
                },
                raised,
                cut_fill,
                bed: Bed::default(),
                up: (up != 0).then_some(up),
                from: (from != 0).then_some(from),
            })
        })
        .collect::<Option<Vec<_>>>()?;
    Some(Road {
        from_site: body.from,
        to_site: body.to,
        surface: body.surface,
        cells,
    })
}

/// The layers a cut road cell has dug out of its ground (`townsfolk` design
/// 32): every layer from the road's ground up to the generated ground's
/// top, by `column::layer_at`'s index. None for a cell that is not cut.
pub fn cut_layers(cell: &RoadCell) -> Vec<u16> {
    (cell.ground_m()..cell.base_m)
        .filter_map(|alt| crate::column::layer_at(alt as f32 + 0.5))
        .map(|l| l as u16)
        .collect()
}

/// The net record for a plan of `roads` roads.
pub fn net_record(roads: usize) -> Record {
    Record::of(
        ROAD_NET_RECORD,
        0,
        RECORD_SCHEMA,
        &NetBody {
            version: NET_VERSION,
            roads: roads as u32,
        },
    )
}

/// The version a world's roads were planned by: `None` before they were.
pub fn net_version(records: &Records) -> Option<u32> {
    records
        .get(ROAD_NET_RECORD, 0)
        .and_then(|r| r.read::<NetBody>())
        .map(|b| b.version)
}

/// Whether the world's roads were planned already.
pub fn planned(records: &Records) -> bool {
    records.get(ROAD_NET_RECORD, 0).is_some()
}

/// Every road the records hold, by id.
pub fn stored(records: &Records) -> Vec<Road> {
    records
        .of_kind(ROAD_RECORD)
        .filter_map(from_record)
        .collect()
}

/// A road cell the player built (`townsfolk` task 8.3): its surface, the
/// neighbour it rises toward and the one it rises from, by key, at the
/// ground's top `base_m` when it was built.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuiltCell {
    pub surface: Surface,
    pub base_m: i32,
    pub up: Option<u32>,
    pub from: Option<u32>,
    /// Paved on a plan (design 40, schema 2): raised a layer on packed
    /// earth, a dip one cell wide, as a world road's ([`RoadCell::raised`]).
    #[serde(default)]
    pub raised: bool,
    /// The plan it was paved on (schema 2); `None` for design 31's cells.
    #[serde(default)]
    pub plan: Option<u64>,
    /// Paved on a river's bed under its water, in gravel (design 42a): a
    /// ford, drawn and wheeled as the world's.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub ford: bool,
    /// A wide road's lane bed (design 42m); plain on a one-wide road.
    #[serde(default, skip_serializing_if = "Bed::is_plain")]
    pub bed: Bed,
    /// A bridge's deck, metres over the sea radius (design 42m).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deck_m: Option<i32>,
}

/// A player's road cell as its record, by the cell's key.
pub fn built_record(key: u32, cell: &BuiltCell) -> Record {
    Record::of(ROAD_CELL_RECORD, u64::from(key), RECORD_SCHEMA, cell)
}

/// Every road cell the player built, by key.
pub fn built(records: &Records) -> BTreeMap<u32, BuiltCell> {
    records
        .of_kind(ROAD_CELL_RECORD)
        .filter(|r| r.schema == RECORD_SCHEMA || r.schema == paving::PAVED_SCHEMA)
        .filter_map(|r| Some((u32::try_from(r.id).ok()?, r.read::<BuiltCell>()?)))
        .collect()
}

/// The neighbour a player's road cell rises toward (design 10): the first,
/// in `toward`'s order (the one faced first), whose ground or road stands
/// exactly a layer over this cell's `base_m`; flat where none does. The
/// slope starts from the side opposite.
pub fn built_up(base_m: i32, toward: &[(u32, i32)]) -> Option<u32> {
    toward
        .iter()
        .find(|(_, h)| *h == base_m + 1)
        .map(|(k, _)| *k)
}

/// A built cell as a road cell, to draw and stand on as the world's are.
pub fn built_cell(key: u32, cell: &BuiltCell) -> RoadCell {
    RoadCell {
        key,
        base_m: cell.base_m,
        kind: match (cell.deck_m, cell.ford) {
            (Some(deck_m), _) => Kind::Bridge { deck_m },
            (None, true) => Kind::Ford,
            (None, false) => Kind::Ground,
        },
        raised: cell.raised,
        cut_fill: 0,
        bed: cell.bed,
        up: cell.up,
        from: cell.from,
    }
}

pub mod paving;
pub mod planner;
pub use planner::{Clash, check_cell, check_step, hex_line};
pub mod piece;

#[cfg(test)]
mod tests;
