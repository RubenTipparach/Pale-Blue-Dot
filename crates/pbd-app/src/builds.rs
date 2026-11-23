//! The player's buildings in the world (`player-building` design B3 to B5):
//! the `piece` records read back from the save, cut a plot at a time by
//! `pbd_core::building::cut`, drawn in the towns' own textures and materials,
//! lit by the field as a town is, and stood on and walked into through the
//! walker's [`PlayerStructures`].
//!
//! Every placement and removal is a record through the save's durable path
//! ([`Builds::place`], [`Builds::remove`]) before anything is drawn.
//!
//! A door (design B8) is a piece that also takes a door from the pack
//! ([`Builds::place_door`]), is drawn as its own leaf ([`PlotDoor`]) swung by
//! [`swing_plot_doors`], and is opened and shut with F ([`use_plot_doors`])
//! through the towns' own `door` record, at `building::door_state_id`.

use crate::field_light::{
    BakedField, LitByField, LitLikeTerrain, RoomLights, UNBAKED, UNBAKED_BEFORE,
};
use crate::planet::{PLANET_RADIUS, PlanetRenderFrame};
use crate::saves::WorldSave;
use crate::towns::{TownAssets, TownFlame, TownRoot};
use crate::walking::PlayerStructures;
use bevy::prelude::*;
use pbd_core::building::cut::{Cut, FLAME, cut_on};
use pbd_core::building::door::{LEAF_TEXTURE, leaf_mesh};
use pbd_core::building::{
    Cells, PIECE_RECORD, PIECE_SCHEMA, PLOT_M, Piece, door_state_id, from_records, placed, plots,
    removed,
};
use pbd_core::goods::Good;
use pbd_core::inventory::{CARRIED, Item, Slots};
use pbd_core::records::{Author, Records};
use pbd_core::settlement::Kits;
use pbd_core::settlement::chart::Patch;
use pbd_core::settlement::record::{DOOR_RECORD, RECORD_SCHEMA, door_open, door_record};
use std::collections::BTreeMap;
use std::sync::Arc;

/// How far round a plot's anchor its cells are fetched, metres: past
/// [`PLOT_M`] by the widest piece (a roof's cells round its first).
pub const PATCH_REACH_M: f32 = PLOT_M + 16.0;

/// The player's pieces and what is drawn of them.
#[derive(Resource, Default)]
pub struct Builds {
    /// Every standing piece, by its record id.
    pub pieces: BTreeMap<u64, Piece>,
    /// Bumped by every change, for whatever is drawn from the pieces.
    pub revision: u64,
    /// The save opening the pieces were read from.
    serial: Option<u64>,
    /// What was cut last, by revision.
    drawn_revision: Option<u64>,
    drawn: Vec<Entity>,
}

impl Builds {
    /// Place pieces: their records stored as the player's, then the pieces
    /// kept. `false`, and nothing kept, once the save's writer has failed.
    pub fn place(&mut self, save: &mut WorldSave, pieces: Vec<Piece>) -> bool {
        if pieces.is_empty() {
            return true;
        }
        let records = pieces.iter().map(placed).collect();
        if save.store(&Author::Player(0), records).is_none() {
            return false;
        }
        save.note_record_kinds(&[(PIECE_RECORD, PIECE_SCHEMA)]);
        for piece in pieces {
            self.pieces.insert(piece.id(), piece);
        }
        self.revision += 1;
        true
    }

    /// Take a piece away: its removal stored, then the piece dropped.
    pub fn remove(&mut self, save: &mut WorldSave, id: u64) -> bool {
        let Some(piece) = self.pieces.get(&id).cloned() else {
            return false;
        };
        if save
            .store(&Author::Player(0), vec![removed(&piece)])
            .is_none()
        {
            return false;
        }
        self.pieces.remove(&id);
        self.revision += 1;
        true
    }

