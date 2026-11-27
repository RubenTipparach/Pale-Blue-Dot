use super::blaze::{self, Air, Ground, Land, Lighting, LitBy, World};
use super::dryness::{self, Cured, Factors, Plant};
use super::scar::{self, Burnt, GroundLook, TreeLook};
use super::torch::{self, Exposure, Soak};
use super::*;
use crate::daylight::DAY_S;
use crate::records::{Record, Records, YieldSet};
use glam::Vec3;
use std::collections::{BTreeMap, BTreeSet};

const RADIUS: f32 = 4_800.0;

fn cfg() -> FireConfig {
    FireConfig::shipped()
}

fn near(a: f32, b: f32, tol: f32) -> bool {
    (a - b).abs() <= tol
}

// ---- design 2's table, row by row ------------------------------------------

fn late_summer() -> f32 {
    dryness::weather(&cfg(), 0.12, 0.32, 29.0)
}

const FULL: Cured = Cured {
    c: 1.0,
    litter: 0.0,
};

#[test]
fn the_shipped_file_parses_and_validates() {
    let c = FireConfig::shipped();
    assert_eq!(c.limits.cells, 300);
    let mut bad = c.clone();
    bad.dryness.cure.cactus.base = 0.1;
    assert!(bad.validate().is_err(), "a cactus that burns is refused");
    let mut bad = c.clone();
    bad.regrow_days.sapling = 100.0;
    assert!(bad.validate().is_err(), "regrowth out of order is refused");
}

#[test]
fn the_fire_weather_matches_design_2s_rows() {
    let c = cfg();
    for (gw, h, t, want) in [
        (0.5, 0.6, 16.0, 0.29),
        (0.25, 0.45, 26.0, 0.70),
        (0.12, 0.32, 29.0, 0.88),
        (0.2, 0.3, 22.0, 0.78),
        (0.1, 0.15, 35.0, 0.90),
        (0.9, 0.85, 28.0, 0.04),
        (0.6, 0.6, 4.0, 0.12),
    ] {
        let w = dryness::weather(&c, gw, h, t);
        assert!(near(w, want, 0.006), "{gw} {h} {t}: {w} not {want}");
    }
}

#[test]
fn plant_dry_matches_design_2s_table_of_examples() {
    let c = cfg();
    let d = &c.dryness;
    let ls = late_summer();
    let grass = |f: Factors| dryness::plant_dry(&c, &f);
    let open = |w: f32, cure_c: f32| {
        Factors::open(
            Plant::Grass,
            Cured {
                c: cure_c,
                litter: 0.0,
            },
            w,
        )
    };
    // Spring meadow, midsummer, late summer.
    assert!(near(
        grass(open(dryness::weather(&c, 0.5, 0.6, 16.0), 0.0)),
        0.06,
        0.01
    ));
    assert!(near(
        grass(open(dryness::weather(&c, 0.25, 0.45, 26.0), 0.5)),
        0.42,
        0.01
    ));
    assert!(near(grass(open(ls, 1.0)), 0.88, 0.01));
    // The river's bank and two cells from it.
    let bank = Factors {
        water: dryness::water(d, Some(1)),
        ..open(ls, 1.0)
    };
    assert!(near(grass(bank), 0.31, 0.01));
    let two = Factors {
        water: dryness::water(d, Some(2)),
        ..open(ls, 1.0)
    };
    assert!(near(grass(two), 0.59, 0.01));
    assert!(near(dryness::water(d, None), 1.0, 1e-6), "three on is dry");
    // Under an oak wood (four trees round, its own tree).
    let wood = Factors {
        shade: dryness::shade(d, Plant::Grass, 4, true),
        ..open(ls, 1.0)
    };
    assert!(near(grass(wood), 0.54, 0.01));
    // In a hollow, on a green patch.
    let hollow = Factors {
        hollow: dryness::hollow(d, -2.0),
        patch: 0.7,
        ..open(ls, 1.0)
    };
    assert!(near(grass(hollow), 0.54, 0.01));
    // Beside a pen.
    let pen = Factors {
        grazed: dryness::grazed(d, Plant::Grass, true),
        ..open(ls, 1.0)
    };
    assert!(near(grass(pen), 0.44, 0.01));
    // An oak in that field (deciduous at cure 0.60).
    let oak = Factors::open(Plant::Deciduous, FULL, ls);
    assert!(near(dryness::plant_dry(&c, &oak), 0.53, 0.01));
    // A mountain pine in late summer.
    let pine = Factors::open(Plant::Conifer, FULL, dryness::weather(&c, 0.2, 0.3, 22.0));
    assert!(near(dryness::plant_dry(&c, &pine), 0.70, 0.01));
    // A desert cactus: never.
    let cactus = Factors::open(Plant::Cactus, FULL, dryness::weather(&c, 0.1, 0.15, 35.0));
    assert_eq!(dryness::plant_dry(&c, &cactus), 0.0);
    // The jungle floor, shaded.
    let jungle = Factors {
        shade: dryness::shade(d, Plant::Grass, 4, false),
        ..open(dryness::weather(&c, 0.9, 0.85, 28.0), 1.0)
    };
    assert!(near(grass(jungle), 0.03, 0.01));
    // Tundra in autumn (c 0.15: cure 0.32).
    assert!(near(
        grass(open(dryness::weather(&c, 0.6, 0.6, 4.0), 0.15)),
        0.04,
        0.01
    ));
    // Anything under snow.
    assert_eq!(
        grass(Factors {
            zeroed: true,
            ..open(ls, 1.0)
        }),
        0.0
    );
}

