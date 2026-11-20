//! The breeding screen (`animal-husbandry` design 13, E9), after M7's
//! Breeding tab: the herd of one kept species, a sire and a dam picked from
//! it, every young the pair can give before they are paired, and a pedigree.
//!
//! B opens it on foot and B or Escape closes it. It is a `menu::Screen`, so it
//! holds the pointer and the keyboard as the field guide does. Every number on
//! it is the core's: the four values of each gene
//! (`breeding::offspring_values`), the coat, pattern and morph odds, and
//! Wright's F off the stored parents (`breeding::Pedigree`). PAIR writes the
//! chosen sire on the dam as the player's (`husbandry::breed::pair`).
//!
//! Labels only on screen (CLAUDE.md, "Notes stay out of the game").

use super::menu::{self, MenuAction, Panel, Screen};
use bevy::prelude::*;
use pbd_app::config::HusbandryConfig;
use pbd_app::fish::Body;
use pbd_app::husbandry::BreedingPick;
use pbd_app::saves::WorldSave;
use pbd_app::ui_kit::{self, Set, UiKit, plaque};
use pbd_app::walking::WalkingReadout;
use pbd_core::husbandry::breeding::{self, Pedigree};
use pbd_core::husbandry::record::{self, AnimalBody, Life};
use pbd_core::husbandry::{Genome, Keeping, LandSpecies, Sex, genome};
use pbd_core::locale::Localized;
use std::collections::BTreeMap;

/// The part of the panel rebuilt when the pick or the herd changes.
#[derive(Component)]
pub struct BreedingBody;

const INK: Color = ui_kit::TEXT;
const DIM: Color = ui_kit::TEXT_DIM;
const AMBER: Color = ui_kit::TITLE;
const SIRE: Color = Color::srgb(0.48, 0.74, 0.88);
const DAM: Color = Color::srgb(0.90, 0.52, 0.62);
const WARN: Color = Color::srgb(0.93, 0.42, 0.36);
/// A gene bar's width, native pixels.
const BAR: f32 = 150.0;

fn text(value: impl Into<String>, size: f32, colour: Color) -> impl Bundle {
    (
        Text::new(value.into()),
        TextFont {
            font_size: size,
            ..default()
        },
        TextColor(colour),
    )
}

/// Build the panel, hidden. `PostStartup`.
pub fn spawn(mut commands: Commands, kit: Res<UiKit>) {
    commands
        .spawn((menu::layer(), GlobalZIndex(10), Panel(Screen::Breeding)))
        .with_children(|layer| {
            layer
                .spawn(menu::framed(&kit, Set::Stone, 600.0))
                .with_children(|panel| {
                    menu::lintel(panel, &kit, Localized::legacy("BREEDING"));
                    panel.spawn((
                        Node {
                            flex_direction: FlexDirection::Column,
                            row_gap: px(4),
                            ..default()
                        },
                        BreedingBody,
                    ));
                });
        });
}

/// B opens the screen on foot and closes it again. `PreUpdate` after
/// `menu::toggle`, like every key a menu owns. In a craft B is the brake or
/// the bail, so only a walker opens it.
pub fn open(
    keys: Res<ButtonInput<KeyCode>>,
    mut screen: ResMut<Screen>,
    walking: Option<Res<WalkingReadout>>,
) {
    if !keys.just_pressed(KeyCode::KeyB) {
        return;
    }
    match *screen {
        Screen::Playing if walking.is_some_and(|w| w.active) => *screen = Screen::Breeding,
        Screen::Breeding => *screen = Screen::Playing,
        _ => {}
    }
}

/// A row, a tab or PAIR pressed.
pub fn press(
    rows: Query<(&Interaction, &MenuAction), Changed<Interaction>>,
    mut pick: ResMut<BreedingPick>,
    mut save: Option<ResMut<WorldSave>>,
) {
    for (interaction, action) in &rows {
        if *interaction != Interaction::Pressed {
            continue;
        }
        match *action {
            MenuAction::BreedSpecies(sp) => {
                if pick.species != sp {
                    *pick = BreedingPick {
                        species: sp,
                        ..default()
                    };
                }
            }
            MenuAction::BreedAnimal(id) => {
                let Some(body) = save
                    .as_ref()
                    .and_then(|s| s.records.get(record::ANIMAL, id))
                    .and_then(|r| r.read::<AnimalBody>())
                else {
                    continue;
                };
                match body.sex {
                    Sex::M => pick.sire = Some(id),
                    Sex::F => pick.dam = Some(id),
                }
                pick.pedigree = Some(id);
                pick.said.clear();
            }
            MenuAction::BreedPair => {
                let (Some(sire), Some(dam), Some(save)) =
                    (pick.sire, pick.dam, save.as_deref_mut())
                else {
                    pick.said = "Pick a sire and a dam".into();
                    continue;
                };
                pick.said = pbd_app::husbandry::breed::pair(save, dam, sire);
            }
            _ => {}
        }
    }
}

