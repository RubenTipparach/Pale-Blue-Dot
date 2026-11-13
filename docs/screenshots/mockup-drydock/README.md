# M23 Drydock and Hex Ship Builder: screenshots

The mockup is `docs/mockups/drydock.html` (change `openspec/changes/vehicle-yards`,
design decision 3). The owner, 2026-11-12: "we need drydocks to build ships too
... I would prefer ocean ships to be made of hex blocks as well so i, as a player
can walk around as the ship sails about".

**Rendered without a GPU.** These were taken in a Claude Code cloud container
with Chromium's software renderer (SwiftShader), at 1280 x 800 CSS pixels and
device scale 2, with the page's `?still` flag, which renders a few frames and
stops. The page is in tabs (Drydock, Ship builder, Sail it, Walk the deck, How
it works) and only the visible tab's scene renders, so every URL below ends in
the tab it shows (`#dock`, `#builder`, `#sail`, `#walk`, `#how`). They show what the page looks like, never how smoothly it runs. On the
owner's desktop the same views render on the GPU (CLAUDE.md, "Render on the
GPU when there is one").

| Shot | What it shows | Page URL flags |
| --- | --- | --- |
| `dock-dry-day.jpg` | The three-master on her keel blocks in the dry dock by day, with staging along her sides. Ten cells by five rows, the most this dock takes. The gantry crane spans the dock; the pump house is at its head. Outside the dock the sea is the new water shader: darker where the chart is deep, lighter over the shallows, foam where it meets the shore. | `?shot&still&time=day&ship=threemaster&view=ship&yaw=0.75&pitch=0.45&dist=62#dock` |
| `dock-night.jpg` | The whole dock at night, dry, with the trader on her blocks. Every floodlight, coping lamp, pump house window, welding flash and ship's lantern lights only what is near it, on the ground and on the water (each lamp lights and is mirrored in the sea near it); the fields and sea outside the pools are near-black and the moon is dim, its path on the water faint. | `?shot&still&time=night#dock` |
| `ship-afloat-day.jpg` | The same by day: the dock water lighter and greener than the open sea (it is shallow), Fresnel sky reflection at a grazing look, sun glint, pixel foam along the copings. | `?shot&still&time=day&view=ship&yaw=2.3&pitch=0.32&dist=36&water=0.03#dock` |
| `ship-afloat-night.jpg` | The dock flooded, the trader afloat at her solved draft (1.22 m), her aftcastle's windows and lanterns lit and reflected in the dock water; she rides its small swell. Shows how a hull of whole hex cells reads (decision D1). | `?shot&still&time=night&view=ship&yaw=2.3&pitch=0.32&dist=36&water=0.03#dock` |
| `dock-floating-out-night.jpg` | Gate open: the caisson has swung along the channel wall and the trader warps out past the jetty. | `?shot&still&time=night&water=0.93&view=gate&yaw=1.9&pitch=0.5&dist=70#dock` |
| `builder.png` | (Taken before the power blocks and the tabs; the builder now fits one screen with the profile beside the stats.) The builder on the trader's main deck (layer 4): the palette, the layer tabs with the waterline layer underlined, the hex grid with the layer below ghosted and walkable cells green, the profile with B, G, M and the waterline, the stats and every check passing. | `?still`, section 2 |
| `walk-deck-night.jpg` | First person on the trader's aftcastle at the helm, at night, sailing her circle: the stern lantern lights the deck, the rail and the water beside her; the island's lighthouse is off the port bow. She heaves and rolls on the swell. The HUD gives the walker's cell in the ship's own frame. | `?still#walk` |
| `walk-deck-day.jpg` | The same on the three-master by day, standing in the waist at the gangway, looking forward along the deck to the forecastle under the foresail. | `?still&wtime=day&walk=threemaster&stand=waist&wyaw=0.35&wpitch=0.12&wt=5.0#walk` |
| `tabs-sail-it.png` | The page at 1280 x 800 on the Sail it tab: the header, the tracker link and the about-the-art note folded at the top, the tab bar, then the whole tab with no scrolling: ship and time buttons, compass bar, the GPS line and advice, and the 3D view beside the chart, with the "Click to play / Press F at the wheel" overlay and the press-and-hold helm buttons. | `?still&sail=demo&demoT=300#sail` (the viewport) |
| `sail-tacking-day.png` | "Sail it" in 3D, the chase camera close behind the sloop at the helm, beating toward waypoint 1 in the trades: the ship large in frame, heeled and riding the swell (she samples the same wave sum the water shader draws), the glowing route and chevron on the water ahead of her bow, the bow wave and wake as foam, sun glint on the sea. The track behind her was sailed by a scripted helmsman following the advice, behind the `?sail=demo` flag, for the picture only: the page advises, it never steers. | `?still&sail=demo&demoT=300#sail` (the 3D view) |
| `sail-tacking-night.png` | The same close chase at night: the deck lit by her lanterns, which also light and reflect on the water around her, red and green running lights, the masthead vane, the faint route ahead on a dark sea and a dim moon path, and the tick on the sail (trimmed). | `?still&sail=demo&demoT=300&stime=night#sail` |
| `sail-tacking-night-route.png` | The same night pulled back (wheel zoom): the route glows faintly across the dark water to the turn. | `?still&sail=demo&demoT=300&camdist=620&campitch=0.6&stime=night#sail` |
| `sail-tack-now.png` | A "Tack now" moment, the whole Sail it panel: the compass bar, GPS line and advice ("Tack now: bring her through the wind ...") above; the 3D view and the chart side by side below. | `?still&sail=demo&demoT=900&demoTack#sail` |
| `sail-on-foot-at-helm.png` | On foot on her deck by the wheel, first person in her frame while she sails on: "F: take the helm". | `?still&sail=demo&demoT=300&sailmode=foot#sail` |
| `sail-overboard-furled.png` | Overboard: the walker stepped off the deck where there was no rail and is swimming in her wake. With no one aboard her sail has furled up to the gaff and she has slowed to a drift; the HUD says "No one aboard: sails furled". | `?still&sail=demo&demoT=300&sailmode=swim#sail` |
| `power-levels.png` | The six power levels, one preset each (rowing launch, sloop, steam packet, oil coaster, fission freighter, battery ferry), with their profiles, power, top speed and range. | `?still`, section 7 |
| `builder-steam.png` | The builder on the steam packet: the twelve power blocks in the palette, and the stats and checks for power, top speed, fuel aboard and range, the funnel to open air and the propeller aft. | `?still&ship=steam`, section 2 |
| `contact-sheet.png` | The new pixel art (28 block icons, 4 textures shown 2 x 2 so a seam would show) beside four of the solitaire collection's references. | `python tools/art/drydock_art.py --sheet` |

## The water

The sea in all three scenes (the dock basin and the open water outside it, the
Walk the deck sea, the Sail it sea) is one shader, built from the M19
vehicles mockup's sea and the game's `sea.wgsl` and `water.wgsl`, as a stand-in
for the game's water: a sum of six sines (the ship heaves, pitches and rolls by
sampling the same sum on the CPU), Fresnel sky reflection, colour by depth from
a baked depth map (deep is darker, the shallows and the dock lighter), sun glint
by day and a dim moon path at night, every lamp lighting and reflecting in the
water near it with the page's `lit()` falloff (dark between the pools at night),
foam on crests, at shorelines, at the bow and in the wake, all on a 0.5 m pixel
grid with nearest sampling. The game's water textures are not used.
