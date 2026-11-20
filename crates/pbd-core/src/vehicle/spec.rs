//! Every number a craft is made of, in `assets/config/vehicles.ron`.
//!
//! `VehicleSpecs::default()` is the one source of the defaults and they are
//! the mockup's (`docs/mockups/vehicles.html`), which is what was approved. The
//! shipped file writes the same set out and a test holds the two together.
//! Coordinates are the craft's reference frame: metres, +x right, +y up, +z
//! aft (forward is -z), the waterline or the ground near y 0.

use super::foil::FoilSpec;
use super::hull::{BilgeSpec, HullSpec, ResistanceSpec};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct VehicleSpecs {
    pub loon: LoonSpec,
    /// The inner tube (`small-boats`): paddled by hand.
    pub tube: LoonSpec,
    /// The raft (`small-boats`): logs, paddled.
    pub raft: LoonSpec,
}

impl Default for VehicleSpecs {
    fn default() -> Self {
        Self {
            loon: Default::default(),
            tube: LoonSpec::tube(),
            raft: LoonSpec::raft(),
        }
    }
}

impl VehicleSpecs {
    pub fn validate(&self) -> Result<(), String> {
        self.loon.validate()?;
        self.tube.validate()?;
        self.raft.validate()
    }
}

/// A point mass: kg at a place.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Part {
    pub mass_kg: f32,
    pub at: [f32; 3],
}

/// A box that stands for the hull's own spread of mass, for its inertia.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct InertiaBox {
    pub mass_kg: f32,
    /// Width (x), height (y), length (z), m.
    pub size_m: [f32; 3],
}

/// A spring-damper against the ground or seabed.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ContactSpec {
    pub at: [f32; 3],
    /// N/m.
    pub stiffness: f32,
    /// N s/m.
    pub damping: f32,
    /// Friction along the craft, rolling and braked, and across it.
    pub friction_rolling: f32,
    pub friction_braked: f32,
    pub friction_side: f32,
}

/// Drag of the parts in the air: area times drag coefficient along each body
/// axis (x across, y up, z along), m^2, acting at `at`.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WindageSpec {
    pub at: [f32; 3],
    pub area_m2: [f32; 3],
}

/// Where a person goes: the seat, how near you must be to board, and where
/// you step off.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SeatSpec {
    /// The eye, for the seat camera.
    pub eye: [f32; 3],
    /// Boarding reach from the craft's centre, m.
    pub reach_m: f32,
    /// Where the walker is put on leaving.
    pub exit: [f32; 3],
}

fn positive(name: &str, values: &[f32]) -> Result<(), String> {
    values
        .iter()
        .all(|v| v.is_finite() && *v > 0.0)
        .then_some(())
        .ok_or_else(|| format!("{name} must be positive"))
}

fn seabed(at: [f32; 3], stiffness: f32, damping: f32) -> ContactSpec {
    ContactSpec {
        at,
        stiffness,
        damping,
        friction_rolling: 0.5,
        friction_braked: 0.5,
        friction_side: 0.5,
    }
}

// ---- Loon ---------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PaddleSpec {
    /// How far out the blade works, and its depth under the waterline mark.
    pub reach_m: f32,
    pub depth_m: f32,
    /// Where a stroke catches (z) and how long it is, m.
    pub catch_z: f32,
    pub stroke_m: f32,
    /// Power and recovery, s.
    pub power_s: f32,
    pub recovery_s: f32,
    pub blade_m2: f32,
    pub blade_cd: f32,
    /// The blade held at the stern as a rudder: where, and how much of it.
    pub rudder_at: [f32; 2],
    pub rudder_m2: f32,
    /// A pole, not a paddle: the stroke pushes on the river bed, so its
    /// push is against the ground whatever the water does, and a raft poles
    /// up a river against its current (the owner, 2026-11-18: "the raft guy
    /// pushing the raft up and down the river"). A paddle only moves a craft
    /// through the water.
    #[serde(default)]
    pub pole: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LoonSpec {
    pub parts: [Part; 1],
    pub inertia_box: InertiaBox,
    pub paddler: Part,
    pub hull: HullSpec,
    pub lateral: FoilSpec,
    pub skeg: FoilSpec,
    pub paddle: PaddleSpec,
    pub resistance: ResistanceSpec,
    /// Windage with the paddler aboard, and empty.
    pub windage: WindageSpec,
    pub windage_empty_m2: [f32; 3],
    pub bilge: BilgeSpec,
    pub contacts: [ContactSpec; 3],
    pub heave_damping: f32,
    pub angular_damping: f32,
    pub bow: [f32; 3],
    pub seat: SeatSpec,
    /// Two oars on a rowing frame (`river-craft` design 9): the raft's. A
    /// kind with them is rowed, not paddled on the Loon's stroke.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rowing: Option<super::oars::RowingSpec>,
    /// Its one anchor (`river-craft` design 4, 10.4): the Loon's and the
    /// raft's; the tube has none.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub anchor: Option<super::anchor::AnchorSpec>,
    /// Its own lantern, lit at night (`river-craft` design 9.8): the
    /// raft's. Its light is the placed post lantern's.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lamp: Option<CraftLampSpec>,
}

