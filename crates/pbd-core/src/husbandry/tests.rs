use super::farm::{Conditions, FarmState};
use super::genome::{self, Genome};
use super::record::{self, AnimalBody, FenceBody, Standing};
use super::*;
use crate::records::{Author, Proposal, Record, Records, YieldSet};
use std::collections::BTreeMap;

fn settings() -> HusbandrySettings {
    HusbandrySettings::default()
}

fn species(id: &str) -> LandSpecies {
    settings()
        .roster(crate::fauna::HOME_BODY)
        .iter()
        .find(|s| s.id == id)
        .unwrap()
        .clone()
}

// ---- 4.1 the kingdom in data ----------------------------------------------

#[test]
fn the_shipped_file_parses_and_validates() {
    let s = settings();
    s.validate().unwrap();
    // A partial file inherits the rest.
    let partial: HusbandrySettings = ron::from_str("(taming: (fear_moving_m: 9.0, fear_still_m: 4.5, reach_m: 2.5, spook_cost: 0.5, decay_per_hour: 5.0, follow_s: 60.0, wild_follow_s: 20.0))").unwrap();
    assert_eq!(partial.taming.spook_cost, 0.5);
    assert_eq!(partial.land, s.land);
    assert!(ron::from_str::<HusbandrySettings>("(tamng: ())").is_err());
}

/// The land roster is append-only: a record names its species by index, so
/// these stay where they are and new ones go after.
#[test]
fn the_land_roster_keeps_its_order() {
    let ids: Vec<String> = settings()
        .roster("home")
        .iter()
        .map(|s| s.id.clone())
        .collect();
    assert_eq!(ids[..3], ["meadow-hen", "woolback", "plodder"]);
}

#[test]
fn every_species_has_a_class_a_status_and_a_guide_entry_and_airless_bodies_have_none() {
    let fauna = crate::fauna::FaunaSettings::default();
    let s = settings();
    let mut ids = std::collections::BTreeSet::new();
    for (body, list) in &fauna.bodies {
        for sp in list {
            // Water life is not only fish since `ocean-environments` 7.1
            // (crabs, clams, turtles, otters, ducks), and none is tamed:
            // each is wild, or protected from the catch (design D9).
            assert!(
                matches!(
                    sp.status,
                    crate::fauna::Status::WildOnly | crate::fauna::Status::Protected
                ),
                "{body}/{}",
                sp.id
            );
            assert!(!sp.guide.entry.is_empty());
            assert!(ids.insert(sp.id.clone()));
        }
    }
    for (body, list) in &s.land {
        for sp in list {
            assert!(!sp.guide.entry.is_empty(), "{body}/{}", sp.id);
            assert!(sp.class != crate::fauna::Class::Fish);
            assert!(ids.insert(sp.id.clone()), "{} is also a fish", sp.id);
        }
    }
    for airless in ["moon", "asteroid", "barren"] {
        assert!(s.roster(airless).is_empty());
        assert!(fauna.roster(airless).is_empty());
    }
}

// ---- 5.1 the genome --------------------------------------------------------

#[test]
fn the_same_seed_gives_the_same_genome_and_the_morph_shows_one_in_64() {
    assert_eq!(Genome::roll(42, 3, 2), Genome::roll(42, 3, 2));
    assert_ne!(Genome::roll(42, 3, 2), Genome::roll(43, 3, 2));
    let rolls = 64_000;
    let shown = (0..rolls)
        .filter(|&k| Genome::roll(crate::fauna::hash2(7, k), 3, 2).morph())
        .count();
    let expected = rolls as f64 / 64.0;
    assert!(
        (shown as f64 - expected).abs() < expected * 0.1,
        "{shown} morphs in {rolls}, expected about {expected}"
    );
    let g = Genome::roll(99, 3, 2);
    assert!(g.coat() < 3 && g.pattern() < 2);
    assert!((0.85..=1.25).contains(&g.size_factor()));
    assert!((0.6..=1.4).contains(&g.yield_factor()));
    assert!((0.6..=1.3).contains(&g.trust_factor()));
}

