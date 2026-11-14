# Industry mockup screenshots

Shots of `docs/mockups/industry.html` (M20, "Pits, Plants and Power").

**Rendered in software.** Every shot here was taken in a Claude Code cloud container with no GPU:
headless Chromium (Playwright) on SwiftShader, 1280 x 800 at device scale 2, the page served by
`python3 -m http.server`. They show what the page looks like, never how smoothly it runs, and they
are to be retaken on the owner's GPU (Edge, `--use-angle=d3d11 --enable-gpu
--ignore-gpu-blocklist`) before the page is gated (CLAUDE.md, "Render on the GPU"). The page's own
frame loop was paused for the capture, after the scene had settled, because SwiftShader takes
about a second a frame here.

## The first set (2026-11-12)

`gate_industry_day.jpg`, `gate_industry_night.jpg`, `gate_industry_mining_night.jpg`, `mining.png`,
`mining-night.png`, `refining-night.png`, `ether-night.png`, `power-fusion-night.png`,
`power-dyson-space.png`, `contact-sheet.png`: the page as first published.

## Oil (2026-11-13; steer-and-sail design, decisions 5 and 6)

| Shot | Shows |
| --- | --- |
| `oil-field.png` | Section 5 by day from above: the oil overlay (an inset hex per cell over the field, sized and coloured by reservoir pressure, the field's edge in amber), the pumpjack on its concrete cell, the crude tank, the flowline, the refinery yard. The side panel's map is the seed-derived field over a stand-in region (one pixel 50 m), the view's square in white. |
| `oil-field-night.png` | The same at 22:00: lamps, the crude heater, the control room's windows and the flare light the yard; the field between them is dark. |
| `oil-well.png` | The pumpjack close up by day: skid, samson post, walking beam and horsehead, crank and weights, wellhead; still, because nothing runs until Run is pressed. |
| `oil-well-night.png` | The pumpjack at night under its lamp. |
| `oil-well-running.png` | After "Run the pumpjack": the beam mid-stroke and the drawdown readout at 2,400 game days pumped (rate falling as the well's share is drawn). |
| `oil-refinery-night.png` | The distillation column (2 cells, 4 layers) and its crude heater, the four product tanks (residue, fuel oil, diesel, polymer feed), the red-and-white flare stack burning, the control room's lit windows, lamps with dark between their pools. |
| `power-oil.png` | The oil plant on the power ladder (tier I, between coal and wind): boiler hall, turbine hall, steel stack and its smoke, two fuel oil tanks in a bund. |
| `power-oil-night.png` | The oil plant at night: burners, turbine hall windows, lamps. |
| `power-ladder-oil.png` | The power sheet's ladder with the oil plant beside the coal plant in tier I. |
| `oil-contact.png` | The new pixel art (`tools/art/industry_art.py`), 4x nearest: pumpjack, column, oil plant, crude, fuel oil, diesel, asphalt, and the asphalt, flare-band and lit-window textures. |

## Mining tiers, rig, platform and transport (2026-11-14; openspec/changes/industry-tiers)

Headless Chromium on SwiftShader (no GPU in the cloud session), 1280 × 800 at 2x. The look only, never the frame rate.
The page was checked across 15 tab, tier and view states with no page errors and one WebGL context; the second canvas
on the oil tab is the 2D overlay map.

| Shot | Shows |
| --- | --- |
| `tiers-mining-crew.png` | Tier 1: four hired townsfolk with picks and a barrow at the marked cells, 1 cell per 180 s each. |
| `tiers-mining-excavator.png` | Tier 2: the excavator, 1 cell per 40 s, on diesel. |
| `tiers-mining-quarry-run.png` | Tier 3, running: the quarry frame over the pit; its head takes every cell inside the frame (37 cells here), 1 per 8 s, 120 kW. |
| `tiers-mining-wheel-run.png` | Tier 4, running: the bucket-wheel excavator facing the bench, 40 cells a minute, 4,000 kW. |
| `tiers-mining-drones-run-night.png` | Tier 5 at night, running: the drone hive and its 12 drones; the hive's lamps and the drones' lights light the bench. |
| `tiers-oil-rig-run.png` | The drilling rig on the oil field, running: the lattice derrick with its travelling block, 300 kW, a well in 2 game days. |
| `tiers-oil-platform-run.png` | The offshore platform, running: 4 legs on the shelf sea, the turbine hall and stacks, the derrick over 6 risers, the flare, the helipad, the tanker at its hose, the sea cable to the shore substation. |
| `tiers-oil-platform-night.png` | The platform at 22:00: its lamps, windows and flare light the deck and water. |
| `tiers-power-platform.png` | The platform on the power ladder, tier I beside coal and oil: 3,100 kW ashore. |
| `tiers-transport-run.png` | Section 6, running: wires on poles and pylons, water and fuel pipes with their pump, the belt from a hopper, the pneumatic tube between stations, and the freighter at the quay with its hired crew. |
| `tiers-transport-auto-run-night.png` | The same at night with the freighter automated (a radio pilothouse, no crew). |

## Hex-built freighters and hex pods (2026-11-14; industry-tiers design 3)

The owner: "freighter should use hex based construction syustem, and cargo should be hex pods". Headless
Chromium on SwiftShader (no GPU), 1280 × 800 at 2x.

| Shot | Shows |
| --- | --- |
| `pods-freighter-hex.png` | The freighter at the quay, close: a hull of 73 whole hex cells (a stepped bow, the bridge built of cells aft), 70 goods pods (two high beside the middle row, one high at the rails) and a middle row of 11 tank pods, each pod a hex prism one cell across and 2 layers tall. |
| `pods-freighter-hex-night.png` | The same automated at 22:00: the bridge windows, the radio mast and the quay crane's floodlight. |
| `pods-tanker-hex.png` | The tanker at the offshore platform: the same hex hull, carrying 16 tank pods (160 t), the hose from the leg tanks. |
| `pods-icons.png` | The redrawn freighter and tanker icons and the new pod icon, 8x nearest (`tools/art/industry_art.py`). |
