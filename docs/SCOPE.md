# CapAnt: scope

CapAnt extracts the capability pattern that PersistAnt (provider registry, open_named), AntSpawn (the launcher certification matrix: matrix/launchers.tsv with yes/no/UNVERIFIED, probe=, reason) and ConfigurAnt each implement separately (CIRESNAVE-EXPECTATIONS 6.17: the developer declares needs with an order of importance; the project returns the providers that satisfy them). A capability is a named claim with a state (yes, no, UNVERIFIED), the evidence for that state (the probe that produced it, the date, the environment) and a reason when it is not yes. A claim without evidence is UNVERIFIED, never yes.

## Rules and traps to design in

Rules to design in from day one:
1. UNVERIFIED is a first-class state and the default; `yes` requires recorded evidence (AntSpawn lesson: a launcher is off until a matrix row says yes).
2. Evidence is data: probe id, tool version, environment, timestamp. A stale row (older than a configurable window or a different tool version) degrades to UNVERIFIED.
3. Needs are ordered; the result explains WHY each provider was chosen or rejected, by need.
4. The matrix file format is stable, diffable text (TSV or TOML), and `--check` mode fails when the file disagrees with the probes (AntSpawn's `matrix --check`).
5. No capability may be satisfied by a provider that was not actually probed on this machine unless the row says so and is marked inherited.
6. Providers are plug-ins behind a trait; the engine knows nothing about persistence, jobs or config.
