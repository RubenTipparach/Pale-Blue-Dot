# Land transport: screenshots

The `cars-*`, `carts-*` and `trains-*` stills are M38's (`docs/mockups/land-transport.html`),
headless Chromium on SwiftShader.

## The game (`land-transport` tasks 5.2 to 5.6)

Fixed-step `--capture` shots in a Claude Code cloud container: **lavapipe, no GPU**, the
`fast` build, new memory-only worlds, 1440 x 900, `--rain 0 --frames 150`. They show the
look only: **frame time was not measured**. `--carts` sets a hand cart and a horse cart
down ahead of the walker by the game's own place path and hitches a bought horse;
`--cart-close hand|horse` sets one broadside close up instead (with a second, loose horse
beside the horse cart, drawn by the farmyard).

| Shot | Command | What it shows |
| --- | --- | --- |
| `game-hand-cart-close-11h00.jpg` | `--walk --carts --cart-close hand --time 11 --pitch -14` | The hand cart let go: resting on its handles' ends, its plank bed and low sides, a spoked wheel with its iron tyre. |
| `game-horse-cart-and-horse-11h00.jpg` | `--walk --carts --cart-close horse --time 11 --pitch -14` | The horse cart with a bought horse in its shafts, side on; a second, loose horse in the right foreground. |
| `game-horse-cart-and-horse-22h30.jpg` | the same, `--time 22.5` | The same at night: cart and horse as dark as the ground, lit as the terrain is. |
| `game-carts-on-and-off-road-11h00.jpg` | `--walk --at 29.60869 4.84535 --yaw 2.6 --carts --time 11 --pitch -10` | On the dirt road from Holbrook: the hand cart at the road's edge, the horse cart off the road among the trees, leaning across its two wheels' ground. |
| `game-carts-on-and-off-road-22h30.jpg` | the same, `--time 22.5` | The same at night, lit by the walker's own light as the road is. |

Not shown yet: a cart in motion (wheels turning, the horse walking) and the hold panel at a
cart's size; both are in the game, and the motion is for the owner's own run.

## Textured and smooth, a plodder in the shafts, riding (designs 12 to 14)

The same container, build profile and settings as above (lavapipe, no GPU, no frame times).
Each `*-before-after-*` is one image, BEFORE on the left and AFTER on the right, labelled in
the frame, the same command on each: BEFORE is the shot above taken before design 12, AFTER
this branch. A capture does not warm (`loading` decision 5), so its log lists
`LATE_PIPELINE` lines by design, before and after alike; the carts' and the coats' meshes
have the pen's vertex layout (positions, normals, UVs, colours) on a textured
`StandardMaterial`, which the farmyard's prewarm templates already build behind the loading
screen.

| Shot | Command | What it shows |
| --- | --- | --- |
| `hand-cart-before-after-11h00.jpg` | `--walk --carts --cart-close hand --time 11 --pitch -14` | The hand cart in the settlement `timber`, grain along every board, handle and spoke and round the rim, its tyre and hub bands in `iron`; 24 rim segments; hub and spokes smooth round, the boards' chamfers kept as creases. |
| `horse-cart-before-after-11h00.jpg` | `--walk --carts --cart-close horse --time 11 --pitch -14` | The horse cart and its horse, a second horse loose and fading in on the right: one smooth lofted horse in the new `coat` texture (hair laid along it, a darker belly, a mane and tail of long hair, horn hooves), tinted by its genome. |
| `horse-cart-before-after-22h30.jpg` | the same, `--time 22.5` | The same at night, lit as the ground is. |
| `game-plodder-cart-11h00.jpg` | `--walk --carts --cart-close plodder --time 11 --pitch -14` | Design 13: a kept plodder in the horse cart's shafts by the one-record hitch ("Plodder hitched"), a second plodder loose beside it. |
| `game-riding-plodder-11h00.jpg` | `--walk --carts --cart-close ride --time 11 --pitch -38 --frames 200` | Design 14: riding a plodder, mounted by the player's own path; the view at the rider's eye on its back (2.1 m over the ground), its neck and horns ahead. |
| `game-riding-plodder-22h30.jpg` | the same, `--time 22.5` | Riding a plodder at night. |

The herd's own BEFORE | AFTER shots (the hen, the woolback, the plodder and the horse side
by side) are in `docs/screenshots/animal-husbandry/`.
