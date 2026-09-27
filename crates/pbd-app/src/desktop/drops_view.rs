//! What a floating drop looks like (`inventory-grid` tasks 3.3 and 4.4).
//!
//! A block's prism is drawn as the ground draws the block: its top tile on
//! top, its side tile round the six sides and its underside tile below, so a
//! grass drop shows the sward's edge over earth, like the block it came out
//! of (survey I4, `inventory-grid` decision 7). Its slot thumbnail is the
//! same side tile. A light's card wears the light's own icon. The drops
//! themselves, and how they are drawn, are `pbd_app::drops`; this only names
//! each item's picture, because the pictures live beside the slot row.

use super::slots::{ATLAS_SHEETS, ATLAS_TILES, ItemIcons, block_art};
use bevy::prelude::*;
use pbd_app::drops::{DropLooks, PrismFaces, prism};
use pbd_app::saves::format::material_of;
use pbd_core::inventory::Item;
use pbd_core::terrain::Material;

/// The part of the atlas a tile covers, as a UV rectangle: the tile's square,
/// a sixty-fourth of a tile in from each edge, as the slot's crop is, so a
/// face never picks up its neighbour's edge.
pub fn tile_rect(sheet: u32, tile: Vec2) -> Rect {
    let sheets = ATLAS_SHEETS as u32;
    let step = 1.0 / (ATLAS_SHEETS * ATLAS_TILES);
    let origin = Vec2::new(
        (sheet % sheets) as f32 / ATLAS_SHEETS + tile.x * step,
        (sheet / sheets) as f32 / ATLAS_SHEETS + tile.y * step,
    );
    let inset = step / 64.0;
    Rect::from_corners(origin + inset, origin + step - inset)
}

/// Every block's and every light's look, once the icons are loaded.
pub fn dress(
    mut commands: Commands,
    icons: Res<ItemIcons>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let mut looks = DropLooks::default();
    // Every block shares one material, the atlas in its own colours; which
    // tiles a prism shows is in its mesh.
    let atlas = materials.add(StandardMaterial {
        base_color_texture: Some(icons.atlas.clone()),
        perceptual_roughness: 0.95,
        ..default()
    });
    for material in (0..=u8::MAX).map_while(material_of) {
        let item = Item::Block(material);
        if let Some(index) = Material::LAMPS.iter().position(|&m| m == material) {
            let Some(icon) = icons.lights.get(index) else {
                continue;
            };
            looks.cards.insert(
                item,
                materials.add(StandardMaterial {
                    base_color_texture: Some(icon.clone()),
                    alpha_mode: AlphaMode::Mask(0.5),
                    double_sided: true,
                    cull_mode: None,
                    perceptual_roughness: 0.9,
                    ..default()
                }),
            );
        } else if let Some(art) = block_art(material) {
            let faces = PrismFaces {
                top: tile_rect(art.sheet, art.top),
                side: tile_rect(art.sheet, art.side),
                under: tile_rect(art.sheet, art.under),
            };
            looks
                .prisms
                .insert(item, (meshes.add(prism(faces)), atlas.clone()));
        }
    }
    commands.insert_resource(looks);
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::mesh::VertexAttributeValues;

    /// Every tile's rectangle lands inside the atlas and inside its own
    /// sheet, and two tiles never share a square.
    #[test]
    fn a_tile_maps_into_its_own_square() {
        let a = tile_rect(5, Vec2::new(3.0, 0.0));
        assert!(a.min.min_element() >= 0.0 && a.max.max_element() <= 1.0);
        // Sheet 5 is the second row's second sheet: x in 1/4..1/2, y in 1/4..1/2.
        assert!(a.min.x > 0.25 + 3.0 / 16.0 && a.max.x < 0.5);
        assert!(a.min.y > 0.25 && a.max.y < 0.25 + 1.0 / 16.0);
        let b = tile_rect(5, Vec2::new(2.0, 0.0));
        assert!(b.max.x < a.min.x, "no overlap");
    }

    /// A grass drop's faces come from three different tiles, as the ground
    /// draws the block: the sward on top, the sward's edge over earth round
    /// the sides, earth underneath. Every vertex's UV sits inside the tile
    /// of the face it belongs to.
    #[test]
    fn a_grass_drop_wears_its_top_side_and_underside() {
        let art = block_art(Material::Grass).expect("grass has art");
        let faces = PrismFaces {
            top: tile_rect(art.sheet, art.top),
            side: tile_rect(art.sheet, art.side),
            under: tile_rect(art.sheet, art.under),
        };
        assert!(faces.top != faces.side && faces.side != faces.under);
        let mesh = prism(faces);
        let Some(VertexAttributeValues::Float32x3(normals)) =
            mesh.attribute(Mesh::ATTRIBUTE_NORMAL)
        else {
            panic!("normals")
        };
        let Some(VertexAttributeValues::Float32x2(uvs)) = mesh.attribute(Mesh::ATTRIBUTE_UV_0)
        else {
            panic!("uvs")
        };
        let mut seen = [0; 3];
        for (n, uv) in normals.iter().zip(uvs) {
            let (which, rect) = if n[1] > 0.5 {
                (0, faces.top)
            } else if n[1] < -0.5 {
                (2, faces.under)
            } else {
                (1, faces.side)
            };
            seen[which] += 1;
            let uv = Vec2::from(*uv);
            assert!(
                uv.cmpge(rect.min - 1e-6).all() && uv.cmple(rect.max + 1e-6).all(),
                "a vertex with normal {n:?} reads {uv} outside its tile {rect:?}"
            );
        }
        assert!(
            seen.iter().all(|&n| n > 0),
            "every face is present: {seen:?}"
        );
    }
}
