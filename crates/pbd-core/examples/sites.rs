//! The planet's city sites, from the rules the game will run
//! (`pbd_core::sites::generate`), for the world map mockup. The measurement
//! instrument for `openspec/changes/city-sites` task 1.1: it prints how long
//! each stage took and each kind's count, and writes the list as JSON.
//!
//!     cargo run --release -p pbd-core --example sites -- [out.json] [threads]
//!
//! The spawn is the game's default spawn direction, so the small town near
//! the spawn (survey C2) is placed where a new player starts.

use glam::Vec3;
use pbd_core::geo;
use pbd_core::planet_gen::TerrainConfig;
use pbd_core::sites::{self, Cells, SiteKind, SitesConfig};
use std::time::Instant;

/// `desktop.rs`'s default spawn direction.
const SPAWN: Vec3 = Vec3::new(0.8776, 0.4794, 0.0);

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let out = args.get(1).cloned().unwrap_or_else(|| "sites.json".into());
    let threads: usize = args
        .get(2)
        .and_then(|a| a.parse().ok())
        .unwrap_or_else(|| std::thread::available_parallelism().map_or(1, |n| n.get()));
    let text = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../assets/config/sites.ron"
    ))
    .expect("assets/config/sites.ron");
    let cfg: SitesConfig = ron::from_str(&text).expect("sites.ron parses");
    cfg.validate().expect("sites.ron is valid");
    let terrain = TerrainConfig::TENEBRIS;
    cfg.validate_pins(&terrain).expect("the pins stand on land");
    let spawn = SPAWN.normalize();

    let t = Instant::now();
    let cells = Cells::new(cfg.level);
    let cells_s = t.elapsed().as_secs_f64();
    let t = Instant::now();
    let found = sites::candidates(&cfg, &terrain, &cells, threads);
    let screen_s = t.elapsed().as_secs_f64();
    let t = Instant::now();
    let list = sites::select(&cfg, &terrain, &cells, &found, spawn);
    let select_s = t.elapsed().as_secs_f64();

    println!(
        "{} cells in {cells_s:.2} s; screen and score on {threads} threads in {screen_s:.2} s, {} candidates; the pins, the home town, the greedy pass, the capital and the names in {select_s:.2} s",
        cells.directions.len(),
        found.len()
    );
    for kind in SiteKind::ALL {
        let candidates = found.iter().filter(|c| c.kind == kind).count();
        let kept = list.sites.iter().filter(|s| s.kind == kind).count();
        let want = cfg.rule(kind).count;
        println!(
            "  {:<15} {kept:>3} of {want:>3} kept, from {candidates:>6} candidates",
            kind.name()
        );
    }
    for s in &list.sites {
        if s.home || s.capital {
            let (lat, lon) = geo::lat_lon(s.direction).degrees();
            println!(
                "  {}: {} ({}) at {lat:.2}, {lon:.2}, {:.0} m from the spawn",
                if s.capital { "capital" } else { "home" },
                s.name,
                s.kind.name(),
                sites::arc_m(s.direction, spawn, terrain.radius_m)
            );
        }
    }

    // The mockup's sites layer: each site's place, kind, name and footprint.
    let json: Vec<serde_json::Value> = list
        .sites
        .iter()
        .map(|s| {
            let (lat, lon) = geo::lat_lon(s.direction).degrees();
            serde_json::json!({
                "id": s.id,
                "kind": format!("{:?}", s.kind),
                "kind_name": s.kind.name(),
                "name": s.name,
                "lat": lat,
                "lon": lon,
                "radius_m": cfg.rule(s.kind).radius_m,
                "capital": s.capital,
                "home": s.home,
                "pinned": s.pinned,
                "river": s.river,
            })
        })
        .collect();
    let (spawn_lat, spawn_lon) = geo::lat_lon(spawn).degrees();
    let doc = serde_json::json!({
        "sites_version": cfg.version,
        "generator": pbd_core::terrain::GENERATOR_VERSION,
        "seed": terrain.seed,
        "spawn": { "lat": spawn_lat, "lon": spawn_lon },
        "kinds": SiteKind::ALL.iter().map(|&k| {
            let rule = cfg.rule(k);
            serde_json::json!({
                "kind": format!("{k:?}"), "name": k.name(), "radius_m": rule.radius_m,
                "spacing_m": rule.spacing_m, "count": rule.count,
                "candidates": found.iter().filter(|c| c.kind == k).count(),
            })
        }).collect::<Vec<_>>(),
        "strikes": cfg.strikes,
        "shortfall": list.shortfall.iter().map(|(k, n)| serde_json::json!({"kind": format!("{k:?}"), "missing": n})).collect::<Vec<_>>(),
        "timings_s": { "cells": cells_s, "screen": screen_s, "select": select_s, "threads": threads },
        "sites": json,
    });
    std::fs::write(&out, serde_json::to_string_pretty(&doc).expect("json"))
        .expect("write the list");
    println!("wrote {} sites to {out}", list.sites.len());
}