/// A craft's own lantern (`river-craft` design 9.8).
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CraftLampSpec {
    /// Where it burns, the craft's frame, m.
    pub at: [f32; 3],
}

impl Default for LoonSpec {
    fn default() -> Self {
        Self {
            parts: [Part {
                mass_kg: 30.0,
                at: [0.0, 0.02, 0.0],
            }],
            inertia_box: InertiaBox {
                mass_kg: 30.0,
                size_m: [0.92, 0.5, 5.0],
            },
            paddler: Part {
                mass_kg: 80.0,
                at: [0.0, 0.35, 0.5],
            },
            hull: HullSpec {
                length_m: 5.0,
                beam_m: 0.92,
                depth_m: 0.36,
                sheer_m: 0.30,
                cell_m: 0.14,
                bow_power: 1.8,
                stern_power: 1.8,
                // Flat-bottomed: a round bottom floats a laden canoe on a
                // waterline too narrow to hold its paddler upright.
                section_power: 4.0,
                deck: false,
            },
            lateral: FoilSpec {
                at: [0.0, -0.02, 0.2],
                chord: [0.0, 0.0, -1.0],
                normal: [1.0, 0.0, 0.0],
                area_m2: 1.1,
                aspect: 0.12,
                cl_max: 0.5,
                cd0: 0.02,
            },
            skeg: FoilSpec {
                at: [0.0, -0.03, 2.2],
                chord: [0.0, 0.0, -1.0],
                normal: [1.0, 0.0, 0.0],
                area_m2: 0.06,
                aspect: 0.5,
                cl_max: 0.9,
                cd0: 0.02,
            },
            paddle: PaddleSpec {
                reach_m: 0.62,
                depth_m: -0.18,
                catch_z: -0.4,
                stroke_m: 1.3,
                power_s: 0.55,
                recovery_s: 0.5,
                blade_m2: 0.11,
                blade_cd: 1.25,
                rudder_at: [0.55, 1.9],
                rudder_m2: 0.08,
                pole: false,
            },
            resistance: ResistanceSpec {
                at: [0.0, -0.02, 0.2],
                wetted_m2: 3.2,
                waterline_m: 4.8,
                lateral_m2: 1.0,
                friction: 0.0045,
            },
            windage: WindageSpec {
                at: [0.0, 0.5, 0.0],
                area_m2: [1.3, 1.2, 0.55],
            },
            windage_empty_m2: [0.8, 1.2, 0.3],
            bilge: BilgeSpec {
                open_m2: 3.6,
                rim_share: 1.0,
                rim_from_z: -99.0,
                drain_kgps: 0.0,
                bail_kgps: 6.0,
                capacity_kg: 700.0,
                at: [0.0, 0.0, 0.0],
                slosh: 1.2,
                slosh_max_m: 0.3,
            },
            contacts: [
                seabed([0.0, -0.06, 0.0], 20_000.0, 2_500.0),
                seabed([0.0, 0.05, -2.3], 20_000.0, 2_500.0),
                seabed([0.0, 0.05, 2.3], 20_000.0, 2_500.0),
            ],
            heave_damping: 17_000.0,
            angular_damping: 6.0,
            bow: [0.0, 0.30, -2.1],
            seat: SeatSpec {
                eye: [0.0, 1.0, 0.5],
                reach_m: 3.5,
                exit: [0.0, 0.3, 0.5],
            },
            rowing: None,
            anchor: Some(super::anchor::AnchorSpec::default()),
            lamp: None,
        }
    }
}

