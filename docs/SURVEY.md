# CapAnt R0 survey

What PersistAnt, AntSpawn and ConfigurAnt each implement of the capability pattern, what is identical, what differs, and what each cannot express. Input to R1 (`docs/DESIGN.md`).

**Refs read** (`git show origin/main:<path>`, never a working tree; fetched 2026-10-23):

| Project | `origin/main` |
|---|---|
| PersistAnt | `c6872b351fb93fbb88c0e6333cf2fd05b35516ea` |
| AntSpawn | `00b2aada5ef33d51b56e8d9c314b74386398ac02` |
| ConfigurAnt | `a7b5a0ab3fcd77eb5689c195509dc8508edd5f54` |

Files read: PersistAnt `crates/persistant-core/src/{matrix,provider,row}.rs`, `src/registry.rs` (module docs and signatures; the `Handle`/`AnyKv` bodies were not read), `matrix/README.md`, `matrix/sqlx-sqlite.tsv` (header and first rows). AntSpawn `src/{matrix,probes,launcher}.rs`, `examples/matrix.rs`, `matrix/launchers.tsv`. ConfigurAnt `src/module.rs`, `docs/DESIGN.md` (lines naming Needs/modules), `grep` of `src/persistant.rs` and `src/edit.rs` for how modules are called. Not read: PersistAnt's `choose.rs` (speed ranking), `bench.rs`, `config.rs`; the CI jobs that run the `--check`-style regeneration. Statements about those are marked below as unread.

## 1. The three implementations

### PersistAnt (`persistant-core` + `persistant::registry`)
- **Matrix**: nine-column TSV `subject_kind subject backend version config enabled reason metadata note`; columns matched **by name**, unknown columns kept in `Row::extra`, `#` comment and blank lines skipped, files can be merged (`Matrix::merge`, nothing deduplicated). `enabled` is `yes`/`no`; `reason` is `missing|wrong|untested` and **must be present exactly when `enabled=no`** (`ReasonMismatch`). `config` and `metadata` are `k=v;k=v` maps. Metadata in the checked-in rows carries `arch`, `os`, `test=<test id>` and probe parameters (`sleep_ms`, `ttl_ms`).
- **Lookup** (`Matrix::lookup`): returns `On | Off(Reason) | NoRow`. A row matches on exact `subject`, `backend`, `version`, and on `config` when every fact the row names is present in the caller's actual config (`holds_in`). **Off wins**: one matching `no` beats any `yes`. No row is off.
- **Provider table** (`provider::providers`): data the caller passes in. A `Provider` has a name, one or more **`Surface`s** (`backend`, `version`, `config`; one per layer, e.g. `kv-on-relational:postgres` plus `postgres`), declared durability (`Option<Declared>`, `None` = unknown) and an `EndpointSpec`. A need is satisfied when each id is `On` on at least one surface and `Off` on none.
- **Needs**: `ProviderNeeds { ids (all must be on), min_durability, preferred (ordered, never excludes) }`.
- **Result**: `Ok(qualified providers, table order)` or `Err(Unmet)`, which lists **every** provider with each missing id and the matrix's own word (`Off(reason)` / `no-row`) plus any durability shortfall. **No closest match, no partial list.** `rank` then orders qualifiers by a measured speed table (`choose`, unread).
- **Registry** (`registry::providers`, `open_named`): the compiled-in table; `Listing::why()` gives per-listing reasons (preferred capabilities met and lacked in order of importance, speed, durability). The operator's pick is stored as a stable string id; `open_named` **re-checks the matrix for the backend it actually opened** and refuses if the row is gone. Ids are append-only, pinned by a test. No dynamic plug-ins: a new backend needs a rebuild.
- **Generation**: `cargo run -p persistant-{opendal,sqlx} --example matrix` writes the files; `matrix/README.md` says CI regenerates on Linux and fails if they differ (the CI job itself was not read).

