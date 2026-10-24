# HANDOFF (capant lane)

Current state: R0, R1 merged. R2 (core crate, in-memory engine, 0.0.5) is PR `r2-core`; waiting on the gate.

## Next task
R3: matrix file reader/writer and `--check` per `docs/DESIGN.md` section 5-6 (nine-column TSV by column name, optional `probed_at tool_version environment inherited_from`, canonical writer, `Prober` trait, `generate` with carry-over, `--check` ignoring only `probed_at`). Add fixtures copied from AntSpawn `matrix/launchers.tsv` and PersistAnt `matrix/sqlx-sqlite.tsv` with their SHAs and licence note in `tests/fixtures/README.md`. Needs RFC 3339 timestamp parsing (hand-written, tested). Worktree `C:\Projects\capant-wt-matrix`.

## Rules to remember
- Messages to the PM are short, compact format (`C:\Projects\MESSAGE-FORMATS.md`).
- "Portfolio checks" is ADVISORY (fix, or put a row on the roadmap).
- Primary checkout stays on `main`; branches live in worktrees (`capant-wt-<topic>`); remove after merge.
