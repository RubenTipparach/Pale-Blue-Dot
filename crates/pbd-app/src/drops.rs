//! Floating drops in the running game: the dug block hovering where it was
//! cut, pulled in when the walker comes near (`inventory-grid` decision 4).
//!
//! The rules are `pbd_core::drops`. This is the frame that runs them, and the
//! durable lines they write: a dig's line carries the drop it made
//! (`desktop/digging.rs`), and a pickup is a line of its own, written before
//! the slots change, so the disk and the hand never disagree about where a
//! block is.
//!
//! They are drawn as Tenebris draws them: a block as a small hex prism of its
//! own ground texture, bobbing and turning, and a light as its own icon on a
//! card that faces the camera. Each is lit by the field, so a drop in a cave
//! is as dark as the cave and one by a lantern is lit by it. Which texture an
//! item wears is the desktop's to say (`desktop/drops_view.rs`, beside the
//! slot thumbnails it shares), through [`DropLooks`].

use crate::hotbar::Hotbar;
use crate::planet::PlanetRenderFrame;
use crate::saves::WorldSave;
use crate::sky::Sun;
use crate::walking::{HALF_HEIGHT, Walker, WalkingReadout};
use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use pbd_core::drops::{self, ItemDrop, Moved};
use pbd_core::inventory::Item;
use std::collections::HashMap;

/// Runs the drops each frame: [`gather`], then [`draw`], in [`DropsSet`].
pub struct DropsPlugin;

/// Where the drops run, for the desktop to order after the dig that makes
/// them.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct DropsSet;

impl Plugin for DropsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Drops>()
            .add_systems(Update, (gather, draw).chain().in_set(DropsSet));
    }
}

/// Every drop floating in the world, in the order it was made.
#[derive(Resource, Default, Debug)]
pub struct Drops {
    pub live: Vec<ItemDrop>,
}

impl Drops {
    /// The drops a save's log says are still floating.
    pub fn restore(save: &WorldSave) -> Self {
        Self {
            live: save.drops.clone(),
        }
    }

    /// The drop a dug block makes: one of `item`, at `centre` pushed aside by
    /// the scatter, named by the save's next id and made at world time
    /// `now_s`. It is not added here: the dig's line has to be accepted first.
    pub fn make(save: &WorldSave, item: Item, centre: Vec3, now_s: f64) -> ItemDrop {
        let id = save.next_drop_id();
        ItemDrop {
            id,
            item,
            count: 1,
            position: drops::scattered(centre, id),
            made_s: now_s,
        }
    }
}

/// The walker's chest, planet-local: where a drop is pulled to.
pub fn chest(walker: Vec3) -> Vec3 {
    let up = walker.normalize_or(Vec3::Y);
    walker - up * HALF_HEIGHT + up * drops::CHEST_M
}

/// One frame of every drop. Those past their time go, and write nothing: a
/// load would not restore them either. On foot, the rest are pulled toward
/// the walker and picked up by the give order. A pickup is written before the
/// slots change; one the save refuses leaves the drop where it is.
pub fn gather(
    time: Res<Time>,
    sun: Res<Sun>,
    walking: Option<Res<WalkingReadout>>,
    walkers: Query<&Transform, With<Walker>>,
    mut drops: ResMut<Drops>,
    mut hotbar: ResMut<Hotbar>,
    mut save: ResMut<WorldSave>,
) {
    let now = sun.clock.seconds;
    drops.live.retain(|drop| !drop.expired(now));
    let on_foot = walking.is_some_and(|readout| readout.active);
    let Some(walker) = walkers.iter().next().filter(|_| on_foot) else {
        return;
    };
    let chest = chest(walker.translation);
    let dt = time.delta_secs();
    if let Some(nearest) = drops
        .live
        .iter()
        .map(|d| (d.position - chest).length())
        .reduce(f32::min)
    {
        trace!(
            "{} drops, nearest {nearest:.2} m from the chest",
            drops.live.len()
        );
    }
    let mut gone = Vec::new();
    for (index, drop) in drops.live.iter_mut().enumerate() {
        let mut moved = *drop;
        let mut slots = hotbar.0.clone();
        match drops::step(&mut moved, chest, dt, &mut slots) {
            Moved::Picked { taken } => {
                if !save.record_pick(moved.id, moved.count, &slots) {
                    error!("pickup BLOCKED: the save did not accept drop {}", moved.id);
                    continue;
                }
                debug!("picked up {taken} of {:?} (drop {})", moved.item, moved.id);
                hotbar.0 = slots;
                *drop = moved;
                if drop.count == 0 {
                    gone.push(index);
                }
            }
            Moved::Pulled => *drop = moved,
            Moved::Stayed => {}
        }
    }
    for index in gone.into_iter().rev() {
        drops.live.remove(index);
    }
}

