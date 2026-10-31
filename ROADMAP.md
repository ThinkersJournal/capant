# CapAnt roadmap

Order matters: survey, then design, then code. Every code row ships with a negative control (a test that fails when the behaviour is removed).

R0. Survey: read PersistAnt's registry (providers/open_named), AntSpawn's matrix (matrix/launchers.tsv, src/probes.rs, examples/matrix.rs) and ConfigurAnt's module selection; list what is identical, what differs, and what each cannot express. Deliver docs/SURVEY.md.
R1. Design: docs/DESIGN.md: data model (Capability, Evidence, Provider, Need), ordering and explanation of choices, matrix file format, staleness, `--check`, and the migration path for the three consumers. Test plan with a negative control per rule above.
R2. Core crate `capant` with an in-memory engine and tests.
R3. Matrix file reader/writer and `--check`.
R4. Migrate AntSpawn first (its matrix is the most developed), then PersistAnt, then ConfigurAnt, each as a PR in the consumer's repo, version bumps per portfolio rules.
R5. A `capant check` CI example for consumers (`examples/capant_check`, `docs/CI-CHECK.md`), run as a process in tests, with a fresh-eyes trial.
R6. Adoption review: for PersistAnt, AntSpawn and ConfigurAnt, read each `origin/main` for its own capability code, list what CapAnt would still replace, an order and what is blocked, with options and a recommendation (`docs/ADOPTION.md`). Docs only; each step would be that repo's own PR, decided by its lane.