/// One animal as the screen reads it.
struct Kept {
    id: u64,
    body: AnimalBody,
    genome: Genome,
    generation: u32,
}

/// The generation of an animal: 0 for a founder, one past its elder parent.
fn generation(id: u64, all: &BTreeMap<u64, AnimalBody>, depth: u32) -> u32 {
    let Some(b) = all.get(&id) else { return 0 };
    if depth > 16 {
        return 0;
    }
    [b.sire, b.dam]
        .into_iter()
        .flatten()
        .map(|p| generation(p, all, depth + 1) + 1)
        .max()
        .unwrap_or(0)
}

/// The coat and pattern a genome shows, by the species' names.
fn looks(keep: &Keeping, g: &Genome) -> String {
    let coat = keep.coats.get(g.coat()).map_or("?", String::as_str);
    let pattern = keep.patterns.get(g.pattern()).map_or("?", String::as_str);
    if g.morph() {
        format!("{} {}, {}", keep.morph, coat, pattern)
    } else {
        format!("{coat}, {pattern}")
    }
}

/// The adult name for a sex: hen or cockerel, ewe or ram.
fn sex_name(keep: &Keeping, sex: Sex) -> &str {
    match sex {
        Sex::F => &keep.young.adult.0,
        Sex::M => &keep.young.adult.1,
    }
}

/// Fill the open screen.
#[allow(clippy::too_many_arguments)]
pub fn paint(
    mut commands: Commands,
    screen: Res<Screen>,
    mut pick: ResMut<BreedingPick>,
    config: Res<HusbandryConfig>,
    body: Res<Body>,
    kit: Res<UiKit>,
    save: Option<Res<WorldSave>>,
    container: Query<Entity, With<BreedingBody>>,
) {
    if *screen != Screen::Breeding {
        return;
    }
    let saved = save.as_ref().is_some_and(|s| s.is_changed());
    if !screen.is_changed() && !pick.is_changed() && !saved {
        return;
    }
    let Ok(container) = container.single() else {
        return;
    };
    let roster = config.0.roster(&body.0);
    let kept_species: Vec<(u16, &LandSpecies)> = roster
        .iter()
        .enumerate()
        .filter(|(_, s)| s.keep.is_some())
        .map(|(i, s)| (i as u16, s))
        .collect();
    if !kept_species.iter().any(|(i, _)| *i == pick.species)
        && let Some((first, _)) = kept_species.first()
    {
        // Open on a species the player keeps, if any.
        let all = save.as_ref().map(|s| record::animals(&s.records));
        let held = kept_species
            .iter()
            .find(|(i, _)| {
                all.as_ref()
                    .is_some_and(|a| a.iter().any(|(_, b)| b.sp == *i && b.body == body.0))
            })
            .map_or(*first, |(i, _)| *i);
        pick.bypass_change_detection().species = held;
    }
    let s = &config.0.breeding;
    let all: BTreeMap<u64, AnimalBody> = save
        .as_ref()
        .map(|s| record::animals(&s.records).into_iter().collect())
        .unwrap_or_default();
    let pedigree = Pedigree::from_animals(&all);
    let sp = pick.species;
    let Some(species) = roster.get(sp as usize) else {
        return;
    };
    let Some(keep) = species.keep.as_ref() else {
        return;
    };
    let herd: Vec<Kept> = all
        .iter()
        .filter(|(_, b)| b.sp == sp && b.body == body.0 && b.state == Life::Alive)
        .map(|(id, b)| Kept {
            id: *id,
            body: b.clone(),
            genome: b.genome(),
            generation: generation(*id, &all, 0),
        })
        .collect();
    let pick = pick.clone();
    commands.entity(container).despawn_children();
    commands.entity(container).with_children(|root| {
        // The species tabs.
        root.spawn(Node {
            column_gap: px(4),
            ..default()
        })
        .with_children(|tabs| {
            for (i, s) in &kept_species {
                tabs.spawn((plaque(&kit, Set::Stone, 60.0), MenuAction::BreedSpecies(*i)))
                    .with_children(|p| {
                        let name = s.name().to_uppercase();
                        p.spawn(text(name, 10.0, ui_kit::LABEL));
                    });
            }
        });
        root.spawn(Node {
            column_gap: px(10),
            ..default()
        })
        .with_children(|columns| {
            // Left: the herd.
            columns
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: px(2),
                    width: px(190),
                    flex_shrink: 0.0,
                    ..default()
                })
                .with_children(|list| {
                    list.spawn(text("THE HERD", 11.0, AMBER));
                    if herd.is_empty() {
                        list.spawn(text("No kept animals", 10.0, DIM));
                    }
                    for k in herd.iter().take(12) {
                        herd_row(list, &kit, keep, k, &pick);
                    }
                    if herd.len() > 12 {
                        list.spawn(text(format!("and {} more", herd.len() - 12), 9.0, DIM));
                    }
                });
            // Right: the pair.
            columns
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: px(3),
                    flex_grow: 1.0,
                    ..default()
                })
                .with_children(|right| {
                    pair_panel(right, &kit, keep, s, &all, &pedigree, &pick);
                });
        });
        // Below: the pedigree of the last animal pressed.
        if let Some(id) = pick.pedigree {
            pedigree_panel(root, keep, &all, &pedigree, id, s.generations);
        }
    });
}

