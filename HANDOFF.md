# HANDOFF (capant lane)

Current state: R0-R2 merged. R3 (matrix file, generate, check, 0.0.6) is PR `r3-matrix`; waiting on the gate.

## Next task
R4: migrate AntSpawn first (its matrix is the most developed), then PersistAnt, then ConfigurAnt, each a PR in the consumer's repo (crates.io dependency on `capant`, never a path or git; so `capant` must be published first: ask the PM). Read `docs/DESIGN.md` section 7-8. AntSpawn's file already parses and rewrites byte-for-byte (tests/file.rs). Do not start a consumer PR until the PM confirms publishing and the lane for that consumer.

## Rules to remember
- Messages to the PM are short, compact format (`C:\Projects\MESSAGE-FORMATS.md`).
- "Portfolio checks" is ADVISORY (fix, or put a row on the roadmap).
- Primary checkout stays on `main`; branches live in worktrees (`capant-wt-<topic>`); remove after merge.