// ---- 5.2 taming -------------------------------------------------------------

#[test]
fn a_plodder_takes_five_feeds_and_a_spook_costs_ten() {
    let plodder = species("plodder");
    let gain = plodder.keep.as_ref().unwrap().trust_gain;
    let mut trust = Trust::default();
    for k in 0..4 {
        assert!(!trust.feed(gain, 1.0), "full after {} feeds", k + 1);
    }
    assert!(trust.feed(gain, 1.0));
    let t = settings().taming;
    let mut half = Trust { value: 50.0 };
    half.spook(&t);
    assert_eq!(half.value, 40.0);
    half.decay(2.0, &t);
    assert_eq!(half.value, 30.0);
    assert_eq!(taming::fear_radius(&t, true, false), 9.0);
    assert_eq!(taming::fear_radius(&t, true, true), 4.5);
    assert_eq!(taming::fear_radius(&t, false, true), 0.0);
}

#[test]
fn a_wild_only_or_protected_species_refuses_with_its_reason() {
    let fish = &crate::fauna::FaunaSettings::default().bodies["home"][0];
    assert_eq!(
        fish.status.refusal(fish.class),
        Some("wild only: fish are kept in tanks, not pens")
    );
    let mut otter = species("woolback");
    otter.status = crate::fauna::Status::Protected;
    otter.keep = None;
    assert_eq!(otter.refusal(), Some("protected: it may not be caught"));
    let mut fox = species("woolback");
    fox.status = crate::fauna::Status::WildOnly;
    fox.keep = None;
    assert_eq!(fox.refusal(), Some("wild only: it cannot be tamed"));
    assert_eq!(species("meadow-hen").refusal(), None);
    // Food (S3): a woolback eats wheat and turf, a hen seed and wheat.
    let keep = species("woolback").keep.unwrap();
    assert!(keep.eats(crate::inventory::Item::Good(crate::goods::Good::WHEAT)));
    assert!(
        !species("meadow-hen")
            .keep
            .unwrap()
            .eats(crate::inventory::Item::Block(
                crate::terrain::Material::Grass
            ))
    );
    assert!(keep.eats(crate::inventory::Item::Block(
        crate::terrain::Material::Grass
    )));
    assert!(!keep.eats(crate::inventory::Item::Block(
        crate::terrain::Material::Stone
    )));
}

#[test]
fn a_tamed_animal_round_trips_its_line_with_its_genome() {
    let g = Genome::roll(1234, 3, 2);
    let body = AnimalBody {
        sp: 1,
        genes: g.0.to_vec(),
        wild: 1234,
        home: 77,
        ..AnimalBody::default()
    };
    let line = body.record(5).line(&Author::Player(0));
    let (author, rec) = Record::parse_line(&line).unwrap();
    assert!(author.is_player());
    assert_eq!(rec.kind, record::ANIMAL);
    let back: AnimalBody = rec.read().unwrap();
    assert_eq!(back.genome(), g);
    assert_eq!(back, body);
    let mut records = Records::new();
    assert_eq!(record::next_animal_id(&records), 1);
    records.put(rec);
    assert_eq!(record::next_animal_id(&records), 6);
    assert_eq!(record::animals(&records).len(), 1);
}

// ---- 5.3 the record's schema -------------------------------------------------

/// A body from an earlier schema - fewer genes, fewer fields - reads under
/// this build with the missing ones at their defaults.
#[test]
fn an_older_record_opens_with_the_missing_gene_and_fields_at_their_defaults() {
    let old = r#"(body: "home", sp: 2, sex: F, born: 3.5, gen: [10, 20, 30, 40, 50, 60, 70, 80, 1, 2, 0, 1])"#;
    let body: AnimalBody = ron::from_str(old).unwrap();
    let g = body.genome();
    assert_eq!(g.alleles(genome::SIZE), (10, 20));
    assert_eq!(g.alleles(genome::PATTERN), (0, 1));
    assert_eq!(g.alleles(genome::MORPH), (0, 0), "the gene it lacked");
    assert_eq!(body.health, 100.0);
    assert!(body.made.is_empty());
    // And no world gains animals it never had.
    assert!(record::animals(&Records::new()).is_empty());
}

