//! The layout rules a building is held to, one function each (`player-building`
//! design B2): the same functions check a player's next piece
//! ([`can_place`]) and a whole layout ([`check_layout`]), which is how a
//! town's buildings, turned into pieces by [`super::blueprint`], are
//! checked. The first rule that fails is the reason, named by [`Rule::id`]
//! as the locale file's `build.refused.*`.

use super::{Class, Opening, Piece, STOREY, Shape, Site};

/// A layout rule.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Rule {
    /// A cell of it is off the plot: not among the cells asked about.
    Outside,
    /// One piece of a class per slot.
    Slot,
    /// No building on a pentagon or a pentagon's neighbour.
    Pentagon,
    /// Something under it holds it up.
    HeldUp,
    /// Headroom at every walkable point: nothing floors a stair's well.
    Headroom,
    /// No wall across a stair's flight.
    StairCut,
    /// A stair's way off its head is open.
    Landing,
    /// A roof is over a room: walls close round its cells, and they are
    /// floored.
    Room,
    /// Roofs never overlap, eaves included.
    RoofOverlap,
    /// A lantern or a piece of furniture does not stand on a stair.
    Clear,
    /// A piece of furniture stays clear of its cell's walls (`furniture`
    /// design 4).
    InWall,
    /// Two pieces of furniture in a cell do not cross.
    Overlap,
    /// No furniture in the clear space inside a door.
    Doorway,
}

impl Rule {
    /// Its id in the locale file's `build.refused.*`.
    pub fn id(self) -> &'static str {
        match self {
            Rule::Outside => "outside",
            Rule::Slot => "slot",
            Rule::Pentagon => "pentagon",
            Rule::HeldUp => "held_up",
            Rule::Headroom => "headroom",
            Rule::StairCut => "stair_cut",
            Rule::Landing => "landing",
            Rule::Room => "room",
            Rule::RoofOverlap => "roof_overlap",
            Rule::Clear => "clear",
            Rule::InWall => "in_wall",
            Rule::Overlap => "overlap",
            Rule::Doorway => "doorway",
        }
    }
}

/// Every rule, in the order they are asked: the first to fail is the
/// reason.
pub const RULES: [Rule; 13] = [
    Rule::Outside,
    Rule::Slot,
    Rule::InWall,
    Rule::Overlap,
    Rule::Pentagon,
    Rule::HeldUp,
    Rule::Headroom,
    Rule::StairCut,
    Rule::Landing,
    Rule::Room,
    Rule::RoofOverlap,
    Rule::Clear,
    Rule::Doorway,
];

/// Whether `rule` holds for `piece` among `others` (the pieces already
/// built, never including `piece` itself).
pub fn holds<'a>(
    rule: Rule,
    site: &Site,
    others: impl Iterator<Item = &'a Piece> + Clone,
    piece: &Piece,
) -> bool {
    let shape = &piece.shape;
    match rule {
        Rule::Outside => shape.cells().iter().all(|&c| site.cells.has(c)) && joined(site, shape),
        Rule::Slot => slot_free(others, shape),
        Rule::Pentagon => shape.cells().iter().all(|&c| {
            !site.cells.pentagon(c)
                && site
                    .cells
                    .neighbours(c)
                    .iter()
                    .flatten()
                    .all(|&n| !site.cells.pentagon(n))
        }),
        Rule::HeldUp => held_up(site, others, shape),
        Rule::Headroom => headroom(others, shape),
        Rule::StairCut => !others
            .clone()
            .any(|o| cuts(&o.shape, shape) || cuts(shape, &o.shape)),
        Rule::Landing => landing(site, others, shape),
        Rule::Room => room(site, others, shape),
        Rule::RoofOverlap => roof_clear(site, others, shape),
        Rule::Clear => match shape {
            Shape::Lantern { cell, layer } | Shape::Furniture { cell, layer, .. } => {
                !others.clone().any(|o| {
                    matches!(&o.shape, Shape::Stair { foot, head, layer: l }
                        if (foot == cell || head == cell) && *layer >= *l && *layer < l + STOREY)
                })
            }
            _ => true,
        },
        Rule::InWall => match shape {
            Shape::Furniture { side, kind, .. } => {
                crate::furniture::fits(&crate::furniture::footprint(*kind, *side))
            }
            _ => true,
        },
        Rule::Overlap => match shape {
            Shape::Furniture {
                cell,
                layer,
                side,
                kind,
            } => {
                let mine = crate::furniture::footprint(*kind, *side);
                !others.clone().any(|o| {
                    matches!(&o.shape, Shape::Furniture { cell: c, layer: l, side: s, kind: k }
                        if c == cell && l == layer && s != side
                            && crate::furniture::overlaps(&mine, &crate::furniture::footprint(*k, *s)))
                })
            }
            _ => true,
        },
        Rule::Doorway => match shape {
            Shape::Furniture {
                cell,
                layer,
                side,
                kind,
            } => !crate::furniture::in_doorway(
                *kind,
                *side,
                &door_sides(site, others, *cell, *layer),
            ),
            Shape::Wall {
                a,
                b,
                layer,
                opening: Opening::Door,
            } => !others.clone().any(|o| match &o.shape {
                Shape::Furniture {
                    cell,
                    layer: l,
                    side,
                    kind,
                } if (cell == a || cell == b) && *l >= *layer && *l < layer + STOREY => {
                    let other = if cell == a { *b } else { *a };
                    site.cells
                        .side(*cell, other)
                        .is_some_and(|d| crate::furniture::in_doorway(*kind, *side, &[d as u8]))
                }
                _ => false,
            }),
            _ => true,
        },
    }
}

