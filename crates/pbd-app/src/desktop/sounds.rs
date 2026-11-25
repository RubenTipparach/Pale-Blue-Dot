//! The desktop's own sound hooks (`audio-and-ambience` build decision 5):
//! digging, breaking and placing at the aimed block, and the screens'
//! pause and duck. They read what `digging.rs` and `menu.rs` keep and write
//! only `pbd_app::audio`'s [`Sfx`] and [`Environment`].

use super::digging::{Aim, Block, Mining, TREE_LAYER, material_of};
use super::menu::Screen;
use bevy::prelude::*;
use pbd_app::audio::{Environment, Listener, Sfx, Where, hooks::named};
use pbd_app::planet::PlanetFine;
use pbd_app::saves::WorldSave;
use pbd_core::audio::{Family, place_cue, surface_family};
use pbd_core::missions::Kind;
use pbd_core::terrain::Material;

pub fn register(app: &mut App) {
    // `digging` reads the tool strokes' impacts, which `HeldPlugin` registers
    // only when there is a walker; a flight with no walker (`--tour`,
    // `--route`) has none, and the reader would fail validation at start.
    app.add_message::<pbd_app::held::StrokeImpact>()
        .add_systems(Update, (digging, screens));
}

/// The material a `block:<name>` code names.
fn material_named(code: &str) -> Option<Material> {
    let name = code.strip_prefix("block:")?;
    Material::ALL
        .into_iter()
        .find(|m| pbd_core::inventory::material_name(*m) == name)
}

/// What the dig at hand is: the block, where it is and what it is made of.
#[derive(Default)]
struct Digging {
    block: Option<Block>,
    at: Option<Vec3>,
    family: Option<Family>,
}

/// A dig sounds on each impact of the tool's stroke but the last
/// (`held-swing` 2.2), a break when the block comes out (on the last
/// impact), a place when one is set (design section 4, Ground).
#[allow(clippy::too_many_arguments)]
fn digging(
    mining: Option<Res<Mining>>,
    aim: Option<Res<Aim>>,
    fine: Option<Res<PlanetFine>>,
    save: Option<Res<WorldSave>>,
    listener: Res<Listener>,
    mut did: MessageReader<pbd_app::missions::Did>,
    mut impacts: MessageReader<pbd_app::held::StrokeImpact>,
    mut sfx: ResMut<Sfx>,
    mut state: Local<Digging>,
) {
    let aimed = aim
        .as_ref()
        .and_then(|a| a.target)
        .map(|t| listener.eye + listener.forward * t.distance);
    if let Some(mining) = &mining {
        match mining.progress() {
            Some((block, _)) => {
                if state.block != Some(block) {
                    let family = if block.1 == TREE_LAYER {
                        Some(Family::Wood)
                    } else {
                        match (&fine, &save) {
                            (Some(fine), Some(save)) => {
                                material_of(fine, save, block).and_then(surface_family)
                            }
                            _ => None,
                        }
                    };
                    *state = Digging {
                        block: Some(block),
                        at: aimed.or(state.at),
                        family: family.filter(|f| f.diggable()),
                    };
                }
            }
            None => {
                state.block = None;
            }
        }
    }
    // The stroke's impacts land on the dig's beat; the last is the break,
    // which sounds as one.
    for impact in impacts.read() {
        if !impact.last
            && let (Some(_), Some(family), Some(at)) = (state.block, state.family, state.at)
        {
            sfx.play(named("dig", family), Where::At(at));
        }
    }
    let hand = aimed
        .or(state.at)
        .map_or(Where::At(listener.eye + listener.forward * 1.5), Where::At);
    for pbd_app::missions::Did(happened) in did.read() {
        match happened.kind {
            Kind::Gather => {
                let family = material_named(&happened.what)
                    .and_then(surface_family)
                    .or(state.family)
                    .filter(|f| f.diggable());
                if let Some(family) = family {
                    let at = state.at.map_or(hand, Where::At);
                    sfx.play(named("break", family), at);
                }
            }
            Kind::Fell => sfx.play("break_wood", hand),
            Kind::Place => {
                if let Some(cue) = material_named(&happened.what).and_then(place_cue) {
                    sfx.play(cue_static(&cue), hand);
                }
            }
            _ => {}
        }
    }
}

/// `place_cue`'s answer as one of the interned names.
fn cue_static(cue: &str) -> &'static str {
    match cue {
        "fire_ignite_small" => "fire_ignite_small",
        "place_lamp" => "place_lamp",
        other => Family::ALL
            .into_iter()
            .find(|f| other == format!("place_{}", f.name()))
            .map_or("place_dirt", |f| named("place", f)),
    }
}

/// The screens: a pause stops the world's sound, a screen over play ducks
/// it (design section 2), and opening or closing one clicks.
fn screens(
    screen: Option<Res<Screen>>,
    mut env: ResMut<Environment>,
    mut sfx: ResMut<Sfx>,
    mut was: Local<Option<Screen>>,
) {
    let Some(screen) = screen else {
        return;
    };
    let now = *screen;
    env.paused = matches!(now, Screen::Pause | Screen::Settings | Screen::Saves);
    env.ducked = !env.paused && now != Screen::Playing;
    if let Some(before) = *was
        && before != now
    {
        let cue = if now == Screen::Playing {
            "ui_back"
        } else {
            "ui_toggle"
        };
        sfx.play(cue, Where::Menu);
    }
    *was = Some(now);
}