impl LoonSpec {
    /// The inner tube (`small-boats` design 2): M34's (`river-craft`, the
    /// Tube tab), a felted-wool ring sealed with pitch, 0.52 m to the middle
    /// of the tube and 0.22 m thick, so 1.48 m across; its paddler sits in
    /// it and paddles with their hands. Round in plan (end powers of 2) and
    /// in section; decked, since the ring takes no water in. Slow, and quick
    /// to turn. 115 kg with its paddler, as M34's.
    pub fn tube() -> Self {
        Self {
            parts: [Part {
                mass_kg: 35.0,
                at: [0.0, 0.0, 0.0],
            }],
            inertia_box: InertiaBox {
                mass_kg: 35.0,
                size_m: [1.48, 0.44, 1.48],
            },
            paddler: Part {
                mass_kg: 80.0,
                at: [0.0, -0.02, 0.06],
            },
            hull: HullSpec {
                length_m: 1.48,
                beam_m: 1.48,
                depth_m: 0.44,
                sheer_m: 0.27,
                cell_m: 0.06,
                bow_power: 2.0,
                stern_power: 2.0,
                section_power: 2.0,
                deck: true,
            },
            lateral: FoilSpec {
                at: [0.0, -0.1, 0.0],
                chord: [0.0, 0.0, -1.0],
                normal: [1.0, 0.0, 0.0],
                area_m2: 0.2,
                aspect: 0.3,
                cl_max: 0.4,
                cd0: 0.05,
            },
            skeg: FoilSpec {
                at: [0.0, -0.1, 0.45],
                chord: [0.0, 0.0, -1.0],
                normal: [1.0, 0.0, 0.0],
                area_m2: 0.02,
                aspect: 0.5,
                cl_max: 0.6,
                cd0: 0.05,
            },
            paddle: PaddleSpec {
                // The rider's feet, kicking over the ring's near side
                // (owner, 2026-11-16: "the guy can just kick his feet around
                // to turn and accelerate"): a foot each side of the middle,
                // under the water ahead, each kick a short sweep aft.
                reach_m: 0.3,
                depth_m: -0.2,
                catch_z: -0.95,
                stroke_m: 0.45,
                power_s: 0.4,
                recovery_s: 0.35,
                blade_m2: 0.05,
                blade_cd: 1.2,
                rudder_at: [0.3, -0.9],
                rudder_m2: 0.02,
                pole: false,
            },
            resistance: ResistanceSpec {
                at: [0.0, -0.06, 0.0],
                wetted_m2: 1.3,
                waterline_m: 1.1,
                lateral_m2: 0.3,
                friction: 0.006,
            },
            windage: WindageSpec {
                at: [0.0, 0.4, 0.0],
                area_m2: [0.6, 0.5, 0.6],
            },
            windage_empty_m2: [0.25, 0.4, 0.25],
            bilge: sealed_bilge(),
            contacts: [
                seabed([0.0, -0.17, 0.0], 8_000.0, 1_200.0),
                seabed([0.0, -0.05, -0.6], 8_000.0, 1_200.0),
                seabed([0.0, -0.05, 0.6], 8_000.0, 1_200.0),
            ],
            heave_damping: 6_000.0,
            angular_damping: 10.0,
            bow: [0.0, 0.2, -0.74],
            // The rider lies back against the ring: the eye low and aft.
            seat: SeatSpec {
                eye: [0.0, 0.5, 0.58],
                reach_m: 2.6,
                exit: [0.0, 0.3, 0.0],
            },
            rowing: None,
            anchor: None,
            lamp: None,
        }
    }

