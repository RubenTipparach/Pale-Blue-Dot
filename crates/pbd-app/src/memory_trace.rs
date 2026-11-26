//! locale: dev
//!
//! MEASUREMENT (`memory-leak`): `PBD_MEMORY_TRACE=<path.csv>` writes one row
//! every two seconds of how much the game is holding: entities in the main and
//! render worlds, and the count and size of each kind of asset (meshes,
//! images, materials). It is how a memory that keeps growing is traced to the
//! one collection that grows with it. Absent, the plugin adds nothing.
//! Changes nothing drawn or simulated.

use std::{
    fs::File,
    io::Write as _,
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

use bevy::{
    asset::RenderAssetUsages,
    mesh::{Indices, Mesh},
    prelude::*,
    render::{Render, RenderApp, RenderSystems},
};

use crate::field_light::FieldLitMaterial;

/// Seconds between rows.
const EVERY_S: f32 = 2.0;

pub struct MemoryTracePlugin;

impl Plugin for MemoryTracePlugin {
    fn build(&self, app: &mut App) {
        let Some(path) = std::env::var_os("PBD_MEMORY_TRACE").map(PathBuf::from) else {
            return;
        };
        let mut file = File::create(&path).expect("PBD_MEMORY_TRACE: cannot create the file");
        writeln!(
            file,
            "since_start_s,entities,render_entities,meshes,mesh_mb,mesh_main_mb,images,image_mb,\
             standard_materials,field_lit_materials,color_materials"
        )
        .expect("PBD_MEMORY_TRACE: cannot write");
        info!("MEMORY_TRACE writing {}", path.display());
        let render_entities = RenderEntities::default();
        app.insert_resource(MemoryTrace { file, next_s: 0.0 })
            .insert_resource(render_entities.clone())
            .add_systems(Last, trace)
            .add_systems(Last, ui_roots);
        if let Some(render) = app.get_sub_app_mut(RenderApp) {
            render
                .insert_resource(render_entities)
                .add_systems(Render, count_render.in_set(RenderSystems::Cleanup));
        }
    }
}

#[derive(Resource)]
struct MemoryTrace {
    file: File,
    next_s: f32,
}

/// The render world's entity count, handed back to the main world.
#[derive(Resource, Clone, Default)]
struct RenderEntities(Arc<AtomicUsize>);

fn count_render(world: &World, out: Res<RenderEntities>) {
    out.0
        .store(world.entities().len() as usize, Ordering::Relaxed);
}

/// Bytes of a mesh's vertex attributes and indices.
fn mesh_bytes(mesh: &Mesh) -> usize {
    // A mesh kept only in the render world has handed its data over: none
    // is held here.
    let attributes: usize = mesh
        .try_attributes()
        .map(|a| a.map(|(_, values)| values.get_bytes().len()).sum())
        .unwrap_or(0);
    let indices = match mesh.try_indices_option() {
        Ok(Some(Indices::U16(i))) => i.len() * 2,
        Ok(Some(Indices::U32(i))) => i.len() * 4,
        _ => 0,
    };
    attributes + indices
}

#[allow(clippy::too_many_arguments)]
fn trace(
    mut out: ResMut<MemoryTrace>,
    time: Res<Time<Real>>,
    entities: &bevy::ecs::entity::Entities,
    render_entities: Res<RenderEntities>,
    meshes: Res<Assets<Mesh>>,
    images: Res<Assets<Image>>,
    standard: Res<Assets<StandardMaterial>>,
    field_lit: Option<Res<Assets<FieldLitMaterial>>>,
    color: Option<Res<Assets<ColorMaterial>>>,
) {
    let now = time.elapsed_secs();
    if now < out.next_s {
        return;
    }
    out.next_s = now + EVERY_S;
    let (mut mesh_all, mut mesh_main) = (0usize, 0usize);
    for (_, mesh) in meshes.iter() {
        let bytes = mesh_bytes(mesh);
        mesh_all += bytes;
        if mesh.asset_usage.contains(RenderAssetUsages::MAIN_WORLD) {
            mesh_main += bytes;
        }
    }
    let image_bytes: usize = images
        .iter()
        .map(|(_, image)| image.data.as_ref().map_or(0, Vec::len))
        .sum();
    let mb = |b: usize| b as f64 / (1024.0 * 1024.0);
    let _ = writeln!(
        out.file,
        "{:.1},{},{},{},{:.2},{:.2},{},{:.2},{},{},{}",
        now,
        entities.len(),
        render_entities.0.load(Ordering::Relaxed),
        meshes.len(),
        mb(mesh_all),
        mb(mesh_main),
        images.len(),
        mb(image_bytes),
        standard.len(),
        field_lit.map_or(0, |a| a.len()),
        color.map_or(0, |a| a.len()),
    );
    let _ = out.file.flush();
}

/// MEASUREMENT (`memory-leak`, towns at 60 fps): the UI's nodes, by root,
/// once every ten seconds: the trees laid out, stacked and hit-tested each
/// frame, shown or not. A root is named by its components.
fn ui_roots(world: &mut World, mut next: Local<f32>) {
    let now = world.resource::<Time<Real>>().elapsed_secs();
    if now < *next {
        return;
    }
    *next = now + 10.0;
    let roots: Vec<Entity> = world
        .query_filtered::<Entity, (With<Node>, Without<ChildOf>)>()
        .iter(world)
        .collect();
    let mut rows: Vec<(usize, String)> = Vec::new();
    for root in roots {
        let mut n = 0usize;
        let mut stack = vec![root];
        while let Some(e) = stack.pop() {
            n += 1;
            if let Some(c) = world.get::<Children>(e) {
                stack.extend(c.iter());
            }
        }
        let shown = world
            .get::<Node>(root)
            .is_some_and(|node| node.display != Display::None);
        // Its first words, which say which screen it is.
        let mut first = String::new();
        let mut stack = vec![root];
        while let Some(e) = stack.pop() {
            if let Some(t) = world.get::<Text>(e)
                && !t.0.trim().is_empty()
            {
                first = t.0.chars().take(24).collect();
                break;
            }
            if let Some(c) = world.get::<Children>(e) {
                stack.extend(c.iter().rev());
            }
        }
        let names = [first];
        rows.push((
            n,
            format!(
                "[{}] {}",
                names.join(" "),
                if shown { "shown" } else { "none" }
            ),
        ));
    }
    rows.sort_by(|a, b| b.0.cmp(&a.0));
    let total: usize = rows.iter().map(|r| r.0).sum();
    let top: Vec<String> = rows
        .iter()
        .take(9)
        .map(|(n, w)| format!("{n} {w}"))
        .collect();
    info!(
        "UI_NODES {total} in {} roots; {}",
        rows.len(),
        top.join(" | ")
    );
}
