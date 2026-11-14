---
name: game-ui-frontend
description: "Design Pale Blue Dot's game UI (the quiet HUD - crosshair, hotbar slots, compass bar - and the menu screens: pack, pause and settings, map, guide, crafting and cooking stations, dialogue and trade, loading screen) with visual direction that serves play, in the stone-and-carved-wood pixel look. OpenAI's game-studio skill via Fleet Command; read the In Pale Blue Dot note first. Use when designing or reviewing a HUD element, a panel, a menu, an overlay, a phone-width layout, a UI mockup in docs/mockups, or the Bevy UI in crates/pbd-app/src/desktop (\"game ui\", \"hud\", \"menu\", \"inventory screen\", \"ui mockup\", \"stone and wood ui\")."
---

## In Pale Blue Dot

The owner, 2026-11-14: "pull in UI skills from my fleet-command repo, and develop ui for this game, do
something like with rock and carved wood". Copied from Fleet Command (`RubenTipparach/fleet-command`,
`.claude/skills/game-ui-frontend` with its `references/`, at `a4a02f6`, 2026-11-13), which took it from
OpenAI's `openai/plugins` (`plugins/game-studio/skills/game-ui-frontend` at `0722921`, MIT). Provenance in
`.claude/skills/PROVENANCE.md`. Below this note are Fleet Command's note and the source text, unchanged; where
they disagree with this note, this note wins.

- **The game is Bevy, not Godot or a browser.** The UI is Bevy UI (`Node`, `ImageNode`, `Text`) in
  `crates/pbd-app/src/desktop/`: `hud.rs` (crosshair and the one instrument line), `slots.rs` (the ten hotbar
  slots, 44 px squares), `pack.rs` (the pack grid), `menu.rs` (pause, settings, saves; it holds the palette
  `INK`, `MINT`, `PANEL_FILL`, `EDGE` and the idle/hovered/pressed fills), `map_screen.rs`, `compass_bar.rs`,
  `guide.rs`, `overlay_ui.rs`, `time_ui.rs`, `weather_ui.rs`, `frame_graph.rs`, and `crates/pbd-app/src/loading.rs`
  (the loading screen). "DOM for text-heavy UI" means Bevy UI nodes over the 3D camera here; in a mockup
  (`docs/mockups/*.html`, three.js) it means DOM, as written.
- **Fleet Command's console does not apply.** There is no full-height console column, no factions, no console
  skins. Pale Blue Dot is the opposite case, so the source's own playfield budget is the rule here, and stricter:
  the owner's quiet screen ("All text is excess on the UI", `openspec/changes/menus-and-a-quiet-screen`). In play
  only controls stay on screen: the crosshair, the hotbar, the compass bar (a navigation control) and transient
  prompts. Everything else is a menu screen that holds the pointer (`MenuOpen`) and closes on Escape.
- **The look is stone and carved wood** (owner, 2026-11-14), drawn as pixel art in the game's style
  (`docs/mockups/ART-DIRECTION.md`; art with the `balatro-pixel-art` kit, `tools/art/ui_art.py`). Panels are
  9-slice images drawn at whole-number scales and sampled nearest, like every terrain texture. The design is
  `openspec/changes/stone-and-wood-ui`; its mockup is `docs/mockups/ui.html`.
- **Phones**: the source's "collapse to chips" holds. A menu screen fills the phone; the hotbar shrinks to the
  slots a thumb can reach. Fleet Command's "panel goes full screen, 3D view to a side strip" does not apply.
- **Mockup rules** (CLAUDE.md): lit at night, nothing spins on its own, one 3D view at a time in tabs, every page
  on the mockup tracker, questions to the owner only with screenshots. Game code waits for the owner's go-ahead
  after the write-up ("Write it up before touching code").

---

*Fleet Command's note and the upstream text, verbatim, below. The original description was: "Design game UI surfaces (HUDs, panels, menus, overlays, responsive desktop and phone layouts) with visual direction that serves play. OpenAI's game-studio skill, read the In Fleet Command note first because the owner's dense console layout overrides its playfield budget. Use when designing or reviewing the console, a panel, a menu, an overlay, the HUD in the 3D view, or a UI mockup."*

## In Fleet Command

The owner, 2026-10-30, asking for "game ui", with this skill's link. Copied from OpenAI's
`openai/plugins` repository (`plugins/game-studio/skills/game-ui-frontend` at `0722921`, MIT), with
the three references it names copied into `references/` here (`frontend-prompts.md`,
`three-hud-layout-patterns.md`, `playtest-checklist.md`; read `../../references/` below as
`references/`). Provenance in `.claude/skills/PROVENANCE.md`. The text below is the source's.

- **CLAUDE.md section 7 wins.** The owner chose the reference game's console: a full-height panel
  column, a top strip and a bottom strip around the 3D view. This skill's "protect the playfield"
  budget (one HUD cluster, 20-25% coverage, no full-width chrome) does not apply to the console. It
  does apply to what is drawn **inside** the 3D view (the target bracket, its label, the weapons
  readout): keep the centre clear and the overlays small.
