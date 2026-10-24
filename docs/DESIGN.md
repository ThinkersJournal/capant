# CapAnt design (R1)

Builds on `docs/SURVEY.md` (refs: PersistAnt `c6872b35`, AntSpawn `00b2aada`, ConfigurAnt `a7b5a0ab` at `origin/main`) and the six rules of `docs/SCOPE.md`. Nothing here is implemented; R2 implements section 1-4, R3 section 5-6, R4 section 8.

## 0. Decisions

| # | Decision | Why |
|---|---|---|
| D1 | Capability states are `Yes`, `No(reason)`, `Unverified(why)`; **`Unverified` is the default** for no row, a `yes` without evidence, a stale row, and a row from another environment that is not marked inherited. | SCOPE 1, 2, 5. The three consumers have two spellings of "unknown" (PersistAnt `NoRow`/`untested`, AntSpawn free-text `UNVERIFIED:`). |
| D2 | **One result type**, `Evaluation`: every provider, ranked, each with a per-need outcome. "Nothing satisfies" is `Evaluation::satisfying().is_empty()`, not an error. Adapters give each consumer its present shape (section 7). | The three consumers disagree (SURVEY section 3); a list that carries per-need data can be projected to all three, an error cannot. |
| D3 | Needs are `required` (all must be `Yes`) and `preferred` (ordered, most important first, never excludes). Non-capability constraints (PersistAnt's minimum durability) are **capabilities** (`durability.process`), so the engine stays ignorant of them. | SCOPE 6; ConfigurAnt's `Needs` has no durability field. |
| D4 | The engine has **no clock and no I/O**: "now", the environment and tool versions arrive in a `Context`. | Deterministic tests; staleness is testable. |
| D5 | File format is the existing nine-column TSV, read by column name, plus optional evidence columns. Files of the old nine columns still parse. | PersistAnt and AntSpawn files must keep working unchanged through R4. |
| D6 | Capability ids are an open, validated string set; the engine never holds a closed enum. | PersistAnt's open set, AntSpawn's enum becomes a consumer-side constant list. |

## 1. Data model (crate `capant`)

```text
CapabilityId   validated string: a-z 0-9 _ . : - (PersistAnt's grammar plus `-`, which AntSpawn's ids use); no leading, trailing or doubled `.`
ProviderId     string, stable and append-only by the consumer's rule (CapAnt only compares)
Config         ordered k=v map, as PersistAnt's `config`/`metadata`

State          Yes | No(Reason) | Unverified(Why)
Reason         Missing | Wrong | Other(text)            // PersistAnt's three-word set plus free text for AntSpawn's reasons
Why            NoRow | NoEvidence | Stale | OtherEnvironment | NoControl | NotSetUp | Untested | Other(text)

Evidence       probe: String            // probe id, e.g. `read`, `kv::kv.get`
               tool_version: Option<String>
               environment: Config      // os, arch, ...
               probed_at: Option<Timestamp>   // seconds since epoch, UTC
               inherited_from: Option<Config> // set only on carried-over rows (D7 below)

Row            subject_kind, subject (CapabilityId), backend, version, config, state, evidence, note, extra columns
Surface        backend, version, config            // PersistAnt's: where the matrix is asked
Provider       id, surfaces (non-empty, first = primary), description text
Need           required: Vec<CapabilityId>, preferred: Vec<CapabilityId> (most important first), allow_inherited: bool (default false)
Context        now: Timestamp, environment: Config, tool_versions: map<probe-tool, version>, policy: Policy
Policy         max_age: Option<Duration> (None = rows never age), require_tool_version_match: bool
```

`Provider` has no behaviour: the trait of SCOPE rule 6 is the **prober** (section 5); selection needs only data. A consumer that holds executables (AntSpawn `Box<dyn Launcher>`, ConfigurAnt `Box<dyn Module>`) keeps them next to its `Vec<Provider>` and maps ids back.

## 2. Resolving one capability on one provider

`resolve(matrix, provider, id, ctx, allow_inherited) -> Resolved { state, evidence_used, rows_considered }`:

1. Collect rows with `subject_kind = capability`, `subject = id` and, for some surface `s` of the provider, equal `backend` and `version` and `row.config.holds_in(s.config)` (PersistAnt's match rule). Rows of other kinds are kept by the reader and never consulted.
2. **No row matches** → `Unverified(NoRow)`.
3. Each matching row is first **degraded** to a per-row state:
   - `yes` with no `probe` evidence → `Unverified(NoEvidence)` (SCOPE 1).
   - `yes` whose `environment` is not contained in `ctx.environment` and has no `inherited_from` → `Unverified(OtherEnvironment)` (SCOPE 5).
   - `yes` with `inherited_from` and `allow_inherited = false` → `Unverified(OtherEnvironment)`; with `true` → `Yes`, flagged inherited in the outcome.
   - `yes` older than `policy.max_age` (a row with no `probed_at` is stale when `max_age` is set and fresh when it is `None`; Q2) or with `tool_version` different from `ctx.tool_versions[probe]` when `require_tool_version_match` → `Unverified(Stale)` (SCOPE 2).
   - `no` stays `No(reason)`; a `no` row is never degraded (an old known-bad is still known-bad).
4. **Combine across rows and surfaces: `No` beats everything, then `Yes`, then `Unverified`** (PersistAnt's "off wins", extended). If nothing is `Yes`, the strongest `Unverified` reason is reported (order `Stale` > `OtherEnvironment` > `NoEvidence` > the rest: the one a person can act on first).

Only `Yes` satisfies. `No` and `Unverified` both reject, with different reasons; this is the whole of rule 1.

## 3. Ordering

`evaluate(table: &[Provider], need: &Need, matrix: &Matrix, ctx: &Context) -> Evaluation`.

For each provider compute, per required id and per preferred id, a `NeedOutcome { id, role: Required | Preferred(rank), state, inherited: bool, evidence, reason_text }`. A provider **satisfies** when every required outcome is `Yes`.

Sort key, ascending (a total order ending in table position, as AntSpawn's stable sort does today). Rejected providers are not re-ranked by how few needs they miss, because AntSpawn orders them by `prefer` then registry order and the R2 test pins that:

1. satisfies first;
2. the preferred outcomes as a vector in importance order, `Yes` before not-`Yes`, compared lexicographically (so the most important preferred capability dominates; matches PersistAnt's "preferences in the order of importance given"); then, if `Need::preferred_providers` names the provider, its position there (AntSpawn's `prefer`, Q4);
3. table position.

**There is no closest match.** Rejected entries are returned so the UI can explain, but `Evaluation::satisfying()` is the only list a consumer may open from, and `open_named` (consumer side) calls `Evaluation::check(provider_id)` which returns `Err(Unmet)` for a rejected or unknown id (PersistAnt and AntSpawn's re-check at open time).

```text
Evaluation { entries: Vec<Entry> }               // ranked
Entry      { provider: ProviderId, satisfies: bool, outcomes: Vec<NeedOutcome> }
Evaluation::satisfying()  -> impl Iterator<Entry>
Evaluation::check(id)     -> Result<&Entry, Unmet>      // Unknown(id) | Rejected(entry)
Evaluation::explain(id)   -> Vec<String>                // one line per need: "kv.ttl: yes (probe kv::kv.ttl, 2026-10-23, linux/x86_64)"
```

Measured speed ranking (PersistAnt `choose`/`bench`, unread in R0) is **not** in the engine: it is a consumer-supplied tie-breaker applied after step 2 (`Evaluation::reorder_satisfiers(|a, b| ...)`). Q1.

## 4. The explanation is data

`NeedOutcome` carries the `State`, the row's `Reason`/`Why`, the `Evidence` and whether it was inherited; `explain` is a pure rendering of it. PersistAnt's `Listing::why()` text and AntSpawn's `unmet` list are both derivable; ConfigurAnt's `Choice.reason` is `explain` joined.

## 5. Matrix file and probes

**Format** (D5): header read by column name. Required: `subject_kind subject backend version config enabled reason metadata note`. Optional, in this order when written: `probed_at tool_version environment inherited_from`. Unknown columns are kept (`extra`) and written back after the known ones, sorted. `#` and blank lines skipped on read, dropped on write. `enabled` is `yes|no`. A row with `enabled=yes` and a `reason` is an error; `enabled=no` requires a non-empty `reason` (PersistAnt's rule). The `reason` text of AntSpawn maps to `Reason::Other`; the words `missing`/`wrong` map to their variants; `UNVERIFIED: <x>` maps to `Unverified(Other(x))` (an AntSpawn compatibility rule: those rows are `no` there, and stay a non-satisfying state here with the better name). `metadata` keeps `k=v;k=v`; `probe=` (else `test=`) is the evidence probe id and is written back as `probe=`; every other key is kept. `os=` and `arch=` in old metadata stay informational (R3 decision: seeding `environment` from them would silently stop PersistAnt's Linux-probed rows from counting on any other OS); only the `environment` column is a claim about where a row holds. `inherited_from` is `?` when the origin is unknown.

Duplicate rows: **not an error** (PersistAnt merges files; AntSpawn rejects). Duplicates for the same `(kind, subject, backend, version, config)` are resolved by section 2 step 4; the reader reports them in a `warnings` list and `--check` fails on them (a generated file has none).

Writer is canonical and deterministic: parse then write then parse is identity; writing is byte-stable (LF only, `\r\n` accepted on read). `to_tsv` never writes `probed_at` from the system clock itself: the caller passes it.

**Probes** (the SCOPE rule 6 trait, in the generator crate part):

```text
trait Prober { fn probe(&self, provider: &Provider, cap: &CapabilityId, ctx: &Context) -> Outcome; }
enum Outcome { Pass(Evidence), Fail { reason: Reason, evidence: Evidence }, Unverified { why: Why, evidence: Evidence } }  // `Unverified` covers an inconclusive probe and a missing one
```

`Outcome::Unverified` is how AntSpawn's network probe says "this host cannot connect, so the control failed": it becomes `Unverified(NoControl)` and is **never** `No` (a failed control proves nothing). A probe that panics or times out is `Inconclusive`, never `Pass`.

**Generate and `--check`** (`capant::generate(table, caps, prober, committed, ctx) -> GeneratedMatrix`; the CLI wrapper is the consumer's example, as today):

1. For each provider the prober can reach, probe every declared capability; no probe for a capability gives `Unverified(Untested)` with `reason = "no probe for this capability yet"`.
2. For a provider the machine cannot probe, **carry over** its committed rows if they are complete (one per declared capability) and every `yes` has probe evidence (AntSpawn's rule), setting `inherited_from` to the row's own `environment` (`?` if it has none) on each carried `yes`, never re-stamping `probed_at`. A carried `yes` counts for a need with `allow_inherited`, **or when its origin names facts that hold in the current environment** (R3 refinement: the Windows machine that probed `account` reads its own committed file without consent; an origin of `?` always needs consent). Otherwise rebuild them `Unverified(NotSetUp)`.
3. `--check` compares the committed file with the generated one **ignoring `probed_at` only** (it changes every run); every other column must be identical, and any warning fails. It also fails when a committed row `Yes` is stale under `ctx.policy`, reporting the rows. Exit codes: 0 equal, 1 differs (prints the generated text), 2 usage/setup.

## 6. Staleness, concretely

A row is stale when `now - probed_at > policy.max_age` (or it has no `probed_at` and `max_age` is set), or its `tool_version` differs from the version in `ctx.tool_versions` for that probe and `require_tool_version_match` is set. Defaults: `max_age = None`, `require_tool_version_match = true` (a different probe tool version is a different measurement; time-based ageing is opt-in because the consumers have no timestamps today). Staleness degrades only `yes`, only at evaluation time; files are never rewritten by evaluating.

## 7. Adapters: one result type, three present behaviours

The R2 tests pin each consumer's **current** behaviour from its own `origin/main` code, as a function of `Evaluation`:

| Consumer | Today | Projection |
|---|---|---|
| PersistAnt | `providers()` returns qualified providers in table order, or `Err(Unmet)` listing **every** provider with each missing id, the matrix word (`Off(reason)`/`no-row`) and durability shortfall; no partial list. | `evaluation.into_unmet()` : `Ok(satisfying)` if non-empty else `Err(Unmet)` whose per-provider `missing` is the non-`Yes` required outcomes (`No(r)` → `Off(r)`, any `Unverified` → `unverified(<why>)`, Q3 as decided in R2). Qualified order is the **table order** in PersistAnt today and the **ranked** order here: a deliberate change, recorded for the migration PR. |
| AntSpawn | `providers()` returns every launcher, satisfying first, then `prefer` position, then registry order, each with `unmet: Vec<Capability>`; `open_named` re-checks. | Direct: `entries` mapped to `Provider { id, unmet = non-Yes required ids }`. Order identical because sort steps 1, 2, 4 reproduce `(!satisfies, prefer rank, registry)` **when `preferred` is a single ranked id list** (AntSpawn's `prefer` is a list of launcher ids, not capabilities: it maps to the ordered **provider** preference, see Q4). |
| ConfigurAnt | `list_choices` returns choices best first, each with a reason; empty on error. | `qualified.map(|e| Choice::new(id, explain(e).join("; ")))`. |

## 8. Migration (R4), each its own consumer PR

1. **AntSpawn**: depend on `capant` (crates.io); keep its `Capability` enum and `launchers.tsv`; replace `matrix.rs`/the `Registry::providers` ordering with CapAnt; `examples/matrix.rs` becomes a `Prober` plus `capant::generate`. The existing 9-column file must parse unchanged and `--check` must pass on it with the default policy (`max_age = None`) before any row is rewritten.
2. **PersistAnt**: replace `persistant-core::Matrix`/`provider.rs` selection with CapAnt; keep `Reason`, `Config` if they are the same types (R2 reads them and reuses by moving, not copying); `choose`/`bench` stay in PersistAnt.
3. **ConfigurAnt**: its `persistant.rs` module keeps calling PersistAnt; only `Needs` gains typed capability ids if its owner agrees (no CapAnt dependency is forced).

Versions follow the portfolio rules; the PM allocates each number at gate time. No consumer is migrated before R2 and R3 are published.

## 9. Test plan: a negative control for every rule

Each test names the behaviour it protects and what breaking it does; the **negative control** is a test double or mutation that must fail the assertion.

| Rule / decision | Test | Negative control |
|---|---|---|
| SCOPE 1 / D1 | no row, `yes` without `probe`, and `unverified` all fail `satisfies` | a matrix whose `yes` has a probe **does** satisfy (positive control); deleting the evidence check makes the first test fail |
| SCOPE 2 / §6 | row with `probed_at` older than `max_age` is `Unverified(Stale)`; same row at `now` is `Yes`; a `no` row stays `No` however old | vary only `now` (hold everything else); a tool-version mismatch is stale only with `require_tool_version_match` |
| SCOPE 3 / §3 | for two providers each rejected for a different need, `explain` names the right need and reason for each; preferred order flips the ranking | swap the order of `preferred` and the ranking must flip |
| SCOPE 4 / §5 | `generate` → write → parse → write is byte-identical; `--check` fails when one `enabled` cell is flipped, passes when only `probed_at` changes | mutate a cell, expect failure; mutate `probed_at`, expect pass |
| SCOPE 5 / §2 | a `yes` from another `os` is `Unverified(OtherEnvironment)`; carried-over rows satisfy only with `allow_inherited` | same row with `inherited_from` set and `allow_inherited` true must satisfy (positive control) |
| SCOPE 6 / D3-D4 | the crate has no dependency beyond `thiserror`-class and std; an in-test provider kind unknown to the crate (`bananas.ripe`) resolves | `cargo tree` assertion in CI; a typo'd id is `Unverified(NoRow)`, never a panic |
| No closest match / §3 | with no satisfier `satisfying()` is empty, `check(id)` is `Err` for every id incl. the closest | one provider made satisfying makes `check` `Ok` for it only |
| Off wins / §2 | two rows, `yes` and `no`, same key → `No` | removing the `no` row gives `Yes` |
| Control failure / §5 | prober returning `Inconclusive(NoControl)` never produces `No` or `Yes` | a `Fail` outcome produces `No` (so the mapping is not constant) |
| Compatibility / D5 | the **checked-in** `launchers.tsv` (AntSpawn, `00b2aada`) and `sqlx-sqlite.tsv` (PersistAnt, `c6872b35`) parse and round-trip content-equal (copied into `tests/fixtures/` with provenance in a README) | a file with a bad `enabled` word or a short row is an error, not a skipped row |
| §7 adapters | one test per consumer in the table above, reproducing that consumer's observable behaviour on the same inputs | flipping a single row changes the projection in each |

## 10. Open questions, and what R2 decided

- **Q1** Decided: consumer-side tie-break, `Evaluation::reorder_satisfiers` (stable; the rejected stay after the satisfiers).
- **Q2** Decided: stale if `max_age` is set (a row that cannot prove its age cannot prove it is fresh), fresh when `max_age = None`.
- **Q3** Decided: `Lookup::NoRow` and `Off(Untested)` both become `Unverified`; `Unmet` text says `unverified(<why>)` instead of `no-row`, a visible wording change for PersistAnt's migration PR (pinned by `persistant_shape_unmet_lists_every_provider_and_never_a_partial_list`).
- **Q4** Decided: `Need::preferred_providers`, consulted after the capability preferences; PersistAnt and ConfigurAnt leave it empty.
- **Q5** Timestamp is `i64` seconds since epoch in the engine (decided); RFC 3339 in the file, a small hand-written parser, is R3's.