/// The sides of `cell` with a door in their wall on the storey of `layer`.
fn door_sides<'a>(
    site: &Site,
    others: impl Iterator<Item = &'a Piece>,
    cell: u32,
    layer: u16,
) -> Vec<u8> {
    others
        .filter_map(|o| match o.shape {
            Shape::Wall {
                a,
                b,
                layer: l,
                opening: Opening::Door,
            } if (a == cell || b == cell) && layer >= l && layer < l + STOREY => {
                let other = if a == cell { b } else { a };
                site.cells.side(cell, other).map(|d| d as u8)
            }
            _ => None,
        })
        .collect()
}

/// The first rule `piece` breaks among `others`, if any.
pub fn first_broken<'a>(
    site: &Site,
    others: impl Iterator<Item = &'a Piece> + Clone,
    piece: &Piece,
) -> Option<Rule> {
    RULES
        .into_iter()
        .find(|&rule| !holds(rule, site, others.clone(), piece))
}

/// Whether `candidate` may be placed among `pieces`: the first rule it
/// breaks, or nothing.
pub fn can_place(site: &Site, pieces: &[Piece], candidate: &Piece) -> Result<(), Rule> {
    match first_broken(site, pieces.iter(), candidate) {
        Some(rule) => Err(rule),
        None => Ok(()),
    }
}

/// Whether a whole layout holds: every piece checked against all the others.
/// The first piece that breaks a rule, by its place in `pieces`, and the
/// rule.
pub fn check_layout(site: &Site, pieces: &[Piece]) -> Result<(), (usize, Rule)> {
    for (i, piece) in pieces.iter().enumerate() {
        let others = pieces
            .iter()
            .enumerate()
            .filter(move |(j, _)| *j != i)
            .map(|(_, p)| p);
        if let Some(rule) = first_broken(site, others, piece) {
            return Err((i, rule));
        }
    }
    Ok(())
}

/// What stands on or hangs from `piece`, so it may not be taken away: the
/// first such piece among `pieces`.
pub fn dependant<'a>(pieces: &'a [Piece], piece: &Piece) -> Option<&'a Piece> {
    pieces
        .iter()
        .find(|o| o.id() != piece.id() && rests_on(&o.shape, &piece.shape))
}

/// Whether `upper` rests on `lower`.
fn rests_on(upper: &Shape, lower: &Shape) -> bool {
    match (upper, lower) {
        (Shape::Lantern { cell, layer }, Shape::Floor { cell: c, layer: l })
        | (Shape::Furniture { cell, layer, .. }, Shape::Floor { cell: c, layer: l }) => {
            cell == c && layer == l
        }
        (Shape::Stair { foot, layer, .. }, Shape::Floor { cell: c, layer: l }) => {
            foot == c && layer == l
        }
        (Shape::Roof { cells, layer, .. }, Shape::Wall { a, b, layer: l, .. }) => {
            *layer == l + STOREY && (cells.contains(a) || cells.contains(b))
        }
        (Shape::Roof { cells, layer, .. }, Shape::Floor { cell, layer: l }) => {
            *layer == l + STOREY && cells.contains(cell)
        }
        (
            Shape::Wall { a, b, layer, .. },
            Shape::Wall {
                a: a2,
                b: b2,
                layer: l,
                ..
            },
        ) => a == a2 && b == b2 && *layer == l + STOREY,
        _ => false,
    }
}

/// A wall's two cells and a stair's two share an edge; a post's three meet.
fn joined(site: &Site, shape: &Shape) -> bool {
    match shape {
        Shape::Wall { a, b, .. }
        | Shape::Stair {
            foot: a, head: b, ..
        } => a != b && site.cells.adjacent(*a, *b),
        Shape::Post { cells, .. } => site.cells.corner(*cells).is_some(),
        Shape::Roof { cells, .. } => !cells.is_empty(),
        _ => true,
    }
}

