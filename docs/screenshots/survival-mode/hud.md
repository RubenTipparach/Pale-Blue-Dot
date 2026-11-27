# Survival HUD in the game (`survival-mode` tasks 2.1, 4.1, 4.2, 4.6, 4.8, 4.9, 6.6)

Fixed-step `--capture` stills from a Claude Code cloud container with no GPU:
software rendering (lavapipe), so they show the look, never the frame rate.
Each state is set by the capture instrument `PBD_SURVIVAL_STATE=<state>`
(`desktop::survival_play::instrument`: the capture's world made a Survival
one, its bars held at the mockup's state), then:

```
PBD_SURVIVAL_STATE=<state> xvfb-run -a -s "-screen 0 1280x720x24" \
  target/fast/pbd-app --walk --frames 300 <args> --capture <png>
```

| File | State | Args | Shows |
| --- | --- | --- | --- |
| `hud-full-hidden-day.jpg` | `full` | `--time 12` | everything full: no survival bar on screen (G7) |
| `hud-thirsty-50-day.jpg` | `thirsty50` | `--time 12` | water at half: its bar appears; "Thirsty" above the hotbar |
| `hud-thirsty-25-night.jpg` | `thirsty25` | `--time 22` | water at a quarter, food 42, health 88, at night; "Very thirsty" |
| `hud-winded-day.jpg` | `winded` | `--time 12` | stamina empty, its strain a faint band, the bar flashing |
| `hud-heat-warning-day.jpg` | `hot` | `--time 13` | 39.7 C rising, the sun as its cause, water nearly dry, "Heat stroke" above the hotbar and the warm wash (G8) |
| `hud-cold-night.jpg` | `cold` | `--time 3` | 34.8 C: the thermometer outside fine, at night |
| `hud-under-water-breath.jpg` | `under` | `--at 3.5 -18.15 --yaw -150 --pitch 10 --time 11` | breath as bubbles over the hotbar, under the sea |
| `hud-salt-water-day.jpg` | `salt` | `--at 3.5 -18.15 --yaw -150 --pitch -30 --time 11` | "Salt water" at the sea |
| `hud-dead-pack-dropped-day.jpg` | `dead` | `--time 12 --survival-kit --pitch -70` | "You died" and Wake up; the pack (the kit) dropped as one bag at the feet, the hotbar empty |
| `hud-worn-pickaxe-day.jpg` | `worn` | `--time 12 --tool pickaxe` | the tool slot's wear bar at a quarter |
| `hud-broken-axe-day.jpg` | `worn` | `--time 12 --tool axe` | a broken axe: its icon greyed (it digs as bare hands) |
| `menu-new-world-mode.jpg` | `full` | `--menu saves` | the new-world screen's Explorer and Survival plaques, and each world's mode in the list |
