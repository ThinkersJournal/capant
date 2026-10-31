# Adoption in PersistAnt, AntSpawn and ConfigurAnt (R6)

Docs only. Which of each consumer's own capability code CapAnt could still replace, in what order, and what blocks it. Nothing here edits another repo; each step would be that repo's own PR, and its lane decides.

**Refs read** (`git show origin/main:<path>` after `git fetch`, 2026-10-31): PersistAnt `c4ed00e`, AntSpawn `ecb6f76`, ConfigurAnt `4220fa1`. CapAnt `7639c53` (0.1.1; crates.io latest is 0.1.0). Not read: PersistAnt's `registry`/`Handle` bodies and its CI jobs; ConfigurAnt's `edit.rs`.

## 1. Where each stands

| Consumer | CapAnt dep | Uses it for | Still its own |
|---|---|---|---|
| AntSpawn | `capant = "0.1.0"` | `launcher.rs` `capant::evaluate`/`check` (selection and `open_named`); `matrix.rs` re-exports `capant::{Matrix, Row}`; `examples/matrix.rs` `capant::generate`/`write`/`check` | `Capability` enum (`capability.rs`, by design, DESIGN D6); the probes (`probes.rs`); a strictness layer in `matrix.rs` (unknown capability or duplicate row refused) |
| PersistAnt | `capant = "0.1.0"` (workspace and `persistant-core`) | `provider.rs:340` `capant::evaluate` (selection); `adapter.rs` `capant::parse`/`write` (file reading and writing) | Its own `Row`, `Reason`, `Config`, `CapabilityId`, `Matrix` and `Lookup` types, mapped to CapAnt's by `Matrix::to_capant` and `adapter.rs`; `Matrix::lookup`, still the gate for the conformance suites and the checked-in-matrix tests; `choose.rs` (speed ranking by measurement) |
| ConfigurAnt | none (CapAnt appears only in `Cargo.lock`, through `persistant`) | nothing directly | `Needs { required: Vec<String>, order: Vec<String> }` (`module.rs`); `persistant.rs` parses the strings to PersistAnt's `CapabilityId` and `ProviderNeeds` and calls PersistAnt's registry |

By use, not by mention: AntSpawn and PersistAnt call `evaluate`; ConfigurAnt calls nothing of CapAnt's.

## 2. What CapAnt could still replace

**AntSpawn: nothing required.** The enum stays (DESIGN D6: the engine never holds a closed enum, the consumer holds a constant list). The strictness layer is deliberate: DESIGN section 5 makes duplicate rows a warning that `--check` fails on, and AntSpawn refuses them on read. Leave it.

**PersistAnt: the duplicate types and `Lookup`.** Three separate things, with different cost:

| Step | What changes | Cost and blockers |
|---|---|---|
| P-a | `Matrix::lookup` answers through `capant::resolve` instead of its own scan. The types stay. | Small. `tests/capant_view.rs` already compares the two for every row of the five checked-in matrices, so the equivalence is tested. The `Lookup` result (`On`/`Off(Reason)`/`NoRow`) stays; `Unverified(no row)` maps to `NoRow`. Blocker: none known. |
| P-b | `Reason`, `Config`, `CapabilityId` become aliases of, or wrappers over, CapAnt's. | Breaking for PersistAnt's public API (they are re-exported from `persistant::capability`). Reason differs in kind: PersistAnt has `missing`/`wrong`/`untested` and refuses any other `no` text; CapAnt's `Reason` has `Other(text)` and the `Unverified` family. The id grammar differs too: CapAnt adds `-`, which PersistAnt refuses (`provider.rs:281` relies on PersistAnt's grammar being inside CapAnt's, one way only). The probe key is `test` here and `probe` in CapAnt (`adapter.rs` maps it). Blocker: a PersistAnt decision to take a breaking change. |
| P-c | `Row` and `Matrix` become CapAnt's. | Everything in P-b plus 80 lines naming `.lookup(` or `Lookup::` in 11 files (`matrix.rs` 15, `provider.rs` 13, `persistant-kv/src/conformance.rs` 3, and the tests: `checked_in_matrix_*` 36, `capant_view.rs` 7, `matrix_reader_equivalence.rs` 4, `reexports.rs` 2; `git grep -c` at `c4ed00e`, a line count, not a call count). Blocker: as P-b, and the conformance suites would then depend on CapAnt's types. |

`choose.rs` and `bench` stay in PersistAnt: they rank by measurement, which CapAnt does not model (DESIGN section 8.2).

**ConfigurAnt: typed `Needs`, or nothing.** DESIGN section 8.3 already says no CapAnt dependency is forced. `Needs` is opaque strings by design, and each module defines their meaning; only `persistant.rs` gives them one. A CapAnt dependency would change `Needs` and the `Module` trait, so it is an API change for every module author. A dependency that nothing calls would be dead weight. The PM closed this as "leave to ConfigurAnt's owner" (2026-10-30).

## 3. Order

1. AntSpawn: done (PRs #24, #26, #31, #33 in its repo).
2. PersistAnt P-a: the only step with a tested equivalence and no API change.
3. PersistAnt P-b/P-c: only if PersistAnt's owner wants one set of types.
4. ConfigurAnt: only if its owner wants typed needs.

## 4. Options

| Option | For | Against |
|---|---|---|
| A. Stop at the current state | Selection and file I/O are already CapAnt's in both consumers; no breaking change | PersistAnt keeps a parallel `Lookup` and a conversion layer that must be kept equivalent (`capant_view.rs` guards it) |
| B. A, plus P-a | Removes the second scan; one place decides what a row means | A small PR in PersistAnt; `Lookup` stays as a wrapper |
| C. B, plus P-b/P-c | One set of types | Breaking for PersistAnt's public API, a reason-vocabulary decision, and a larger change than the duplication costs |

**Recommendation: B.** It removes the one real duplication (two implementations of "does this row hold"), has its equivalence test already, and breaks nothing. C waits for PersistAnt's owner to ask for it. ConfigurAnt stays as it is.

## 5. What is blocked

- P-b/P-c: a PersistAnt decision on a breaking release; the reason-vocabulary question (Q3 in `DESIGN.md` decided `NoRow` to `Unverified` wording for the unmet text, not the `Reason` type).
- ConfigurAnt typed `Needs`: its owner, and its #64 ports slice, which is in the gate.
- Nothing is blocked on CapAnt: no gap in 0.1.0 was found that these steps need.
