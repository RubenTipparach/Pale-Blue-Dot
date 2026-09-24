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

#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct VehicleSpecs {
    pub kestrel: KestrelSpec,
    pub tern: TernSpec,
    pub loon: LoonSpec,
}

impl VehicleSpecs {
    pub fn validate(&self) -> Result<(), String> {
        self.kestrel.validate()?;
        self.tern.validate()?;
        self.loon.validate()
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

fn non_negative(name: &str, values: &[f32]) -> Result<(), String> {
    values
        .iter()
        .all(|v| v.is_finite() && *v >= 0.0)
        .then_some(())
        .ok_or_else(|| format!("{name} must not be negative"))
}

// ---- Kestrel ------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WingSpec {
    /// How far out each panel's centre is, m.
    pub panel_at: [f32; 3],
    pub panel_area_m2: f32,
    pub aspect: f32,
    pub incidence_deg: f32,
    pub dihedral_deg: f32,
    pub cl_max: f32,
    pub cd0: f32,
    /// Aileron throw as a shift of the angle of attack, rad.
    pub aileron_rad: f32,
    /// Flap as the nacelles come up, rad at vertical.
    pub flap_rad: f32,
}

impl WingSpec {
    /// Left and right foils in the reference frame, shared by forces and drawing.
    pub fn panels(&self) -> [FoilSpec; 2] {
        [-1.0, 1.0].map(|side| {
            let inc = (self.incidence_deg as f64).to_radians();
            let turn = glam::DQuat::from_rotation_z((self.dihedral_deg as f64).to_radians() * side);
            let chord = turn * glam::DVec3::new(0.0, inc.sin(), -inc.cos());
            let normal = turn * glam::DVec3::new(0.0, inc.cos(), inc.sin());
            let [x, y, z] = self.panel_at;
            FoilSpec {
                at: [x * side as f32, y, z],
                chord: chord.as_vec3().to_array(),
                normal: normal.as_vec3().to_array(),
                area_m2: self.panel_area_m2,
                aspect: self.aspect,
                cl_max: self.cl_max,
                cd0: self.cd0,
            }
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RotorSpec {
    /// The right rotor's hub; the left is its mirror.
    pub at: [f32; 3],
    /// Static thrust of one rotor at full power, N.
    pub thrust_n: f32,
    /// Inflow along the axis at which thrust is gone, m/s.
    pub inflow_zero_mps: f32,
    pub radius_m: f32,
    /// The most ground effect adds, as a factor.
    pub ground_effect_max: f32,
    /// Drag of a turning rotor to air crossing its disc, N per m/s at full.
    pub edgewise_drag: f32,
    /// How fast the nacelles tilt, rad/s.
    pub tilt_rate: f32,
    /// Share of power moved between rotors for full roll.
    pub roll_mix: f32,
    /// Cyclic pitch and differential-tilt yaw at full power, N m.
    pub pitch_torque: f32,
    pub yaw_torque: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AssistSpec {
    /// Bank and pitch a full stick asks for in the hover, rad.
    pub hover_bank: f32,
    pub hover_pitch: f32,
    /// Rates a full stick asks for in wingborne flight, rad/s.
    pub roll_rate: f32,
    pub pitch_rate: f32,
    pub yaw_rate: f32,
    /// Attitude error to rate, 1/s (roll, pitch).
    pub attitude_gain: [f32; 2],
    /// Rate error to command (roll, pitch, yaw).
    pub rate_gain: [f32; 3],
    /// Climb and sink a full collective asks for in the hover, m/s.
    pub climb_mps: f32,
    pub climb_gain: f32,
    pub climb_integral: f32,
    /// Tilt against drift over the ground: rad per (m/s)/g, and the most.
    pub drift_gain: f32,
    pub drift_tilt: f32,
    /// How fast an empty Kestrel lets itself down, m/s.
    pub unattended_sink_mps: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct KestrelSpec {
    pub mass_kg: f32,
    /// Principal moments about right, up, aft, kg m^2.
    pub inertia: [f32; 3],
    pub wing: WingSpec,
    pub tail: FoilSpec,
    pub elevator_rad: f32,
    pub fin: FoilSpec,
    pub rudder_rad: f32,
    pub rotor: RotorSpec,
    pub assist: AssistSpec,
    /// Drag of the fuselage, m^2 per body axis.
    pub body_drag_m2: [f32; 3],
    /// Aerodynamic damping of turning not already in the surfaces, N m s.
    pub angular_damping: f32,
    pub gear: [ContactSpec; 3],
    /// Fuselage float cells: where, and each one's volume, m^3.
    pub floats: [[f32; 3]; 9],
    pub float_m3: f32,
    /// Drag of each float in water, m^2.
    pub float_drag_m2: f32,
    /// A wet wing: CLmax lost and cd0 gained at `wet_full_mmh` of rain.
    pub wet_cl_loss: f32,
    pub wet_cd_gain: f32,
    pub wet_full_mmh: f32,
    pub seat: SeatSpec,
}

impl Default for KestrelSpec {
    fn default() -> Self {
        let gear = |at: [f32; 3]| ContactSpec {
            at,
            stiffness: 90_000.0,
            damping: 9_000.0,
            friction_rolling: 0.02,
            friction_braked: 0.6,
            friction_side: 0.7,
        };
        Self {
            mass_kg: 1200.0,
            inertia: [2600.0, 4200.0, 1900.0],
            wing: WingSpec {
                panel_at: [2.7, 0.55, 0.05],
                panel_area_m2: 8.0,
                aspect: 6.25,
                incidence_deg: 3.0,
                dihedral_deg: 4.0,
                cl_max: 1.45,
                cd0: 0.012,
                aileron_rad: 0.14,
                flap_rad: 0.12,
            },
            tail: FoilSpec {
                at: [0.0, 0.95, 4.9],
                chord: [0.0, 0.0, -1.0],
                normal: [0.0, 1.0, 0.0],
                area_m2: 3.4,
                aspect: 4.5,
                cl_max: 1.2,
                cd0: 0.012,
            },
            elevator_rad: 0.28,
            fin: FoilSpec {
                at: [0.0, 1.6, 5.0],
                chord: [0.0, 0.0, -1.0],
                normal: [1.0, 0.0, 0.0],
                area_m2: 2.2,
                aspect: 1.6,
                cl_max: 1.1,
                cd0: 0.012,
            },
            rudder_rad: 0.3,
            rotor: RotorSpec {
                // Over the centre of mass: hover thrust aft of it is a
                // nose-down moment the cyclic spends itself holding.
                at: [5.3, 0.62, 0.0],
                thrust_n: 24_000.0,
                inflow_zero_mps: 150.0,
                radius_m: 2.0,
                ground_effect_max: 1.2,
                edgewise_drag: 55.0,
                tilt_rate: 0.26,
                roll_mix: 0.12,
                // Full-power ratings: preserve the measured hover authority
                // at nominal collective 0.625 after removing the old 0.25
                // torque floor (0.875 / 0.625 = 1.4).
                pitch_torque: 12600.0,
                yaw_torque: 9800.0,
            },
            assist: AssistSpec {
                hover_bank: 0.44,
                hover_pitch: 0.3,
                roll_rate: 1.2,
                pitch_rate: 0.8,
                yaw_rate: 0.7,
                attitude_gain: [2.0, 1.8],
                rate_gain: [2.2, 2.6, 2.4],
                climb_mps: 7.0,
                climb_gain: 0.07,
                climb_integral: 0.08,
                drift_gain: 0.9,
                drift_tilt: 0.3,
                unattended_sink_mps: 3.0,
            },
            body_drag_m2: [3.2, 4.0, 0.4],
            angular_damping: 150.0,
            gear: [
                gear([0.0, -1.25, -3.3]),
                gear([-1.4, -1.25, 0.9]),
                gear([1.4, -1.25, 0.9]),
            ],
            floats: std::array::from_fn(|i| [0.0, -0.55, -3.0 + i as f32 * 0.75]),
            float_m3: 0.216,
            float_drag_m2: 0.3,
            wet_cl_loss: 0.15,
            wet_cd_gain: 0.6,
            wet_full_mmh: 150.0,
            seat: SeatSpec {
                eye: [0.0, 0.85, -2.3],
                reach_m: 6.0,
                exit: [-2.2, -0.9, -2.0],
            },
        }
    }
}

impl KestrelSpec {
    fn validate(&self) -> Result<(), String> {
        positive(
            "kestrel masses and sizes",
            &[
                self.mass_kg,
                self.inertia[0],
                self.inertia[1],
                self.inertia[2],
                self.wing.panel_area_m2,
                self.wing.aspect,
                self.wing.cl_max,
                self.rotor.thrust_n,
                self.rotor.inflow_zero_mps,
                self.rotor.radius_m,
                self.rotor.tilt_rate,
                self.float_m3,
                self.wet_full_mmh,
                self.seat.reach_m,
            ],
        )?;
        non_negative(
            "kestrel drags and gains",
            &[
                self.wing.cd0,
                self.body_drag_m2[0],
                self.body_drag_m2[1],
                self.body_drag_m2[2],
                self.angular_damping,
                self.rotor.edgewise_drag,
                self.wet_cl_loss,
                self.wet_cd_gain,
                self.assist.climb_mps,
                self.assist.unattended_sink_mps,
            ],
        )?;
        if self.rotor.ground_effect_max < 1.0 {
            return Err("kestrel.rotor.ground_effect_max must be at least 1".into());
        }
        if self.wet_cl_loss >= 1.0 {
            return Err("kestrel.wet_cl_loss must be below 1".into());
        }
        self.tail.validate("kestrel.tail")?;
        self.fin.validate("kestrel.fin")
    }
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