fn slot_free<'a>(mut others: impl Iterator<Item = &'a Piece>, shape: &Shape) -> bool {
    !others.any(|o| {
        if o.shape.slot() == shape.slot() {
            return true;
        }
        match (&o.shape, shape) {
            // Two flights that share a cell on the same storey.
            (
                Shape::Stair {
                    foot: f1,
                    head: h1,
                    layer: l1,
                },
                Shape::Stair {
                    foot: f2,
                    head: h2,
                    layer: l2,
                },
            ) => l1 == l2 && [f1, h1].iter().any(|c| *c == f2 || *c == h2),
            // A roof over a cell another roof already covers at its eaves.
            (
                Shape::Roof {
                    cells: c1,
                    layer: l1,
                    ..
                },
                Shape::Roof {
                    cells: c2,
                    layer: l2,
                    ..
                },
            ) => l1 == l2 && c1.iter().any(|c| c2.contains(c)),
            _ => false,
        }
    })
}

/// Floors in `cell` at `layer` among `others`.
fn floor_at<'a>(mut others: impl Iterator<Item = &'a Piece>, cell: u32, layer: u16) -> bool {
    others.any(|o| matches!(o.shape, Shape::Floor { cell: c, layer: l } if c == cell && l == layer))
}

fn held_up<'a>(
    site: &Site,
    others: impl Iterator<Item = &'a Piece> + Clone,
    shape: &Shape,
) -> bool {
    // A wall or post a storey down on the cell's edges or corners.
    let walled_below = |cell: u32, layer: u16| {
        layer >= STOREY
            && others.clone().any(|o| match &o.shape {
                Shape::Wall { a, b, layer: l, .. } => {
                    (*a == cell || *b == cell) && l + STOREY == layer
                }
                Shape::Post { cells, layer: l } => cells.contains(&cell) && l + STOREY == layer,
                _ => false,
            })
    };
    let floor_beside = |cell: u32, layer: u16| {
        site.cells
            .neighbours(cell)
            .iter()
            .flatten()
            .any(|&n| floor_at(others.clone(), n, layer))
    };
    let stands = |cell: u32, layer: u16| {
        site.on_ground(cell, layer) || floor_at(others.clone(), cell, layer)
    };
    match *shape {
        Shape::Floor { cell, layer } => {
            site.on_ground(cell, layer) || walled_below(cell, layer) || floor_beside(cell, layer)
        }
        Shape::Wall { a, b, layer, .. } => {
            stands(a, layer)
                || stands(b, layer)
                || others.clone().any(|o| {
                    matches!(o.shape, Shape::Wall { a: a2, b: b2, layer: l, .. }
                        if a2 == a && b2 == b && l + STOREY == layer)
                })
        }
        Shape::Post { cells, layer } => {
            cells.iter().any(|&c| stands(c, layer))
                || others.clone().any(|o| {
                    matches!(o.shape, Shape::Post { cells: c, layer: l } if c == cells && l + STOREY == layer)
                })
        }
        Shape::Stair { foot, layer, .. }
        | Shape::Lantern { cell: foot, layer }
        | Shape::Furniture {
            cell: foot, layer, ..
        } => stands(foot, layer),
        // A roof is held by its room's walls: the room rule.
        Shape::Roof { .. } => true,
    }
}

/// The layers a stair's well takes over its cells: from a layer over its
/// foot to the walker's head at its top.
fn well(layer: u16) -> std::ops::RangeInclusive<u16> {
    layer + 1..=layer + STOREY + 2
}

fn headroom<'a>(others: impl Iterator<Item = &'a Piece> + Clone, shape: &Shape) -> bool {
    match *shape {
        Shape::Floor { cell, layer } => !others.clone().any(|o| match o.shape {
            Shape::Stair {
                foot,
                head,
                layer: l,
            } => (foot == cell || head == cell) && well(l).contains(&layer),
            // A floor less than a storey over or under another.
            Shape::Floor { cell: c, layer: l } => {
                c == cell && l != layer && l.abs_diff(layer) < STOREY
            }
            _ => false,
        }),
        Shape::Stair { foot, head, layer } => !others.clone().any(|o| {
            matches!(o.shape, Shape::Floor { cell, layer: l }
                if (cell == foot || cell == head) && well(layer).contains(&l))
        }),
        // A floor over a piece of furniture closer than its height.
        Shape::Furniture {
            cell, layer, kind, ..
        } => !others.clone().any(|o| {
            matches!(o.shape, Shape::Floor { cell: c, layer: l }
                if c == cell && l > layer && f32::from(l - layer) < kind.height_m())
        }),
        _ => true,
    }
}

