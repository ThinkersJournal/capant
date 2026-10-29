# HANDOFF (capant lane)

State 2026-10-29: main=5b616b7 (0.0.15, README only; not yet published). R0-R4 done: PersistAnt (persistant#170, 0.12.0) and AntSpawn migrated; ConfigurAnt chose "no change" (strings stay; it reaches capant through PersistAnt). crates.io has capant 0.0.14 (tag v0.0.14), published by the PM.

## Task in hand (PM [TASK] 2026-10-29)
R5: a `capant check` CI example for consumers, one PR, 0.0.16. Branch `check-example`, worktree `C:\Projects\capant-wt-check`. Built: `examples/capant_check/` (main.rs, matrix.tsv, capant-check.yml), `docs/CI-CHECK.md`, `tests/ci_check.rs` (runs the example as a process; negative control run: disabling the unprobed loop fails exactly `a_declared_capability_nobody_probes_fails` and `a_file_that_records_the_hole_still_fails`), a `capant-check` job in `ci.yml` (not a required check; the PM adds it to protection if wanted). Remaining: fresh-eyes trial result into the PR, merge, then the PM publishes 0.0.15 and 0.0.16 together and tags. Remove the worktree after merge.

## Rules to remember
- Messages to the PM are short, compact format (`C:\Projects\MESSAGE-FORMATS.md`).
- Primary checkout stays on `main`; branches live in worktrees (`capant-wt-<topic>`); remove after merge.
- CHANGELOG.md, Cargo.*, README.md, ci.yml are CRLF: scripted edits must keep the file's newline; read the result back.
- `cargo test --test ci_check` alone does not rebuild the example; the test refuses a stale binary. Use `cargo test`.
- Untracked `capant-seq.sh` in the primary checkout is not mine; ask before touching.
- Open thread: PersistAnt lane asked for an `Evaluation::explain`-based fn; answered A (additive; exists as `Evaluation::explain(id)`).
