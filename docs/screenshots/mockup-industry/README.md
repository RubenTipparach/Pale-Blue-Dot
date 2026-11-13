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
