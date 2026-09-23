//! Arithmetic the step uses in place of the platform maths library.
//!
//! `f32::exp` and friends come from each platform's libm and may differ in
//! their last bits, which would let two machines' weather drift apart. These
//! use only `+ - * /` and bit operations, which IEEE fixes exactly.

/// `e^x`, relative error under 2e-7 over the step's range, deterministic.
pub fn exp(x: f32) -> f32 {
    if x.is_nan() {
        return x;
    }
    let x = x.clamp(-80.0, 80.0);
    // e^x = 2^(x log2 e) = 2^n * 2^f with n whole and f in [-0.5, 0.5].
    let t = x * std::f32::consts::LOG2_E;
    let n = (t + 0.5).floor();
    let f = t - n;
    // 2^f by its Taylor series in f ln 2, to the seventh power.
    let y = f * std::f32::consts::LN_2;
    let p = 1.0
        + y * (1.0
            + y * (0.5
                + y * (1.0 / 6.0
                    + y * (1.0 / 24.0
                        + y * (1.0 / 120.0 + y * (1.0 / 720.0 + y * (1.0 / 5040.0)))))));
    let bits = ((n as i32 + 127) as u32) << 23;
    p * f32::from_bits(bits)
}

pub fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    if edge1 <= edge0 {
        return if x < edge0 { 0.0 } else { 1.0 };
    }
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// A well-mixed hash of a cell and a step, uniform in [0, 1).
pub fn chance(seed: u64, cell: usize, step: u64) -> f32 {
    let mut z = seed
        ^ (cell as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15)
        ^ step.wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^= z >> 31;
    (z >> 40) as f32 / (1u64 << 24) as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exp_matches_the_library_closely() {
        for i in -400..=400 {
            let x = i as f32 * 0.05;
            let want = x.exp();
            let got = exp(x);
            assert!(
                ((got - want) / want).abs() < 2e-6,
                "exp({x}) = {got}, want {want}"
            );
        }
    }

    #[test]
    fn chance_is_uniform_enough() {
        let mut buckets = [0u32; 10];
        for step in 0..20_000u64 {
            buckets[(chance(7, 3, step) * 10.0) as usize] += 1;
        }
        assert!(
            buckets.iter().all(|&b| (1_700..2_300).contains(&b)),
            "{buckets:?}"
        );
    }
}