    /// Hang a door (design B8): one door taken from `carried`, and the piece
    /// and the slots after it stored as one line. Nothing changes when none
    /// is carried or the save's writer has failed.
    pub fn place_door(
        &mut self,
        save: &mut WorldSave,
        piece: Piece,
        carried: &mut Slots,
    ) -> Result<(), DoorRefused> {
        let door = Item::Good(Good::DOOR);
        let Some(index) =
            (0..CARRIED).find(|&i| carried.get(i).is_some_and(|stack| stack.item == door))
        else {
            return Err(DoorRefused::NoDoor);
        };
        let mut after = carried.clone();
        after.take(index, 1);
        if !save.store_with_hands(placed(&piece), PIECE_SCHEMA, &after) {
            return Err(DoorRefused::Save);
        }
        *carried = after;
        self.pieces.insert(piece.id(), piece);
        self.revision += 1;
        Ok(())
    }

    /// Take a door down (design B8): it is written shut, so a door hung
    /// there later starts shut, and given back to `carried`. Refused with no
    /// room to carry it.
    pub fn remove_door(
        &mut self,
        save: &mut WorldSave,
        id: u64,
        carried: &mut Slots,
    ) -> Result<(), DoorRefused> {
        let Some(piece) = self.pieces.get(&id).cloned() else {
            return Err(DoorRefused::Save);
        };
        let mut after = carried.clone();
        if after.give(Item::Good(Good::DOOR), 1) > 0 {
            return Err(DoorRefused::NoRoom);
        }
        if save
            .store(
                &Author::Player(0),
                vec![door_record(door_state_id(id), false)],
            )
            .is_none()
            || !save.store_with_hands(removed(&piece), PIECE_SCHEMA, &after)
        {
            return Err(DoorRefused::Save);
        }
        *carried = after;
        self.pieces.remove(&id);
        self.revision += 1;
        Ok(())
    }

    /// Open or shut the door piece `id`: its record written before anything
    /// moves. `false`, and nothing written, once the save's writer has
    /// failed.
    pub fn set_door(save: &mut WorldSave, id: u64, open: bool) -> bool {
        if save
            .store(
                &Author::Player(0),
                vec![door_record(door_state_id(id), open)],
            )
            .is_none()
        {
            return false;
        }
        save.note_record_kinds(&[(DOOR_RECORD, RECORD_SCHEMA)]);
        true
    }

    /// The pieces with a cell within `reach_m` of `direction`.
    pub fn near(&self, direction: Vec3, reach_m: f32) -> Vec<Piece> {
        let cos = (reach_m / PLANET_RADIUS).cos();
        let d = direction.normalize_or(Vec3::Y);
        self.pieces
            .values()
            .filter(|p| {
                p.shape
                    .cells()
                    .iter()
                    .any(|&c| pbd_core::cell_key::key_direction(c).is_some_and(|k| k.dot(d) > cos))
            })
            .cloned()
            .collect()
    }

    /// Read the pieces from the save when a different world is open.
    pub fn follow_save(&mut self, save: &WorldSave) {
        if self.serial != Some(save.serial()) {
            self.pieces = from_records(&save.records);
            self.serial = Some(save.serial());
            self.revision += 1;
        }
    }
}

/// Why a door was not hung or taken down.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DoorRefused {
    /// None in the pack.
    NoDoor,
    /// No room in the pack to take it back.
    NoRoom,
    /// The save's writer has failed.
    Save,
}

impl DoorRefused {
    /// What the line under the crosshair says.
    pub fn text(self) -> pbd_core::locale::Localized {
        use pbd_core::locale::tr;
        match self {
            DoorRefused::NoDoor => tr("build.no_door"),
            DoorRefused::NoRoom => tr("breed.no_room_carry"),
            DoorRefused::Save => tr("breed.save_refused"),
        }
    }
}

