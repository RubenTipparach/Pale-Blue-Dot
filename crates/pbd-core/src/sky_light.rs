//! The new lighting model's sun and sky (`lighting-model` L1 and L2): the
//! sun's colour after the air it came through, and the light the whole sky
//! gives a surface, as nine spherical-harmonic coefficients.
//!
//! The arithmetic is the mockup's (`docs/mockups/lighting.html`, the owner's
//! sign-off of 2026-10-28), number for number. Directions here are in a
//! LOCAL frame: y is up, and the sun stands in the x-y plane, so its vector
//! is `(sqrt(1 - e^2), e, 0)` for an elevation sine `e`. The sky is
//! symmetric about that plane, so a shader that rebuilds the frame per
//! fragment (up its own radial, x its own sun azimuth) reads the same table
//! anywhere on the body.

/// Rayleigh optical depth at the zenith, red, green, blue.
pub const TAU: [f32; 3] = [0.040, 0.100, 0.240];
/// Mie (haze) optical depth at the zenith, all colours alike.
pub const TAU_M: f32 = 0.07;
/// Today's sun colour, linear: the terrain shader's `(1.12, 1.03, 0.87)`.
/// The new sun matches it with the sun 60 degrees up.
pub const SUN_NOW: [f32; 3] = [1.12, 1.03, 0.87];
const KR: f32 = 0.9;
const KM: f32 = 0.5;
/// The sunlit ground's albedo, for the light it bounces up.
pub const GROUND_ALBEDO: [f32; 3] = [0.18, 0.26, 0.11];
/// The sun's true angular radius, radians (L3 scales it: the owner's x12).
pub const SUN_RADIUS_RAD: f32 = 0.004_65;
/// The least a surface the sky reaches gets at night (the mockup's floor).
pub const AMBIENT_FLOOR: [f32; 3] = [0.006, 0.008, 0.011];

fn smooth(a: f32, b: f32, x: f32) -> f32 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Air masses along a ray `e` (sine of its elevation) above the horizon:
/// Kasten and Young's, one at the zenith, about 38 at the horizon.
pub fn airmass(e: f32) -> f32 {
    let h = e.clamp(0.0, 1.0).asin();
    1.0 / (h.sin() + 0.50572 * (h.to_degrees() + 6.07995).powf(-1.6364))
}

fn transmittance(e: f32) -> [f32; 3] {
    let m = airmass(e);
    TAU.map(|t| (-(t + TAU_M) * m).exp())
}

/// What gets through at 60 degrees, the normalisation that keeps noon as it
/// is today. The shader's `SUN_NORM` holds these.
pub fn sun_norm() -> [f32; 3] {
    transmittance((60f32).to_radians().sin())
}

/// The sun after the air (L1): white-gold high, gold at ten degrees, red at
/// the horizon, gone below it. `e` is the sine of the sun's elevation.
pub fn sun_air(e: f32) -> [f32; 3] {
    let t = transmittance(e);
    let n = sun_norm();
    let k = smooth(-0.02, 0.01, e);
    [0, 1, 2].map(|i| t[i] / n[i] * SUN_NOW[i] * k)
}

/// The light that lights the upper sky: the sun as seen from a little higher
/// (its elevation plus 0.22), so the zenith stays blue at sunset while the
/// horizon round the sun goes red. Fades out over twilight.
pub fn sun_high(e: f32) -> [f32; 3] {
    let up = (e + 0.22).min(1.0);
    let k = smooth(-0.12, 0.0, e) / smooth(-0.02, 0.01, up).max(1e-3);
    sun_air(up).map(|v| v * k)
}

fn hg(mu: f32, g: f32) -> f32 {
    (1.0 - g * g) / (4.0 * std::f32::consts::PI * (1.0 + g * g - 2.0 * g * mu).powf(1.5))
}

/// The sky's radiance toward `d` (local frame, unit, y up) with the sun at
/// `s` and `cover` (0..1) of it clouded. At night the moon and the stars
/// light the sky, and a cloud deck catches that light and spreads it: an
/// overcast night is an even grey-blue, never black (owner, 2026-10-28).
pub fn sky_radiance(d: [f32; 3], s: [f32; 3], cover: f32) -> [f32; 3] {
    let ev = d[1].max(0.0);
    let mv = airmass(ev).min(14.0);
    let mu = d[0] * s[0] + d[1] * s[1] + d[2] * s[2];
    let sa = sun_air(s[1]);
    let sh = sun_high(s[1]);
    let hi = smooth(0.0, 0.5, ev);
    let pm = hg(mu, 0.76);
    let tm = 1.0 - (-TAU_M * mv).exp();
    let tw = smooth(-0.2, 0.0, s[1]) * (1.0 - smooth(0.0, 0.2, s[1]));
    let twilight = [0.012, 0.02, 0.045];
    let starlight = [0.0012, 0.0018, 0.0035];
    let mut l = [0, 1, 2].map(|i| {
        let tr = (1.0 - (-TAU[i] * mv).exp()) * (-TAU[i] * mv * 0.3).exp();
        let illum = sa[i].max(0.0).powf(0.75) * (1.0 - hi) + sh[i].max(0.0).powf(0.75) * hi;
        illum * (tr * 0.75 * (1.0 + mu * mu) * KR + tm * pm * KM) + twilight[i] * tw + starlight[i]
    });
    let night = 1.0 - smooth(-0.12, 0.02, s[1]);
    let moon_sky = [0.008, 0.011, 0.020];
    let deck_glow = [0.026, 0.029, 0.038];
    for i in 0..3 {
        l[i] += moon_sky[i] * night;
    }
    let g = 0.2126 * l[0] + 0.7152 * l[1] + 0.0722 * l[2];
    let day = smooth(-0.1, 0.1, s[1]);
    let grey = [0.10, 0.105, 0.11];
    let k = cover.clamp(0.0, 1.0) * 0.92;
    [0, 1, 2].map(|i| l[i] + ((g * 0.75 + grey[i] * day * 0.6 + deck_glow[i] * night) - l[i]) * k)
}

