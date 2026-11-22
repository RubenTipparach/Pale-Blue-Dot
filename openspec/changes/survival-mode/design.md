# Design

## Context

- **No survival today.** There is no health, no death, no hunger and no
  breath. `cooking` (decision 1, the owner's Q1 answer, asked with
  screenshots) made meals give buffs, with no hunger bar. `swimming` gives
  unlimited breath. `animal-husbandry` keeps predators wild-only, and they
  never attack the player.
- **The design doc says so too.** `docs/GAME_DESIGN.md` section 1 lists
  "survival through constant meter maintenance" as a non-goal. Section 8 says
  "survival creates planning, not chores", and that hazards "telegraph
  cause, trend, threshold, and mitigation".
- **Multiplayer (3b) comes first.** The server owns damage, and every
  survival rule is written for it from the start.
- **Positions and speeds** live in the floating origin. The walker's speed is
  a local `f32`, and "up" and gravity come from the body (`gravity-model`).

## Goals / Non-Goals

**Goals:**
- Survival as an opt-in mode per world, leaving Explorer exactly as it is.
- Pressures that read before they hurt: every damage source shows its cause
  and its trend before the first loss.
- Nothing a player made is lost to death: the pack waits where they fell.
- One set of rules, run by the server in multiplayer and by the in-process
  server alone.

**Non-Goals:**
- PvP, armour, ranged weapons, cold and heat, disease, sleep.
- Difficulty levels. There is one tuning file, `assets/config/survival.ron`.
- Damage to craft from crashes.

## Decisions

### 1. Mode per world, Explorer by default for old saves

The world's identity record gains `mode: Explorer | Survival`. A save without
it is Explorer, so no world changes under a player (CLAUDE.md, "Saved games
survive every change"). The new-world screen offers both, and so does the
world list.

Recommendation taken (ask only with screenshots): the mode is fixed once
chosen. A switch would let a player step out of Survival at its hardest
moment, and switching back would leave a world with no history of being
survived. If the owner wants a switch, it is one stored field and a menu
entry.

### 2. Health

- One bar, 100 points, kept in the player record. Old saves get 100.
- It refills 1 point every 4 s while hunger is above a quarter, and at half
  that rate below a quarter.
- The numbers are in `survival.ron` with units, validated at load.

### 3. Falls

- Damage comes from the downward speed at impact, along the body's up, in the
  body-local frame: `max(0, v - v_safe) * k`.
- `v_safe = sqrt(2 * g * h_safe)`, with `h_safe` = 3.5 m (three layers and a
  half) and `g` the body's own surface gravity. A step off a two-layer ledge
  never hurts, and on a body with half the gravity the same height hurts
  less, as it should.
- `k` is tuned so a twenty-layer fall onto rock takes about 80 points on the
  home planet.
- Landing in a water cell at least 2 layers deep cancels the damage. So does
  a hay or leaf block, if one exists later.

### 4. Hunger

- A bar of 100 that empties in about 45 minutes of play from full. It counts
  play time only, so a closed game never starves anyone (the owner's world
  time rule).
- Every dish gains a `fills` value, scaled by its stars like its buff. Raw
  food fills a little.
- Empty: health stops refilling, then drains 1 point every 10 s. That gives
  over 16 minutes from empty to death, telegraphed by the bar the whole way.
- The buffs stay exactly as `cooking` made them, in both modes.

### 5. Breath and suit air

- **Breath:** 30 s while the head's cell is water, refilled in 3 s once out.
  Then health drains 5 points a second.
- **Suit air:** 60 s on foot where the body's atmosphere is below a
  breathable density (vacuum, airless bodies, very high altitude). It
  refills inside a sealed craft cabin or a breathable atmosphere. A larger
  suit tank is a crafted item, and waits for the rocketry tiers.
- The bar shows only while it is not full, so a walk on dry land shows
  nothing extra.

### 6. Predators

- The wild-only predators gain, in Survival only, a hunt state:
  - **notice**, by range, scaled up at night;
  - **stalk**;
  - **warn**: a growl and a crouch, held for at least 1.5 s before the first
    attack;
  - **attack**: a lunge, with damage from the species' table;
  - **flee**: below a third of its health, or when the player is out of reach
    for a while.
- They have health. The held tool is the weapon: the axe hits hardest, the
  pickaxe less, bare hands least. That uses the swing's own hit moment
  (`held-swing`).
- A killed predator drops its species' drops.
- In Explorer nothing changes: they never attack the player.
- In multiplayer the server runs them (`multiplayer` design 8), and each
  attack is a server change.

### 7. Townsfolk

- In Survival, a struck townsperson fights or flees by role. A guard with a
  spear fights, and the nearby guards come.
- The fight ends when the player leaves the town's walls or is downed.
  A downed player wakes at their bed, not in a cell (no jail in this phase).
- The town's standing with the player drops, through `townsfolk`'s
  reputation, which is built in phase 3.
- In Explorer, a swing at a townsperson does nothing.

### 8. Death and the dropped pack

- At zero health the whole pack, the hotbar included, drops as one bag
  record at the death place, body-relative `f64` like any drop.
- The bag is shown with an icon and a map marker, and never despawns.
- Walking into it returns everything that fits. What does not fit stays in
  the bag.
- The player wakes at the last bed they slept in (a bed is a placed block that
  sets a spawn; `player-building` makes beds) or at the world's spawn, with
  full health, half hunger and full breath.

### 9. The HUD

- Survival only:
  - health and hunger as two short bars beside the hotbar;
  - breath or suit air as a third bar, only while not full;
  - a red edge on the screen for a moment when hit;
  - a plain line on death, "You died", with one button, "Wake up".
- In the stone-and-carved-wood UI (M29), a mockup first, lit at night. No
  explaining text on screen (CLAUDE.md, "Game text is plain").

### 10. Rules in the core, run by the server

- `pbd_core::survival` holds the bars, the damage table and the fall rule, as
  pure functions of state, time step and events, with tests.
- The in-process server (`multiplayer` design 5) ticks them.
- A client predicts its own fall and breath for display, and the server's
  numbers win.
- No player can damage another (PvP stays off).

### 11. Tool wear

The owner, 2026-11-22: "for planning survival: can you add tool degredation?".

- **Survival only.** In Explorer a tool never wears, as today. Every
  recommendation below is taken (ask only with screenshots); the look goes
  to the owner on the HUD mockup (task 1.1).
- **Wear is counted in uses.** Each owned tool carries the uses it has left.
  A use is one finished action: a block broken, a tree felled, a field cell
  tilled, a fish landed, a hit on a creature or a townsperson. A swing at
  nothing, a cast that catches nothing, and a dig given up half way cost
  nothing, so practice and missed swings are free.
- **Harder work costs more.** Breaking a block whose tier is above the
  tool's own (allowed by `tool-crafting` decision 6) costs 2 uses; anything
  else costs 1.
- **The tier sets how long a tool lasts**, in `assets/config/survival.ron`
  (`tool_uses`, by tier): stone 60, bronze 160, iron 320, steel 640. The rod
  has one number per tier as well. These are starting values for the owner
  to play with, not a balance claim.
- **A worn-out tool breaks but is not lost.** At zero uses it becomes a
  broken tool in the same slot: it works as bare hands (`held-swing`'s
  punch), its icon is cracked, and the line "Pickaxe broke" shows once.
  Nothing the player made disappears, in keeping with "Saved games survive".
- **Repair.**
  - At the workbench: half the tool's head materials, rounded up (`tool-crafting`'s recipe for that tool and tier), restore all its uses.
  - In a town: the smith or the trader repairs it for coins, priced from the tool's value (`townsfolk` design 5) times the share worn.
  - A broken tool is repaired the same way.
  - Repairing never raises a tier.