/// Every plot of `pieces` cut, each door's leaf open as `records` hold it
/// (no record is a shut door). `ground` is the first layer over the ground
/// in a cell, for the footings ([`ground_layer`] in the game).
pub fn cut_plots(
    pieces: &BTreeMap<u64, Piece>,
    kits: &Kits,
    repeat: &dyn Fn(&str) -> f32,
    records: &Records,
    ground: &dyn Fn(u32) -> Option<u16>,
) -> Vec<Cut> {
    let mut out = Vec::new();
    for (anchor, ids) in plots(pieces, PLANET_RADIUS, pbd_core::cell_key::key_direction) {
        let Some(direction) = pbd_core::cell_key::key_direction(anchor) else {
            continue;
        };
        let patch = patch_round(direction, PATCH_REACH_M);
        let cells = Cells::new(&patch);
        let of: Vec<&Piece> = ids.iter().filter_map(|id| pieces.get(id)).collect();
        let Some(mut cut) = cut_on(&cells, anchor, &of, kits, repeat, PLANET_RADIUS, ground) else {
            continue;
        };
        for (leaf, id) in cut.solids.doors.iter_mut().zip(&cut.door_ids) {
            leaf.open = door_open(records, door_state_id(*id));
        }
        out.push(cut);
    }
    out
}

/// The finest cells round a direction, for the rules and the cut.
pub fn patch_round(direction: Vec3, reach_m: f32) -> Patch {
    crate::towns::patch_round(direction, PLANET_RADIUS, reach_m)
}

/// The kits, from the towns' assets where they are loaded.
pub fn kits_in(world: &World) -> Arc<Kits> {
    world
        .get_resource::<TownAssets>()
        .map(|a| a.kits.clone())
        .unwrap_or_else(|| Arc::new(crate::towns::load_kits()))
}

/// The first id a plot's shadow casters take in the sun's caster list,
/// past every site's id (`player-building` design 9).
pub const PLOT_CASTER_ID: u32 = 0xF000_0000;

/// The first layer over the generated ground in a cell (towns' ground
/// included), which a floor's footing stands on (design 8).
pub fn ground_layer(cell: u32) -> Option<u16> {
    let d = pbd_core::cell_key::key_direction(cell)?;
    let surface = crate::planet::surface_height(d);
    pbd_core::column::layer_at(surface - 0.5).map(|l| (l + 1) as u16)
}

/// The root of a plot's meshes.
#[derive(Component)]
pub struct BuildRoot;

/// Read the save's pieces, and cut and draw them again whenever they change.
pub fn sync(world: &mut World) {
    let Some(save) = world.get_resource::<WorldSave>() else {
        return;
    };
    let serial = save.serial();
    if world.resource::<Builds>().serial != Some(serial) {
        let save = world.resource::<WorldSave>();
        let pieces = from_records(&save.records);
        let mut builds = world.resource_mut::<Builds>();
        builds.pieces = pieces;
        builds.serial = Some(serial);
        builds.revision += 1;
    }
    let (revision, drawn_revision) = {
        let b = world.resource::<Builds>();
        (b.revision, b.drawn_revision)
    };
    if drawn_revision == Some(revision) {
        return;
    }
    let old = std::mem::take(&mut world.resource_mut::<Builds>().drawn);
    for e in old {
        if let Ok(e) = world.get_entity_mut(e) {
            e.despawn();
        }
    }
    let kits = kits_in(world);
    let repeat = crate::towns::repeat_in(world);
    let pieces = world.resource::<Builds>().pieces.clone();
    let cuts = cut_plots(
        &pieces,
        &kits,
        &repeat,
        &world.resource::<WorldSave>().records,
        &ground_layer,
    );
    let mut solids = Vec::new();
    let mut drawn = Vec::new();
    let mut casting = Vec::new();
    for (plot, c) in cuts.into_iter().enumerate() {
        let root = spawn_plot(world, &c.meshes, &c.solids.lights);
        spawn_leaves(world, root, plot, &c);
        drawn.push(root);
        casting.push(std::sync::Arc::new(crate::towns::casting_of(&c.meshes)));
        solids.push(c.solids);
    }
    info!("BUILDS {} pieces in {} plots", pieces.len(), solids.len());
    world.insert_resource(PlayerStructures(solids));
    // The plots cast the sun's shadow as a town does (design 9), under ids
    // past every site's.
    let mut casters =
        world.get_resource_or_insert_with(crate::planet::shadow::TownCasters::default);
    casters.0.retain(|(id, _)| *id < PLOT_CASTER_ID);
    for (k, c) in casting.into_iter().enumerate() {
        casters.0.push((PLOT_CASTER_ID + k as u32, c));
    }
    let mut builds = world.resource_mut::<Builds>();
    builds.drawn = drawn;
    builds.drawn_revision = Some(revision);
}