#[test]
fn the_hazard_grows_as_the_square_of_the_margin_and_is_nought_when_green() {
    let c = cfg();
    assert_eq!(dryness::burn(&c, 0.49), 0.0);
    assert_eq!(dryness::burn(&c, 0.0), 0.0);
    assert!(near(dryness::burn(&c, 0.6), 0.04, 1e-4));
    assert!(near(dryness::burn(&c, 0.75), 0.25, 1e-4));
    assert!(near(dryness::burn(&c, 0.9), 0.64, 1e-4));
    assert!(!dryness::can_catch(&c, 0.4999));
}

#[test]
fn the_dry_season_is_half_a_year_apart_across_the_equator() {
    let c = cfg();
    let lat = (45f32).to_radians().sin();
    let ls = late_summer();
    // Late summer in the north (local 0.10).
    let north = dryness::cured(&c, 0.10, lat);
    let south = dryness::cured(&c, 0.10, -lat);
    let dry = |cu: Cured| dryness::plant_dry(&c, &Factors::open(Plant::Grass, cu, ls));
    assert!(dry(north) > 0.8, "north's late summer: {}", dry(north));
    assert!(
        dry(south) < 0.5,
        "south's late winter is too green: {}",
        dry(south)
    );
    // And half a year on, the other way round.
    let north = dryness::cured(&c, 0.60, lat);
    let south = dryness::cured(&c, 0.60, -lat);
    assert!(dry(south) > 0.8 && dry(north) < 0.5);
    // Spring (local 0.75) is green everywhere.
    assert!(dry(dryness::cured(&c, 0.75, lat)) < 0.5);
}

// ---- a test patch ----------------------------------------------------------

/// Ground round a point away from any face edge: the real lattice's cells,
/// with land and weather from closures of a cell's east and north metres.
struct Patch {
    centre: Vec3,
    east: Vec3,
    north: Vec3,
    cells: BTreeMap<u32, (Vec3, Vec<u32>)>,
    land: Box<dyn Fn(f32, f32, u32) -> Option<Land>>,
    air: Air,
}

impl Patch {
    fn new(rings: usize, land: impl Fn(f32, f32, u32) -> Option<Land> + 'static, air: Air) -> Self {
        let centre = Vec3::new(0.31, 0.62, 0.72).normalize();
        let east = Vec3::Y.cross(centre).normalize();
        let north = centre.cross(east);
        let start = crate::roads::nearest_key(centre).unwrap();
        let mut cells = BTreeMap::new();
        let mut ring = vec![start];
        let mut seen = BTreeSet::from([start]);
        for _ in 0..=rings {
            let mut next = Vec::new();
            for &k in &ring {
                let ns = crate::roads::neighbours(k);
                for &n in &ns {
                    if seen.insert(n) {
                        next.push(n);
                    }
                }
                cells.insert(k, (crate::roads::centre(k).unwrap(), ns));
            }
            ring = next;
        }
        Self {
            centre,
            east,
            north,
            cells,
            land: Box::new(land),
            air,
        }
    }

    fn local(&self, c: Vec3) -> (f32, f32) {
        let p = c / c.dot(self.centre) - self.centre;
        (p.dot(self.east) * RADIUS, p.dot(self.north) * RADIUS)
    }

    fn at(&self, east: f32, north: f32) -> u32 {
        let d =
            (self.centre + self.east * (east / RADIUS) + self.north * (north / RADIUS)).normalize();
        crate::roads::nearest_key(d).unwrap()
    }

    fn xy(&self, key: u32) -> (f32, f32) {
        self.local(self.cells[&key].0)
    }
}

impl Ground for Patch {
    fn land(&self, key: u32) -> Option<Land> {
        let (c, _) = self.cells.get(&key)?;
        let (e, n) = self.local(*c);
        (self.land)(e, n, key)
    }
    fn air(&self, _key: u32) -> Air {
        self.air
    }
    fn radius_m(&self) -> f32 {
        RADIUS
    }
    fn centre(&self, key: u32) -> Option<Vec3> {
        self.cells.get(&key).map(|c| c.0)
    }
    fn neighbours(&self, key: u32) -> Vec<u32> {
        self.cells
            .get(&key)
            .map_or_else(|| crate::roads::neighbours(key), |c| c.1.clone())
    }
}

