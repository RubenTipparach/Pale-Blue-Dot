//! `world-frames` guard 4: the code that simulates in local space takes
//! "up" and a radius only through the origin (`SimOrigin::up_at`,
//! `radius_at`, `with_radius`), never from a local position's own
//! direction, which is meaningless once the origin is not the planet's
//! centre. This reads the sources of the converted simulation (test modules
//! left out) and fails on a new `position.0.normalize()`-style site.

use std::path::Path;

/// The files whose simulation runs in local space.
const FILES: [&str; 16] = [
    "crates/pbd-app/src/walking.rs",
    "crates/pbd-app/src/flight_view.rs",
    "crates/pbd-app/src/fish.rs",
    "crates/pbd-app/src/sim_origin.rs",
    "crates/pbd-core/src/fauna/school.rs",
    "crates/pbd-core/src/fishing.rs",
    // `world-frames` 10: the herd and drops (the decks went with the cog).
    "crates/pbd-app/src/husbandry.rs",
    "crates/pbd-app/src/husbandry/breed.rs",
    "crates/pbd-app/src/drops.rs",
    // `fish-behaviour`: the drawn poses and the line's fish.
    "crates/pbd-core/src/fauna/swim.rs",
    // `ocean-environments` B29: the beavers and otters drawn on the fauna's
    // creatures.
    "crates/pbd-app/src/critters.rs",
    // `townsfolk` 25: the townspeople, placed in the planet's frame in f64.
    "crates/pbd-core/src/townsfolk.rs",
    "crates/pbd-app/src/townsfolk.rs",
    // `townsfolk` 28: the lost villager, stored and walked in the body's
    // frame in f64.
    "crates/pbd-app/src/economy/special.rs",
    // `land-transport` 11: the carts, kept in the body's frame in f64.
    "crates/pbd-core/src/vehicle/cart.rs",
    "crates/pbd-app/src/vehicles/carts.rs",
];

/// What taking up or a radius from a local position looks like.
const PATTERNS: [&str; 20] = [
    "position.0.normalize",
    "position.0.length()",
    "position.normalize",
    "position.length()",
    "accepted.normalize",
    "accepted.length()",
    "candidate.normalize",
    "candidate.length()",
    "float.normalize",
    "float.length()",
    "feet.normalize",
    "anchor.normalize",
    // An animal's feet and goal (`husbandry.rs`), a drop's walker, a deck's
    // carried point.
    "pos.normalize",
    "pos.length()",
    "goal.normalize",
    "goal.length()",
    "feet.length()",
    "walker.normalize",
    "on_deck_at.normalize",
    "next.normalize",
];

/// The source with every `#[cfg(test)]` item cut out.
fn without_tests(source: &str) -> String {
    let mut out = String::new();
    let mut lines = source.lines();
    while let Some(line) = lines.next() {
        if line.trim_start().starts_with("#[cfg(test)]") {
            // Skip the item: to the end of its braces, or its one line.
            let mut depth = 0i32;
            let mut opened = false;
            for next in lines.by_ref() {
                depth += next.matches('{').count() as i32 - next.matches('}').count() as i32;
                opened |= next.contains('{');
                if (opened && depth <= 0) || (!opened && next.trim_end().ends_with(';')) {
                    break;
                }
            }
            continue;
        }
        out.push_str(line);
        out.push('\n');
    }
    out
}

#[test]
fn local_simulation_takes_up_only_through_the_origin() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut found = Vec::new();
    for file in FILES {
        let source = std::fs::read_to_string(root.join(file)).expect(file);
        for (n, line) in without_tests(&source).lines().enumerate() {
            let code = line.split("//").next().unwrap_or("");
            for pattern in PATTERNS {
                if code.contains(pattern) {
                    found.push(format!("{file}: `{}`", line.trim()));
                }
            }
            let _ = n;
        }
    }
    assert!(
        found.is_empty(),
        "up or a radius taken from a local position, not through the origin \
         (`world-frames` design 6):\n{}",
        found.join("\n")
    );
}
