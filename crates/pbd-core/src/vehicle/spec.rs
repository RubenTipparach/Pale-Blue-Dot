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
    pub tern: TernSpec,
    pub loon: LoonSpec,
    /// The harbour's cog (`sail-the-cog`).
    pub cog: CogSpec,
    /// The harbour's rowboat (`cities-in-the-world` task 4.2b): the
    /// mockup's, rowed with the Loon's stroke.
    pub rowboat: LoonSpec,
    /// The inner tube (`small-boats`): paddled by hand.
    pub tube: LoonSpec,
    /// The raft (`small-boats`): logs, paddled.
    pub raft: LoonSpec,
}

impl Default for VehicleSpecs {
    fn default() -> Self {
        Self {
            tern: Default::default(),
            loon: Default::default(),
            cog: Default::default(),
            rowboat: LoonSpec::rowboat(),
            tube: LoonSpec::tube(),
            raft: LoonSpec::raft(),
        }
    }
}

impl VehicleSpecs {
    pub fn validate(&self) -> Result<(), String> {
        self.tern.validate()?;
        self.loon.validate()?;
        self.rowboat.validate()?;
        self.tube.validate()?;
        self.raft.validate()?;
        self.cog.validate()
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

// ---- Tern ---------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SailSpec {
    /// Where the mast steps on the deck.
    pub mast: [f32; 3],
    /// Boom height over the step and its length, and the head's height over
    /// the step, m.
    pub boom_height_m: f32,
    pub boom_length_m: f32,
    pub head_height_m: f32,
    pub area_m2: f32,
    pub aspect: f32,
    pub cl_max: f32,
    pub cd0: f32,
    /// The boom's reach at sheet in and at sheet out, deg.
    pub sheet_deg: [f32; 2],
    /// How fast the sheet is hauled or eased, share per second.
    pub sheet_rate: f32,
    /// How fast the boom swings, rad/s, plus per m/s of apparent wind.
    pub swing_rate: f32,
    pub swing_per_mps: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CrewSpec {
    pub mass_kg: f32,
    pub at: [f32; 3],
    /// How far out the crew can hike, m, and how fast they move, m/s.
    pub hike_m: f32,
    pub hike_rate: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TernSpec {
    pub parts: [Part; 3],
    pub inertia_box: InertiaBox,
    pub crew: CrewSpec,
    pub hull: HullSpec,
    pub sail: SailSpec,
    pub keel: FoilSpec,
    pub rudder: FoilSpec,
    /// The tiller's reach, rad, and how fast it is put over, rad/s.
    pub rudder_max_rad: f32,
    pub rudder_rate: f32,
    pub resistance: ResistanceSpec,
    pub windage: WindageSpec,
    pub bilge: BilgeSpec,
    pub contacts: [ContactSpec; 3],
    /// Heave damping per m^3 submerged, N s/m^4; and damping of turning.
    pub heave_damping: f32,
    pub angular_damping: f32,
    /// Where a mooring line is made fast.
    pub bow: [f32; 3],
    pub seat: SeatSpec,
    /// How its keel lifts in shallow water.
    pub lift: KeelLift,
}

/// A keel that lifts (`cities-in-the-world` task 4.2b): it goes as deep as
/// the water under it allows, its ballast with it. It rises at once when the
/// seabed comes up under it, as a keel kicks up on the bottom, and lowers at
/// its rate when the water deepens.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct KeelLift {
    /// How far it rises from all the way down to all the way up, m.
    pub rise_m: f32,
    /// How fast it lowers: its travel a second.
    pub rate: f32,
    /// How far its tip keeps off the seabed, m.
    pub clearance_m: f32,
    /// Which of the craft's parts is its ballast, and which contact its tip.
    pub ballast: usize,
    pub tip: usize,
}

impl KeelLift {
    /// How far the keel is lifted, m, `down` of the way down (1 all the way).
    pub fn raised_m(&self, down: f64) -> f64 {
        self.rise_m as f64 * (1.0 - down.clamp(0.0, 1.0))
    }
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

impl Default for TernSpec {
    fn default() -> Self {
        let part = |mass_kg, at| Part { mass_kg, at };
        Self {
            parts: [
                part(380.0, [0.0, -0.05, 0.2]),
                part(220.0, [0.0, -1.45, 0.15]),
                part(40.0, [0.0, 3.0, -0.4]),
            ],
            inertia_box: InertiaBox {
                mass_kg: 380.0,
                size_m: [2.3, 1.0, 6.2],
            },
            crew: CrewSpec {
                mass_kg: 80.0,
                at: [0.0, 0.75, 1.6],
                hike_m: 1.1,
                hike_rate: 1.4,
            },
            hull: HullSpec {
                length_m: 6.2,
                beam_m: 2.3,
                depth_m: 1.0,
                sheer_m: 0.62,
                cell_m: 0.25,
                bow_power: 1.6,
                stern_power: 3.5,
                section_power: 1.8,
                deck: true,
            },
            sail: SailSpec {
                mast: [0.0, 0.62, -0.6],
                boom_height_m: 1.3,
                boom_length_m: 3.3,
                head_height_m: 7.0,
                area_m2: 9.4,
                aspect: 3.4,
                cl_max: 1.5,
                cd0: 0.06,
                sheet_deg: [5.0, 85.0],
                sheet_rate: 0.35,
                swing_rate: 1.5,
                swing_per_mps: 0.25,
            },
            keel: FoilSpec {
                at: [0.0, -0.95, 0.15],
                chord: [0.0, 0.0, -1.0],
                normal: [1.0, 0.0, 0.0],
                area_m2: 1.0,
                aspect: 2.2,
                cl_max: 1.2,
                cd0: 0.01,
            },
            rudder: FoilSpec {
                at: [0.0, -0.4, 3.0],
                chord: [0.0, 0.0, -1.0],
                normal: [1.0, 0.0, 0.0],
                area_m2: 0.3,
                aspect: 1.6,
                cl_max: 1.1,
                cd0: 0.012,
            },
            rudder_max_rad: 0.45,
            rudder_rate: 2.2,
            resistance: ResistanceSpec {
                at: [0.0, -0.25, 0.2],
                wetted_m2: 9.0,
                waterline_m: 5.6,
                lateral_m2: 1.8,
                friction: 0.0045,
            },
            windage: WindageSpec {
                at: [0.0, 1.2, 0.0],
                area_m2: [2.8, 3.0, 1.2],
            },
            bilge: BilgeSpec {
                open_m2: 1.6,
                rim_share: 0.3,
                rim_from_z: 0.8,
                drain_kgps: 0.6,
                bail_kgps: 6.0,
                capacity_kg: 900.0,
                at: [0.0, -0.2, 0.5],
                slosh: 1.6,
                slosh_max_m: 0.8,
            },
            contacts: [
                seabed([0.0, -1.5, 0.15], 60_000.0, 6_000.0),
                seabed([0.0, -0.35, -2.8], 60_000.0, 6_000.0),
                seabed([0.0, -0.35, 2.8], 60_000.0, 6_000.0),
            ],
            heave_damping: 17_000.0,
            angular_damping: 40.0,
            bow: [0.0, 0.62, -2.6],
            seat: SeatSpec {
                eye: [0.0, 1.55, 1.6],
                reach_m: 4.5,
                exit: [0.0, 0.62, 0.0],
            },
            // The keel's tip, 1.5 m down, rises to just under the hull.
            lift: KeelLift {
                rise_m: 1.08,
                rate: 0.35,
                clearance_m: 0.1,
                ballast: 1,
                tip: 0,
            },
        }
    }
}

impl TernSpec {
    fn validate(&self) -> Result<(), String> {
        let masses: Vec<f32> = self.parts.iter().map(|p| p.mass_kg).collect();
        positive("tern.parts", &masses)?;
        positive(
            "tern sizes",
            &[
                self.crew.mass_kg,
                self.sail.area_m2,
                self.sail.aspect,
                self.sail.cl_max,
                self.sail.boom_length_m,
                self.sail.sheet_rate,
                self.rudder_max_rad,
                self.rudder_rate,
                self.heave_damping,
            ],
        )?;
        let [lo, hi] = self.sail.sheet_deg;
        if !(0.0..hi).contains(&lo) || hi > 90.0 {
            return Err("tern.sail.sheet_deg must rise within 0..=90".into());
        }
        self.hull.validate("tern")?;
        self.keel.validate("tern.keel")?;
        let lift = &self.lift;
        if !(lift.rise_m >= 0.0 && lift.rate > 0.0 && lift.clearance_m >= 0.0)
            || lift.ballast >= self.parts.len()
            || lift.tip >= self.contacts.len()
        {
            return Err(
                "tern.lift: rise_m and clearance_m at least 0, rate over 0, and ballast and tip naming a part and a contact"
                    .into(),
            );
        }
        self.rudder.validate("tern.rudder")
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
        }
    }
}

impl LoonSpec {
    /// The harbour's rowboat: the towns mockup's (`BOATS.rowboat`), 4.2 m
    /// by 1.45 m and 0.6 m deep, its gunwale 0.35 m over the waterline, so
    /// it draws about 0.25 m. Its oars are the Loon's stroke, out at the
    /// rowlocks' reach.
    pub fn rowboat() -> Self {
        Self {
            parts: [Part {
                mass_kg: 110.0,
                at: [0.0, 0.0, 0.0],
            }],
            inertia_box: InertiaBox {
                mass_kg: 110.0,
                size_m: [1.45, 0.6, 4.2],
            },
            paddler: Part {
                mass_kg: 80.0,
                at: [0.0, 0.3, 0.3],
            },
            hull: HullSpec {
                length_m: 4.2,
                beam_m: 1.45,
                depth_m: 0.6,
                sheer_m: 0.35,
                cell_m: 0.1,
                bow_power: 2.4,
                stern_power: 2.0,
                // Flat-floored, as the mockup's U section is (about 3 to 4):
                // a round bottom floats it with its rower on a waterline too
                // narrow to hold them upright, as the Loon's.
                section_power: 3.5,
                deck: false,
            },
            lateral: FoilSpec {
                at: [0.0, -0.12, 0.1],
                chord: [0.0, 0.0, -1.0],
                normal: [1.0, 0.0, 0.0],
                area_m2: 1.0,
                aspect: 0.15,
                cl_max: 0.5,
                cd0: 0.02,
            },
            skeg: FoilSpec {
                at: [0.0, -0.15, 1.8],
                chord: [0.0, 0.0, -1.0],
                normal: [1.0, 0.0, 0.0],
                area_m2: 0.08,
                aspect: 0.5,
                cl_max: 0.9,
                cd0: 0.02,
            },
            paddle: PaddleSpec {
                reach_m: 1.1,
                depth_m: -0.15,
                catch_z: -0.6,
                stroke_m: 1.4,
                power_s: 0.7,
                recovery_s: 0.6,
                blade_m2: 0.12,
                blade_cd: 1.25,
                rudder_at: [0.8, 1.8],
                rudder_m2: 0.08,
            },
            resistance: ResistanceSpec {
                at: [0.0, -0.1, 0.1],
                wetted_m2: 4.5,
                waterline_m: 3.9,
                lateral_m2: 1.2,
                friction: 0.0045,
            },
            windage: WindageSpec {
                at: [0.0, 0.4, 0.0],
                area_m2: [1.2, 1.4, 0.6],
            },
            windage_empty_m2: [0.8, 1.4, 0.4],
            bilge: BilgeSpec {
                open_m2: 5.0,
                rim_share: 1.0,
                rim_from_z: -99.0,
                drain_kgps: 0.0,
                bail_kgps: 6.0,
                capacity_kg: 1200.0,
                at: [0.0, -0.1, 0.0],
                slosh: 1.2,
                slosh_max_m: 0.4,
            },
            contacts: [
                seabed([0.0, -0.25, 0.0], 20_000.0, 2_500.0),
                seabed([0.0, 0.0, -1.9], 20_000.0, 2_500.0),
                seabed([0.0, 0.0, 1.9], 20_000.0, 2_500.0),
            ],
            heave_damping: 17_000.0,
            angular_damping: 8.0,
            bow: [0.0, 0.35, -2.0],
            seat: SeatSpec {
                eye: [0.0, 1.05, 0.3],
                reach_m: 3.5,
                exit: [0.0, 0.35, 0.3],
            },
        }
    }

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
                at: [0.0, -0.1, 0.0],
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
                reach_m: 0.62,
                depth_m: -0.12,
                catch_z: -0.3,
                stroke_m: 0.55,
                power_s: 0.45,
                recovery_s: 0.4,
                blade_m2: 0.035,
                blade_cd: 1.2,
                rudder_at: [0.55, 0.5],
                rudder_m2: 0.02,
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
            seat: SeatSpec {
                eye: [0.0, 0.85, 0.05],
                reach_m: 2.6,
                exit: [0.0, 0.3, 0.0],
            },
        }
    }

