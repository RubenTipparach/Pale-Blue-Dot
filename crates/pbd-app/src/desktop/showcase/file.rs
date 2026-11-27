//! A showcase's shot list (`step-videos` 2.1, 5.1): `assets/showcase/<name>.ron`.
//!
//! A showcase is a bot playing the game toward a goal it states at the start
//! (CLAUDE.md, "Demos: tech montages and a bot playing to a goal"). The file
//! says where the bot starts, what its goal is, and the shots: each a caption,
//! the missions it must have finished by its end, how long it may take, the
//! clock's pace while it runs, and the acts the bot plays through the
//! player's own inputs.

use serde::Deserialize;
use std::path::PathBuf;

/// The whole file.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Showcase {
    /// The goal, said as the first caption: "Goal: finish the starter
    /// missions".
    pub goal: String,
    /// `--spawn`'s place names (`fields`, `beach`, ...), for a new world's
    /// start.
    #[serde(default)]
    pub spawn: Option<String>,
    /// `--at <lat> <lon>`, degrees.
    #[serde(default)]
    pub at: Option<(f32, f32)>,
    /// `--yaw`, degrees right of the default heading.
    #[serde(default)]
    pub yaw: Option<f32>,
    /// The hour the clock starts at; it runs from there.
    #[serde(default = "nine")]
    pub hour: f32,
    /// The day of the year, for a season.
    #[serde(default)]
    pub day: Option<u32>,
    /// The new world is a Survival one (`survival-mode` 6.4); Explorer when
    /// absent.
    #[serde(default)]
    pub survival: bool,
    pub shots: Vec<Shot>,
}

fn nine() -> f32 {
    9.0
}

fn two_minutes() -> f32 {
    120.0
}

/// One shot.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Shot {
    /// What the caption bar says while it runs.
    pub caption: String,
    /// Missions that are done by the shot's end. A shot whose acts finish
    /// before they are plays its acts again (`repeat`) or fails the run.
    #[serde(default)]
    pub until: Vec<String>,
    /// Play the acts again until `until` is met.
    #[serde(default)]
    pub repeat: bool,
    /// The most real seconds the shot may take before the run fails as
    /// stalled.
    #[serde(default = "two_minutes")]
    pub max_s: f32,
    /// The clock's pace while the shot runs, times its own; one when absent.
    /// Said in the caption whenever it is more ("Three days later").
    #[serde(default)]
    pub rate: Option<f32>,
    pub acts: Vec<Act>,
}

/// When a looping act stops.
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub enum Until {
    /// This many of an item carried (`log`, `fibre`, `fish`, `seed`).
    Have(String, u32),
    /// A mission done.
    Mission(String),
    /// The act's own deed this many times (a block dug, a fish caught).
    Times(u32),
}