fn grass_land() -> Land {
    Land {
        plant: Plant::Grass,
        top: 0,
        water: false,
        snow: false,
        pen: false,
    }
}

fn dry_air(wind_east: f32, patch: &Vec3) -> Air {
    let east = Vec3::Y.cross(*patch).normalize();
    Air {
        ground_wet: 0.12,
        humidity: 0.32,
        air_c: 29.0,
        rain_mm_h: 0.0,
        wind: east * wind_east,
        snow: false,
    }
}

fn spring_air() -> Air {
    Air {
        ground_wet: 0.5,
        humidity: 0.6,
        air_c: 16.0,
        ..Air::default()
    }
}

/// Late summer in the patch's hemisphere (it is north).
const LATE_SUMMER: f64 = 0.10;
const SPRING: f64 = 0.75;

/// Fields after two dry weeks: grass, one tree in eight, a shrub in twelve.
fn fields(_e: f32, _n: f32, key: u32) -> Option<Land> {
    let r = crate::tree_kit::roll(key, 0x77);
    let plant = if r < 0.125 {
        Plant::Deciduous
    } else if r < 0.21 {
        Plant::Shrub
    } else {
        Plant::Grass
    };
    Some(Land {
        plant,
        ..grass_land()
    })
}

struct Run {
    records: Records,
    yields: YieldSet,
    seed: u64,
    now_s: f64,
    year: f64,
}

impl Run {
    fn new(year: f64) -> Self {
        Self {
            records: Records::new(),
            yields: YieldSet::new(),
            seed: 4242,
            now_s: 1_000_000.0,
            year,
        }
    }

    fn world<'a, G: Ground>(&'a self, cfg: &'a FireConfig, g: &'a G) -> World<'a, G> {
        World {
            cfg,
            seed: self.seed,
            ground: g,
            records: &self.records,
            yields: &self.yields,
            now_s: self.now_s,
            year_fraction: self.year,
        }
    }

    fn light<G: Ground>(&mut self, cfg: &FireConfig, g: &G, cell: u32) -> Option<u64> {
        let lighting = blaze::ignite(&self.world(cfg, g), cell, LitBy::Player(0));
        let Lighting::Blaze(b) = lighting else {
            return None;
        };
        let id = blaze::next_id(&self.records);
        self.records.put(b.record(id));
        Some(id)
    }

    /// One second: every live blaze ticks.
    fn second<G: Ground>(&mut self, cfg: &FireConfig, g: &G) {
        self.now_s += 1.0;
        let ids: Vec<u64> = blaze::live(&self.records)
            .iter()
            .map(|(id, _)| *id)
            .collect();
        let proposals = blaze::run(&self.world(cfg, g), &ids);
        for (_, p) in proposals {
            for r in p.records {
                self.records.put(r);
            }
        }
    }

    fn burn_out<G: Ground>(&mut self, cfg: &FireConfig, g: &G, most_s: u32) -> u32 {
        for s in 0..most_s {
            if blaze::live(&self.records).is_empty() {
                return s;
            }
            self.second(cfg, g);
        }
        most_s
    }

    fn burnt_cells(&self) -> BTreeSet<u32> {
        self.records
            .of_kind(blaze::BURNT_KIND)
            .map(|r| r.id as u32)
            .collect()
    }

    fn lines(&self) -> Vec<String> {
        self.records
            .iter()
            .map(|r| r.line(&blaze::author(1)))
            .collect()
    }
}

#[test]
fn spring_green_grass_never_catches_anywhere_on_the_patch() {
    let c = cfg();
    let g = Patch::new(12, |_, _, _| Some(grass_land()), spring_air());
    let mut run = Run::new(SPRING);
    for &k in g.cells.keys() {
        assert!(
            run.world(&c, &g).plant_dry(k) < 0.1,
            "spring grass is green"
        );
        assert_eq!(
            blaze::ignite(&run.world(&c, &g), k, LitBy::Player(0)),
            Lighting::Damp,
            "a torch on spring grass makes a damp flame"
        );
    }
    // And a burning cell set among it spreads to nothing.
    let origin = g.at(0.0, 0.0);
    let b = Blaze {
        started_s: run.now_s,
        origin,
        lit_by: LitBy::Player(0),
        state: State::Burning,
        tick: 0,
        burning: vec![Burning {
            cell: origin,
            plant: Plant::Grass,
            left_ds: 150,
            crown: false,
        }],
        cells: 1,
        limited: false,
    };
    run.records.put(b.record(1));
    run.burn_out(&c, &g, 60);
    assert_eq!(run.burnt_cells(), BTreeSet::from([origin]));
}

