# Design: a cloud tower grows without re-forming

## 1. The step, from the code

`puffs_at`: `r = spacing (0.5 + 0.35 cover)(0.8 + 0.4 size)`, `height = max(top
* thickness * 0.9, 80 m)`, `stack = clamp(ceil(height / (1.4 r)), 1, 3)`,
`each = height / stack`; billboard k centred at `base + each (k + 0.5)`,
`radius_up = 0.6 each`. As `top` crosses `height = 1.4 r k`, `stack` steps,
`each` drops from `1.4 r` to `1.4 r k / (k + 1)`, and every billboard's centre
and size jump. The weather blend moves `top` every frame, so towers re-form
in one frame wherever the weather is changing.

## 2. Continuous stacks

`each = max(1.4 r, height / 3)` (continuous: the two meet where the count
reaches three). The count `n = min(height / each, 3)` is continuous; billboards
`k < ceil(n)` are drawn, billboard 0 whole, billboard k >= 1 at presence
`clamp(n - k, 0, 1)` on its opacity. Billboard k's centre is
`base + min(each (k + 0.5), height - each / 2)` for k >= 1: a billboard fading
in sits inside the column and rises to its place as it fills. Billboard 0
keeps the old single puff's shape while the column is under one spacing tall
(`each0 = min(each, height)`).

## 3. What moves

A tall tower's billboards are a little larger than before between steps (the
spacing no longer shrinks to fit), and its top billboard fades in and out with
the top instead of the tower re-splitting. Low cloud is as it was.
