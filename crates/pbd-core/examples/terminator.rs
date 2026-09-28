//! Where the day-night line crosses the equator at an hour of day 0: the
//! longitudes where `Clock::daylight` is one half. A measurement instrument
//! for `world-map` task 3.1, checked against a capture of the map's night.
//!
//!     cargo run -p pbd-core --example terminator -- <hour>
use pbd_core::daylight::Clock;
use pbd_core::geo::{self, LatLon};

fn main() {
    let hour: f32 = std::env::args()
        .nth(1)
        .and_then(|a| a.parse().ok())
        .unwrap_or(12.0);
    let clock = Clock::at_hour(hour);
    let (lat, lon) = geo::lat_lon(clock.sun()).degrees();
    println!("hour {hour}: the sun is overhead at {lat:.2} N {lon:.2} E");
    let day = |lon: f32| {
        clock.daylight(geo::direction(LatLon {
            lat: 0.0,
            lon: lon.to_radians(),
        }))
    };
    let mut crossings = Vec::new();
    let steps = 36_000;
    for i in 0..steps {
        let (a, b) = (
            -180.0 + 360.0 * i as f32 / steps as f32,
            -180.0 + 360.0 * (i + 1) as f32 / steps as f32,
        );
        if (day(a) - 0.5).signum() != (day(b) - 0.5).signum() {
            crossings.push((a + b) / 2.0);
        }
    }
    for lon in crossings {
        let side = if day(lon + 1.0) > day(lon - 1.0) {
            "dawn"
        } else {
            "dusk"
        };
        println!("the equator's {side} line: {lon:.2} E");
    }
}
