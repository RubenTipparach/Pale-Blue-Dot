//! The player's buildings in the world (`player-building` design B3 to B5):
//! the `piece` records read back from the save, cut a plot at a time by
//! `pbd_core::building::cut`, drawn in the towns' own textures and materials,
//! lit by the field as a town is, and stood on and walked into through the
//! walker's [`PlayerStructures`].
//!
//! Every placement and removal is a record through the save's durable path
//! ([`Builds::place`], [`Builds::remove`]) before anything is drawn.

use crate::field_light::{
    BakedField, LitByField, LitLikeTerrain, RoomLights, UNBAKED, UNBAKED_BEFORE,
};
use crate::planet::{PLANET_RADIUS, PlanetRenderFrame};
use crate::saves::WorldSave;
use crate::towns::{TownAssets, TownFlame, TownRoot};
use crate::walking::PlayerStructures;
use bevy::prelude::*;
use pbd_core::building::cut::{FLAME, cut_on};
use pbd_core::building::{
    Cells, PIECE_RECORD, PIECE_SCHEMA, PLOT_M, Piece, from_records, placed, plots, removed,
};
use pbd_core::records::Author;
use pbd_core::settlement::Kits;
use pbd_core::settlement::chart::Patch;
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
    let mut solids = Vec::new();
    let mut drawn = Vec::new();
    let mut casting = Vec::new();
    for (anchor, ids) in plots(&pieces, PLANET_RADIUS, pbd_core::cell_key::key_direction) {
        let Some(direction) = pbd_core::cell_key::key_direction(anchor) else {
            continue;
        };
        let patch = patch_round(direction, PATCH_REACH_M);
        let cells = Cells::new(&patch);
        let of: Vec<&Piece> = ids.iter().filter_map(|id| pieces.get(id)).collect();
        let Some(c) = cut_on(
            &cells,
            anchor,
            &of,
            &kits,
            &repeat,
            PLANET_RADIUS,
            &ground_layer,
        ) else {
            continue;
        };
        drawn.push(spawn_plot(world, &c.meshes, &c.solids.lights));
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

/// The player's buildings: [`Builds`], kept current with the save, and
/// [`PlayerStructures`] for the walker.
pub struct BuildsPlugin;

impl Plugin for BuildsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Builds>()
            .init_resource::<PlayerStructures>()
            .add_systems(Update, sync.run_if(resource_exists::<WorldSave>));
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