- **Keep**: visual direction first (genre, materials, type, palette, motion), theme tokens in one
  place, hierarchy (combat-critical first), readable over motion, desktop and phone from the start,
  input gated while a panel or menu is open, restrained motion with reduced-motion respected.
- **In Godot** the "DOM for text-heavy UI" advice means `Control` nodes and a `Theme` for the console,
  with the 3D view in a `SubViewportContainer`. In three.js mockups it means DOM, as written.
- On phones, CLAUDE.md section 7 replaces "collapse to chips": a panel expands to full screen and the
  live 3D view moves to a side strip.

---

*The source text, verbatim, below.*

# Game UI Frontend

## Overview

Use this skill whenever the game needs a visible interface layer. The job is not to produce generic dashboard UI. The job is to produce a readable, thematic browser-game interface that supports the play experience.

Default assumption: build the game world in canvas or WebGL, and build text-heavy UI in DOM.

## Frontend Standards

1. Establish visual direction before coding.
   - Genre and fantasy
   - Material language
   - Typography
   - Palette
   - Motion tone
2. Use CSS variables for the UI theme.
3. Build clear hierarchy.
   - Critical combat or survival information first
   - Secondary tools second
   - Rarely used settings behind menus or drawers
4. Protect the playfield first, especially in 3D.
   - The initial screen should feel playable within a few seconds.
   - Default to one primary persistent HUD cluster and at most one small secondary cluster.
   - Keep the center of the playfield clear during normal play.
   - Keep the lower-middle playfield mostly clear during normal play.
   - Put lore, field notes, quest details, and long control lists behind drawers, toggles, or pause surfaces.
   - Prefer contextual prompts and transient hints over permanent boxed panels.
5. Keep overlays readable over motion.
   - Use backing panels, edge treatment, contrast, and restrained blur where needed.
6. Design for both desktop and mobile from the start.
7. Design 3D UI around camera and input control boundaries.
   - Pause or gate camera-control input when menus, dialogs, or pointer-driven UI are active.
   - Keep pointer-lock, drag-to-look, and menu interaction states explicit.

## 3D Starter Defaults

For exploration, traversal, or third-person starter scaffolds, prefer this UI budget:

- one compact objective chip or status strip at the edge
- one transient controls hint or interaction prompt
- one optional collapsible secondary surface such as a journal, map, or quest log

Do not open every informational surface on first load. The scene should be readable before the user opens any deeper UI.

As a default implementation constraint for 3D browser games:

- no always-on full-width header plus multi-card body plus full-width footer layout
- no large center-screen or lower-middle overlays during normal movement
- no more than roughly 20-25% of the viewport covered by persistent HUD on desktop unless the user explicitly requests a denser layout
- on mobile, collapse to a narrow stack or contextual chips before covering the playfield with larger panels

## Prompting Rules

When asking the model to design or implement game UI, include:

- the game fantasy
- the camera or viewpoint
- the player verbs
- the HUD layers
- the camera or control mode when the game is 3D
- the tone of motion
- desktop and mobile expectations
- playfield protection and disclosure strategy
- explicit anti-patterns to avoid

Use `../../references/frontend-prompts.md` for concrete prompt shapes.

## Motion Rules

- Prefer a few meaningful transitions over constant micro-animation.
- Reserve strong motion for state change, reward, danger, and onboarding.
- Respect reduced-motion settings for non-essential animation.
- Keep 3D HUD motion from competing with camera motion.

## What Good Looks Like

- HUD elements are legible without flattening the scene.
- Menus feel native to the game world, not like a SaaS admin panel.
- Layout adapts cleanly across breakpoints.
- Pointer, keyboard, and game-state feedback are obvious.
- In 3D games, menu and HUD states do not fight camera control or pointer-lock.
- In 3D games, the first playable view keeps most of the viewport available for movement, aiming, and spatial reading.
- Persistent information density is low enough that screenshots still read as game scenes, not UI comps.

## Anti-Patterns

- Generic app dashboard layouts
- Flat placeholder styling with no theme
- Default font stacks without intent
- Dense overlays that obscure the playfield
- Large title cards or multi-paragraph notes sitting over a live playable scene
- Equal-weight boxed panels distributed around every edge of the viewport
- Controls, objectives, notes, and lore all expanded at once on first load
- Full-width top-and-bottom chrome with large always-on center or body panels in 3D play
- Excessive motion on every element
- Canvas-only UI when DOM would be clearer and cheaper
- Forcing HUD controls into the 3D scene when standard DOM would be clearer
- Letting camera input remain active under modals or inventory panels

## References

- Shared architecture: `../web-game-foundations/SKILL.md`
- Prompt recipes: `../../references/frontend-prompts.md`
- Low-chrome 3D layout patterns: `../../references/three-hud-layout-patterns.md`
- React-hosted 3D UI context: `../react-three-fiber-game/SKILL.md`
- Playtest review: `../../references/playtest-checklist.md`