/// The nine real SH basis functions at a unit direction, in the order the
/// shader's `sh_irradiance` reads them.
pub fn sh_basis(d: [f32; 3]) -> [f32; 9] {
    let [x, y, z] = d;
    [
        0.282095,
        0.488603 * y,
        0.488603 * z,
        0.488603 * x,
        1.092548 * x * y,
        1.092548 * y * z,
        0.315392 * (3.0 * z * z - 1.0),
        1.092548 * x * z,
        0.546274 * (x * x - y * y),
    ]
}

/// The cosine lobe's convolution per band, over pi: irradiance in the units
/// of the fill it replaces.
const BAND: [f32; 9] = [
    1.0,
    2.0 / 3.0,
    2.0 / 3.0,
    2.0 / 3.0,
    0.25,
    0.25,
    0.25,
    0.25,
    0.25,
];

/// Evenly spread unit directions (a Fibonacci sphere).
fn sphere(n: usize) -> impl Iterator<Item = [f32; 3]> {
    (0..n).map(move |i| {
        let y = 1.0 - 2.0 * (i as f32 + 0.5) / n as f32;
        let r = (1.0 - y * y).max(0.0).sqrt();
        let p = i as f32 * 2.399963;
        [p.cos() * r, y, p.sin() * r]
    })
}

const SAMPLES: usize = 256;

/// Projects radiance over the sphere to irradiance coefficients (L2):
/// `radiance(d)` in, nine RGB coefficients out, already convolved with the
/// cosine lobe and over pi.
pub fn project(radiance: impl Fn([f32; 3]) -> [f32; 3]) -> [[f32; 3]; 9] {
    let mut c = [[0.0f32; 3]; 9];
    let w = 4.0 * std::f32::consts::PI / SAMPLES as f32;
    for d in sphere(SAMPLES) {
        let l = radiance(d);
        let y = sh_basis(d);
        for k in 0..9 {
            for i in 0..3 {
                c[k][i] += l[i] * y[k] * w;
            }
        }
    }
    for (k, ck) in c.iter_mut().enumerate() {
        for v in ck.iter_mut() {
            *v *= BAND[k];
        }
    }
    c
}

/// The sky's light from all round with the sun at elevation sine `e` and
/// `cover` clouded (L2): the sky above, and below the horizon the ground,
/// lit by the sun that reaches it (`sun`, already dimmed by the cloud) and by
/// that sky.
pub fn sky_sh(e: f32, cover: f32, sun: [f32; 3]) -> [[f32; 3]; 9] {
    let e = e.clamp(-1.0, 1.0);
    let s = [(1.0 - e * e).max(0.0).sqrt(), e, 0.0];
    let mut up = [0.0f32; 3];
    let mut w = 0.0;
    for d in sphere(SAMPLES).filter(|d| d[1] > 0.0) {
        let l = sky_radiance(d, s, cover);
        for i in 0..3 {
            up[i] += l[i] * d[1];
        }
        w += d[1];
    }
    let up = up.map(|v| v / w.max(1e-6));
    let ground = [0, 1, 2].map(|i| GROUND_ALBEDO[i] * (sun[i] * e.max(0.0) + up[i]));
    project(|d| {
        if d[1] > 0.0 {
            sky_radiance(d, s, cover)
        } else {
            ground
        }
    })
}

