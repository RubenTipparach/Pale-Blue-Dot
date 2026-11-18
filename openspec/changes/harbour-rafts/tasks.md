## 1. Rafts at the harbour

- [ ] 1.1 `harbour::boats_for` makes a raft at each `rowboat` or `sail` berth
      at least 5.1 m from the rafts before it, tagged `(site, RAFT_BERTH +
      n)`, anchored, on the tide. Test: a harbour moors its canoes and its
      rafts once each, no two rafts closer than 5.1 m, each afloat and
      anchored; asked again, it makes none.
- [ ] 1.2 Old worlds: a berth held by a retired record still gets its raft,
      and the held record is unchanged. Test.
- [ ] 1.3 "Take a boat" counts a harbour raft (a craft with a berth).

## 2. Close

- [ ] 2.1 Spec delta into `openspec/specs/player/vehicles/spec.md` with the
      commit that makes it true.
- [ ] 2.2 Screenshot of a harbour with its rafts in
      `docs/screenshots/retire-rowboat-and-cog/`.
- [ ] 2.3 fmt, clippy, tests, `openspec validate --all`.
