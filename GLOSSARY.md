# Glossary

Terms as `docs/SCOPE.md` uses them. Where a term has a fuller definition, the pointer says where.

- **capability**: a named claim with a state, the evidence for that state, and a reason when the state is not `yes`.
- **state**: `yes`, `no` or `UNVERIFIED`. `UNVERIFIED` is the default; a claim without evidence is never `yes`.
- **evidence**: data recorded with a claim: probe id, tool version, environment, timestamp.
- **probe**: the check that produced a piece of evidence, run on this machine.
- **stale row**: a row older than the configured window or recorded against a different tool version; it degrades to `UNVERIFIED`.
- **inherited**: a row satisfied by a provider that was not probed on this machine; the row must say so.
- **need**: a capability the developer declares, in an order of importance.
- **provider**: a plug-in behind a trait that can satisfy needs. The engine knows nothing about persistence, jobs or config.
- **matrix**: the stable, diffable text file of capability rows. `--check` fails when it disagrees with the probes.
- **capability graph**: the portfolio-wide graph of capabilities and their providers (`C:\Projects\CAPABILITY-GRAPH.json`, tooling in `tools/capability_graph.py`).
- **provides / presents**: the graph fields for what a project supplies itself and what it exposes by way of another; see `docs/SCOPING-PRESENTS-FIELD.md`.