// ---- 6.1 pens ----------------------------------------------------------------

fn key(q: i32, r: i32) -> u32 {
    (((q + 500) as u32) << 16) | (r + 500) as u32
}

fn axial(k: u32) -> (i32, i32) {
    ((k >> 16) as i32 - 500, (k & 0xffff) as i32 - 500)
}

const DIRS: [(i32, i32); 6] = [(1, 0), (1, -1), (0, -1), (-1, 0), (-1, 1), (0, 1)];

fn flat(k: u32) -> Option<Vec<u32>> {
    let (q, r) = axial(k);
    Some(DIRS.iter().map(|(dq, dr)| key(q + dq, r + dr)).collect())
}

fn ring_fence(gate_at: Option<usize>, gap_at: Option<usize>) -> BTreeMap<Edge, bool> {
    // Every edge from a cell within one of the centre to a cell two out.
    let dist = |q: i32, r: i32| (q.abs() + r.abs() + (q + r).abs()) / 2;
    let mut edges = Vec::new();
    for q in -1..=1 {
        for r in -1..=1 {
            if dist(q, r) > 1 {
                continue;
            }
            for (dq, dr) in DIRS {
                if dist(q + dq, r + dr) == 2 {
                    edges.push(Edge::new(key(q, r), key(q + dq, r + dr)));
                }
            }
        }
    }
    edges.sort();
    edges.dedup();
    assert_eq!(edges.len(), 18);
    edges
        .into_iter()
        .enumerate()
        .filter(|(i, _)| Some(*i) != gap_at)
        .map(|(i, e)| (e, Some(i) == gate_at))
        .collect()
}

#[test]
fn a_closed_loop_with_a_gate_is_a_pen_and_a_broken_loop_is_not() {
    let pen = find_pen(key(0, 0), flat, &ring_fence(Some(3), None), 64).unwrap();
    assert_eq!(pen.cells.len(), 7);
    assert_eq!(pen.gates, 1);
    assert_eq!(pen.id, *pen.cells.iter().min().unwrap());
    // Found from any cell inside, it is the same pen.
    let again = find_pen(key(1, -1), flat, &ring_fence(Some(3), None), 64).unwrap();
    assert_eq!(again, pen);
    assert_eq!(
        find_pen(key(0, 0), flat, &ring_fence(Some(3), Some(7)), 64),
        Err(PenRefusal::Open)
    );
    assert_eq!(
        find_pen(key(0, 0), flat, &ring_fence(None, None), 64),
        Err(PenRefusal::NoGate)
    );
    // Outside the ring is no pen.
    assert_eq!(
        find_pen(key(3, 0), flat, &ring_fence(Some(3), None), 64),
        Err(PenRefusal::Open)
    );
    assert_eq!(Edge::new(5, 9), Edge::new(9, 5));
    assert_eq!(Edge::new(5, 9).cells(), (5, 9));
}

#[test]
fn a_removed_piece_is_not_standing() {
    let mut records = Records::new();
    let e = Edge::new(1, 2);
    records.put(
        FenceBody {
            gate: true,
            state: Standing::Placed,
        }
        .record(e),
    );
    records.put(FenceBody::default().record(Edge::new(2, 3)));
    assert_eq!(record::pieces(&records).len(), 2);
    assert!(record::pieces(&records)[&e]);
    records.put(
        FenceBody {
            gate: true,
            state: Standing::Removed,
        }
        .record(e),
    );
    assert_eq!(record::pieces(&records).len(), 1);
}

// ---- 6.2 needs and catch-up ----------------------------------------------------

fn advance_all(s: &FarmState, to: f64, c: Conditions, sp: &LandSpecies, sex: Sex) -> FarmState {
    s.advance(
        to,
        c,
        sp.keep.as_ref().unwrap(),
        sex,
        1.0,
        &settings().needs,
    )
}