/// A plot's meshes under one root, drawn as a town's outside is: field-lit
/// like the terrain, baked from the field vertex by vertex, its lanterns'
/// flames unlit, and its lanterns lighting its own faces.
pub fn spawn_plot(
    world: &mut World,
    meshes: &pbd_core::settlement::pieces::Meshes,
    lights: &[pbd_core::settlement::pieces::RoomLight],
) -> Entity {
    let centre = world
        .get_resource::<PlanetRenderFrame>()
        .map_or(Vec3::ZERO, |f| f.center.as_vec3());
    let parts: Vec<(Mesh, Option<Handle<Image>>)> = {
        let assets = world.resource::<AssetServer>();
        meshes
            .iter()
            .map(|(name, buf)| {
                let mesh = crate::towns::to_mesh(buf)
                    .with_inserted_attribute(
                        Mesh::ATTRIBUTE_UV_1,
                        vec![UNBAKED; buf.positions.len()],
                    )
                    .with_inserted_attribute(
                        Mesh::ATTRIBUTE_COLOR,
                        vec![UNBAKED_BEFORE; buf.positions.len()],
                    );
                let image = (name != FLAME).then(|| crate::towns::texture(assets, name));
                (mesh, image)
            })
            .collect()
    };
    let root = world
        .spawn((
            Name::new("Player building"),
            BuildRoot,
            TownRoot,
            LitByField,
            LitLikeTerrain,
            Transform::from_translation(centre),
            Visibility::default(),
        ))
        .id();
    for (mesh, image) in parts {
        let flame = image.is_none();
        let mesh = world.resource_mut::<Assets<Mesh>>().add(mesh);
        let material = crate::towns::town_material(world, image, false);
        let mut child = world.spawn((Mesh3d(mesh), MeshMaterial3d(material), Transform::default()));
        if flame {
            child.insert(TownFlame);
        } else {
            child.insert((BakedField::default(), RoomLights(lights.to_vec())));
        }
        let child = child.id();
        world.entity_mut(root).add_child(child);
    }
    root
}

/// A player's door leaf in the world (design B8): its plot and door in the
/// walker's [`PlayerStructures`], its piece's id, and how far it has swung
/// (0 shut, a quarter turn open).
#[derive(Component, Clone, Copy, Debug)]
pub struct PlotDoor {
    pub plot: usize,
    pub door: usize,
    pub piece: u64,
    pub angle: f32,
}

/// Each door leaf of a plot as its own mesh under the plot's root, so it is
/// lit by the field as the walls round it are.
fn spawn_leaves(world: &mut World, root: Entity, plot: usize, c: &Cut) {
    if c.solids.doors.is_empty() {
        return;
    }
    let image = crate::towns::texture(world.resource::<AssetServer>(), LEAF_TEXTURE);
    let material = crate::towns::town_material(world, Some(image), false);
    for (k, (leaf, &piece)) in c.solids.doors.iter().zip(&c.door_ids).enumerate() {
        let angle = if leaf.open {
            crate::towns::OPEN_ANGLE
        } else {
            0.0
        };
        let mesh = leaf_entity_mesh(leaf, c.solids.frame, angle);
        let mesh = world.resource_mut::<Assets<Mesh>>().add(mesh);
        let child = world
            .spawn((
                Name::new("Door"),
                Mesh3d(mesh),
                MeshMaterial3d(material.clone()),
                Transform::default(),
                PlotDoor {
                    plot,
                    door: k,
                    piece,
                    angle,
                },
            ))
            .id();
        world.entity_mut(root).add_child(child);
    }
}

fn leaf_entity_mesh(
    leaf: &pbd_core::settlement::pieces::DoorLeaf,
    frame: pbd_core::settlement::pieces::Frame,
    angle: f32,
) -> Mesh {
    let meshes = leaf_mesh(leaf, frame, angle);
    crate::towns::to_mesh(meshes.values().next().expect("a leaf is planks"))
}