### AntSpawn (`antspawn::matrix`, `launcher::Registry`, `examples/matrix.rs`, `probes.rs`)
- **Matrix**: the same nine columns and header, but a **fixed** header (any other header is `BadHeader`), strict 9 fields, `enabled` `yes|no`, duplicates on `(kind, subject, backend)` rejected, capability ids checked against a **closed enum** (`UnknownCapability`). `reason` is free text (`UNVERIFIED: not set up`, `no probe for this capability yet`, `job read a host file outside its environment`); `metadata` is `probe=<name>`. Rows of other kinds (`measurement`) are kept but grant nothing. `version` is the constant `1`; `config` is empty everywhere.
- **Gate** (`Matrix::enabled`): on only if a `capability` row for `(launcher, cap)` says `yes`; no row or `no` is off.
- **Provider selection** (`Registry::providers`): for each launcher, `unmet` = needed capabilities not enabled. All launchers are returned, **satisfying first**, then the spec's `prefer` list position, then registry order (stable sort). Unlike PersistAnt this is a ranking with the unmet set attached, not an error. `open_named` re-checks and returns `OpenError::Unmet { id, unmet }` or `Unknown`.
- **Evidence and probes**: `probes.rs` holds negative controls (`violates_read/write/clean_slate/kill_runaway/network`); the `naive` launcher must violate all of them, which proves they can fail. The network probe returns `None` (no verdict, row stays `no` with `UNVERIFIED`) when the **host itself cannot connect**: a positive control for the control.
- **`--check`** (`examples/matrix.rs`): regenerates every row, asserts it parses and is canonical (`parse(to_tsv(x)) == x`), compares to the committed file (CRLF normalised) and exits 1 on a difference; without `--check` it writes. Launchers this machine cannot probe (`account` off Windows-with-setup, `windows-sandbox` always) are **carried over** from the committed file, but only if the rows are complete (one per capability) and every `yes` names its probe; otherwise they are rebuilt as `UNVERIFIED`. A capability with no probe is `no` with `no probe for this capability yet`.

### ConfigurAnt (`module.rs`, `persistant.rs`, `edit.rs`)
- **No matrix and no evidence.** `Module` is a plug-in trait: `list_choices(&Needs) -> Vec<Choice>` (best first, each `Choice { id, reason }`), `validate`, `on_change -> Plan` (data only; the front end executes after confirmation). `Modules` registers by name with an interface-version check (`INTERFACE_VERSION = 1`) and refuses duplicates.
- **Needs** are `{ required: Vec<String>, order: Vec<String> }`: opaque strings whose meaning each module defines. The only real module read, `persistant.rs`, parses them as PersistAnt `CapabilityId`s into `ProviderNeeds` (minimum durability is a module setting because `Needs` has no field for it, `docs/DESIGN.md`) and delegates to PersistAnt's registry. So ConfigurAnt **consumes** the pattern and owns no state of it.
- **Failure shape**: `list_choices` cannot return an error (an empty list stands in; `try_list_choices` carries the reason). `ChoiceList` is the trivial module over a fixed list: it ignores needs entirely.

## 2. What is identical