/// One pressable row of the herd: sex, id, generation and coat, in the
/// sire's or the dam's colour when picked.
fn herd_row(
    list: &mut ChildSpawnerCommands,
    kit: &UiKit,
    keep: &Keeping,
    k: &Kept,
    pick: &BreedingPick,
) {
    let marked = if pick.sire == Some(k.id) {
        Some(SIRE)
    } else if pick.dam == Some(k.id) {
        Some(DAM)
    } else {
        None
    };
    list.spawn((
        plaque(kit, Set::Stone, 180.0),
        MenuAction::BreedAnimal(k.id),
    ))
    .insert(Node {
        align_items: AlignItems::Center,
        column_gap: px(5),
        height: px(20),
        padding: UiRect::axes(px(6), px(2)),
        flex_shrink: 0.0,
        ..default()
    })
    .with_children(|row| {
        row.spawn((
            Node {
                width: px(4),
                height: px(12),
                ..default()
            },
            BackgroundColor(marked.unwrap_or(Color::NONE)),
        ));
        let name = k
            .body
            .name
            .clone()
            .unwrap_or_else(|| format!("{} {}", sex_name(keep, k.body.sex), k.id));
        row.spawn(text(name, 10.0, ui_kit::LABEL));
        row.spawn(text(format!("gen {}", k.generation), 9.0, ui_kit::TAB_DIM));
        row.spawn(text(looks(keep, &k.genome), 9.0, DIM));
    });
}