#[test]
fn a_cell_below_one_half_never_catches_from_a_neighbour() {
    let c = cfg();
    // Every cell but the origin is marginal: a patch whose soil keeps it
    // just under the line.
    let mut g = Patch::new(6, |_, _, _| Some(grass_land()), Air::default());
    g.air = dry_air(3.0, &g.centre);
    let origin = g.at(0.0, 0.0);
    let mut run = Run::new(LATE_SUMMER);
    // Make every other cell's dryness 0.49 through a doused-free green patch:
    // a ground whose neighbours are all water two rings out is simplest; here
    // we check the gate itself on the computed dryness.
    let w = run.world(&c, &g);
    for &k in g.cells.keys() {
        let d = w.plant_dry(k);
        if d < c.dryness.dry_min {
            assert!(
                g.neighbours(k).iter().all(|&n| w.hazard(n, k).is_none()),
                "a cell at {d} never takes a hazard"
            );
        }
    }
    // And the gate on a hand-built table: 0.4999 burns nothing.
    assert_eq!(dryness::burn(&c, 0.4999), 0.0);
    let _ = run.light(&c, &g, origin);
}

#[test]
fn a_late_summer_grass_fire_stays_small_and_slow() {
    let c = cfg();
    for wind in [0.0f32, 3.0, 8.0] {
        let mut g = Patch::new(40, fields, Air::default());
        g.air = dry_air(wind, &g.centre);
        let mut biggest = 0;
        let mut most_burning = 0;
        let mut speeds = Vec::new();
        for (i, (e, n)) in [
            (0.0, 0.0),
            (6.0, 3.0),
            (-9.0, 4.0),
            (3.0, -8.0),
            (12.0, 12.0),
        ]
        .into_iter()
        .enumerate()
        {
            let mut run = Run::new(LATE_SUMMER);
            run.seed = 7 + i as u64;
            let origin = g.at(e, n);
            if run.light(&c, &g, origin).is_none() {
                continue;
            }
            let (oe, on) = g.xy(origin);
            let mut s = 0;
            // The front's speed: how far downwind (east) of the origin the
            // fire has reached, over the time it took, while it still spreads.
            let mut front = (0.0f32, 0u32);
            while !blaze::live(&run.records).is_empty() && s < 900 {
                run.second(&c, &g);
                s += 1;
                for (_, b) in blaze::live(&run.records) {
                    most_burning = most_burning.max(b.burning.len());
                    if b.limited {
                        continue;
                    }
                    for burning in &b.burning {
                        let (x, y) = g.xy(burning.cell);
                        let reach = if wind > 0.0 {
                            x - oe
                        } else {
                            ((x - oe).powi(2) + (y - on).powi(2)).sqrt()
                        };
                        if reach > front.0 {
                            front = (reach, s);
                        }
                    }
                }
            }
            assert!(
                s < 600,
                "out within ten minutes of world time (wind {wind}): {s} s"
            );
            if front.1 > 30 {
                speeds.push(front.0 / front.1 as f32);
            }
            biggest = biggest.max(run.burnt_cells().len());
        }
        assert!(biggest <= 300, "wind {wind}: {biggest} cells");
        assert!(
            most_burning <= 40,
            "wind {wind}: {most_burning} burning at once"
        );
        let mean = speeds.iter().sum::<f32>() / speeds.len().max(1) as f32;
        let fastest = speeds.iter().copied().fold(0.0, f32::max);
        assert!(fastest < 1.0, "wind {wind}: a front ran {fastest} m/s");
        assert!(
            (0.05..0.6).contains(&mean),
            "wind {wind}: fronts {mean} m/s"
        );
        println!(
            "wind {wind}: biggest {biggest} cells, {most_burning} burning, fronts {speeds:.2?} m/s"
        );
    }
}

#[test]
fn a_fire_on_open_dry_grass_stops_at_its_limits() {
    let c = cfg();
    // Dry open grass everywhere, a hot dry wind: the limit, not the land, ends it.
    let land = |_e: f32, _n: f32, _k: u32| {
        Some(Land {
            plant: Plant::Grass,
            ..grass_land()
        })
    };
    let mut g = Patch::new(40, land, Air::default());
    g.air = Air {
        ground_wet: 0.0,
        humidity: 0.1,
        air_c: 35.0,
        ..dry_air(3.0, &g.centre)
    };
    let mut run = Run::new(LATE_SUMMER);
    let id = run.light(&c, &g, g.at(0.0, 0.0)).expect("dry grass lights");
    let mut most = 0;
    for _ in 0..1200 {
        run.second(&c, &g);
        if let Some(b) = run
            .records
            .get(blaze::BLAZE_KIND, id)
            .and_then(|r| r.read::<Blaze>())
        {
            most = most.max(b.burning.len());
            assert!(b.cells <= c.limits.cells);
        }
    }
    let b: Blaze = run
        .records
        .get(blaze::BLAZE_KIND, id)
        .unwrap()
        .read()
        .unwrap();
    assert!(b.limited, "the limit stopped it");
    assert!(!b.is_burning(), "and it burnt out there");
    assert!(run.burnt_cells().len() as u32 <= c.limits.cells);
    assert!(most <= c.limits.burning);
}