    /// The raft (`small-boats` design 2): M34's (`river-craft`, the Raft
    /// tab), five logs 4.4 m long and 0.25 m round lashed side by side under
    /// three cross planks, 2.4 m across, worked with a pole. Boxy in plan and
    /// round in section, as a row of logs is, and decked, so it does not
    /// swamp. It floats with its logs half under, as M34's draws it.
    pub fn raft() -> Self {
        Self {
            parts: [Part {
                mass_kg: 2000.0,
                at: [0.0, -0.04, 0.0],
            }],
            inertia_box: InertiaBox {
                mass_kg: 2000.0,
                size_m: [2.4, 0.5, 4.4],
            },
            paddler: Part {
                mass_kg: 80.0,
                at: [0.35, 0.9, 0.7],
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
                blade_m2: 0.28,
                blade_cd: 1.25,
                rudder_at: [1.2, 2.0],
                rudder_m2: 0.14,
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
            // The poler stands on the planks aft of the middle, as M34's.
            seat: SeatSpec {
                eye: [0.35, 1.75, 0.7],
                reach_m: 3.8,
                exit: [0.35, 0.35, 0.7],
            },
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

// ---- Cog ----------------------------------------------------------------------

/// A square sail bent to a yard, which the braces turn about the mast
/// (`sail-the-cog` design 1).
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SquareSailSpec {
    /// Where the mast steps on the deck.
    pub mast: [f32; 3],
    /// The yard's height over the step and the sail's foot's, and the sail's
    /// width, m: its area is the width by the height between.
    pub yard_height_m: f32,
    pub foot_height_m: f32,
    pub width_m: f32,
    pub aspect: f32,
    pub cl_max: f32,
    pub cd0: f32,
    /// How far the braces turn the yard either way off square, deg, and how
    /// fast, deg/s.
    pub brace_deg: f32,
    pub brace_rate_deg: f32,
    /// The luff, deg: a square sail's weather edge is a free leech, not a
    /// spar, so with the wind closer to the yard than `luff_deg` it falls in
    /// and the sail lifts nothing; it fills, and lifts as the foil it is,
    /// from `fill_deg`. Drag is whole throughout: a luffing sail flogs.
    pub luff_deg: f32,
    pub fill_deg: f32,
}

impl SquareSailSpec {
    /// The sail's area, m^2.
    pub fn area_m2(&self) -> f32 {
        self.width_m * (self.yard_height_m - self.foot_height_m)
    }
}

/// The cog: the harbour's ship, 15 m by 5 m, the mockup's (`towns.html`,
/// `cog()`), sailed from its tiller on the aftcastle. Its reference frame's
/// origin is its waterline's middle, as the harbour's moored piece's is.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CogSpec {
    /// The hull's timber, its ballast, and the mast and yard.
    pub parts: [Part; 3],
    pub inertia_box: InertiaBox,
    /// Whoever has the helm, at the tiller.
    pub helmsman: Part,
    pub hull: HullSpec,
    pub sail: SquareSailSpec,
    pub keel: FoilSpec,
    pub rudder: FoilSpec,
    /// The tiller's reach, rad, and how fast it is put over, rad/s.
    pub rudder_max_rad: f32,
    pub rudder_rate: f32,
    pub resistance: ResistanceSpec,
    pub windage: WindageSpec,
    pub bilge: BilgeSpec,
    pub contacts: [ContactSpec; 3],
    pub heave_damping: f32,
    pub angular_damping: f32,
    /// Where a mooring line is made fast.
    pub bow: [f32; 3],
    pub seat: SeatSpec,
}

impl Default for CogSpec {
    fn default() -> Self {
        let part = |mass_kg, at| Part { mass_kg, at };
        Self {
            parts: [
                part(22_000.0, [0.0, 0.0, 0.4]),
                part(9_500.0, [0.0, -0.95, 0.0]),
                part(2_500.0, [0.0, 8.5, -1.5]),
            ],
            inertia_box: InertiaBox {
                mass_kg: 22_000.0,
                size_m: [5.0, 3.2, 15.0],
            },
            helmsman: part(80.0, [0.0, 4.4, 5.6]),
            hull: HullSpec {
                length_m: 15.0,
                beam_m: 5.0,
                depth_m: 3.1,
                sheer_m: 1.8,
                cell_m: 0.8,
                bow_power: 2.0,
                stern_power: 2.6,
                section_power: 4.0,
                deck: true,
            },
            sail: SquareSailSpec {
                mast: [0.0, 1.9, -1.5],
                yard_height_m: 11.0,
                foot_height_m: 4.0,
                width_m: 8.8,
                aspect: 1.3,
                cl_max: 1.0,
                cd0: 0.12,
                brace_deg: 60.0,
                brace_rate_deg: 12.0,
                luff_deg: 15.0,
                fill_deg: 30.0,
            },
            keel: FoilSpec {
                at: [0.0, -1.0, 0.0],
                chord: [0.0, 0.0, -1.0],
                normal: [1.0, 0.0, 0.0],
                area_m2: 7.0,
                aspect: 0.6,
                cl_max: 0.9,
                cd0: 0.02,
            },
            rudder: FoilSpec {
                at: [0.0, -0.5, 7.2],
                chord: [0.0, 0.0, -1.0],
                normal: [1.0, 0.0, 0.0],
                area_m2: 1.6,
                aspect: 1.4,
                cl_max: 1.0,
                cd0: 0.015,
            },
            rudder_max_rad: 0.6,
            rudder_rate: 0.6,
            resistance: ResistanceSpec {
                at: [0.0, -0.5, 0.0],
                wetted_m2: 70.0,
                waterline_m: 13.5,
                lateral_m2: 14.0,
                friction: 0.004,
            },
            windage: WindageSpec {
                at: [0.0, 3.0, 0.0],
                area_m2: [35.0, 60.0, 12.0],
            },
            bilge: BilgeSpec {
                open_m2: 4.0,
                rim_share: 0.1,
                rim_from_z: -7.5,
                drain_kgps: 5.0,
                bail_kgps: 20.0,
                capacity_kg: 8_000.0,
                at: [0.0, -0.6, 0.0],
                slosh: 1.0,
                slosh_max_m: 1.5,
            },
            contacts: [
                seabed([0.0, -1.35, 0.0], 2_000_000.0, 200_000.0),
                seabed([0.0, -0.9, -6.5], 2_000_000.0, 200_000.0),
                seabed([0.0, -0.9, 6.5], 2_000_000.0, 200_000.0),
            ],
            heave_damping: 17_000.0,
            angular_damping: 200_000.0,
            bow: [0.0, 1.9, -7.2],
            seat: SeatSpec {
                eye: [0.0, 5.1, 5.6],
                reach_m: 3.0,
                exit: [0.0, 3.5, 4.6],
            },
        }
    }
}

impl CogSpec {
    fn validate(&self) -> Result<(), String> {
        let masses: Vec<f32> = self.parts.iter().map(|p| p.mass_kg).collect();
        positive("cog.parts", &masses)?;
        positive(
            "cog sizes",
            &[
                self.helmsman.mass_kg,
                self.sail.width_m,
                self.sail.aspect,
                self.sail.cl_max,
                self.sail.brace_rate_deg,
                self.rudder_max_rad,
                self.rudder_rate,
                self.heave_damping,
            ],
        )?;
        // A square sail needs a yard over its foot to be bent to.
        if !(self.sail.yard_height_m > self.sail.foot_height_m && self.sail.foot_height_m >= 0.0) {
            return Err("cog.sail: the yard must stand over the sail's foot".into());
        }
        if !(0.0..=90.0).contains(&self.sail.brace_deg) {
            return Err("cog.sail.brace_deg must be within 0..=90".into());
        }
        if !(0.0 <= self.sail.luff_deg
            && self.sail.luff_deg < self.sail.fill_deg
            && self.sail.fill_deg <= 90.0)
        {
            return Err("cog.sail: luff_deg must rise to fill_deg within 0..=90".into());
        }
        self.hull.validate("cog")?;
        self.keel.validate("cog.keel")?;
        self.rudder.validate("cog.rudder")
    }
}
