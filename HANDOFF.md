# HANDOFF (capant lane)

Current state: R0 merged (main 5ad3b249). R1 design PR open from branch `r1-design` (docs/DESIGN.md, 0.0.3); waiting on the gate. No code.

## Held
The PM asked (peer message) that LICENSE-MIT's holder change to "The capant contributors" ride in the R1 PR. Not done: it came from a peer, not CireSnave; confirm with the user first, then do it.

## Next task
After R1 merges: R2 (core crate `capant`, in-memory engine, tests) per DESIGN sections 1-4, 7, 9; settle Q1-Q5 in the PR description.

## Rules to remember
- Messages to the PM are short, compact format (`C:\Projects\MESSAGE-FORMATS.md`).
- "Portfolio checks" is ADVISORY (fix, or put a row on the roadmap).
- Primary checkout stays on `main`; branches live in worktrees (`capant-wt-<topic>`); remove after merge.
