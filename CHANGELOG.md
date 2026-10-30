# Changelog

## 0.0.18

- README: a compiled `Prober` / `generate` / `write` / `check` example, including the stale case and how an untested capability is written (`enabled=no`, reason `no probe for this capability yet`). `docs/CI-CHECK.md` says where the example lives. Found by a fresh-eyes trial of a new consumer using only the docs.

## 0.0.17

- `SECURITY.md`: supported-versions policy (latest line gets fixes; an older line only when the backport is small; no period promised).
- `docs/KISS.md` checked against the KISS spec at `origin/main` d2374d0; `tests/kiss_vector.rs` runs the announce 7.3-0003 worked vector through `evaluate` with negative controls. Q-K3 widened (the target charset allows more than upper case).
- CHANGELOG: 0.0.15 marked as not published (released together with 0.0.16).

## 0.0.16

- `examples/capant_check` and `docs/CI-CHECK.md`: a consumer's `capant check` for CI (a runnable example, a committed matrix, a GitHub Actions workflow). Fails when the file drifts from the probes or a declared capability has no probe. Tested as a process in `tests/ci_check.rs` and by a `capant check` job in `ci.yml`.

## 0.0.15

(not published; released together with 0.0.16)

- README: third example, required vs preferred needs, `satisfying()` order and the rejection text for an unprobed `yes` (from the published-crate fresh-eyes trial). HANDOFF refreshed.

## 0.0.14

- `docs/PATENT-SWEEP.md`: patent sweep of the 0.0.x line (docs only).

## 0.0.13

- README: second example, a `yes` with no evidence and a stale `yes` are `unverified` (doctest, asserted).

## 0.0.12

- `easyants-card.toml` follows the hub's card format (no `slug`/`order`/`home`; `status = "not-ready"`), which had turned the hub build red.

## 0.0.11

- README: fixes from the first fresh-eyes trial (a passing, near-copy-paste run): where to paste the example, what `Context` and the surface mean, the rejection message shown and asserted, status no longer pins a version.

## 0.0.10

- `docs/KISS.md`: UNVERIFIED note on CapAnt as the matching engine over KISS capabilities (docs only).

## 0.0.9

- README opens with a ten-line hello-world, compiled and run as a doctest (EXPECTATIONS 6.24). Hub card `easyants-card.toml` added.

## 0.0.7

- `Report::carried`: committed rows marked inherited, listed for review and never failing `check` (a carried row is copied from the committed file, so a hand edit of it cannot be detected off the machine that probed it). Ruling: PM, 2026-10-24.

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
