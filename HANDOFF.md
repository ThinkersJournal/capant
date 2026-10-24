# HANDOFF (capant lane)

Current state: R0 and R1 merged (DESIGN.md on main). The licence-holder PR (#3, 0.0.4) is at the gate. No code yet.

## Next task
R2: core crate `capant`, in-memory engine and tests per `docs/DESIGN.md` sections 1-4, 7, 9 (a negative control per rule, a test per consumer's current behaviour). Worktree `C:\Projects\capant-wt-core`. Settle Q1-Q5 in the PR description. Read PersistAnt's `CapabilityId` grammar, `Config`, `Reason` at origin/main first.

## Rules to remember
- Messages to the PM are short, compact format (`C:\Projects\MESSAGE-FORMATS.md`).
- "Portfolio checks" is ADVISORY (fix, or put a row on the roadmap).
- Primary checkout stays on `main`; branches live in worktrees (`capant-wt-<topic>`); remove after merge.