fn close(a: &FarmState, b: &FarmState) {
    assert!((a.as_of - b.as_of).abs() < 1e-9);
    assert!((a.health - b.health).abs() < 1e-3, "{a:?} {b:?}");
    assert!((a.hungry - b.hungry).abs() < 1e-3, "{a:?} {b:?}");
    for (k, v) in &a.made {
        assert!((v - b.made[k]).abs() < 1e-3, "{k}: {a:?} {b:?}");
    }
}

/// `world-persistence`'s rule for every process: N hours stepwise equals N
/// hours at once, across a hand feed running out, sickness and recovery.
#[test]
fn the_closed_form_for_n_hours_equals_n_stepwise_hours() {
    let n = settings().needs;
    for sp in ["meadow-hen", "woolback", "plodder"].map(species) {
        for c in [
            Conditions {
                grazing: true,
                water: false,
                shelter: false,
                space: true,
            },
            Conditions {
                grazing: false,
                water: true,
                shelter: false,
                space: true,
            },
            Conditions {
                grazing: false,
                water: false,
                shelter: false,
                space: true,
            },
            Conditions::default(),
        ] {
            let mut start = FarmState::new(10.0);
            start.health = 15.0;
            start.hand_feed(10.0, &n);
            let hours = 24 * 9;
            let at_once = advance_all(&start, 10.0 + hours as f64 / 24.0, c, &sp, Sex::F);
            let mut stepped = start.clone();
            for h in 1..=hours {
                stepped = advance_all(&stepped, 10.0 + h as f64 / 24.0, c, &sp, Sex::F);
            }
            close(&at_once, &stepped);
        }
    }
}

#[test]
fn neglect_never_kills_and_an_unfed_animal_strays_after_five_days() {
    let n = settings().needs;
    let sp = species("woolback");
    let bare = Conditions {
        space: true,
        ..Conditions::default()
    };
    let s = advance_all(&FarmState::new(0.0), 30.0, bare, &sp, Sex::F);
    assert_eq!(s.health, 0.0);
    assert!(s.sick(&n));
    assert!(s.strayed(&n));
    assert_eq!(s.waiting("wool"), 0, "a sick animal gives nothing");
    let early = advance_all(&FarmState::new(0.0), 4.0, bare, &sp, Sex::F);
    assert!(!early.strayed(&n));
    // Food brings it back.
    let mut back = s.clone();
    back.hand_feed(30.0, &n);
    assert!(!back.strayed(&n));
    let healed = advance_all(&back, 32.0, bare, &sp, Sex::F);
    assert_eq!(healed.health, 20.0);
}

/// The world never overwrites what the player wrote: a proposal changing an
/// animal's pen, which the player set, is refused whole.
#[test]
fn a_process_proposal_changing_a_player_written_field_is_refused() {
    let mut records = Records::new();
    let mut yields = YieldSet::new();
    let body = AnimalBody {
        pen: Some(9),
        ..AnimalBody::default()
    };
    let rec = body.record(1);
    yields.touch_record(None, &rec);
    records.put(rec);
    let moved = AnimalBody {
        pen: Some(10),
        ..body.clone()
    };
    let refused = yields.check(
        &Proposal {
            edits: vec![],
            records: vec![moved.record(1)],
        },
        &records,
    );
    assert!(refused.is_err());
}

// ---- 6.3 products --------------------------------------------------------------