#[test]
fn a_front_goes_out_at_a_green_river_bank() {
    let c = cfg();
    // A river running north-south 15 m east of the start.
    let land = |e: f32, _n: f32, _k: u32| {
        Some(Land {
            plant: if (14.0..20.0).contains(&e) {
                Plant::None
            } else {
                Plant::Grass
            },
            water: (14.0..20.0).contains(&e),
            ..grass_land()
        })
    };
    let mut g = Patch::new(30, land, Air::default());
    g.air = Air {
        ground_wet: 0.0,
        humidity: 0.1,
        air_c: 35.0,
        ..dry_air(6.0, &g.centre)
    };
    let mut run = Run::new(LATE_SUMMER);
    run.light(&c, &g, g.at(0.0, 0.0)).expect("lit");
    run.burn_out(&c, &g, 1200);
    for cell in run.burnt_cells() {
        let (e, _) = g.xy(cell);
        assert!(e < 14.0, "nothing burns across the river");
        assert!(
            g.neighbours(cell)
                .iter()
                .all(|&n| !g.land(n).is_some_and(|l| l.water)),
            "the bank beside the water stays green"
        );
    }
}

#[test]
fn rain_stops_a_front() {
    let c = cfg();
    let land = |_e: f32, _n: f32, _k: u32| Some(grass_land());
    let mut g = Patch::new(20, land, Air::default());
    g.air = Air {
        ground_wet: 0.0,
        humidity: 0.1,
        air_c: 35.0,
        ..dry_air(3.0, &g.centre)
    };
    let mut run = Run::new(LATE_SUMMER);
    let id = run.light(&c, &g, g.at(0.0, 0.0)).unwrap();
    for _ in 0..30 {
        run.second(&c, &g);
    }
    let before = run.burnt_cells().len()
        + run
            .records
            .get(blaze::BLAZE_KIND, id)
            .unwrap()
            .read::<Blaze>()
            .unwrap()
            .burning
            .len();
    g.air.rain_mm_h = 4.0;
    let s = run.burn_out(&c, &g, 120);
    assert!(s <= 6, "4 mm/h puts grass out in about 5 s: {s}");
    let b: Blaze = run
        .records
        .get(blaze::BLAZE_KIND, id)
        .unwrap()
        .read()
        .unwrap();
    assert_eq!(b.cells as usize, before, "nothing caught in the rain");
}

#[test]
fn a_bucket_puts_out_a_cell_and_its_neighbours() {
    let c = cfg();
    let mut g = Patch::new(20, |_, _, _| Some(grass_land()), Air::default());
    g.air = Air {
        ground_wet: 0.0,
        humidity: 0.1,
        air_c: 35.0,
        ..dry_air(0.0, &g.centre)
    };
    let mut run = Run::new(LATE_SUMMER);
    let id = run.light(&c, &g, g.at(0.0, 0.0)).unwrap();
    for _ in 0..20 {
        run.second(&c, &g);
    }
    let b: Blaze = run
        .records
        .get(blaze::BLAZE_KIND, id)
        .unwrap()
        .read()
        .unwrap();
    let target = b.burning[0].cell;
    for r in blaze::douse(&g, target, run.now_s) {
        run.records.put(r);
    }
    run.second(&c, &g);
    let b: Blaze = run
        .records
        .get(blaze::BLAZE_KIND, id)
        .unwrap()
        .read()
        .unwrap();
    assert!(b.burning_at(target).is_none(), "the poured cell is out");
    for n in g.neighbours(target) {
        assert!(b.burning_at(n).is_none(), "and its neighbours");
        assert_eq!(run.world(&c, &g).plant_dry(n), 0.0, "left wet");
    }
}

#[test]
fn a_dug_line_and_the_players_field_hold() {
    let c = cfg();
    let mut g = Patch::new(30, |_, _, _| Some(grass_land()), Air::default());
    g.air = Air {
        ground_wet: 0.0,
        humidity: 0.1,
        air_c: 35.0,
        ..dry_air(0.0, &g.centre)
    };
    let mut run = Run::new(LATE_SUMMER);
    // A line dug down to soil 10 m east, two cells wide; and a planted
    // field west, both the player's.
    let mut line = BTreeSet::new();
    let mut field = BTreeSet::new();
    for &k in g.cells.keys() {
        let (e, n) = g.xy(k);
        if (10.0..14.0).contains(&e) {
            line.insert(k);
        }
        if (-30.0..-18.0).contains(&e) && n.abs() < 8.0 {
            field.insert(k);
        }
    }
    for &k in line.iter().chain(&field) {
        run.yields.touch_cell(k);
    }
    run.light(&c, &g, g.at(0.0, 0.0)).unwrap();
    run.burn_out(&c, &g, 1200);
    let burnt = run.burnt_cells();
    assert!(!burnt.is_empty());
    for cell in &burnt {
        let (e, _) = g.xy(*cell);
        assert!(e < 10.0, "nothing past the line");
        assert!(
            !field.contains(cell) && !line.contains(cell),
            "the player's cells untouched"
        );
    }
}

