# HANDOFF (capant lane)

Current state: R0-R3 merged, capant 0.0.6 published, AntSpawn migrated (antspawn#24). 0.0.7 (`Report::carried`) is PR `r0-carried`; waiting on the gate, then the PM publishes.

## Next task
R4 continues in other lanes: PersistAnt step 1 (parse/write via capant, no behaviour change; TASK relayed by the PM to the persistant lane, 0.9.40 proposed), then ConfigurAnt after a PersistAnt release. This lane's part: answer API questions, and when PersistAnt step 1 merges, draft the step-2 [TASK] (selection via `capant::evaluate`, Unmet wording `unverified(<why>)`, ranked order, durability as capabilities). Otherwise idle; ROADMAP has no row after R4.

## Rules to remember
- Messages to the PM are short, compact format (`C:\Projects\MESSAGE-FORMATS.md`).
- "Portfolio checks" is ADVISORY (fix, or put a row on the roadmap).
- Primary checkout stays on `main`; branches live in worktrees (`capant-wt-<topic>`); remove after merge.
