//! The weather overlays: which field of the simulated atmosphere each one
//! shows, in what unit and over what range.
//!
//! An overlay is a READING of `Atmosphere::sample`, never a formula of its
//! own: humidity, ground sunlight, temperature and the rest are computed once,
//! in the atmosphere, and the overlay picks one. So what an overlay shows and
//! what the clouds and the rain are doing cannot disagree.

use crate::atmosphere::Sample;
use glam::Vec3;

/// One overlay. The order is the order M steps through them.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Overlay {
    /// The surface wind: streamlines coloured by speed.
    Wind,
    /// The wind at cloud height, which carries the cloud.
    Jet,
    /// The ocean's surface current.
    Currents,
    /// Cloud cover.
    Cloud,
    /// Precipitation, with snow in its own colour.
    Rain,
    /// Relative humidity at the surface.
    Humidity,
    /// Sunlight reaching the ground, after the cloud.
    Sunlight,
    /// Surface temperature at the ground's own height.
    Temperature,
}

/// Which colour ramp an overlay is drawn in. The ramps themselves are the
/// renderer's; this names them so the core can say which one a field wants.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Ramp {
    /// Calm blue through green and yellow to red: a speed.
    Speed,
    /// Clear to white: cloud cover.
    Cover,
    /// Pale to deep blue for rain; the negative half is snow's violet.
    Rain,
    /// Dry brown to wet teal.
    Humidity,
    /// Dark to bright yellow.
    Sunlight,
    /// Cold blue through white to hot red.
    Temperature,
}

impl Ramp {
    /// Every ramp, in the order the renderer's table holds them.
    pub const ALL: [Ramp; 6] = [
        Ramp::Speed,
        Ramp::Cover,
        Ramp::Rain,
        Ramp::Humidity,
        Ramp::Sunlight,
        Ramp::Temperature,
    ];

    /// The ramp's row in the renderer's table.
    pub fn index(self) -> usize {
        Ramp::ALL
            .iter()
            .position(|r| *r == self)
            .expect("every ramp is in ALL")
    }
}

impl Overlay {
    /// Every overlay, in the order M steps through them.
    pub const ALL: [Overlay; 8] = [
        Overlay::Wind,
        Overlay::Jet,
        Overlay::Currents,
        Overlay::Cloud,
        Overlay::Rain,
        Overlay::Humidity,
        Overlay::Sunlight,
        Overlay::Temperature,
    ];

    /// The overlay after `current`, and off after the last one: one key walks
    /// through all of them and back to the plain view.
    pub fn next(current: Option<Overlay>) -> Option<Overlay> {
        match current {
            None => Some(Overlay::ALL[0]),
            Some(overlay) => {
                let at = Overlay::ALL
                    .iter()
                    .position(|o| *o == overlay)
                    .expect("every overlay is in ALL");
                Overlay::ALL.get(at + 1).copied()
            }
        }
    }