#[test]
fn the_same_blaze_burns_the_same_cells_on_two_runs_and_across_a_reload() {
    let c = cfg();
    let mut g = Patch::new(30, fields, Air::default());
    g.air = dry_air(3.0, &g.centre);
    let origin = g.at(0.0, 0.0);
    let straight = |reload_at: Option<u32>| {
        let mut run = Run::new(LATE_SUMMER);
        run.light(&c, &g, origin).unwrap();
        for s in 0..900 {
            if Some(s) == reload_at {
                // Close and reopen: the records as lines and back.
                let mut again = Records::new();
                for line in run.lines() {
                    let (_, r) = Record::parse_line(&line).expect("a line");
                    again.put(r);
                }
                run.records = again;
            }
            if blaze::live(&run.records).is_empty() {
                break;
            }
            run.second(&c, &g);
        }
        run.records
    };
    let a = straight(None);
    let b = straight(None);
    assert_eq!(a, b, "two runs agree");
    let c2 = straight(Some(40));
    assert_eq!(a, c2, "a reload mid-fire changes nothing");
    assert!(a.of_kind(blaze::BURNT_KIND).count() > 1, "it spread");
}

#[test]
fn a_pentagon_cell_tries_its_five_neighbours() {
    let (vertices, _) = crate::topology::icosahedron();
    let key = crate::roads::nearest_key(vertices[0]).unwrap();
    let ns = crate::roads::neighbours(key);
    assert_eq!(ns.len(), 5);
    // A ground of dry grass round it: every one of the five can be reached.
    struct Pent(Vec<u32>, u32);
    impl Ground for Pent {
        fn land(&self, key: u32) -> Option<Land> {
            (key == self.1 || self.0.contains(&key)).then_some(grass_land())
        }
        fn air(&self, _: u32) -> Air {
            Air {
                ground_wet: 0.0,
                humidity: 0.1,
                air_c: 35.0,
                ..Air::default()
            }
        }
        fn radius_m(&self) -> f32 {
            RADIUS
        }
    }
    let g = Pent(ns.clone(), key);
    let c = cfg();
    let run = Run::new(LATE_SUMMER);
    let lat = vertices[0].y;
    let year = if lat >= 0.0 {
        LATE_SUMMER
    } else {
        LATE_SUMMER + 0.5
    };
    let w = World {
        year_fraction: year,
        ..run.world(&c, &g)
    };
    let reached = ns.iter().filter(|&&n| w.hazard(key, n).is_some()).count();
    assert_eq!(reached, 5);
}

// ---- the scar --------------------------------------------------------------

#[test]
fn the_scar_grows_back_by_day() {
    let c = cfg();
    let day = f64::from(DAY_S);
    let tree = Burnt {
        blaze: 1,
        out_s: 0.0,
        plant: Plant::Conifer,
        crown: true,
    };
    let foot = Burnt {
        crown: false,
        plant: Plant::Deciduous,
        ..tree
    };
    let grass = Burnt {
        plant: Plant::Grass,
        crown: false,
        ..tree
    };
    let cell = 12345;
    assert_eq!(
        scar::ground_look(&c, cell, &grass, 0.5 * day),
        GroundLook::Ash
    );
    assert_eq!(
        scar::ground_look(&c, cell, &grass, 1.0 * day),
        GroundLook::Ash,
        "the next day"
    );
    assert_eq!(
        scar::ground_look(&c, cell, &grass, 2.5 * day),
        GroundLook::Shoots
    );
    assert_eq!(
        scar::ground_look(&c, cell, &grass, 8.0 * day),
        GroundLook::Grass
    );
    let back = scar::grass_back_day(&c, cell);
    assert!((4.0..=8.0).contains(&back));
    assert_eq!(
        scar::tree_look(&c, &tree, day),
        TreeLook::Snag,
        "a crown fire's snag"
    );
    assert_eq!(scar::tree_look(&c, &foot, day), TreeLook::Scorched);
    assert_eq!(
        scar::tree_look(&c, &tree, 30.0 * day),
        TreeLook::Sapling { grown: 0.25 },
        "day 30"
    );
    assert_eq!(
        scar::tree_look(&c, &tree, 60.0 * day),
        TreeLook::Whole,
        "sixty days on"
    );
    assert!(!scar::shrub_back(&c, &grass, 9.0 * day) && scar::shrub_back(&c, &grass, 10.0 * day));
    assert!(!scar::burns_again(&c, &grass, 7.9 * day) && scar::burns_again(&c, &grass, 8.0 * day));
    assert!(!scar::burns_again(&c, &tree, 19.0 * day) && scar::burns_again(&c, &tree, 20.0 * day));
    assert_eq!(scar::snag_gives(&c, &tree, day), Some(("charcoal", 1)));
    assert_eq!(
        scar::snag_gives(&c, &tree, 25.0 * day),
        None,
        "a sapling gives none"
    );
    assert_eq!(scar::snag_gives(&c, &grass, day), None);
    assert!(scar::healed(&c, cell, &tree, 60.0 * day));
    assert!(!scar::healed(&c, cell, &tree, 59.0 * day));
}