1. The question: the developer declares needs plus an order of importance, the project returns the providers that satisfy them (CIRESNAVE-EXPECTATIONS 6.17).
2. **Off by default**: no row is off (PersistAnt `NoRow`, AntSpawn "no row grants nothing").
3. The nine-column TSV and its header (AntSpawn states it copied PersistAnt's so one reader serves both).
4. A stable string provider id that the application stores, and an `open_named` that **re-checks the gate at open time** and refuses with the unmet needs.
5. Ordered preference only orders; it never excludes (PersistAnt `preferred`, AntSpawn `prefer`).
6. Strict parsing: a malformed row is an error, never a skipped row, because a skipped row silently turns a capability off.
7. Canonical, diffable output with a round-trip test; matrix files are generated from probes and committed.

## 3. What differs

| Aspect | PersistAnt | AntSpawn | ConfigurAnt |
|---|---|---|---|
| Capability id | validated `CapabilityId` string, open set | closed `enum Capability` (parse rejects unknowns) | opaque `String`, module-defined |
| Header | by name, extra columns kept | exact string only | n/a |
| Reason | enum `missing/wrong/untested`, required iff `no` | free text | `Choice::reason` free text (output only) |
| Metadata | `k=v` map: arch, os, test id | `probe=<name>` | n/a |
| `version`/`config` | real (`0.9+3.51.3`, `dialect=sqlite`), matched with `holds_in` | constant `1` / empty | n/a |
| Conflicting rows | Off wins | duplicates rejected at parse | n/a |
| Several layers per provider | `Surface`s | none | none |
| No satisfier | `Err(Unmet)` listing every provider and what it lacks, no partial list | `Vec<Provider>` of all, satisfying first, `unmet` attached | whatever the module returns (possibly empty) |
| Order of importance | `preferred`, then measured speed (`rank`) | `prefer` ids, then registry order | `Needs::order`, meaning is the module's |
| Extra constraint besides capabilities | min durability | none | none (module setting) |
| Why a provider was chosen | `Listing::why()` text | none (only `unmet`) | `Choice::reason` text |
| Provider set | compiled-in table | `Vec<Box<dyn Launcher>>` | registered modules |
| Check mode | CI regenerates and diffs (CI job unread) | `matrix --check`, with carry-over of unmeasurable rows | none |

## 4. What each cannot express

**PersistAnt**
- **Staleness**: the `version` column makes a new driver version a miss, but there is no timestamp column and no age window; a row from any date stays valid for its version string. (Metadata could hold a date; nothing reads it. Read from `row.rs`/`matrix.rs`; `bench.rs` unread.)
- **UNVERIFIED as a state**: `no` + `reason=untested` and "no row" are two different spellings of "unknown", and `Lookup::NoRow` vs `Off(Untested)` are distinct values that callers must both treat as off. Nothing distinguishes "tried and could not conclude" from "never tried".
- **Inherited rows**: a row cannot say "not probed here, inherited from another environment"; `arch`/`os` in metadata are informational and are not matched.
- **Why-chosen per need**: `why()` explains a listing in prose, not as data per need and per rejected provider; `Unmet` carries rejection data only when **no** provider qualifies.
- A capability that is satisfied partially or with a degree (the row is boolean).

**AntSpawn**
- **Version and config**: both are hard-coded constants, so it cannot say "isolated on Windows 11 build X only".
- **Evidence beyond a probe name**: no tool version, environment or time; "UNVERIFIED" is a free-text prefix inside `reason`, not a state, and the carry-over soundness check is a string test on `metadata` (`starts_with("probe=")`).
- **Open capability set**: adding a capability is a code change to the enum and the generator's match.
- **Rejection reasons**: `unmet` is a list of ids; the matrix reason for each is not returned.
- **Conflict** between rows: duplicates are refused rather than resolved, so one launcher cannot carry a `no` for a configuration and a `yes` for another.
- Probes and `--check` are written per project; the carry-over rule exists only in this example.

**ConfigurAnt**
- Everything: it holds no capabilities, evidence or matrix, and `Needs` is untyped strings, so a typo in a need is a module-time failure, not a type error. It cannot express a minimum durability (or any non-capability constraint) in `Needs`. `list_choices` cannot report an error. Choosing is delegated and unexplained beyond a string.

## 5. Findings that bear on the design

1. **PersistAnt's matrix is the superset** of AntSpawn's: AntSpawn is the same file with `version=1`, `config` empty, a closed enum, and free-text reasons. A single reader (by-name columns, extra columns kept) can read both; the migration for AntSpawn is mostly its reason and capability-id typing.
2. **The three disagree on "no satisfier"** (error with details / sorted list with `unmet` / module-defined). CapAnt must pick one result type that can carry both (a ranked list where each entry says met/unmet **per need with the row's reason**; "no provider fully satisfies" is then a property of the list, not a separate error). This is a design decision for R1, not settled here.
3. **UNVERIFIED, evidence and staleness do not exist in any of them as data.** Rules 1 and 2 of `docs/SCOPE.md` are new work, not extraction.
4. **Both reproducible-matrix mechanisms share the same shape** (probe, generate, round-trip, diff against committed, fail on difference) but only AntSpawn has the explicit carry-over of unmeasurable rows. That carry-over is the closest existing form of SCOPE rule 5 (inherited), and it is a source of silent staleness: a carried-over `yes` is never re-measured until someone runs it on the right machine.
5. **ConfigurAnt's `Needs` is stringly typed**; CapAnt's `Need` type is what its `persistant.rs` module would parse into.

## 6. Not established (open for R1)

- How PersistAnt's `choose`/`bench` speed ranking would sit on top of CapAnt's order (unread; ranking by measurement may stay in PersistAnt).
- Whether PersistAnt's CI regeneration job and AntSpawn's `--check` compare identical things (the PersistAnt workflow was not read).
- Whether `persistant.rs` in ConfigurAnt is the only module that calls into a registry.
