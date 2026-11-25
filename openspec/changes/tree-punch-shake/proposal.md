# Proposal: punch a tree for logs, and trees that shake when hit

## Why

The owner, 2026-11-18: "Need tree shaking animation when chopping or
punching trees. Please allow punching trees to get wood logs for the first
axe. Just takes longer".

## What Changes

- Bare hands work a tree for logs as an axe does, up to the same logs a tree
  gives, one log at a time at the hands' slower speed. Today hands take one
  branch and nothing more.
- A tree being chopped or punched shakes: each hit sets its crown and trunk
  swaying, a short damped sway from the base up, and it settles when the
  hits stop.

## Impact

- `pbd_core::gather::tree_gives`: hands take logs to `axe_logs`; the
  `BranchTaken` refusal goes. Old saves' `tree` records (logs taken) read as
  before; a tree a hand took a branch from has three more to give.
- `desktop/digging.rs`: the log note and the fell mission unchanged (an axe's
  last log fells and counts; a hand's last log fells the tree too).
- The shake: a per-tree uniform (the worked tree and when it was last hit)
  read by the near tree's vertex shader. Drawn only; no rule reads it.
