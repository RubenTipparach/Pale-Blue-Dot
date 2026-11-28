use super::*;
use pbd_core::townsfolk::face::Mouth;
use pbd_core::townsfolk::face_card::{self, CardLook, FaceLod};

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
    assert_eq!(ks, (0..28).collect());
    // The mouth stands on the head's front, a little proud of it.
    let head = parts[5].1.unwrap();
    let mouth = parts.iter().find(|(k, _)| *k == 26).unwrap().1.unwrap();
    let front = head.transform_point3(Vec3::new(0.0, figure::MOUTH_Y, figure::HA));
    let at = mouth.transform_point3(Vec3::ZERO);
    let proud = (at - front).length();
    assert!(proud > 0.001 && proud < 0.004, "{proud}");
    // The far card stands 1 cm proud of the head's front, over every piece.
    let card = parts.iter().find(|(k, _)| *k == CARD).unwrap().1.unwrap();
    let front = head.transform_point3(Vec3::new(0.0, 0.0, figure::HA));
    let proud = (card.transform_point3(Vec3::ZERO) - front).length();
    assert!((proud - face_card::CARD_PROUD_M).abs() < 1e-4, "{proud}");
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
            walk: None,
            door: None,
            chosen: Chosen {
                expression: Expression::Neutral,
                focus: false,
            },
            talking: false,
            held: None,
            behind: 0.0,
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
    // `furniture` task 4.3: walked in through the door to a seat, walking
    // on the way and seated at the end; let go, walked back out to the post
    // and released there.
    use pbd_core::furniture::walk::{Doorway, Route, WALK_MPS};
    let post = DVec3::new(0.0, 100.0, 8.0);
    let door = Doorway {
        middle: DVec3::new(0.0, 100.0, 0.0),
        out: DVec3::Z,
    };
    let seat = DVec3::new(1.0, 100.0, -2.0);
    folk.drive(id, post, DVec3::Z, false);
    let route = Route::in_through(post, door, seat);
    let time = route.length() / WALK_MPS;
    let sit = Posture::Sit { seat_m: 0.46 };
    assert!(folk.walk(id, route, Some((seat, DVec3::X, sit)), Some(door)));
    assert_eq!(folk.heading_to(id), Some(seat));
    folk.clock += time * 0.5;
    let clock = folk.clock;
    let l = folk.live_mut(id).unwrap();
    step_walk(l, clock);
    assert!(
        l.driven.unwrap().walking && l.posture.is_none(),
        "on the way"
    );
    folk.clock += time;
    let clock = folk.clock;
    step_walk(folk.live_mut(id).unwrap(), clock);
    assert_eq!(folk.resting(id), Some(sit), "seated at the end");
    assert_eq!(folk.live(id).unwrap().driven.unwrap().feet, seat);
    assert!(folk.leave(id, post));
    folk.clock += 60.0;
    let clock = folk.clock;
    step_walk(folk.live_mut(id).unwrap(), clock);
    let l = folk.live(id).unwrap();
    assert!(l.driven.is_none() && l.posture.is_none() && l.walk.is_none());
    assert!(!folk.leave(id, post), "a person not resting is left alone");
}

/// A person talked to holds where they stood, standing, for the length of
/// the talk; let go, they go on from that place, their round as far behind
/// the town's as the talk lasted (`playtest-fixes-2026-11-25` 1).
#[test]
fn a_talk_holds_a_person_and_they_go_on_from_where_they_stopped() {
    let mut folk = Townsfolk::default();
    let mut person = proxy_person();
    person.id = PersonId { site: 4, index: 0 };
    let id = person.id;
    let radius = 100.0;
    folk.clock = 10.0;
    let placed = person.at(folk.clock, radius);
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
            walk: None,
            door: None,
            chosen: Chosen {
                expression: Expression::Neutral,
                focus: false,
            },
            talking: false,
            held: None,
            behind: 0.0,
            person: person.clone(),
        }],
    });
    assert!(folk.set_talking(id, true));
    // Held every frame the talk is open: the first call holds.
    folk.clock = 25.0;
    assert!(folk.set_talking(id, true));
    let l = folk.live(id).unwrap();
    let held = place_of(l, 40.0, radius);
    assert_eq!(held.feet, placed.feet, "stands where the talk began");
    assert!(!held.walking, "and does not walk");
    folk.clock = 40.0;
    assert!(folk.set_talking(id, false));
    let l = folk.live(id).unwrap();
    assert!((l.behind - 30.0).abs() < 1e-9, "thirty seconds behind");
    assert_eq!(
        place_of(l, 40.0, radius).feet,
        placed.feet,
        "goes on from the same place, no jump"
    );
    assert_eq!(place_of(l, 45.0, radius).feet, person.at(15.0, radius).feet);
}