/// The pair: who they are, F, each gene's four young on a bar, the odds of
/// coat, pattern and morph, and PAIR.
fn pair_panel(
    right: &mut ChildSpawnerCommands,
    kit: &UiKit,
    keep: &Keeping,
    s: &pbd_core::husbandry::BreedingSettings,
    all: &BTreeMap<u64, AnimalBody>,
    pedigree: &Pedigree,
    pick: &BreedingPick,
) {
    let name = |id: Option<u64>, sex: Sex| -> String {
        id.map_or_else(
            || format!("pick a {}", sex_name(keep, sex)),
            |id| format!("{} {id}", sex_name(keep, sex)),
        )
    };
    right
        .spawn(Node {
            column_gap: px(6),
            align_items: AlignItems::Center,
            ..default()
        })
        .with_children(|head| {
            head.spawn(text(name(pick.sire, Sex::M), 12.0, SIRE));
            head.spawn(text("x", 11.0, DIM));
            head.spawn(text(name(pick.dam, Sex::F), 12.0, DAM));
        });
    let (Some(sire), Some(dam)) = (
        pick.sire.and_then(|id| all.get(&id)),
        pick.dam.and_then(|id| all.get(&id)),
    ) else {
        return;
    };
    let (sg, dg) = (sire.genome(), dam.genome());
    let f = pedigree.f_of_young(pick.sire.unwrap_or(0), pick.dam.unwrap_or(0), s.generations);
    let close = f > s.warn_f + 1e-6;
    right.spawn(text(
        if close {
            format!(
                "F {f:.3}  close kin: hardiness -{:.0}%, litters -{:.0}%",
                s.hardiness_per_f * f * 100.0,
                f * 100.0
            )
        } else {
            format!("F {f:.3}")
        },
        10.0,
        if close { WARN } else { INK },
    ));
    for (label, gene) in [
        ("SIZE", genome::SIZE),
        ("YIELD", genome::YIELD),
        ("TEMPER", genome::TEMPERAMENT),
        ("HARDY", genome::HARDINESS),
    ] {
        gene_bar(right, label, &sg, &dg, gene);
    }
    // Coat, pattern and morph odds, in quarters.
    let odds = |gene: usize, names: &[String]| -> String {
        breeding::discrete_odds(&sg, &dg, gene)
            .into_iter()
            .map(|(i, q)| format!("{} {}%", names.get(i).map_or("?", String::as_str), q * 25))
            .collect::<Vec<_>>()
            .join("  ")
    };
    right.spawn(text(
        format!("COAT  {}", odds(genome::COAT, &keep.coats)),
        10.0,
        INK,
    ));
    right.spawn(text(
        format!("PATTERN  {}", odds(genome::PATTERN, &keep.patterns)),
        10.0,
        INK,
    ));
    right.spawn(text(
        format!(
            "{}  {}%",
            keep.morph.to_uppercase(),
            breeding::morph_quarters(&sg, &dg) * 25
        ),
        10.0,
        INK,
    ));
    right
        .spawn(Node {
            column_gap: px(8),
            align_items: AlignItems::Center,
            ..default()
        })
        .with_children(|row| {
            row.spawn((plaque(kit, Set::Stone, 60.0), MenuAction::BreedPair))
                .with_children(|p| {
                    p.spawn(text("PAIR", 10.0, ui_kit::LABEL));
                });
            let paired = dam.mate.is_some() && dam.mate == pick.sire;
            let said = if pick.said.is_empty() && paired {
                "Paired".to_string()
            } else {
                pick.said.clone()
            };
            row.spawn(text(said, 10.0, AMBER));
        });
}

/// One gene: a bar with the four equally likely young as ticks, the band
/// they span, the sire's mark above and the dam's below, and where the band
/// sits against the parents.
fn gene_bar(
    right: &mut ChildSpawnerCommands,
    label: &str,
    sire: &Genome,
    dam: &Genome,
    gene: usize,
) {
    let ticks = breeding::offspring_values(sire, dam, gene);
    let lo = ticks.iter().copied().fold(f32::MAX, f32::min);
    let hi = ticks.iter().copied().fold(f32::MIN, f32::max);
    let (sv, dv) = (sire.value(gene), dam.value(gene));
    let arrow = if lo > sv.max(dv) {
        "+"
    } else if hi < sv.min(dv) {
        "-"
    } else {
        "="
    };
    right
        .spawn(Node {
            column_gap: px(6),
            align_items: AlignItems::Center,
            height: px(14),
            ..default()
        })
        .with_children(|row| {
            row.spawn((
                text(label, 9.0, DIM),
                Node {
                    width: px(44),
                    ..default()
                },
            ));
            row.spawn((
                Node {
                    width: px(BAR),
                    height: px(12),
                    ..default()
                },
                BackgroundColor(ui_kit::WELL),
            ))
            .with_children(|bar| {
                let at = |v: f32| px((v.clamp(0.0, 1.0) * BAR).round());
                bar.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: at(lo),
                        width: px(((hi - lo) * BAR).max(2.0)),
                        top: px(3),
                        height: px(6),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.98, 0.69, 0.36, 0.35)),
                ));
                for t in ticks {
                    bar.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            left: at(t),
                            width: px(2),
                            top: px(2),
                            height: px(8),
                            ..default()
                        },
                        BackgroundColor(AMBER),
                    ));
                }
                for (v, colour, top) in [(sv, SIRE, 0.0), (dv, DAM, 9.0)] {
                    bar.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            left: at(v),
                            width: px(3),
                            top: px(top),
                            height: px(3),
                            ..default()
                        },
                        BackgroundColor(colour),
                    ));
                }
            });
            row.spawn(text(
                format!("{:.0}-{:.0} {arrow}", lo * 100.0, hi * 100.0),
                9.0,
                INK,
            ));
        });
}