/// What each item's drop looks like: a block's prism wears its ground tile,
/// and a light's card its icon. An item with no look is not drawn, which is
/// a missing picture rather than a wrong one.
#[derive(Resource, Default)]
pub struct DropLooks {
    pub prisms: HashMap<Item, Handle<StandardMaterial>>,
    pub cards: HashMap<Item, Handle<StandardMaterial>>,
}

/// The drawn drop with this id.
#[derive(Component)]
pub struct DropView(pub u64);

/// A hex prism, [`drops::PRISM_RADIUS_M`] to its corners and
/// [`drops::PRISM_HALF_HEIGHT_M`] each way from its middle, standing on its
/// local Y. Each face is a whole tile: the caps fit the tile's square round
/// the hexagon, and each side is the tile stretched along its edge.
pub fn prism() -> Mesh {
    let (r, h) = (drops::PRISM_RADIUS_M, drops::PRISM_HALF_HEIGHT_M);
    let corner = |k: usize| {
        let a = k as f32 * std::f32::consts::TAU / 6.0;
        Vec2::new(a.cos(), a.sin())
    };
    let (mut positions, mut normals, mut uvs, mut indices) =
        (Vec::new(), Vec::new(), Vec::new(), Vec::<u32>::new());
    for (y, up) in [(h, 1.0f32), (-h, -1.0)] {
        let base = positions.len() as u32;
        positions.push([0.0, y, 0.0]);
        normals.push([0.0, up, 0.0]);
        uvs.push([0.5, 0.5]);
        for k in 0..6 {
            let c = corner(k);
            positions.push([c.x * r, y, c.y * r]);
            normals.push([0.0, up, 0.0]);
            uvs.push([0.5 + 0.5 * c.x, 0.5 + 0.5 * c.y]);
        }
        for k in 0..6u32 {
            let (a, b) = (base + 1 + k, base + 1 + (k + 1) % 6);
            // Counter-clockwise seen from outside the cap.
            if up > 0.0 {
                indices.extend([base, b, a]);
            } else {
                indices.extend([base, a, b]);
            }
        }
    }
    for k in 0..6 {
        let (a, b) = (corner(k), corner(k + 1));
        let normal = (a + b).normalize();
        let base = positions.len() as u32;
        for (c, y, uv) in [
            (a, h, [0.0, 0.0]),
            (b, h, [1.0, 0.0]),
            (b, -h, [1.0, 1.0]),
            (a, -h, [0.0, 1.0]),
        ] {
            positions.push([c.x * r, y, c.y * r]);
            normals.push([normal.x, 0.0, normal.y]);
            uvs.push(uv);
        }
        indices.extend([base, base + 1, base + 2, base, base + 2, base + 3]);
    }
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_indices(Indices::U32(indices))
}

/// Where a drop is drawn at time `t` s, in the render frame: its floating
/// point plus its bob along the local up. A prism turns about the up; a card
/// turns to face `eye`, upright.
fn pose(drop: &ItemDrop, centre: Vec3, t: f32, card: bool, eye: Option<Vec3>) -> Transform {
    let up = drop.position.normalize_or(Vec3::Y);
    let at = centre + drop.position + up * drop.bob(t);
    let standing = Quat::from_rotation_arc(Vec3::Y, up);
    let rotation = match (card, eye) {
        (true, Some(eye)) => {
            let toward = eye - at;
            let flat = toward - up * toward.dot(up);
            match flat.try_normalize() {
                Some(facing) => {
                    Transform::from_translation(at)
                        .looking_to(-facing, up)
                        .rotation
                }
                None => standing,
            }
        }
        _ => Quat::from_axis_angle(up, drop.spin(t)) * standing,
    };
    Transform::from_translation(at).with_rotation(rotation)
}

