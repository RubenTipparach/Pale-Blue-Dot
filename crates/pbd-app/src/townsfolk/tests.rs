use super::*;
use pbd_core::townsfolk::face::Mouth;

/// Every colour a person can wear has its own swatch inside the atlas, and
/// a swatch reads back its colour.
#[test]
fn every_swatch_is_its_colour() {
    let assets = crate::towns::TownAssets::load();
    let tunics = [
        &assets.village,
        &assets.walled,
        &assets.harbour,
        &assets.desert,
        &assets.tundra,
        &assets.jungle,
    ]
    .into_iter()
    .flat_map(|t| t.people.iter().map(|p| p.tunic));
    let colours = look::swatches(tunics);
    let (px, swatch) = atlas_pixels(&colours);
    assert_eq!(swatch.len(), colours.len());
    for c in colours {
        let [u, v] = swatch[&c];
        let (x, y) = ((u * ATLAS_W as f32) as usize, (v * ATLAS_H as f32) as usize);
        assert!((SWATCH_Y..ATLAS_H).contains(&y), "{c:06x} at row {y}");
        let at = (y * ATLAS_W + x) * 4;
        assert_eq!(
            [px[at], px[at + 1], px[at + 2], px[at + 3]],
            [(c >> 16) as u8, (c >> 8) as u8, c as u8, 255],
            "{c:06x}"
        );
    }
}

/// A mouth quad shows its own cell and no other: its UVs stay inside it.
#[test]
fn a_mouth_quad_shows_its_cell() {
    for m in Mouth::ALL {
        let mesh = cell_mesh(0.24, 0.12, m);
        let Some(bevy::mesh::VertexAttributeValues::Float32x2(uvs)) =
            mesh.attribute(Mesh::ATTRIBUTE_UV_0)
        else {
            panic!("uvs");
        };
        let c = m.cell();
        let (ox, oy) = ((c % 4) * face::CELL_W, (c / 4) * face::CELL_H);
        for [u, v] in uvs {
            let (x, y) = (u * ATLAS_W as f32, v * ATLAS_H as f32);
            assert!(
                x >= ox as f32 - 1e-3 && x <= (ox + face::CELL_W) as f32 + 1e-3,
                "{m:?}"
            );
            assert!(
                y >= oy as f32 - 1e-3 && y <= (oy + face::CELL_H) as f32 + 1e-3,
                "{m:?}"
            );
        }
    }
}

/// Every part of a figure is posed each frame, and the face's pieces sit on
/// the front of the head, proud of it.
#[test]
fn every_part_is_posed_on_the_figure() {
    let person = proxy_person();
    let f = Figure::of(&person.look);
    let mut face = Face::new(1, Expression::Happy);
    let fp = face.step(
        0.016,
        Chosen {
            expression: Expression::Happy,
            focus: false,
        },
        None,
        false,
    );
    let pose = Body::new(1).step(0.016, Doing::Standing, 0.0);
    let parts = pose_parts(&f, &pose, &fp, &person.look);
    let ks: std::collections::BTreeSet<usize> = parts.iter().map(|(k, _)| *k).collect();
    assert_eq!(ks, (0..27).collect());
    // The mouth stands on the head's front, a little proud of it.
    let head = parts[5].1.unwrap();
    let mouth = parts.iter().find(|(k, _)| *k == 26).unwrap().1.unwrap();
    let front = head.transform_point3(Vec3::new(0.0, figure::MOUTH_Y, figure::HA));
    let at = mouth.transform_point3(Vec3::ZERO);
    let proud = (at - front).length();
    assert!(proud > 0.001 && proud < 0.004, "{proud}");
    // Feet on the ground, head at the figure's height.
    let top = head.transform_point3(Vec3::new(0.0, figure::HH, 0.0)).y;
    assert!(
        (top - f.height_m()).abs() < 0.03,
        "{top} against {}",
        f.height_m()
    );
}

/// What other parts use: an expression held on a person, a person driven
/// and released, all by their id.
#[test]
fn others_can_set_a_face_and_drive_a_person() {
    let mut folk = Townsfolk::default();
    let mut person = proxy_person();
    person.id = PersonId { site: 4, index: 0 };
    let placed = person.at(0.0, 100.0);
    folk.towns.push(Folk {
        site: 4,
        live: vec![Live {
            mood: Mood::new(Expression::Neutral),
            face: Face::new(1, Expression::Neutral),
            body: Body::new(1),
            forward: placed.forward,
            placed,
            driven: None,
            posture: None,
            chosen: Chosen {
                expression: Expression::Neutral,
                focus: false,
            },
            talking: false,
            person,
        }],
    });
    let id = PersonId { site: 4, index: 0 };
    assert!(folk.set_expression(id, Expression::Delighted, 2.0));
    let l = folk.live_mut(id).unwrap();
    assert_eq!(
        l.mood.step(0.1, Seen::default()).expression,
        Expression::Delighted
    );
    assert_eq!(l.body.acting(), Some(Act::Hop), "delighted hops");
    let there = DVec3::new(0.0, 101.0, 3.0);
    assert!(folk.drive(id, there, DVec3::Z, true));
    assert_eq!(folk.live(id).unwrap().driven.unwrap().feet, there);
    assert!(folk.release(id));
    assert!(folk.live(id).unwrap().driven.is_none());
    assert!(!folk.set_expression(PersonId { site: 9, index: 0 }, Expression::Happy, 1.0));
    assert_eq!(folk.people(4).count(), 1);
}