#[test]
fn a_burnt_cell_cannot_burn_again_until_its_grass_is_whole() {
    let c = cfg();
    let mut g = Patch::new(8, |_, _, _| Some(grass_land()), Air::default());
    g.air = dry_air(0.0, &g.centre);
    let mut run = Run::new(LATE_SUMMER);
    let cell = g.at(0.0, 0.0);
    run.records.put(Record::of(
        blaze::BURNT_KIND,
        u64::from(cell),
        blaze::BURNT_SCHEMA,
        &Burnt {
            blaze: 1,
            out_s: run.now_s - 3.0 * f64::from(DAY_S),
            plant: Plant::Grass,
            crown: false,
        },
    ));
    assert_eq!(run.world(&c, &g).plant_dry(cell), 0.0);
    assert!(!matches!(
        blaze::ignite(&run.world(&c, &g), cell, LitBy::Player(0)),
        Lighting::Blaze(_)
    ));
    run.now_s += 6.0 * f64::from(DAY_S);
    assert!(
        run.world(&c, &g).plant_dry(cell) > 0.5,
        "whole again on day 9"
    );
}

impl Lighting {
    fn blaze(self) -> Blaze {
        match self {
            Lighting::Blaze(b) => b,
            other => panic!("not a blaze: {other:?}"),
        }
    }
}

// ---- starting one ----------------------------------------------------------

#[test]
fn a_torch_on_late_summer_grass_starts_a_blaze_and_a_third_is_a_damp_flame() {
    let c = cfg();
    let mut g = Patch::new(20, |_, _, _| Some(grass_land()), Air::default());
    g.air = Air {
        ground_wet: 0.0,
        humidity: 0.1,
        air_c: 35.0,
        ..dry_air(0.0, &g.centre)
    };
    let mut run = Run::new(LATE_SUMMER);
    assert!(run.light(&c, &g, g.at(0.0, 0.0)).is_some());
    assert!(run.light(&c, &g, g.at(30.0, 0.0)).is_some());
    assert_eq!(
        blaze::ignite(&run.world(&c, &g), g.at(-30.0, 0.0), LitBy::Player(0)),
        Lighting::Damp,
        "two blazes already burn"
    );
    // Rain: a damp flame.
    let mut wet = Run::new(LATE_SUMMER);
    g.air.rain_mm_h = 2.0;
    assert_eq!(
        blaze::ignite(&wet.world(&c, &g), g.at(0.0, 0.0), LitBy::Player(0)),
        Lighting::Damp
    );
    wet.records = Records::new();
    // Nothing on bare ground.
    let bare = Patch::new(
        4,
        |_, _, _| {
            Some(Land {
                plant: Plant::None,
                ..grass_land()
            })
        },
        Air::default(),
    );
    assert_eq!(
        blaze::ignite(&wet.world(&c, &bare), bare.at(0.0, 0.0), LitBy::Player(0)),
        Lighting::Nothing
    );
    // The ignition is the player's own record.
    let r = blaze::ignition_record(3, 99, 5.0);
    assert_eq!(r.kind, "ignition");
    assert_eq!(blaze::author(3).token(), "@wfire:blaze/3");
}

#[test]
fn lightning_starts_a_blaze_only_when_dry_and_once_in_its_quiet_spell() {
    let c = cfg();
    let mut g = Patch::new(10, |_, _, _| Some(grass_land()), Air::default());
    g.air = Air {
        ground_wet: 0.0,
        humidity: 0.1,
        air_c: 35.0,
        ..dry_air(0.0, &g.centre)
    };
    let mut run = Run::new(LATE_SUMMER);
    let cell = g.at(0.0, 0.0);
    assert!(blaze::lightning_starts(&run.world(&c, &g), cell));
    let mut b = blaze::ignite(&run.world(&c, &g), cell, LitBy::Lightning).blaze();
    b.state = State::Out(run.now_s);
    run.records.put(b.record(1));
    assert!(
        !blaze::lightning_starts(&run.world(&c, &g), g.at(20.0, 0.0)),
        "the quiet spell"
    );
    run.now_s += 11.0 * f64::from(DAY_S);
    assert!(
        blaze::lightning_starts(&run.world(&c, &g), g.at(20.0, 0.0)),
        "ten days on"
    );
    g.air.rain_mm_h = 0.6;
    assert!(
        !blaze::lightning_starts(&run.world(&c, &g), cell),
        "a wet strike"
    );
}

