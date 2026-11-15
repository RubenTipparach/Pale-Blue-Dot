# Starter missions: design

## Decisions

Recommendations taken (ask only with screenshots) unless quoted.

1. **Data, not code.** Each mission is
   `{ id, title, help, needs: [ids], objectives: [{ kind, what, count }] }`.
   - Kinds: `craft`, `place`, `gather`, `catch`, `cook`, `plant`,
     `harvest`, `tame`, `pen`, `collect`, `smelt`, `fell`, `cast`.
   - Ids are append-only short codes, so a saved record keeps its meaning.
2. **Counted from events the game already raises.** The systems that do the
   thing send an event, and the missions system counts it:
   - the recipe completes;
   - `apply_edit` places a block;
   - the fishing catch is landed;
   - the harvest happens.

   No mission reads the screen, and no mission gives an item. Rewards are the
   next mission. Coins come with the townsfolk phase.
3. **Only the current missions count.**
   - An open mission counts events from the moment it opens.
   - Events from before the mission opened are not counted, except that
     state-shaped objectives can already be met when it opens: `craft` of a
     tool owned, or `place` of a station already in the world.
4. **A new world starts the chain. An old world joins it**, with every
   state-met mission marked done on open.
5. **One line on the HUD** shows the current mission and its count ("Catch
   fish 1/3"). The pack's Missions tab lists the chain: done, current, and
   locked missions with their titles.
6. **Skippable.** A setting hides the HUD line. The chain still records
   progress.

## Risks

- **The chain can name something that is not built yet.** Each mission is
  added to the data only once its system is in the game, and a data test
  checks every `what` against the goods, recipes and species tables.