/// The player's door in reach of the walker: its plot, its door and its
/// piece, and how far.
fn plot_door_in_reach(
    walking: &crate::walking::WalkingState,
    at: Vec3,
    structures: &PlayerStructures,
    doors: &Query<&PlotDoor>,
) -> Option<(f32, PlotDoor)> {
    let (eye, look) = crate::towns::eye_and_look(walking, at);
    doors
        .iter()
        .filter_map(|door| {
            let b = structures.0.get(door.plot)?;
            let d = b.doors.get(door.door)?;
            crate::towns::leaf_in_reach(eye, look, b, d).map(|dist| (dist, *door))
        })
        .min_by(|a, b| a.0.total_cmp(&b.0))
}

/// Offer the player's door in reach to the interact key, as a town's is.
pub fn offer_plot_doors(
    walking: Option<Res<crate::walking::WalkingState>>,
    walker: Query<&avian3d::prelude::Position, With<crate::walking::Walker>>,
    structures: Res<PlayerStructures>,
    doors: Query<&PlotDoor>,
    mut interact: ResMut<crate::interact::Interact>,
    space: crate::SimSpace,
) {
    let (Some(walking), Ok(position)) = (walking, walker.single()) else {
        return;
    };
    if !walking.active {
        return;
    }
    let at = space.origin().query_point(position.0);
    let Some((_, door)) = plot_door_in_reach(&walking, at, &structures, &doors) else {
        return;
    };
    let open = structures
        .0
        .get(door.plot)
        .and_then(|b| b.doors.get(door.door))
        .is_some_and(|d| d.open);
    use crate::interact::{Act, Offer};
    interact.offer(Offer::new(
        if open { Act::ShutDoor } else { Act::OpenDoor },
        pbd_core::locale::tr("towns.door"),
    ));
}

/// F opens or shuts the player's door in reach: its record written before
/// the leaf moves, and a save that refuses it leaves it as it was.
#[allow(clippy::too_many_arguments)]
pub fn use_plot_doors(
    keys: Option<Res<ButtonInput<KeyCode>>>,
    walking: Option<Res<crate::walking::WalkingState>>,
    walker: Query<&avian3d::prelude::Position, With<crate::walking::Walker>>,
    mut structures: ResMut<PlayerStructures>,
    save: Option<ResMut<WorldSave>>,
    doors: Query<&PlotDoor>,
    interact: Option<Res<crate::interact::Interact>>,
    space: crate::SimSpace,
) {
    let (Some(keys), Some(walking)) = (keys, walking) else {
        return;
    };
    if !keys.just_pressed(KeyCode::KeyF) || !walking.active {
        return;
    }
    if !(walking.captured || walking.scripted) {
        return;
    }
    if !crate::interact::may(interact.as_deref(), crate::interact::Reader::Doors) {
        return;
    }
    let Ok(position) = walker.single() else {
        return;
    };
    let at = space.origin().query_point(position.0);
    let Some((_, door)) = plot_door_in_reach(&walking, at, &structures, &doors) else {
        return;
    };
    let Some(leaf) = structures
        .0
        .get_mut(door.plot)
        .and_then(|b| b.doors.get_mut(door.door))
    else {
        return;
    };
    let open = !leaf.open;
    if let Some(mut save) = save
        && !Builds::set_door(&mut save, door.piece, open)
    {
        warn!("the save refused the door; it stays as it was");
        return;
    }
    leaf.open = open;
}