    /// Its name, as the legend and the menu print it.
    pub fn name(self) -> &'static str {
        match self {
            Overlay::Wind => "WIND",
            Overlay::Jet => "JET",
            Overlay::Currents => "CURRENTS",
            Overlay::Cloud => "CLOUD",
            Overlay::Rain => "RAIN",
            Overlay::Humidity => "HUMIDITY",
            Overlay::Sunlight => "SUNLIGHT",
            Overlay::Temperature => "TEMPERATURE",
        }
    }

    /// The unit its scalar is in.
    pub fn unit(self) -> &'static str {
        match self {
            Overlay::Wind | Overlay::Jet | Overlay::Currents => "m/s",
            Overlay::Cloud | Overlay::Humidity => "%",
            Overlay::Rain => "mm/h",
            Overlay::Sunlight => "W/m2",
            Overlay::Temperature => "C",
        }
    }

    /// The scalar values the ramp's two ends stand for. The rain's low end is
    /// negative because snow is written as negative rain.
    pub fn range(self) -> (f32, f32) {
        match self {
            // Surface winds: the band means run 0-3.5 m/s and the storms
            // faster (`examples/climate.rs`), so the top is where a storm is.
            Overlay::Wind => (0.0, 8.0),
            // The jet's core runs to its cap, `jet_max_mps`.
            Overlay::Jet => (0.0, 45.0),
            // Surface currents are slow (the fastest is about 0.06 m/s); the
            // ramp is set so a gyre reads.
            Overlay::Currents => (0.0, 0.08),
            Overlay::Cloud | Overlay::Humidity => (0.0, 100.0),
            Overlay::Rain => (-10.0, 10.0),
            Overlay::Sunlight => (0.0, 1000.0),
            Overlay::Temperature => (-30.0, 40.0),
        }
    }

    pub fn ramp(self) -> Ramp {
        match self {
            Overlay::Wind | Overlay::Jet | Overlay::Currents => Ramp::Speed,
            Overlay::Cloud => Ramp::Cover,
            Overlay::Rain => Ramp::Rain,
            Overlay::Humidity => Ramp::Humidity,
            Overlay::Sunlight => Ramp::Sunlight,
            Overlay::Temperature => Ramp::Temperature,
        }
    }

    /// Whether it carries a flow the renderer draws streamlines along.
    pub fn flows(self) -> bool {
        self.vector(&Sample::default()).is_some()
    }

    /// Whether it fades out toward zero: clear sky and dry ground are the
    /// scene itself, so the cloud and rain overlays mark only where there IS
    /// cloud or rain, the way a radar map does.
    pub fn fades(self) -> bool {
        matches!(self, Overlay::Cloud | Overlay::Rain)
    }

    fn vector(self, sample: &Sample) -> Option<Vec3> {
        match self {
            Overlay::Wind => Some(sample.wind),
            Overlay::Jet => Some(sample.upper),
            Overlay::Currents => Some(sample.current),
            _ => None,
        }
    }

    /// What the overlay shows at one sample: the scalar in its unit, and the
    /// flow in m/s in the body frame (zero for an overlay with none). This is
    /// the texel the overlay map holds.
    pub fn texel(self, sample: &Sample) -> [f32; 4] {
        let flow = self.vector(sample).unwrap_or(Vec3::ZERO);
        let scalar = match self {
            Overlay::Wind | Overlay::Jet | Overlay::Currents => flow.length(),
            Overlay::Cloud => sample.cover * 100.0,
            Overlay::Rain => {
                let mmh = sample.rain_rate * 3600.0;
                if sample.snow { -mmh } else { mmh }
            }
            Overlay::Humidity => sample.humidity * 100.0,
            Overlay::Sunlight => sample.sunlight,
            Overlay::Temperature => sample.temperature,
        };
        [scalar, flow.x, flow.y, flow.z]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn m_walks_every_overlay_once_and_back_to_off() {
        let mut seen = Vec::new();
        let mut at = Overlay::next(None);
        while let Some(overlay) = at {
            assert!(!seen.contains(&overlay), "{overlay:?} came round twice");
            seen.push(overlay);
            at = Overlay::next(at);
        }
        assert_eq!(seen, Overlay::ALL);
    }

    #[test]
    fn each_overlay_reads_its_own_field_in_its_own_unit() {
        let sample = Sample {
            cover: 0.4,
            rain_rate: 2.0 / 3600.0,
            snow: true,
            wind: Vec3::new(3.0, 0.0, 4.0),
            upper: Vec3::new(0.0, 30.0, 0.0),
            current: Vec3::new(0.1, 0.0, 0.0),
            humidity: 0.75,
            sunlight: 640.0,
            temperature: -4.0,
            ..Sample::default()
        };
        let at = |o: Overlay| o.texel(&sample);
        assert_eq!(at(Overlay::Wind), [5.0, 3.0, 0.0, 4.0]);
        assert_eq!(at(Overlay::Jet)[0], 30.0);
        assert!((at(Overlay::Currents)[0] - 0.1).abs() < 1e-6);
        assert!((at(Overlay::Cloud)[0] - 40.0).abs() < 1e-4);
        // Snow is written as negative rain, in mm/h.
        assert!((at(Overlay::Rain)[0] + 2.0).abs() < 1e-4);
        assert_eq!(at(Overlay::Humidity)[0], 75.0);
        assert_eq!(at(Overlay::Sunlight)[0], 640.0);
        assert_eq!(at(Overlay::Temperature)[0], -4.0);
        // A scalar overlay carries no flow, so it draws no streamlines.
        assert_eq!(&at(Overlay::Cloud)[1..], &[0.0, 0.0, 0.0]);
        let flowing: Vec<_> = Overlay::ALL.into_iter().filter(|o| o.flows()).collect();
        assert_eq!(
            flowing,
            [Overlay::Wind, Overlay::Jet, Overlay::Currents],
            "only the flows draw streamlines"
        );
    }

    #[test]
    fn every_range_runs_upward_and_every_ramp_has_a_row() {
        for overlay in Overlay::ALL {
            let (lo, hi) = overlay.range();
            assert!(lo < hi, "{overlay:?}");
            assert!(overlay.ramp().index() < Ramp::ALL.len());
        }
    }
}