/// A day's output for a known herd against the Economy tab's numbers: a hen
/// lays an egg a day, a woolback gives 3 wool every 4 days, a plodder cow a
/// pail a day, at a yield factor of one.
#[test]
fn a_known_herds_output_matches_the_economy_tab() {
    let content = Conditions {
        grazing: true,
        water: true,
        shelter: false,
        space: true,
    };
    let mut g = Genome::default();
    g.0[2 * genome::YIELD] = 127;
    g.0[2 * genome::YIELD + 1] = 128;
    assert!((g.yield_factor() - 1.0).abs() < 1e-6);
    let n = settings().needs;
    let herd = [
        ("meadow-hen", Sex::F, 3),
        ("meadow-hen", Sex::M, 1),
        ("woolback", Sex::F, 2),
        ("plodder", Sex::F, 1),
        ("plodder", Sex::M, 1),
    ];
    let mut total: BTreeMap<String, u32> = BTreeMap::new();
    for (id, sex, count) in herd {
        let sp = species(id);
        for _ in 0..count {
            let s = FarmState::new(0.0).advance(
                4.0,
                content,
                sp.keep.as_ref().unwrap(),
                sex,
                g.yield_factor(),
                &n,
            );
            for good in ["egg", "wool", "milk"] {
                *total.entry(good.into()).or_default() += s.waiting(good);
            }
        }
    }
    assert_eq!(total["egg"], 12, "three hens, four days");
    assert_eq!(total["wool"], 6, "two woolbacks, one shearing each");
    assert_eq!(total["milk"], 4, "one cow, four days");
    // Collecting keeps the fraction, and the wait is capped.
    let sp = species("plodder");
    let mut s =
        FarmState::new(0.0).advance(2.5, content, sp.keep.as_ref().unwrap(), Sex::F, 1.0, &n);
    assert_eq!(s.collect("milk"), 2);
    assert!((s.made["milk"] - 0.5).abs() < 1e-4);
    let long =
        FarmState::new(0.0).advance(100.0, content, sp.keep.as_ref().unwrap(), Sex::F, 1.0, &n);
    assert_eq!(long.waiting("milk"), 4, "four days at most");
}

// ---- herds ---------------------------------------------------------------------

#[test]
fn herds_are_derived_from_the_seed_alone() {
    let s = settings();
    let roster = s.roster("home");
    let radius = 4800.0;
    let squares = herds::squares_near(
        glam::Vec3::new(0.3, 0.5, 0.8).normalize(),
        400.0,
        radius,
        &s.herds,
    );
    assert!(squares.len() > 9);
    let all: Vec<_> = squares
        .iter()
        .filter_map(|q| herds::herd_in(42, *q, radius, &s.herds, roster))
        .collect();
    assert!(!all.is_empty());
    let again: Vec<_> = squares
        .iter()
        .filter_map(|q| herds::herd_in(42, *q, radius, &s.herds, roster))
        .collect();
    assert_eq!(all, again);
    for h in &all {
        let sp = &roster[h.species as usize];
        assert!(h.members.len() >= sp.herd.0 as usize && h.members.len() <= sp.herd.1 as usize);
        assert_eq!(herds::square_of(h.centre, radius, &s.herds), h.square);
    }
}

// ---- 7 and 8: eggs, young and breeding (design 13) -------------------------

mod breeding_tests {
    use super::super::breeding::{self, BroodBody, Fitness, Pedigree, Place};
    use super::*;

    fn index(id: &str) -> u16 {
        settings()
            .roster(crate::fauna::HOME_BODY)
            .iter()
            .position(|s| s.id == id)
            .unwrap() as u16
    }

    fn adult(sp: &str, sex: Sex, born: f64, pen: u32, seed: u64) -> AnimalBody {
        let k = species(sp).keep.unwrap();
        AnimalBody {
            sp: index(sp),
            sex,
            born,
            genes: Genome::roll(seed, k.coats.len(), k.patterns.len())
                .0
                .to_vec(),
            pen: Some(pen),
            home: pen,
            as_of: born + f64::from(k.adult_days),
            fed_until: born + f64::from(k.adult_days),
            wild: seed,
            ..AnimalBody::default()
        }
    }

    fn content(_: u64, _: &AnimalBody) -> Fitness {
        Fitness {
            mood: 75.0,
            health: 100.0,
        }
    }

    fn lit(_: u32) -> bool {
        true
    }

    fn apply(records: &mut Records, events: &[breeding::FarmEvent]) {
        for e in events {
            for r in &e.records {
                records.put(r.clone());
            }
        }
    }