/// What the bot does, each through the player's own input path: keys and
/// buttons pressed in `ButtonInput`, the look moved by the mouse's motion,
/// the hotbar picked by its number keys and the tool by G and the wheel, a
/// panel's controls pressed as the pointer presses them.
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub enum Act {
    /// Stand and watch for this many seconds.
    Wait(f32),
    /// Stand and watch until the clock reaches this hour (with the shot's
    /// `rate`, a captioned wait for the morning).
    Hour(f32),
    /// Turn the look: yaw right by degrees, to an absolute pitch, over
    /// seconds.
    Look(f32, f32, f32),
    /// Walk ahead for seconds.
    Walk(f32),
    /// Smelt at the nearest forge: open it with F, press the recipe's
    /// plaque this many times, collect.
    Smelt(String, u32),
    /// Open the map with M for seconds, then close it.
    Map(f32),
    /// Walk this many metres from where the act began, over dry ground,
    /// keeping on as straight as the land allows.
    Away(f32),
    /// Mark a place this many metres away on the map: open it with M, put
    /// the pointer on dry ground that far off, right-click, close it.
    Mark(f32),
    /// Plan from the map (`mission-playthrough` 5.8): open it with M, pick a
    /// layer by its legend row (`places`, `biomes`), find the pixel of a
    /// class of its key (`reef`, `kelp_forest`, `desert`) nearest the
    /// player, zoom in on it, right-click a mark there, close the map. The
    /// `ToMark` after it walks there.
    MapGoal(String, String),
    /// Walk to the newest waypoint on the map, in legs; a mark at sea ends
    /// at the coast, facing the water, for a `Swim`.
    ToMark,
    /// From the coast into the water and across, swimming on the surface to
    /// the place of a kind (`reef`: the nearest reef rock to the mark;
    /// `mark`: the mark), then down to it, looking down for a `Dig`
    /// (`mission-playthrough` 5.7, legs 2 to 4).
    Swim(String),
    /// The right button once, on what is aimed at, with what is in hand
    /// (the bucket filled at the water or poured).
    Use,
    /// The tool in hand by G and the wheel: `axe`, `shovel`, `pickaxe`,
    /// `rod`, or `hand` for bare hands.
    Hand(String),
    /// Put an item in the selected slot, from the pack if need be.
    Select(String),
    /// Work trees for logs until.
    Chop(Until),
    /// Dig fresh ground ahead, `depth` blocks down a cell, until.
    Dig(Until, u32),
    /// Cut a stair down into the ground ahead with the tool in hand, level
    /// `depth` layers down, until (a player mining for stone and ore).
    Mine(Until, u32),
    /// The pack's Craft panel: a recipe, this many times.
    Craft(String, u32),
    /// Set a block or a station down on the ground ahead.
    Place(String),
    /// At the nearest placed workbench: a recipe, this many times.
    Bench(String, u32),
    /// At the nearest placed campfire: a dish, this many times.
    Cook(String, u32),
    /// Walk to the sea's edge and face the water.
    Shore,
    /// Walk to a river's or a lake's edge (water above the sea's level) and
    /// face it; the `Fish` after it fishes there.
    Fresh,
    /// Go to the sea's edge, cast, wait for the bite, reel, until.
    Fish(Until),
    /// Till this many cells of a field with the shovel.
    Till(u32),
    /// Sow the field's tilled cells with the seed in hand.
    Plant(u32),
    /// Pick this many wild plants.
    Pick(u32),
    /// Pick up stones, or pull fibre from bushes, shrubs and ferns, with F
    /// (`gather-by-hand`), until: `Have("stone", n)` or `Have("fibre", n)`.
    PickUp(Until),
    /// Harvest the field's ripe crops.
    Harvest(u32),
    /// Feed a wild animal of this species (`any` for the nearest one the
    /// carried food suits) until it is tamed.
    Tame(String),
    /// A ring of fence round the cell underfoot and its neighbours, a gate
    /// last.
    Pen,
    /// Lead a kept animal of this species into the pen and let it stay.
    Lead(String),
    /// F on a kept animal of this species until it gives what it has.
    Collect(String),
    /// Eat the dish in hand (selected first).
    Eat(String),
    /// Open the missions tab (O) for seconds, then shut it.
    Missions(f32),
    /// Open the pack with Tab for seconds, then shut it with Tab
    /// (`starter-missions`, "Open your pack").
    Pack(f32),
    /// Walk to the nearest chest, open it with F and put one stack of the
    /// ground's own blocks (soil, sand, stone) in it (`starter-missions`,
    /// "A chest").
    Store,
    /// Open the pack and drop every stack of the ground's own blocks (soil,
    /// sand, stone, grass) off the panels, then shut it: a full pack cleared
    /// before a shot that needs room.
    Declutter,
    /// Pick the mission to be on: open the tree (O), click its card, shut
    /// it (`mission-playthrough` 3.5).
    Focus(String),
    /// Talk to this many townsfolk, each one not talked to before: walk up
    /// to the nearest, look at them, F, and shut the talk with F
    /// (`mission-playthrough` design 6, the town tree's "meet").
    Talk(u32),
    /// At the town's trader: talk to townsfolk until one offers TRADE,
    /// press it, sell this many goods (one press each, the first row the
    /// trader buys), buy this many (each press the next row), and shut the
    /// talk (`mission-playthrough` design 6, "first-trade").
    Trade(u32, u32),
    /// One town job, played through: townsfolk asked until one offers WORK,
    /// the first posted job of a kind the bot plays (trade, fish, mine)
    /// taken (any other given back), its work done, and handed in
    /// (`mission-playthrough` design 6, "first-job").
    Job,
    /// A town's furniture used as a player uses it: "sit" on a seat, "lie"
    /// on a bed (then up again with F), or "loot" a store (opened with F,
    /// its coins or one stack taken, shut). The nearest free piece in a
    /// town; one that will not take the bot is passed over
    /// (`mission-playthrough` design 6, "sit", "lie", "find").
    Furniture(String),
    /// A still, when `--showcase-stills` names a directory.
    Still,
    /// Go to the nearest place of a kind (`mission-playthrough` design 5):
    /// `hot`, `temperate` or `cold` dry ground, a `reef` (swum onto) or a
    /// `town`.
    Seek(String),
    /// Pick a path on the path picker (`survival-mode` 6.5): the Missions
    /// screen opened with O (the picker comes up with it while a path is to
    /// be picked, from its Paths plaque otherwise), the path's card pressed,
    /// the screen shut.
    Path(String),
    /// An empty hand: no tool (G and the wheel) and an empty hotbar slot
    /// picked by its number key.
    Empty,
    /// F this many times at what is ahead, a second and a half apart: a sip
    /// at the water each, with an empty hand.
    Drink(u32),
    /// Hunt with the weapon in hand: walk up to the nearest wild animal of
    /// these roster ids (`|` between them), look at it and hold the left
    /// button, a carcass struck again to dress it; until.
    Hunt(String, Until),
    /// After a death (`survival-mode` design 8): Enter on "You died" to wake
    /// up, then walk to the newest dropped bag until it is picked up. The
    /// bot puts this in front of whatever it was doing when it dies.
    Revive,
    /// A key pressed once, as the player presses it (`F`, `W`, ...).
    Tap(String),
    /// A key held for seconds.
    Hold(String, f32),
}