/// 15.2: past the line a person's face is the card and none of its pieces;
/// inside it the pieces and no card; during a fade both, each through its
/// half of the mask. The body is drawn whatever the face shows.
#[test]
fn far_off_the_card_shows_and_the_pieces_do_not() {
    let near = FaceLod::new(10.0).card();
    let far = FaceLod::new(face_card::FACE_CARD_M + 5.0).card();
    assert_eq!((near, far), (0.0, 1.0));
    for k in 6..=26 {
        assert!(part_drawn(k, true, near), "piece {k} near");
        assert!(!part_drawn(k, true, far), "piece {k} far");
        assert!(part_drawn(k, true, 0.5), "piece {k} fading");
        assert!(!part_drawn(k, false, near), "a piece its pose hides");
    }
    assert!(!part_drawn(CARD, true, near), "no card near");
    assert!(part_drawn(CARD, true, far), "the card far");
    assert!(part_drawn(CARD, true, 0.5), "the card fading");
    for k in 0..6 {
        assert!(part_drawn(k, true, near) && part_drawn(k, true, far));
    }
    // The two halves of the mask: every threshold is drawn by exactly one.
    for share in [0.0, 0.25, 0.5, 0.75, 1.0] {
        let (leave, come) = (
            CrossFaded::Leaving(share).packed(),
            CrossFaded::Coming(share).packed(),
        );
        for i in 0..16 {
            let mask = (i as f32 + 0.5) / 16.0;
            let pieces = !(leave > 0.0 && mask < leave) && !(leave < 0.0 && mask >= leave + 2.0);
            let card = !(come > 0.0 && mask < come) && !(come < 0.0 && mask >= come + 2.0);
            assert!(pieces != card, "share {share}, threshold {mask}");
        }
    }
    // A talking person far off shows Talking's tile; asleep, Sleepy's.
    assert_eq!(
        card_expression(Expression::Happy, true, false),
        Expression::Talking
    );
    assert_eq!(
        card_expression(Expression::Neutral, false, true),
        Expression::Sleepy
    );
    assert_eq!(
        card_expression(Expression::Cross, false, false),
        Expression::Cross
    );
}

/// 15.1 and 15.2: a card is one flat mesh on the face's area, each texel
/// its colour's swatch, and an expression's card differs from Neutral's.
#[test]
fn a_face_card_is_its_tile_in_swatches() {
    let person = proxy_person();
    let figure = Figure::of(&person.look);
    let colours = look::swatches([]);
    let (_, swatch) = atlas_pixels(&colours);
    let kit = Kit {
        swatch: swatch.clone(),
        solid: Handle::default(),
        masked: Handle::default(),
        card: Handle::default(),
        white: Handle::default(),
        pupil: Handle::default(),
        glint: Handle::default(),
        lash: Handle::default(),
        arc: Handle::default(),
        mouths: HashMap::new(),
    };
    let who = CardLook::of(&person.look, &figure);
    let mut neutral = None;
    for e in Expression::ALL {
        let tile = face_card::bake_tile(&face_card::resting_pose(e), &who, face_card::TILE_PX);
        let mesh = card_mesh(&kit, &tile, face_card::TILE_PX);
        let Some(bevy::mesh::VertexAttributeValues::Float32x3(at)) =
            mesh.attribute(Mesh::ATTRIBUTE_POSITION)
        else {
            panic!("positions");
        };
        let Some(bevy::mesh::VertexAttributeValues::Float32x2(uvs)) =
            mesh.attribute(Mesh::ATTRIBUTE_UV_0)
        else {
            panic!("uvs");
        };
        assert!(!at.is_empty() && at.len() % 6 == 0, "{e:?}");
        let w = face_card::CARD_W / 2.0 + 1e-4;
        let [y0, y1] = face_card::CARD_Y;
        for p in at {
            assert!(p[0].abs() <= w && p[1] >= y0 - 1e-4 && p[1] <= y1 + 1e-4);
            assert_eq!(p[2], 0.0, "{e:?}: one flat piece");
        }
        let drawn: std::collections::BTreeSet<u32> = tile.iter().flatten().copied().collect();
        let used: Vec<[u32; 2]> = uvs.iter().map(|uv| uv.map(f32::to_bits)).collect();
        for c in &drawn {
            let uv = swatch[c].map(f32::to_bits);
            assert!(used.contains(&uv), "{e:?}: {c:06x} drawn");
        }
        let positions: Vec<[u32; 3]> = at.iter().map(|p| p.map(f32::to_bits)).collect();
        match &neutral {
            None => neutral = Some(positions),
            Some(n) => assert!(
                *n != positions || e == Expression::Neutral,
                "{e:?} draws Neutral's card"
            ),
        }
    }
}