/// Keep one drawn drop per live drop: spawn the new, move the rest, and let
/// the picked-up and the expired go.
#[allow(clippy::too_many_arguments)]
fn draw(
    mut commands: Commands,
    drops: Res<Drops>,
    looks: Option<Res<DropLooks>>,
    frame: Res<PlanetRenderFrame>,
    time: Res<Time>,
    cameras: Query<(&GlobalTransform, &Camera), With<Camera3d>>,
    mut views: Query<(Entity, &DropView, &mut Transform)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut shapes: Local<Option<(Handle<Mesh>, Handle<Mesh>)>>,
) {
    let Some(looks) = looks else {
        return;
    };
    let (prism_mesh, card_mesh) = shapes
        .get_or_insert_with(|| {
            let side = 2.0 * drops::ICON_HALF_M;
            (meshes.add(prism()), meshes.add(Rectangle::new(side, side)))
        })
        .clone();
    let centre = frame.center.as_vec3();
    let eye = cameras
        .iter()
        .find(|(_, camera)| camera.is_active)
        .map(|(transform, _)| transform.translation());
    let t = time.elapsed_secs();
    let by_id: HashMap<u64, &ItemDrop> = drops.live.iter().map(|d| (d.id, d)).collect();
    let mut drawn = std::collections::HashSet::new();
    for (entity, view, mut transform) in &mut views {
        match by_id.get(&view.0) {
            Some(drop) => {
                let card = looks.cards.contains_key(&drop.item);
                *transform = pose(drop, centre, t, card, eye);
                drawn.insert(view.0);
            }
            None => commands.entity(entity).despawn(),
        }
    }
    for drop in &drops.live {
        if drawn.contains(&drop.id) {
            continue;
        }
        let (mesh, material, card) =
            match (looks.cards.get(&drop.item), looks.prisms.get(&drop.item)) {
                (Some(card), _) => (card_mesh.clone(), card.clone(), true),
                (None, Some(prism)) => (prism_mesh.clone(), prism.clone(), false),
                (None, None) => continue,
            };
        commands.spawn((
            Name::new(format!("drop {}", drop.id)),
            DropView(drop.id),
            crate::field_light::LitByField,
            Mesh3d(mesh),
            MeshMaterial3d(material),
            pose(drop, centre, t, card, eye),
            Visibility::default(),
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pbd_core::inventory::{CARRIED, Stack};
    use pbd_core::terrain::Material;

    const FEET: Vec3 = Vec3::new(0.0, 300.0, 0.0);

    fn app(drops: Vec<ItemDrop>, slots: Hotbar) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(Sun::default())
            .insert_resource(WalkingReadout {
                active: true,
                ..default()
            })
            .insert_resource(Drops { live: drops })
            .insert_resource(slots)
            .insert_resource(WorldSave::memory_only())
            .add_systems(Update, gather);
        app.world_mut().spawn((
            Walker,
            Transform::from_translation(FEET + Vec3::Y * HALF_HEIGHT),
        ));
        app
    }

    fn dirt(id: u64, at: Vec3, made_s: f64) -> ItemDrop {
        ItemDrop {
            id,
            item: Item::Block(Material::Dirt),
            count: 1,
            position: at,
            made_s,
        }
    }

    fn now(app: &App) -> f64 {
        app.world().resource::<Sun>().clock.seconds
    }

    /// A drop beside the walker is picked up into the slots, and the drop is
    /// gone.
    #[test]
    fn a_drop_by_the_walker_is_picked_up() {
        let mut app = app(vec![], Hotbar::default());
        let t = now(&app);
        app.world_mut().resource_mut::<Drops>().live = vec![dirt(0, FEET + Vec3::X * 0.5, t)];
        app.update();
        assert!(app.world().resource::<Drops>().live.is_empty());
        let slots = &app.world().resource::<Hotbar>().0;
        assert_eq!(
            slots.get(0),
            Some(Stack::new(Item::Block(Material::Dirt), 1))
        );
    }

    /// With everything full it floats where it is, and nothing is taken.
    #[test]
    fn a_full_pack_leaves_it_floating() {
        let mut full = Hotbar::default();
        for k in 0..CARRIED {
            full.0.set(k, Some(Stack::new(Item::Fish(k as u16), 16)));
        }
        let mut app = app(vec![], full);
        let t = now(&app);
        let at = FEET + Vec3::X * 0.5;
        app.world_mut().resource_mut::<Drops>().live = vec![dirt(3, at, t)];
        for _ in 0..3 {
            app.update();
        }
        let live = &app.world().resource::<Drops>().live;
        assert_eq!(live.len(), 1);
        assert_eq!(live[0].count, 1);
    }

    /// Far off, a drop stays put; past its time, it goes.
    #[test]
    fn a_far_drop_stays_and_an_old_one_goes() {
        let mut app = app(vec![], Hotbar::default());
        let t = now(&app);
        let far = FEET + Vec3::X * 10.0;
        app.world_mut().resource_mut::<Drops>().live =
            vec![dirt(1, far, t), dirt(2, far, t - drops::LIFE_S - 1.0)];
        app.update();
        let live = &app.world().resource::<Drops>().live;
        assert_eq!(live.len(), 1, "the old one went");
        assert_eq!(live[0].id, 1);
        assert_eq!(live[0].position, far);
    }

    /// The prism is closed and every face points out: each triangle's
    /// winding agrees with its normal, and its corners reach the radius.
    #[test]
    fn the_prism_is_closed_and_faces_out() {
        let mesh = prism();
        let Some(bevy::mesh::VertexAttributeValues::Float32x3(p)) =
            mesh.attribute(Mesh::ATTRIBUTE_POSITION)
        else {
            panic!("positions");
        };
        let Some(bevy::mesh::VertexAttributeValues::Float32x3(n)) =
            mesh.attribute(Mesh::ATTRIBUTE_NORMAL)
        else {
            panic!("normals");
        };
        let Some(Indices::U32(i)) = mesh.indices() else {
            panic!("indices");
        };
        assert_eq!(
            i.len(),
            3 * (12 + 12),
            "two caps of six and six sides of two"
        );
        for tri in i.chunks(3) {
            let [a, b, c] = [tri[0], tri[1], tri[2]].map(|k| Vec3::from(p[k as usize]));
            let face = (b - a).cross(c - a);
            assert!(
                face.dot(Vec3::from(n[tri[0] as usize])) > 0.0,
                "{tri:?} faces in"
            );
        }
        let reach = p
            .iter()
            .map(|v| Vec2::new(v[0], v[2]).length())
            .fold(0.0, f32::max);
        assert!((reach - drops::PRISM_RADIUS_M).abs() < 1e-5);
    }

    /// A drop is drawn where it floats, bobbing along its up, and a card
    /// turns to the eye.
    #[test]
    fn a_drop_is_drawn_where_it_floats() {
        let drop = dirt(0, FEET, 0.0);
        let still = pose(&drop, Vec3::ZERO, 0.0, false, None);
        let up = FEET.normalize();
        assert!(
            (still.translation - FEET).cross(up).length() < 1e-4,
            "along the up"
        );
        assert!((still.translation - FEET).length() <= drops::BOB_M + 1e-5);
        let eye = FEET + Vec3::X * 3.0;
        let card = pose(&drop, Vec3::ZERO, 0.0, true, Some(eye));
        let facing = card.rotation * Vec3::Z;
        assert!(
            facing.dot(Vec3::X) > 0.99,
            "the card's face is toward the eye"
        );
    }

    /// A new drop is named by the save's next id and scattered off the
    /// cell's centre.
    #[test]
    fn a_new_drop_takes_the_next_id() {
        let save = WorldSave::memory_only();
        let drop = Drops::make(&save, Item::Block(Material::Stone), FEET, 50.0);
        assert_eq!(drop.id, 0);
        assert_eq!(drop.count, 1);
        assert!(((drop.position - FEET).length() - drops::SCATTER_M).abs() < 1e-4);
    }
}