/// The pedigree of one animal: it, its parents, its grandparents.
fn pedigree_panel(
    root: &mut ChildSpawnerCommands,
    keep: &Keeping,
    all: &BTreeMap<u64, AnimalBody>,
    pedigree: &Pedigree,
    id: u64,
    generations: u32,
) {
    let card = |col: &mut ChildSpawnerCommands, who: Option<u64>| {
        let line = match who.and_then(|w| all.get(&w).map(|b| (w, b))) {
            Some((w, b)) => {
                let ended = if b.state == Life::Ended {
                    ", ended"
                } else {
                    ""
                };
                format!(
                    "{} {w}{ended}  {}",
                    sex_name(keep, b.sex),
                    looks(keep, &b.genome())
                )
            }
            None => "wild-caught founder".to_string(),
        };
        let colour = match who.and_then(|w| all.get(&w)).map(|b| b.sex) {
            Some(Sex::M) => SIRE,
            Some(Sex::F) => DAM,
            None => DIM,
        };
        col.spawn((
            Node {
                padding: UiRect::axes(px(5), px(2)),
                border: UiRect::left(px(2)),
                ..default()
            },
            BorderColor::all(colour),
            BackgroundColor(ui_kit::WELL),
        ))
        .with_children(|c| {
            c.spawn(text(line, 9.0, INK));
        });
    };
    let parents = |w: Option<u64>| -> (Option<u64>, Option<u64>) {
        w.and_then(|w| all.get(&w))
            .map_or((None, None), |b| (b.sire, b.dam))
    };
    root.spawn(text(
        format!(
            "PEDIGREE  {} {id}  F {:.3}",
            all.get(&id).map_or("", |b| sex_name(keep, b.sex)),
            pedigree.f(id, generations)
        ),
        10.0,
        AMBER,
    ));
    root.spawn(Node {
        column_gap: px(6),
        align_items: AlignItems::Center,
        ..default()
    })
    .with_children(|cols| {
        cols.spawn(Node {
            flex_direction: FlexDirection::Column,
            ..default()
        })
        .with_children(|c| card(c, Some(id)));
        let (s, d) = parents(Some(id));
        cols.spawn(Node {
            flex_direction: FlexDirection::Column,
            row_gap: px(10),
            ..default()
        })
        .with_children(|c| {
            card(c, s);
            card(c, d);
        });
        cols.spawn(Node {
            flex_direction: FlexDirection::Column,
            row_gap: px(2),
            ..default()
        })
        .with_children(|c| {
            for p in [s, d] {
                let (gs, gd) = parents(p);
                card(c, gs);
                card(c, gd);
            }
        });
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    /// B on foot opens the screen and B again closes it; in a craft B is the
    /// brake and opens nothing.
    #[test]
    fn b_opens_the_breeding_screen_on_foot_only() {
        let mut app = App::new();
        app.init_resource::<ButtonInput<KeyCode>>()
            .insert_resource(Screen::Playing)
            .insert_resource(WalkingReadout {
                active: true,
                ..default()
            })
            .add_systems(Update, open);
        let tap = |app: &mut App| {
            let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            keys.clear();
            keys.release(KeyCode::KeyB);
            keys.clear();
            keys.press(KeyCode::KeyB);
            app.update();
        };
        tap(&mut app);
        assert_eq!(*app.world().resource::<Screen>(), Screen::Breeding);
        tap(&mut app);
        assert_eq!(*app.world().resource::<Screen>(), Screen::Playing);
        app.world_mut().resource_mut::<WalkingReadout>().active = false;
        tap(&mut app);
        assert_eq!(*app.world().resource::<Screen>(), Screen::Playing);
    }

    /// A founder is generation 0 and a young is one past its elder parent.
    #[test]
    fn generations_count_from_the_founders() {
        let mut all = BTreeMap::new();
        all.insert(1, AnimalBody::default());
        all.insert(2, AnimalBody::default());
        all.insert(
            3,
            AnimalBody {
                sire: Some(1),
                dam: Some(2),
                ..AnimalBody::default()
            },
        );
        all.insert(
            4,
            AnimalBody {
                sire: Some(1),
                dam: Some(3),
                ..AnimalBody::default()
            },
        );
        assert_eq!(generation(1, &all, 0), 0);
        assert_eq!(generation(3, &all, 0), 1);
        assert_eq!(generation(4, &all, 0), 2);
    }
}