    /// 7.1: an egg set on day d hatches on day d + 3 into a chick that is a
    /// laying hen from day d + 13, and not before.
    #[test]
    fn an_egg_set_on_day_d_hatches_on_d_plus_3_and_lays_from_d_plus_13() {
        let s = settings();
        let hen_sp = species("meadow-hen");
        let keep = hen_sp.keep.clone().unwrap();
        let mut records = Records::new();
        let hen = adult("meadow-hen", Sex::F, 0.0, 7, 11);
        let cock = adult("meadow-hen", Sex::M, 0.0, 7, 12);
        records.put(hen.record(1));
        records.put(cock.record(2));
        let d = 40.0;
        assert_eq!(
            breeding::cockerel_for(&records, &hen, &keep, d),
            Some(2),
            "her eggs are fertile"
        );
        let clutch = BroodBody {
            sp: index("meadow-hen"),
            dam: 1,
            sire: 2,
            eggs: 4,
            laid: d - 0.5,
            set: Some(d),
            place: Place::Hen(1),
            ..BroodBody::default()
        };
        records.put(clutch.record(1));
        let roster = s.roster(crate::fauna::HOME_BODY);
        let before = breeding::farm_events(&records, roster, &s.breeding, d + 2.99, &content, &lit);
        assert!(before.is_empty(), "nothing hatches before d + 3");
        let events = breeding::farm_events(&records, roster, &s.breeding, d + 3.0, &content, &lit);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].day, d + 3.0);
        assert_eq!(events[0].records.len(), 4, "every egg hatches");
        let chick: AnimalBody = events[0].records[0].read().unwrap();
        assert_eq!(chick.born, d + 3.0);
        assert_eq!(chick.brood, Some(1));
        assert_eq!((chick.sire, chick.dam), (Some(2), Some(1)));
        assert_eq!(keep.young.stage(chick.sex, chick.born, d + 3.5).0, "chick");
        assert_eq!(keep.young.stage(Sex::F, chick.born, d + 9.0).0, "pullet");
        assert_eq!(keep.young.stage(Sex::F, chick.born, d + 13.0), ("hen", 1.0));
        // She lays from d + 13: products only from adulthood.
        let lays_from = chick.born + f64::from(keep.adult_days);
        assert_eq!(lays_from, d + 13.0);
        let c = Conditions {
            grazing: true,
            water: true,
            shelter: false,
            space: true,
        };
        let farm = FarmState::new(chick.born);
        let at_12 = farm.advance_from(d + 12.99, c, &keep, Sex::F, 1.0, &s.needs, lays_from);
        assert_eq!(at_12.waiting("egg"), 0, "a pullet lays nothing");
        let at_15 = farm.advance_from(d + 15.0, c, &keep, Sex::F, 1.0, &s.needs, lays_from);
        assert_eq!(at_15.waiting("egg"), 2, "two days of a hen's laying");
        // Hatched once: the brood is done.
        apply(&mut records, &events);
        assert!(
            breeding::farm_events(&records, roster, &s.breeding, d + 30.0, &content, &lit)
                .is_empty()
        );
    }

    /// 7.1: an incubator without a lamp beside it holds its eggs; carried
    /// eggs go cold after a day and give no parentage.
    #[test]
    fn an_incubator_needs_its_lamp_and_carried_eggs_go_cold() {
        let s = settings();
        let mut records = Records::new();
        records.put(adult("meadow-hen", Sex::F, 0.0, 7, 11).record(1));
        records.put(adult("meadow-hen", Sex::M, 0.0, 7, 12).record(2));
        let clutch = BroodBody {
            sp: index("meadow-hen"),
            dam: 1,
            sire: 2,
            eggs: 3,
            laid: 20.0,
            set: Some(20.2),
            place: Place::Incubator(99),
            ..BroodBody::default()
        };
        records.put(clutch.record(1));
        let roster = s.roster(crate::fauna::HOME_BODY);
        let dark = breeding::farm_events(&records, roster, &s.breeding, 30.0, &content, &|_| false);
        assert!(dark.is_empty(), "no lamp, no hatch");
        let lit_now = breeding::farm_events(&records, roster, &s.breeding, 30.0, &content, &lit);
        assert_eq!(
            lit_now[0].day, 23.2,
            "born on its due day, however late it is seen"
        );
        let carried = BroodBody {
            sp: index("meadow-hen"),
            dam: 1,
            sire: 2,
            eggs: 5,
            laid: 40.0,
            place: Place::Carried,
            ..BroodBody::default()
        };
        records.put(carried.record(2));
        let (warm, left) = breeding::take_parentage(&records, 3, 40.5, &s.breeding);
        assert_eq!((warm.len(), warm[0].2, left), (1, 3, 0));
        let (cold, left) = breeding::take_parentage(&records, 3, 41.5, &s.breeding);
        assert!(cold.is_empty());
        assert_eq!(left, 3, "every egg is cold");
    }

    /// 8.1: a ewe and a ram penned together conceive at the first morning,
    /// lamb a gestation later, wait a gestation, and go again; catching up
    /// a long absence in one evaluation writes exactly what evaluating every
    /// half day writes.
    #[test]
    fn catching_up_in_one_evaluation_writes_what_half_days_write() {
        let s = settings();
        let roster = s.roster(crate::fauna::HOME_BODY);
        let mut base = Records::new();
        base.put(adult("woolback", Sex::F, 0.0, 5, 31).record(1));
        base.put(adult("woolback", Sex::M, 0.0, 5, 32).record(2));
        base.put(adult("woolback", Sex::F, 1.0, 5, 33).record(3));
        let end = 60.0;
        let once = breeding::farm_events(&base, roster, &s.breeding, end, &content, &lit);
        let mut stepped = base.clone();
        let mut steps = Vec::new();
        let mut t = 0.0;
        while t <= end {
            let e = breeding::farm_events(&stepped, roster, &s.breeding, t, &content, &lit);
            apply(&mut stepped, &e);
            steps.extend(e);
            t += 0.5;
        }
        assert_eq!(once, steps);
        assert!(
            once.len() > 6,
            "several conceptions and births: {}",
            once.len()
        );
        // The first conception is the first morning after the pair is adult
        // and penned (as_of 12 for the first ewe and the ram).
        assert_eq!(once[0].day, 12.25);
        assert_eq!(once[0].author.0, "pen");
        // Her first birth is a gestation (5 days) later.
        let first_birth = once
            .iter()
            .find(|e| e.author.0 == breeding::BROOD)
            .expect("a birth");
        assert_eq!(first_birth.day, 17.25);
        let lamb: AnimalBody = first_birth.records[0].read().unwrap();
        assert_eq!(lamb.via, record::Via::Born);
        assert_eq!(lamb.pen, Some(5));
    }

    /// 8.1: a world with no broods, or animals that cannot pair, writes
    /// nothing however long it is left.
    #[test]
    fn a_world_with_no_broods_writes_nothing() {
        let s = settings();
        let roster = s.roster(crate::fauna::HOME_BODY);
        let mut records = Records::new();
        // Two ewes, no ram; a loose ram; an unhappy pen.
        records.put(adult("woolback", Sex::F, 0.0, 5, 1).record(1));
        records.put(adult("woolback", Sex::F, 0.0, 5, 2).record(2));
        let mut loose = adult("woolback", Sex::M, 0.0, 5, 3);
        loose.pen = None;
        records.put(loose.record(3));
        assert!(
            breeding::farm_events(&records, roster, &s.breeding, 500.0, &content, &lit).is_empty()
        );
        records.put(adult("woolback", Sex::M, 0.0, 5, 4).record(4));
        let sad = |_: u64, _: &AnimalBody| Fitness {
            mood: 50.0,
            health: 100.0,
        };
        assert!(breeding::farm_events(&records, roster, &s.breeding, 500.0, &sad, &lit).is_empty());
        assert!(
            breeding::farm_events(&Records::new(), roster, &s.breeding, 500.0, &content, &lit)
                .is_empty()
        );
    }

    /// 8.1: the four predicted values of a quantitative gene are exactly the
    /// values births produce, over many births, apart from the mutated.
    #[test]
    fn the_four_predicted_values_are_the_ones_births_produce() {
        let s = settings();
        let keep = species("woolback").keep.unwrap();
        let sire = Genome::roll(91, 3, 2);
        let dam = Genome::roll(92, 3, 2);
        for gene in [
            genome::SIZE,
            genome::YIELD,
            genome::TEMPERAMENT,
            genome::HARDINESS,
        ] {
            let predicted = breeding::offspring_values(&sire, &dam, gene);
            let mut seen = std::collections::BTreeSet::new();
            let mut mutated = 0;
            for n in 0..4000 {
                let (young, muts) = breeding::inherit((7, &sire), (8, &dam), n, &keep, &s.breeding);
                if muts.contains(&gene) {
                    mutated += 1;
                    continue;
                }
                let v = young.value(gene);
                let at = predicted
                    .iter()
                    .position(|p| (p - v).abs() < 1e-6)
                    .unwrap_or_else(|| panic!("gene {gene}: {v} is not one of {predicted:?}"));
                seen.insert(at);
            }
            assert_eq!(seen.len(), 4, "gene {gene}: all four turn up");
            // Mutation one allele in 32: about 1 young in 16 for a gene.
            assert!(
                (150..400).contains(&mutated),
                "gene {gene}: {mutated} mutated"
            );
        }
        // The morph: two carriers give it one young in four.
        let mut a = Genome::default();
        a.0[2 * genome::MORPH] = 1;
        let b = a;
        assert_eq!(breeding::morph_quarters(&a, &b), 1);
    }

    /// 8.1: Wright's F from the pedigree: full siblings 0.25, half
    /// siblings 0.125, first cousins 0.0625, unrelated 0.
    #[test]
    fn siblings_give_f_a_quarter() {
        let mut p = Pedigree::default();
        let mut add = |id, s: Option<u64>, d: Option<u64>, born: f64| {
            p.of.insert(id, (s, d, born));
        };
        add(1, None, None, 0.0);
        add(2, None, None, 0.0);
        add(3, None, None, 0.0);
        add(4, None, None, 0.0);
        add(10, Some(1), Some(2), 20.0); // full siblings
        add(11, Some(1), Some(2), 21.0);
        add(12, Some(1), Some(3), 22.0); // half sibling of 10
        add(13, Some(4), Some(3), 22.0);
        add(20, Some(10), Some(13), 40.0); // cousins' parents: 10 and 12 are half sibs
        add(21, Some(11), Some(4), 41.0);
        let g = 4;
        assert_eq!(p.f_of_young(10, 11, g), 0.25);
        assert_eq!(p.f_of_young(10, 12, g), 0.125);
        assert_eq!(p.f_of_young(1, 4, g), 0.0);
        assert_eq!(p.f_of_young(20, 21, g), 0.0625, "first cousins");
        assert_eq!(p.f(10, g), 0.0, "a founder pair's young");
    }

    /// E10: an animal record written before breeding reads with no brood and
    /// no mate, and brood records round-trip.
    #[test]
    fn records_before_breeding_read_unchanged() {
        let old = "(body:\"home\",sp:1,sex:F,born:3.0,gen:[1,2,3,4,5,6,7,8,9,10,11,12,13,14],pen:Some(5),home:5,as_of:4.0)";
        let r = Record {
            kind: record::ANIMAL.into(),
            id: 9,
            schema: 1,
            body: old.into(),
        };
        let a: AnimalBody = r.read().unwrap();
        assert_eq!((a.brood, a.mate), (None, None));
        assert_eq!(a.pen, Some(5));
        let b = BroodBody {
            sp: 1,
            dam: 3,
            sire: 4,
            laid: 2.5,
            place: Place::Womb,
            ..BroodBody::default()
        };
        assert_eq!(b.record(7).read::<BroodBody>(), Some(b));
    }
}