- **What the player sees, Survival only:**
  - a thin bar along the bottom of the tool's slot icon (UI, so the icon's
    1 px outline rule applies), shown once the tool is below full;
  - the bar turns red below a tenth;
  - a short crack sound when it breaks.
  The held model in the world does not change.
- **Saving.** The owned-tool record gains `uses` (additive; a save without
  it reads as full). A use is written in the same durable line as the
  action that spent it (the dig's drop, the catch, the hit), so wear never
  costs an extra write and is never lost on a crash.
- **Multiplayer.** The server counts uses like any damage (decision 10). A
  client shows its own prediction, and the server's number wins.
- **Not in this decision:** tool enchantments or upgrades, wear on armour
  (no armour yet), wear on vehicles and boats (craft damage stays a
  follow-up), and tools wearing in Explorer.

## Risks / Trade-offs

- **Wear can feel like a chore.** That is why missed swings and abandoned digs are free, why a broken tool is kept and can be repaired, and why the numbers live in one file.

- **It reverses a decision the owner made with screenshots** (`cooking` Q1),
  but only in Survival. Explorer keeps it. The owner sees both on the HUD
  mockup.
- **Predators that hunt can make the night a chore.** That is why every
  attack is warned first, why they flee when hurt, and why the tuning lives
  in one file the owner can play with.
- **Fall damage depends on the walker reporting impact speed correctly** in
  the floating origin, on every body. A test lands on the home planet and on a
  lower-gravity body.
- **Fighting townsfolk needs `townsfolk`** (phase 3) to have reputation and
  guards. Phase 4 comes after it, so it will.