/// Swing each player's door toward its state, redrawing it as it goes.
pub fn swing_plot_doors(
    time: Res<Time>,
    structures: Res<PlayerStructures>,
    mut doors: Query<(&mut PlotDoor, &Mesh3d)>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let step = crate::towns::SWING_RAD_S * time.delta_secs().max(1.0 / 60.0);
    for (mut door, mesh) in &mut doors {
        let Some(b) = structures.0.get(door.plot) else {
            continue;
        };
        let Some(leaf) = b.doors.get(door.door) else {
            continue;
        };
        let target = if leaf.open {
            crate::towns::OPEN_ANGLE
        } else {
            0.0
        };
        if door.angle == target {
            continue;
        }
        door.angle = if door.angle < target {
            (door.angle + step).min(target)
        } else {
            (door.angle - step).max(target)
        };
        if let Some(m) = meshes.get_mut(&mesh.0) {
            *m = leaf_entity_mesh(leaf, b.frame, door.angle);
        }
    }
}

/// The player's buildings: [`Builds`], kept current with the save, and
/// [`PlayerStructures`] for the walker.
pub struct BuildsPlugin;

impl Plugin for BuildsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Builds>()
            .init_resource::<PlayerStructures>()
            .add_systems(Update, sync.run_if(resource_exists::<WorldSave>))
            .add_systems(
                Update,
                (use_plot_doors, swing_plot_doors).chain().after(sync),
            )
            .add_systems(
                PostUpdate,
                offer_plot_doors
                    .in_set(crate::interact::InteractSet::Offer)
                    .run_if(resource_exists::<crate::interact::Interact>),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pbd_core::building::rules::RULES;
    use pbd_core::building::{Class, Opening, Shape};
    use pbd_core::locale::Locale;

    /// `player-building` design 9: a plot's walls and floors cast into the
    /// sun's cascades, its lanterns' flames do not, and its ids are past
    /// every site's.
    #[test]
    fn a_plot_casts_the_suns_shadow_but_its_flames_do_not() {
        let mut meshes = pbd_core::settlement::pieces::Meshes::new();
        let tri = |x: f32| [[x, 0.0, 0.0], [x + 1.0, 0.0, 0.0], [x, 1.0, 0.0]];
        let mut boards = pbd_core::settlement::pieces::MeshBuf::default();
        boards.positions.extend(tri(0.0));
        boards.positions.extend(tri(40.0));
        meshes.insert("boards".into(), boards);
        let mut flame = pbd_core::settlement::pieces::MeshBuf::default();
        flame.positions.extend(tri(2.0));
        meshes.insert(FLAME.into(), flame);
        let casting = crate::towns::casting_of(&meshes);
        assert_eq!(casting.positions.len(), 6, "the boards' two triangles");
        assert_eq!(casting.chunks.len(), 2, "chunked by where they stand");
        assert!(
            casting
                .chunks
                .iter()
                .all(|c| c.cascades as usize == crate::planet::shadow::CASCADE_COUNT)
        );
        assert!(PLOT_CASTER_ID > 1 << 24, "past every site id");
    }

    /// `player-building` design B7: the footing's texture has its manifest
    /// row, is the town textures' 32 px, opaque, and every colour of it is
    /// one of the town kits' stones (fieldstone, stone, rubble), so it sits
    /// in their palette.
    #[test]
    fn the_footing_is_drawn_in_the_stones_palette() {
        use pbd_core::building::cut::FOOTING;
        let repeats = crate::towns::load_repeats();
        assert!(repeats.get(FOOTING).is_some_and(|r| *r > 0.0), "its row");
        let read = |name: &str| {
            let path = crate::paths::assets_dir().join(format!("textures/settlement/{name}.png"));
            image::open(&path)
                .unwrap_or_else(|e| panic!("{}: {e}", path.display()))
                .to_rgba8()
        };
        let stones: std::collections::BTreeSet<[u8; 4]> = ["fieldstone", "stone", "rubble"]
            .iter()
            .flat_map(|n| read(n).pixels().map(|p| p.0).collect::<Vec<_>>())
            .collect();
        let footing = read(FOOTING);
        assert_eq!(footing.dimensions(), (32, 32));
        for p in footing.pixels() {
            assert_eq!(p.0[3], 255, "opaque");
            assert!(stones.contains(&p.0), "{:?} is not a stone's colour", p.0);
        }
    }

    /// Every reason, piece and kit build mode names by id has its words.
    #[test]
    fn every_rule_piece_and_kit_has_its_words() {
        let locale = Locale::english();
        for rule in RULES {
            let key = format!("build.refused.{}", rule.id());
            assert!(locale.get(&key).is_some(), "{key}");
        }
        for class in [
            Class::Wall,
            Class::Floor,
            Class::Stair,
            Class::Post,
            Class::Lantern,
            Class::Roof,
            Class::Door,
        ] {
            let key = format!("build.piece.{}", class.id());
            assert!(locale.get(&key).is_some(), "{key}");
        }
        for kit in &crate::towns::load_kits().kits {
            let key = format!("build.kit.{}", kit.name);
            assert!(locale.get(&key).is_some(), "{key}");
        }
    }

    /// A placed piece is a record in the save, and a world opened again from
    /// those records builds it; a removal takes it out of both.
    #[test]
    fn a_piece_placed_is_kept_and_a_removal_is_too() {
        let mut save = WorldSave::memory_only();
        let mut builds = Builds::default();
        builds.follow_save(&save);
        let piece = Piece::new("timber", Shape::wall(5, 9, 150, Opening::Door));
        assert!(builds.place(&mut save, vec![piece.clone()]));
        assert_eq!(from_records(&save.records).get(&piece.id()), Some(&piece));
        assert!(builds.remove(&mut save, piece.id()));
        assert!(from_records(&save.records).is_empty());
        assert!(builds.pieces.is_empty());
    }

    /// A one-cell house on the planet's real cells: its floor, five walls
    /// and a doorway, as a world before doors (design B8) holds one.
    fn house_with_a_doorway() -> (Vec<Piece>, u32, u32) {
        let direction = Vec3::new(0.3, 0.5, 0.8).normalize();
        let patch = patch_round(direction, 30.0);
        let cells = Cells::new(&patch);
        let k = patch.keys[patch.nearest(direction).unwrap()];
        let layer = 160;
        let mut pieces = vec![Piece::new("timber", Shape::Floor { cell: k, layer })];
        let mut doorway = None;
        for (s, n) in cells.neighbours(k).into_iter().enumerate() {
            let n = n.expect("on the patch");
            let opening = if s == 0 {
                doorway = Some(n);
                Opening::Door
            } else {
                Opening::None
            };
            pieces.push(Piece::new("timber", Shape::wall(k, n, layer, opening)));
        }
        (pieces, k, doorway.unwrap())
    }

    /// The cut of `pieces` as the game cuts it, for `records`' doors.
    fn cut_of(pieces: &[Piece], records: &Records) -> Vec<Cut> {
        let map: BTreeMap<u64, Piece> = pieces.iter().map(|p| (p.id(), p.clone())).collect();
        cut_plots(&map, &crate::towns::load_kits(), &|_| 2.0, records, &|_| {
            None
        })
    }

    /// Design B8: a house saved before doors could be made comes back from
    /// the disk with its doorway open, no leaf in it, and nothing written.
    #[test]
    fn an_old_house_comes_back_with_its_doorways_open() {
        let root = std::env::temp_dir().join(format!("pbd-old-house-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let slot = crate::saves::create(&root, "Old", 41).unwrap();
        let (house, _, _) = house_with_a_doorway();
        {
            let mut save = WorldSave::open(root.clone(), slot.clone());
            let mut builds = Builds::default();
            builds.follow_save(&save);
            assert!(builds.place(&mut save, house.clone()));
            save.drain();
        }
        let reopened = WorldSave::open(root.clone(), crate::saves::list(&root)[0].clone());
        let back = from_records(&reopened.records);
        assert_eq!(back.len(), house.len(), "piece for piece");
        assert!(
            back.values()
                .all(|p| !matches!(p.shape, Shape::Door { .. }))
        );
        let cuts = cut_of(&back.into_values().collect::<Vec<_>>(), &reopened.records);
        assert_eq!(cuts.len(), 1);
        assert!(cuts[0].solids.doors.is_empty(), "an open doorway, no leaf");
        let _ = std::fs::remove_dir_all(&root);
    }

    /// Design B8: a door is hung from the pack, opened, kept through a
    /// reload open, and taken down back into the pack; with no door in the
    /// pack nothing is hung.
    #[test]
    fn a_door_hung_and_opened_comes_back_open() {
        let root = std::env::temp_dir().join(format!("pbd-door-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let slot = crate::saves::create(&root, "Door", 43).unwrap();
        let (house, k, out) = house_with_a_doorway();
        let door = Piece::new("timber", Shape::door(k, out, 160));
        let mut carried = Slots::new();
        {
            let mut save = WorldSave::open(root.clone(), slot.clone());
            let mut builds = Builds::default();
            builds.follow_save(&save);
            assert!(builds.place(&mut save, house.clone()));
            assert_eq!(
                builds.place_door(&mut save, door.clone(), &mut carried),
                Err(DoorRefused::NoDoor)
            );
            carried.give(Item::Good(Good::DOOR), 2);
            assert_eq!(
                builds.place_door(&mut save, door.clone(), &mut carried),
                Ok(())
            );
            let shut = cut_of(
                &builds.pieces.values().cloned().collect::<Vec<_>>(),
                &save.records,
            );
            assert_eq!(shut[0].door_ids, vec![door.id()]);
            assert!(!shut[0].solids.doors[0].open, "it hangs shut");
            assert!(Builds::set_door(&mut save, door.id(), true));
            save.drain();
        }
        let mut save = WorldSave::open(root.clone(), crate::saves::list(&root)[0].clone());
        let back = from_records(&save.records);
        assert_eq!(back.get(&door.id()), Some(&door), "the door is kept");
        let cuts = cut_of(&back.values().cloned().collect::<Vec<_>>(), &save.records);
        assert!(cuts[0].solids.doors[0].open, "and it is open");
        let pack = save.carried.clone().expect("the slots were kept with it");
        let doors = |slots: &Slots| {
            slots
                .iter()
                .flatten()
                .filter(|s| s.item == Item::Good(Good::DOOR))
                .map(|s| s.count)
                .sum::<u16>()
        };
        assert_eq!(doors(&pack), 1, "one door taken from two");
        let mut builds = Builds::default();
        builds.follow_save(&save);
        let mut carried = pack;
        assert_eq!(
            builds.remove_door(&mut save, door.id(), &mut carried),
            Ok(())
        );
        assert_eq!(doors(&carried), 2, "given back");
        assert!(
            !door_open(&save.records, door_state_id(door.id())),
            "written shut"
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    /// Task 3.1: a room built in a world with an old log comes back from
    /// the disk piece for piece, the old log's edit is untouched, and the
    /// world's identity names the kind.
    #[test]
    fn a_built_room_survives_a_reload_and_an_old_log_reads_as_it_did() {
        let root = std::env::temp_dir().join(format!("pbd-builds-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let slot = crate::saves::create(&root, "Built", 31).unwrap();
        std::fs::write(root.join(&slot.id).join("edits.v1.log"), "77 200 0\n").unwrap();
        let room: Vec<Piece> = (0..6)
            .map(|s| Piece::new("timber", Shape::wall(100, 200 + s, 150, Opening::None)))
            .chain(std::iter::once(Piece::new(
                "timber",
                Shape::Floor {
                    cell: 100,
                    layer: 150,
                },
            )))
            .collect();
        {
            let mut save = WorldSave::open(root.clone(), slot.clone());
            let mut builds = Builds::default();
            builds.follow_save(&save);
            assert!(builds.place(&mut save, room.clone()));
            save.drain();
        }
        let reopened = WorldSave::open(root.clone(), crate::saves::list(&root)[0].clone());
        let back = from_records(&reopened.records);
        assert_eq!(back.len(), room.len());
        for piece in &room {
            assert_eq!(back.get(&piece.id()), Some(piece));
        }
        assert_eq!(
            reopened.edits.for_cell(77),
            &[(200, pbd_core::terrain::Material::Air)]
        );
        assert_eq!(
            reopened.identity.records.get(PIECE_RECORD),
            Some(&PIECE_SCHEMA)
        );
        let _ = std::fs::remove_dir_all(&root);
    }
}
