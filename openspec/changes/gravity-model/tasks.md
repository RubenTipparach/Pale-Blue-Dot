# Tasks

## 1. The owner's decision
- [ ] Is `tenebris-rs` the definitive spec for gravity as it is for hex size? If
      yes, 25 m/s^2 at 1 g and the `1.4 R` / `1.8 R` bands are the standard. If
      no, the two-field shape is still worth taking and the constant is chosen
      here, with its reason recorded.

## 2. The anchor field
- [ ] A query returning the strongest-pull body, its multiplier and the
      altitude, with the band shape from `design.md` and per-body overrides
      where zero means "use the default".
- [ ] The space transition reads that query rather than a separate threshold.
- [ ] Strongest pull, not nearest centre. A test with two overlapping wells
      where the nearer centre is the wrong answer.

## 3. One falloff, one place
- [ ] `walking.rs` computes the falloff inline. Collapse it onto the core well
      so the walker and the ship cannot disagree about gravity.

## 4. The surface constant
- [ ] One declared value shared by the anchor and orbital fields, so they agree
      at the surface by construction rather than by two numbers matching.
- [ ] Record why it is what it is. The absence of that reason for today's 9.0
      is what made this whole comparison necessary.

## 5. Measure the feel
- [ ] Fall time through one cell height, before and after, against Tenebris's
      0.28 s. That is the number that reads as float, more than the apex does.
