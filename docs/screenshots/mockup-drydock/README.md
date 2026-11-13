# M23 Drydock and Hex Ship Builder: screenshots

The mockup is `docs/mockups/drydock.html` (change `openspec/changes/vehicle-yards`,
design decision 3). The owner, 2026-11-12: "we need drydocks to build ships too
... I would prefer ocean ships to be made of hex blocks as well so i, as a player
can walk around as the ship sails about".

**Rendered without a GPU.** These were taken in a Claude Code cloud container
with Chromium's software renderer (SwiftShader), at 1280 x 800 CSS pixels and
device scale 2, with the page's `?still` flag, which renders a few frames and
stops. They show what the page looks like, never how smoothly it runs. On the
owner's desktop the same views render on the GPU (CLAUDE.md, "Render on the
GPU when there is one").

| Shot | What it shows | Page URL flags |
| --- | --- | --- |
| `dock-dry-day.jpg` | The three-master on her keel blocks in the dry dock by day, with staging along her sides. Ten cells by five rows, the most this dock takes. The gantry crane spans the dock; the pump house is at its head. | `?shot&still&time=day&ship=threemaster&view=ship&yaw=0.75&pitch=0.45&dist=62` |
| `dock-night.jpg` | The whole dock at night, dry, with the trader on her blocks. Every floodlight, coping lamp, pump house window, welding flash and ship's lantern lights only what is near it; the fields outside the pools are near-black and the moon is dim. | `?shot&still&time=night` |
| `ship-afloat-night.jpg` | The dock flooded, the trader afloat at her solved draft (1.22 m), her aftcastle's windows and lanterns lit. Shows how a hull of whole hex cells reads (decision D1). | `?shot&still&time=night&view=ship&yaw=2.3&pitch=0.32&dist=36&water=0.03` |
| `dock-floating-out-night.jpg` | Gate open: the caisson has swung along the channel wall and the trader warps out past the jetty. | `?shot&still&time=night&water=0.93&view=gate&yaw=1.9&pitch=0.5&dist=70` |
| `builder.png` | (Taken before the power blocks.) The builder on the trader's main deck (layer 4): the palette, the layer tabs with the waterline layer underlined, the hex grid with the layer below ghosted and walkable cells green, the profile with B, G, M and the waterline, the stats and every check passing. | `?still`, section 2 |
| `walk-deck-night.jpg` | First person on the trader's aftcastle at the helm, at night, sailing her circle: the stern lantern lights the deck and rail; the island's lighthouse is off the port bow. The HUD gives the walker's cell in the ship's own frame. | `?still`, section 3 |
| `walk-deck-day.jpg` | The same on the three-master by day, standing in the waist at the gangway, looking forward along the deck to the forecastle under the foresail. | `?still&wtime=day&walk=threemaster&stand=waist&wyaw=0.35&wpitch=0.12&wt=5.0`, section 3 |
| `sail-tacking-day.png` | "Sail it" in 3D (decision 3a): the sloop at the helm, beating toward waypoint 1 dead upwind in the trades. The chase camera is pulled back (wheel zoom) so the glowing route on the water shows its close-hauled legs and the chevron at the next turn. Above: the compass bar (amber waypoint, cyan best heading), the GPS line and the advice ("Steady on 114", "Sail trimmed (18°)", VMG now against best). Below: the chart, now the map beside the 3D view. The track behind her was sailed by a scripted helmsman following the advice, behind the `?sail=demo` flag, for the picture only: the page advises, it never steers. | `?still&sail=demo&demoT=300&camdist=620&campitch=0.6`, section 4 |
| `sail-tacking-night.png` | The chase camera close behind her at night: the deck lit by her lanterns, red and green running lights, the masthead vane, the faint route ahead on a dark sea, and the tick on the sail (trimmed). | `?still&sail=demo&demoT=300&stime=night`, section 4 |
| `sail-tacking-night-route.png` | The same night pulled back: the route glows faintly across the dark water to the turn. | `?still&sail=demo&demoT=300&camdist=620&campitch=0.6&stime=night` |
| `sail-tack-now.png` | A "Tack now" moment: she has reached the lane's edge on the port-tack leg; the advice says "Tack now: bring her through the wind to starboard (head 114)", the route leaves her bow to starboard and the chevron points the leg after. | `?still&sail=demo&demoT=900&demoTack&camdist=620&campitch=0.6` |
| `sail-on-foot-at-helm.png` | On foot on her deck by the wheel, first person in her frame while she sails on: "F: take the helm". | `?still&sail=demo&demoT=300&sailmode=foot` |
| `sail-overboard-furled.png` | Overboard: the walker stepped off the deck where there was no rail and is swimming in her wake. With no one aboard her sail has furled up to the gaff and she has slowed to a drift (0.5 m/s and falling); the HUD says "No one aboard: sails furled". | `?still&sail=demo&demoT=300&sailmode=swim` |
| `power-levels.png` | The six power levels, one preset each (rowing launch, sloop, steam packet, oil coaster, fission freighter, battery ferry), with their profiles, power, top speed and range. | `?still`, section 7 |
| `builder-steam.png` | The builder on the steam packet: the twelve power blocks in the palette, and the stats and checks for power, top speed, fuel aboard and range, the funnel to open air and the propeller aft. | `?still&ship=steam`, section 2 |
| `contact-sheet.png` | The new pixel art (28 block icons, 4 textures shown 2 x 2 so a seam would show) beside four of the solitaire collection's references. | `python tools/art/drydock_art.py --sheet` |