/// Whether `wall` stands across `stair`'s flight: on the edge between its
/// two cells, on its storey.
fn cuts(wall: &Shape, stair: &Shape) -> bool {
    match (wall, stair) {
        (
            Shape::Wall { a, b, layer, .. },
            Shape::Stair {
                foot,
                head,
                layer: l,
            },
        ) => ((a == foot && b == head) || (a == head && b == foot)) && layer == l,
        _ => false,
    }
}

fn landing<'a>(
    site: &Site,
    others: impl Iterator<Item = &'a Piece> + Clone,
    shape: &Shape,
) -> bool {
    // The wall on a stair's way off at its top.
    let blocks = |wall: &Shape, stair: &Shape| match (wall, stair) {
        (
            Shape::Wall {
                a,
                b,
                layer,
                opening,
            },
            Shape::Stair {
                foot,
                head,
                layer: l,
            },
        ) => {
            let Some(on) = site.cells.straight_on(*foot, *head) else {
                return false;
            };
            *layer == l + STOREY && *opening != Opening::Door && {
                let (x, y) = (on.min(*head), on.max(*head));
                *a == x && *b == y
            }
        }
        _ => false,
    };
    if let Shape::Stair { foot, head, .. } = shape
        && site.cells.straight_on(*foot, *head).is_none()
    {
        return false;
    }
    !others
        .clone()
        .any(|o| blocks(&o.shape, shape) || blocks(shape, &o.shape))
}

fn room<'a>(site: &Site, others: impl Iterator<Item = &'a Piece> + Clone, shape: &Shape) -> bool {
    let Shape::Roof { cells, layer, .. } = shape else {
        return true;
    };
    if *layer < STOREY {
        return false;
    }
    let floor = layer - STOREY;
    cells.iter().all(|&c| {
        // Floored, or the top of a stair's well: the flight comes up there.
        (site.on_ground(c, floor)
            || floor_at(others.clone(), c, floor)
            || others.clone().any(|o| {
                matches!(o.shape, Shape::Stair { foot, head, layer: l }
                    if (foot == c || head == c) && l + STOREY == floor)
            }))
            && site.cells.neighbours(c).iter().all(|n| match n {
                Some(n) if cells.contains(n) => true,
                Some(n) => others.clone().any(|o| {
                    matches!(o.shape, Shape::Wall { a, b, layer: l, .. }
                        if l == floor && ((a == c && b == *n) || (a == *n && b == c)))
                }),
                None => false,
            })
    })
}

fn roof_clear<'a>(
    site: &Site,
    others: impl Iterator<Item = &'a Piece> + Clone,
    shape: &Shape,
) -> bool {
    let Shape::Roof { cells, layer, .. } = shape else {
        return true;
    };
    // Its eaves reach over the cells next to its own.
    let reach: Vec<u32> = cells
        .iter()
        .flat_map(|&c| std::iter::once(c).chain(site.cells.neighbours(c).into_iter().flatten()))
        .collect();
    !others.clone().any(|o| match &o.shape {
        Shape::Roof {
            cells: theirs,
            layer: l,
            ..
        } => l.abs_diff(*layer) < 2 * STOREY && theirs.iter().any(|c| reach.contains(c)),
        _ => false,
    })
}

/// The pieces of `class` among `pieces`.
pub fn of_class(pieces: &[Piece], class: Class) -> impl Iterator<Item = &Piece> {
    pieces.iter().filter(move |p| p.shape.class() == class)
}

/// `pieces` in an order each can be placed in, among `built` and the ones
/// before it: pass after pass, each placing what it can, so a floor held up
/// only by its neighbour goes after that neighbour (a blueprint on uneven
/// ground). Refused with the rule the first piece left breaks, once a pass
/// places nothing.
pub fn placing_order(site: &Site, built: &[Piece], pieces: &[Piece]) -> Result<Vec<Piece>, Rule> {
    let mut so_far = built.to_vec();
    let mut left: Vec<Piece> = pieces.to_vec();
    let mut out = Vec::new();
    while !left.is_empty() {
        let before = left.len();
        let mut still = Vec::new();
        for piece in left {
            if can_place(site, &so_far, &piece).is_ok() {
                so_far.push(piece.clone());
                out.push(piece);
            } else {
                still.push(piece);
            }
        }
        if still.len() == before {
            return Err(can_place(site, &so_far, &still[0]).expect_err("refused"));
        }
        left = still;
    }
    Ok(out)
}
