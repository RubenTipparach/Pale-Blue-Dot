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