/// Irradiance (over pi) on a surface facing `n` (local frame) from the nine
/// coefficients: the shader's `sh_irradiance`, word for word.
pub fn sh_irradiance(c: &[[f32; 3]; 9], n: [f32; 3]) -> [f32; 3] {
    let y = sh_basis(n);
    [0, 1, 2].map(|i| (0..9).map(|k| c[k][i] * y[k]).sum::<f32>())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn luma(c: [f32; 3]) -> f32 {
        0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2]
    }

    #[test]
    fn the_sun_at_sixty_degrees_is_todays() {
        let s = sun_air((60f32).to_radians().sin());
        for i in 0..3 {
            assert!((s[i] - SUN_NOW[i]).abs() < 1e-4, "{s:?}");
        }
    }

    #[test]
    fn the_low_sun_is_redder_and_gone_below_the_horizon() {
        let noon = sun_air(0.95);
        let low = sun_air((5f32).to_radians().sin());
        assert!(
            low[0] / low[2] > 1.5 * noon[0] / noon[2],
            "{noon:?} {low:?}"
        );
        assert!(luma(low) < luma(noon));
        assert_eq!(sun_air(-0.1), [0.0; 3]);
    }

    #[test]
    fn a_constant_sky_gives_constant_irradiance() {
        let c = project(|_| [0.3, 0.4, 0.5]);
        for n in [
            [0.0, 1.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, -1.0, 0.0],
            [0.6, 0.0, 0.8],
        ] {
            let e = sh_irradiance(&c, n);
            // a constant radiance L gives irradiance pi L, over pi: L
            for (got, want) in e.iter().zip([0.3, 0.4, 0.5]) {
                assert!((got - want).abs() < 0.02, "{n:?} {e:?}");
            }
        }
    }

    #[test]
    fn at_noon_the_sky_above_is_blue_and_the_ground_below_is_its_own_colour() {
        // A sunlit meadow bounces about as much as the blue sky gives, so the
        // two differ in colour, not in brightness: blue from above, the
        // ground's green-gold from below.
        let c = sky_sh(0.9, 0.0, sun_air(0.9));
        let up = sh_irradiance(&c, [0.0, 1.0, 0.0]);
        let down = sh_irradiance(&c, [0.0, -1.0, 0.0]);
        assert!(up[2] > up[0] && up[2] > down[2], "{up:?} {down:?}");
        assert!(down[1] > down[2] && down[0] > down[2], "{down:?}");
        let wall = sh_irradiance(&c, [0.0, 0.0, 1.0]);
        assert!(luma(wall) > 0.1 && luma(wall) < 0.4, "{wall:?}");
    }

    #[test]
    fn an_overcast_night_is_soft_and_never_black() {
        let clear = sky_sh(-0.4, 0.0, [0.0; 3]);
        let deck = sky_sh(-0.4, 1.0, [0.0; 3]);
        let c = sh_irradiance(&clear, [0.0, 1.0, 0.0]);
        let o = sh_irradiance(&deck, [0.0, 1.0, 0.0]);
        assert!(luma(o) > luma(c), "{c:?} {o:?}");
        assert!(luma(o) > 0.015, "{o:?}");
        // even: a wall gets most of what a cap does under the deck
        let wall = sh_irradiance(&deck, [1.0, 0.0, 0.0]);
        assert!(luma(wall) > 0.5 * luma(o), "{wall:?} {o:?}");
    }
}

/// Khronos PBR Neutral (`lighting-model` L6, the owner's pick): the display
/// transform, linear in, linear 0..1 out. Below its knee it only takes a
/// small offset off the darks; above it the peak rolls off toward white and
/// the brightest colours desaturate a little. The shader `tone_grade.wgsl`'s
/// `pbr_neutral` is this, and a test holds its constants.
pub fn pbr_neutral(c: [f32; 3]) -> [f32; 3] {
    const START: f32 = 0.8 - 0.04;
    const DESAT: f32 = 0.15;
    let x = c[0].min(c[1]).min(c[2]);
    let offset = if x < 0.08 { x - 6.25 * x * x } else { 0.04 };
    let c = c.map(|v| v - offset);
    let peak = c[0].max(c[1]).max(c[2]);
    if peak < START {
        return c;
    }
    let d = 1.0 - START;
    let new_peak = 1.0 - d * d / (peak + d - START);
    let c = c.map(|v| v * new_peak / peak);
    let g = 1.0 - 1.0 / (DESAT * (peak - new_peak) + 1.0);
    c.map(|v| v + (new_peak - v) * g)
}

#[cfg(test)]
mod tone_tests {
    use super::pbr_neutral;

    #[test]
    fn khronos_neutral_matches_the_specs_worked_values() {
        // Mid grey keeps its value less the 0.04 offset; white rolls off to
        // 1 - 0.24^2 / 0.44; the darks lose x - 6.25 x^2.
        let g = pbr_neutral([0.18; 3]);
        assert!((g[0] - 0.14).abs() < 1e-6, "{g:?}");
        let w = pbr_neutral([1.0; 3]);
        assert!((w[0] - (1.0 - 0.0576 / 0.44)).abs() < 1e-5, "{w:?}");
        let d = pbr_neutral([0.04; 3]);
        assert!((d[0] - 6.25 * 0.04 * 0.04).abs() < 1e-6, "{d:?}");
        // very bright saturates toward white, and stays under one
        let hot = pbr_neutral([40.0, 8.0, 2.0]);
        assert!(hot.iter().all(|v| *v <= 1.0) && hot[2] > 0.5, "{hot:?}");
    }
}