    /// The raft (`small-boats` design 2): M34's (`river-craft`, the Raft
    /// tab), five logs 4.4 m long and 0.25 m round lashed side by side under
    /// three cross planks, 2.4 m across, worked with a pole. Boxy in plan and
    /// round in section, as a row of logs is, and decked, so it does not
    /// swamp. It floats with its logs half under, as M34's draws it.
    pub fn raft() -> Self {
        // Nine 4.4 m logs about 0.3 m thick, green, and the deck boards:
        // about 1,200 kg, so it rides with its deck about 20 cm clear with its
        // poler aboard (the owner, 2026-11-18: "please ensure the raft is
        // boyant"; at 2,000 kg it sat awash, its deck 7 cm over the river).
        Self {
            parts: [Part {
                mass_kg: 1200.0,
                at: [0.0, -0.04, 0.0],
            }],
            inertia_box: InertiaBox {
                mass_kg: 1200.0,
                size_m: [2.4, 0.5, 4.4],
            },
            // Seated on the rowing frame's bench (`river-craft` design 9).
            paddler: Part {
                mass_kg: 80.0,
                at: [0.0, 0.85, 0.52],
            },
            hull: HullSpec {
                length_m: 4.4,
                beam_m: 2.4,
                depth_m: 0.5,
                sheer_m: 0.21,
                cell_m: 0.12,
                bow_power: 8.0,
                stern_power: 8.0,
                section_power: 2.0,
                deck: true,
            },
            lateral: FoilSpec {
                at: [0.0, -0.08, 0.0],
                chord: [0.0, 0.0, -1.0],
                normal: [1.0, 0.0, 0.0],
                area_m2: 0.9,
                aspect: 0.12,
                cl_max: 0.45,
                cd0: 0.03,
            },
            skeg: FoilSpec {
                at: [0.0, -0.2, 2.0],
                chord: [0.0, 0.0, -1.0],
                normal: [1.0, 0.0, 0.0],
                area_m2: 0.08,
                aspect: 0.5,
                cl_max: 0.8,
                cd0: 0.03,
            },
            paddle: PaddleSpec {
                reach_m: 1.35,
                depth_m: -0.3,
                catch_z: -0.2,
                stroke_m: 1.4,
                power_s: 0.8,
                recovery_s: 0.7,
                // A pole's push on the bed: enough for a poler to work the
                // raft up a river against its current.
                blade_m2: 0.7,
                blade_cd: 1.25,
                rudder_at: [1.2, 2.0],
                rudder_m2: 0.14,
                // Not worked: the raft is rowed (`rowing`), and nothing
                // pushes off the bed (the owner, 2026-11-19: "no pole just
                // oar is best").
                pole: false,
            },
            resistance: ResistanceSpec {
                at: [0.0, -0.06, 0.0],
                wetted_m2: 11.0,
                waterline_m: 4.3,
                lateral_m2: 1.2,
                friction: 0.006,
            },
            windage: WindageSpec {
                at: [0.0, 0.5, 0.0],
                area_m2: [2.0, 10.5, 1.4],
            },
            windage_empty_m2: [1.2, 10.5, 0.8],
            bilge: sealed_bilge(),
            contacts: [
                seabed([0.0, -0.29, 0.0], 60_000.0, 8_000.0),
                seabed([0.0, -0.2, -2.0], 60_000.0, 8_000.0),
                seabed([0.0, -0.2, 2.0], 60_000.0, 8_000.0),
            ],
            heave_damping: 40_000.0,
            angular_damping: 8.0,
            bow: [0.0, 0.25, -2.2],
            // The rower sits on the rowing frame's bench amidships, facing
            // the bow (`river-craft` design 9.10): the eye is the rowing
            // spec's at rest.
            seat: SeatSpec {
                eye: [0.0, 1.36, 0.42],
                reach_m: 3.8,
                exit: [0.35, 0.35, 0.7],
            },
            rowing: Some(super::oars::RowingSpec::default()),
            anchor: Some(super::anchor::AnchorSpec::default()),
            // The lantern on the bow post (`draw.rs`, `Builder::lantern`).
            lamp: Some(CraftLampSpec {
                at: [-0.95, 1.45, -1.95],
            }),
        }
    }

    fn validate(&self) -> Result<(), String> {
        positive(
            "loon sizes",
            &[
                self.parts[0].mass_kg,
                self.paddler.mass_kg,
                self.paddle.stroke_m,
                self.paddle.power_s,
                self.paddle.recovery_s,
                self.paddle.blade_m2,
                self.heave_damping,
            ],
        )?;
        self.hull.validate("loon")?;
        if let Some(rowing) = &self.rowing {
            rowing.validate("raft")?;
        }
        if let Some(anchor) = &self.anchor {
            anchor.validate("loon")?;
        }
        self.lateral.validate("loon.lateral")?;
        self.skeg.validate("loon.skeg")
    }
}

/// A decked float's bilge (`small-boats`): no rim lets water in, no rain
/// lies on it, and what does come aboard runs off at once.
fn sealed_bilge() -> BilgeSpec {
    BilgeSpec {
        open_m2: 0.0,
        rim_share: 0.0,
        rim_from_z: 99.0,
        drain_kgps: 50.0,
        bail_kgps: 6.0,
        capacity_kg: 20.0,
        at: [0.0, 0.0, 0.0],
        slosh: 0.0,
        slosh_max_m: 0.0,
    }
}
