# Changelog

## 0.0.6

- R3: matrix file reader and canonical writer (`parse`, `write`), RFC 3339 `time`, `Prober`, `generate` with carry-over, `check` (ignores only `probed_at`). `Row` gains `metadata` and `extra`. An inherited row whose origin is the current environment counts without consent. Fixtures from AntSpawn and PersistAnt.

## 0.0.5

- R2: core crate: ids, config facts, states with evidence, in-memory matrix, `resolve`, `evaluate`, `Evaluation` (per-consumer projections), tests with a negative control per rule. DESIGN section 3 simplified (no re-ranking of rejected providers) and Q1-Q5 decided.

## 0.0.4

- LICENSE-MIT holder is "The capant contributors" (CireSnave, decision 263, 2026-10-24: "Go with your recommendations.").

## 0.0.3

- R1 design: `docs/DESIGN.md` (data model, one `Evaluation` result type, matrix format, staleness, `--check`, adapters, migration, test plan). No code.

## 0.0.2

- R0 survey: `docs/SURVEY.md` (PersistAnt registry, AntSpawn matrix, ConfigurAnt modules, read at origin/main). No code.

## 0.0.1

- Repository seeded by the PM: CI, deny policy, licences, roadmap and first handoff. No code yet.
