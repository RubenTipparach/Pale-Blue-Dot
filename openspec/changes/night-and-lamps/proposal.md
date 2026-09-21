# Proposal: a night to be dark in, and things that burn in it

## Why

**The owner's words: "Add torches. And lighting support. Add flowers to the
game, and add day night cycle..at night some flowers illuminate the ground as
they are bio florescent. All lighting here is adding to baked vertex colors
which should be super efficient. If an object is non static like player, items
or enties like animals they should receive dynamic lightomg from these
sources".**

The change before this one gave the world a sky light field, so a cave is dark
and a crease is darker. What it has no answer for is the other half of a
lighting model: **light that is not the sun.** And there is nowhere yet for
that to matter, because the sun in this world does not set -
`sky::SUN_DIRECTION` is a `const` at `(0.65, 0.75, 0.35)` and six call sites
normalise their own copy of it. A lamp is worth nothing at noon, and noon is
all there is.

**Flowers already exist**, which is worth saying because the rule here is to
find a feature before building it: the vertex shader grows one on a hashed
share of green cells - a slim stem and a four-triangle head, `SALT_FLOWER`,
`clutter_chance.y`. What they do not do is glow, and what the world does not
have is a night for them to glow in.

## What

Five pieces, each useful on its own and in this order because each needs the
one before it.

**1. A day and a night.** One clock, one sun direction derived from it, and
every consumer reading that one answer rather than normalising its own copy of
a constant. The length of a day is a tunable with a unit. `--time` sets the
hour for a capture, because a harness cannot wait six minutes for dusk.

**2. The block channel.** The field already floods sky light; this floods a
second nibble from EMITTERS, on the same seed-and-flood machinery, and the
shader adds it to the sky term rather than multiplying it by daylight - which
is the whole point of a second channel: one of them goes out at night.

**This is a reversal of the change before it, and the reason is a
measurement.** That change deliberately left the block channel out, citing the
reference: it pulled torches out of its BFS after measuring "~1000 ms / 17M
ops" per placement against "0.4 ms" for an ordinary edit, and made them a
per-vertex proximity sum with no occlusion instead. **That measurement is
about a planet-wide grid of 160k tiles.** Ours is a tier of 3,105 columns and
a full rebake measured **6.0 ms**. So the thing the reference could not afford
is the thing we can, and the BFS is strictly better than what it settled for:
its own comment admits its torch light "ignores walls entirely - it leaks
through stone". Ours will not.

**3. Bioluminescent flowers.** A share of the flowers a cell already grows are
a glowing species, and at night they are emitters. **Which cells glow is the
CPU's answer, not the shader's** - the bake has to know where the light is,
and a hash rolled in two languages is two answers. The record carries a bit and
the shader draws the glowing head where the CPU lit it.

**4. Torches.** A `Material` like any other, so it flows through the whole of
the machinery that already exists: the hotbar holds it, `aim` targets it, the
edit path places and removes it, the save records it, the tier relights. It is
non-solid, so a player walks through it, and transparent to light, so it does
not shadow itself. It gets an icon in the same change, which is this
repository's rule about items.

**5. Dynamic lighting for what moves.** A public `light_at(point)` that reads
the same field, for anything that is not part of the baked geometry. The
static world takes its light at its vertices, which is what "adding to baked
vertex colors" means and is why it is cheap; a thing that moves cannot, so it
samples.

## What is honest about part 5

**There is nothing in this world yet that it can be demonstrated on.** The
player is first person and draws no body, items go straight from the ground to
the hotbar without ever standing in the world, and there are no animals. The
one thing that moves and is drawn is the ship.

So this ships the SAMPLER, applied to the ship, with a test that a point in a
lit cave reads lit and a point in a dark one reads dark. When there is a
dropped item or an animal, the function it needs is there and is the same one
the ground uses. Building the sampler now and the entities later is the right
way round; building a lighting path per entity kind later is how two of them
end up disagreeing.

## What this is NOT

- **No coloured light.** A torch is warm and the tint is the shader's, over
  one intensity channel. RGB is three nibbles instead of one and a widening of
  the same field; it is named in the design as what it would cost.
- **No moon, no stars moving, no seasons.** The sun goes round; the sky
  shader's night is what it already draws.
- **No burning out, no fuel, no fire spread.** A torch is a block that emits.
