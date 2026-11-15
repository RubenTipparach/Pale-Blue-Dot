# Design: new-world-arrival

## Decisions

1. **One path for opening a world.** `load_world` stops rebuilding in place
   and hands the loading screen a *switch*: the slot to open, the root, and
   the direction to build round. `loading::begin_switch` puts `WorldLoading`
   back (so `Main` is gated again, as at launch: only `First`, `Loading`,
   `PostUpdate` and `Last` run, and every game system that needs the open
   world is held by `world_open`), spawns the screen and starts the loading
   thread, which opens the save (`WorldSave::open`, the log replay) and runs
   the same `prepare` a launch runs. `Startup` is not run again.

2. **What a switch does when the thread is done.** Instead of handing
   `PreparedPlanet` to `create_planet`, the prepared planet is installed in
   place (`planet::install_prepared`, the second half of `rebuild_planet`:
   a new base generation, a new fine-set version, the in-flight LOD task
   abandoned); the old towns' entities are taken down and the prepared
   `Towns` put in (`towns::replace`); `WorldSites::opened` and the new save
   are inserted (the old save's writer is dropped, which drains and joins
   it). A `loading::Arrived` marker is left for the desktop, and `Warming`
   starts as at launch. The desktop's `arrive` (in `PreUpdate`, on the first
   open frame) restores what the binary owns: hotbar, tool, drops, the clock,
   the air, the fishery, the map raster, and the walker (decision 3).

3. **Where the walker stands.** A world with a pose stands where it was. A
   world without one stands where a launch puts a new world: the launch's
   spawn direction (`NewWorldStart`, saved from the launch before anything
   is restored), the same level-ground search and four metres off the trunk
   (`walking::new_world_up`, the code `setup_walking` used inline, now shared),
   and `WalkingState::spawn_direction` becomes that, so the reset key also
   returns to this world's spawn. The thread builds the planet round the
   pose, or round the spawn direction for a new world.

4. **The leaving world is written first**: its pose and clock
   (`snapshot`), its weather, its craft (`put_away`), then `drain`, before the
   switch starts. The same writes the exit path makes.

5. **The clock.** A world with a saved clock resumes it; one without starts
   on the new-world clock (`sun_clock(launch, None)`): day 0 at the start
   hour, or the launch's `--time`/`--day`. Recommendation taken (ask only
   with screenshots): a brand-new world is new in time too.

6. **The banner.** A centred two-line label, the world's name and "A NEW
   WORLD" / "WELCOME BACK", shown when the screen lifts and faded out over
   four seconds. Whether the world is new is decided when it is opened (no
   pose in its world file), not when the banner shows, since an autosave may
   have written one by then. A memory-only world (a capture with no
   `--world`) shows no banner, and a capture shows none either, so existing
   shots do not change.

7. **The generator switch** happens on the main thread before the thread
   starts (`switch_generator` from the slot's identity), as `load_world` did,
   so the thread surveys and builds with the world's generator.

8. **A capture's switch** does not pause virtual time (a launch's does);
   `--new-world` and `--load` are instruments, and their shots are taken
   after `READY` like any other.

## Risks

- `Main` is gated while a world is open. Every game system in `PostUpdate`
  and `Last` that needs the world already runs `if world_open`; those that
  do not ran under the launch's gate before `Startup`, when even less
  existed.
- The towns' ground is a process global (`ground::install`), replaced by the
  thread while the old planet still stands. Nothing simulates while gated,
  and the in-flight LOD task is abandoned when the new planet lands.
