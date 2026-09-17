# Tasks

## 1. The owner's decision
- [x] Adopt the handoff's settled 25 m/s^2 at 1 g and `1.4 R` / `1.8 R`
      bands; record the human-scale reason and retain assisted-flight scope.

## 2. The anchor field
- [x] A query returning the strongest-pull body, its multiplier and the
      altitude, with the band shape from `design.md` and per-body overrides
      using explicit optional overrides (zero is a value, never inheritance).
- [x] The space transition reads that query rather than a separate threshold.
- [x] Strongest pull, not nearest centre. A test with two overlapping wells
      where the nearer centre is the wrong answer.

## 3. One falloff, one place
- [x] `walking.rs` computes the falloff inline. Collapse it onto the core well
      so the walker and the ship cannot disagree about gravity.

## 4. The surface constant
- [x] One declared value shared by the anchor and orbital fields, so they agree
      at the surface by construction rather than by two numbers matching.
- [x] Record why it is what it is. The absence of that reason for today's 9.0
      is what made this whole comparison necessary.

## 5. Measure the feel
- [x] Fall time through one cell height, before and after, against Tenebris's
      0.28 s. That is the number that reads as float, more than the apex does.

Measured using the actual Avian walker at 60 Hz: a one-metre fall takes
28 ticks (0.467 s) at the old 9 m/s^2 and 17 ticks (0.283 s) at 25 m/s^2.
The still-six-metre preview steps take 69 ticks (1.150 s) and 42 ticks
(0.700 s), respectively. The jump, hover, shared walker/ship acceleration,
overlap and space-edge regressions pass. See `docs/handoff-validation.md`.