/// Where the showcases live.
pub fn dir() -> PathBuf {
    pbd_app::config::config_dir()
        .parent()
        .map_or_else(|| PathBuf::from("assets"), PathBuf::from)
        .join("showcase")
}

/// Read and parse `assets/showcase/<name>.ron`.
pub fn load(name: &str) -> Result<Showcase, String> {
    let path = dir().join(format!("{name}.ron"));
    let text = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    parse(&text).map_err(|e| format!("{}: {e}", path.display()))
}

/// Parse a shot list.
pub fn parse(text: &str) -> Result<Showcase, String> {
    let file: Showcase = ron::from_str(text).map_err(|e| e.to_string())?;
    if file.shots.is_empty() {
        return Err("a showcase has at least one shot".into());
    }
    for (n, shot) in file.shots.iter().enumerate() {
        if shot.acts.is_empty() {
            return Err(format!("shot {} has no acts", n + 1));
        }
        if !(shot.max_s.is_finite() && shot.max_s > 0.0) {
            return Err(format!("shot {} max_s must be positive", n + 1));
        }
        if let Some(rate) = shot.rate
            && !(rate.is_finite() && rate > 0.0)
        {
            return Err(format!("shot {} rate must be positive", n + 1));
        }
        if shot.repeat && shot.until.is_empty() {
            return Err(format!("shot {} repeats with nothing to wait for", n + 1));
        }
    }
    if !(0.0..24.0).contains(&file.hour) {
        return Err("hour must be in 0..24".into());
    }
    Ok(file)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_short_showcase_parses() {
        let file = parse(
            r#"(
                goal: "Goal: a log",
                spawn: Some("fields"),
                shots: [
                    (caption: "Chop", until: ["gather"], repeat: true, max_s: 60.0,
                     acts: [Chop(Have("log", 1)), Dig(Mission("gather"), 2)]),
                    (caption: "Three days later", rate: Some(200.0),
                     acts: [Wait(5.0), Still]),
                ],
            )"#,
        )
        .unwrap();
        assert_eq!(file.shots.len(), 2);
        assert_eq!(file.hour, 9.0);
        assert_eq!(
            file.shots[0].acts[0],
            Act::Chop(Until::Have("log".into(), 1))
        );
        assert_eq!(file.shots[1].rate, Some(200.0));
    }

    #[test]
    fn the_map_goal_and_the_swim_parse() {
        let file = parse(
            r#"(goal: "x", shots: [(caption: "Reef",
                acts: [MapGoal("places", "reef"), ToMark, Swim("reef")])])"#,
        )
        .unwrap();
        assert_eq!(
            file.shots[0].acts,
            vec![
                Act::MapGoal("places".into(), "reef".into()),
                Act::ToMark,
                Act::Swim("reef".into()),
            ]
        );
    }

    #[test]
    fn a_bad_showcase_is_refused() {
        assert!(parse("(goal: \"x\", shots: [])").is_err());
        assert!(
            parse("(goal: \"x\", shots: [(caption: \"a\", repeat: true, acts: [Wait(1.0)])])")
                .is_err()
        );
    }

    /// Every showcase in `assets/showcase` parses, and names only missions
    /// the starter tree or the town tree has.
    #[test]
    fn every_showcase_file_parses() {
        let read = |f: &str| std::fs::read_to_string(pbd_app::config::config_dir().join(f));
        let chain_text = read("missions.ron").unwrap() + &read("town_missions.ron").unwrap();
        let mut count = 0;
        for entry in std::fs::read_dir(dir()).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().is_none_or(|e| e != "ron") {
                continue;
            }
            let file = parse(&std::fs::read_to_string(&path).unwrap())
                .unwrap_or_else(|e| panic!("{}: {e}", path.display()));
            for shot in &file.shots {
                for id in &shot.until {
                    assert!(
                        chain_text.contains(&format!("id: \"{id}\"")),
                        "{}: no mission {id}",
                        path.display()
                    );
                }
            }
            count += 1;
        }
        assert!(count > 0, "no showcases in {}", dir().display());
    }
}