#[test]
fn snow_water_and_a_cactus_never_burn() {
    let c = cfg();
    let ls = late_summer();
    for f in [
        Factors {
            zeroed: true,
            ..Factors::open(Plant::Grass, FULL, ls)
        },
        Factors::open(Plant::Cactus, FULL, ls),
        Factors::open(Plant::None, FULL, ls),
    ] {
        assert_eq!(dryness::plant_dry(&c, &f), 0.0);
    }
    assert_eq!(
        Plant::of_tree(crate::tree_kit::Species::Pine),
        Plant::Conifer
    );
    assert_eq!(
        Plant::of_tree(crate::tree_kit::Species::Oak),
        Plant::Deciduous
    );
    assert_eq!(
        Plant::of_tree(crate::tree_kit::Species::Palm),
        Plant::Evergreen
    );
    assert_eq!(
        Plant::of_tree(crate::tree_kit::Species::Cactus),
        Plant::Cactus
    );
}

// ---- torches ---------------------------------------------------------------

#[test]
fn a_torch_goes_out_in_the_rain_under_the_sky_and_not_under_a_roof() {
    let c = cfg();
    let rain = Exposure {
        rain_seen: 0.6,
        snow: false,
        open_sky: true,
    };
    let mut placed = Soak::default();
    let went: Vec<bool> = (0..25).map(|_| placed.step(&c, rain, false, 1.0)).collect();
    assert_eq!(went.iter().position(|&w| w), Some(19), "out after 20 s");
    let mut held = Soak::default();
    let went: Vec<bool> = (0..12).map(|_| held.step(&c, rain, true, 1.0)).collect();
    assert_eq!(
        went.iter().position(|&w| w),
        Some(9),
        "held: out after 10 s"
    );
    let mut roof = Soak::default();
    let roofed = Exposure {
        open_sky: false,
        ..rain
    };
    assert!(
        (0..60).all(|_| !roof.step(&c, roofed, false, 1.0)),
        "a roof keeps it lit"
    );
    let mut snow = Soak::default();
    let snowing = Exposure { snow: true, ..rain };
    assert!(
        (0..60).all(|_| !snow.step(&c, snowing, false, 1.0)),
        "snow does not"
    );
    assert!(torch::open_sky(&c, 10, |_| false));
    assert!(!torch::open_sky(&c, 10, |l| l == 15), "an overhang five up");
    assert!(torch::open_sky(&c, 10, |l| l == 19), "nine up is open sky");
}

#[test]
fn a_placed_torch_out_is_the_worlds_record_and_relit_the_players() {
    let mut records = Records::new();
    let (cell, layer) = (777, 41);
    assert!(!torch::is_out(&records, cell, layer));
    records.put(torch::out_record(&records, cell, layer));
    assert!(torch::is_out(&records, cell, layer));
    assert_eq!(torch::out_lamps(&records), vec![(cell, layer)]);
    // A reload keeps it out.
    let mut again = Records::new();
    for r in records.iter() {
        let line = r.line(&torch::author(cell));
        assert!(line.starts_with("rec @wfire:lamp/777 lamp_out "));
        again.put(Record::parse_line(&line).unwrap().1);
    }
    assert!(torch::is_out(&again, cell, layer));
    // The player lights it; the world can put it out again.
    let lit = torch::lit_record(&again, cell, layer);
    let mut yields = YieldSet::new();
    yields.touch_record(again.get(&lit.kind, lit.id), &lit);
    again.put(lit);
    assert!(!torch::is_out(&again, cell, layer));
    let out = torch::out_record(&again, cell, layer);
    let p = crate::records::Proposal {
        edits: vec![],
        records: vec![out.clone()],
    };
    assert!(yields.check(&p, &again).is_ok(), "the world's own record");
    again.put(out);
    assert!(torch::is_out(&again, cell, layer));
}

#[test]
fn harm_is_four_a_second_in_the_fire_and_one_beside_it() {
    let c = cfg();
    assert_eq!(harm_per_s(&c, true, true), 4.0);
    assert_eq!(harm_per_s(&c, false, true), 1.0);
    assert_eq!(harm_per_s(&c, false, false), 0.0);
    let kinds = record_kinds();
    assert!(kinds.contains(&("blaze", 1)) && kinds.contains(&("lamp_out", 1)));
}
