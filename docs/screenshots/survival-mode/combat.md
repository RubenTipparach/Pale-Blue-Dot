# Combat in the game (`survival-mode` tasks 1.6, 4.3, 4.4, 4.5)

Headless `--capture` shots in a Claude Code cloud container, rendered on
lavapipe (software Vulkan, no GPU) at 1440 x 900 under Xvfb, stepped at a
fixed 1/60 s: they show what things look like, never how fast or how smoothly
they draw. Smoothness is judged on the owner's machine. Every shot is a
memory-only world made a Survival one by `--combat <scene>`, a capture
instrument (nothing a player meets); the commands are
`tools/capture_survival_combat.sh`. The motion follows the drafted mockups
(`docs/mockups/survival.html#weapons`, In hand, and `#hunt`), to be retuned
when the owner reviews them.

| Shot | What it shows |
| --- | --- |
| `combat-fox-stalk-day.jpg`, `combat-fox-stalk-night.jpg` | A red fox placed 9 m off on the grass noticed the player (14 m, x1.5 at night) and stalks in, low, at 1.4 m/s (the log: `Stalk`). At night the lamps of `--lamps` light it. |
| `combat-fox-warn-day.jpg`, `combat-fox-warn-night.jpg` | The same fox at 5 m: it stops, crouches and faces the player, the warning held while the player stays out of its lunge (the log: `Warn`). |
| `combat-hound-warn-day.jpg` | A dust hound warning, crouched. |
| `combat-swing-hit-day.jpg`, `combat-swing-hit-night.jpg` | A bronze sword's slash at the rule's hit moment (0.45 of 0.5 s after the press) on a fox held 1.7 m ahead: the log's `COMBAT_HIT red-fox for 28 -> Hurt`. |
| `combat-axe-hit-day.jpg` | An iron hunting axe's chop landing on the fox for 30: killed, it lies, and its bone drops (`COMBAT_KILL red-fox dropping [("bone", 1)]`). |
| `combat-spear-flight-day.jpg` | A stone spear raised 0.5 s and thrown at a fox 7 m off: in flight, seen from behind (`COMBAT_LAUNCH stone-spear`), later landing as a drop (`COMBAT_PROJECTILE_LANDED stone-spear kept true`). |

Not shot yet: the guards coming (`--combat guard`). The town spots the
talk captures used now open inside a building or a walled yard with no guard
within 40 m (the log: `COMBAT_SCRIPT struck ... standing Some(-10), 0
guards`); the fight itself is pinned by
`combat::tests::a_struck_villager_brings_the_guards_and_lowers_the_towns_standing`.
The spear by night is not shot either: the night lamps the capture lays
stand across its line.

Looks to judge:

- The predators' crouch is the rig's lie-down partway; the attack's gather,
  spring and landing squash are the mockup's, not yet the game's.
- A killed animal lies as a resting one does.
- The swings' poses were moved for the game's view: the sword's hit and
  recoil were raised (it was below the frame at the hit), in the mockup and
  `held.ron` together.
