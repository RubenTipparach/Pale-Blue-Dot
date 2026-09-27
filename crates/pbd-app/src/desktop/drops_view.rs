//! What a floating drop looks like (`inventory-grid` task 3.3).
//!
//! A block's prism wears the tile its slot thumbnail is cropped from, at the
//! same tint, so a drop and the slot it lands in agree about what dirt looks
//! like. A light's card wears the light's own icon. The drops themselves, and
//! how they are drawn, are `pbd_app::drops`; this only names each item's
//! picture, because the pictures live beside the slot row.

use super::slots::{ATLAS_SHEETS, ATLAS_TILES, ItemIcons, thumbnail};
use bevy::math::Affine2;
use bevy::prelude::*;
use pbd_app::drops::DropLooks;
use pbd_app::saves::format::material_of;
use pbd_core::inventory::Item;
use pbd_core::terrain::Material;

/// The part of the atlas a tile covers, as a UV transform: the tile's square,
/// half a texel in from each edge as the slot's crop is.
pub fn tile_transform(slot: u32, tile: Vec2) -> Affine2 {
    let sheets = ATLAS_SHEETS as u32;
    let step = 1.0 / (ATLAS_SHEETS * ATLAS_TILES);
    let origin = Vec2::new(
        (slot % sheets) as f32 / ATLAS_SHEETS + tile.x * step,
        (slot / sheets) as f32 / ATLAS_SHEETS + tile.y * step,
    );
    let inset = step / 64.0;
    Affine2::from_scale_angle_translation(Vec2::splat(step - 2.0 * inset), 0.0, origin + inset)
}

/// Every block's and every light's look, once the icons are loaded.
pub fn dress(
    mut commands: Commands,
    icons: Res<ItemIcons>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut looks = DropLooks::default();
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
        } else if let Some((slot, tile, tint)) = thumbnail(material) {
            looks.prisms.insert(
                item,
                materials.add(StandardMaterial {
                    base_color: tint,
                    base_color_texture: Some(icons.atlas.clone()),
                    uv_transform: tile_transform(slot, tile),
                    perceptual_roughness: 0.95,
                    ..default()
                }),
            );
        }
    }
    commands.insert_resource(looks);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every tile's transform lands inside the atlas and inside its own
    /// sheet, and two tiles never share a square.
    #[test]
    fn a_tile_maps_into_its_own_square() {
        let a = tile_transform(5, Vec2::new(3.0, 0.0));
        let low = a.transform_point2(Vec2::ZERO);
        let high = a.transform_point2(Vec2::ONE);
        assert!(low.min_element() >= 0.0 && high.max_element() <= 1.0);
        // Sheet 5 is the second row's second sheet: x in 1/4..1/2, y in 1/4..1/2.
        assert!(low.x > 0.25 + 3.0 / 16.0 && high.x < 0.5);
        assert!(low.y > 0.25 && high.y < 0.25 + 1.0 / 16.0);
        let b = tile_transform(5, Vec2::new(2.0, 0.0));
        assert!(b.transform_point2(Vec2::ONE).x < low.x, "no overlap");
    }
}
