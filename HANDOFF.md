# HANDOFF (capant lane)

Current state: R0 survey written (`docs/SURVEY.md`, PR from branch `r0-survey`, version 0.0.2); waiting on the PM's gate. No code.

## Next task
R1 (design): `docs/DESIGN.md` per ROADMAP. Start it in a worktree `C:\Projects\capant-wt-design` once R0 is merged (or stacked on `r0-survey`). Settle first the open points in SURVEY section 5 (one result type for "no satisfier") and 6. No library code before the design is merged.

## Rules to remember
- Messages to the PM are short, compact format (`C:\Projects\MESSAGE-FORMATS.md`).
- The check named "Portfolio checks" is ADVISORY (do not silence rules; fix or put a row on the roadmap).
- Primary checkout stays on `main`; branches live in worktrees. Remove the worktree after the merge.
