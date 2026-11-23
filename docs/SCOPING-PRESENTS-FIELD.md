# Scoping: the `presents` field beyond claude-code

Row 3 of the unowned stack-gap rows in `AGENT-STACK-GAPS.md`. A scoping note, no code. Written 2026-11-22; facts read from `C:\Projects\CAPABILITY-GRAPH.json` and `C:\Projects\tools\capability_graph.py` on disk (not from a git ref: `tools` is not a git repo).

## What exists
- The graph has 31 projects; `presents` is set on exactly one, claude-code (`["run-model", "sampling"]`). Control: the same scan finds `provides` on all 31.
- The tool already supports it. `capability_graph.py` validates that every presented id is a known capability (the "presents unknown capability" error) and `matrix` draws `^` for "presented but implemented elsewhere". So the gap is data and a rule, **not a schema change**; the row's "touches the PM-owned graph" is true only of the data edits.

## What is missing
1. **No rule for when a project may use it.** claude-code's entry is justified by a free-text note; nothing says what "presents" means for another project.
2. **No link to the implementer.** `^` says "implemented elsewhere" but not where. For claude-code that is Anthropic's model, which is outside the graph, so nothing could be linked; for an in-portfolio case it could be.
3. **No check that a presenting project does not also claim it as built.** A project that both `provides` built and `presents` the same capability is contradictory; the validator does not flag it.

## Proposal (for the PM's decision, not made)
- Define: `presents` = the project exposes the capability to its users but the implementation lives in a different project or an outside service; it never counts toward `gaps` (it already does not: `gaps` reads `provides` only).
- Add one optional per-id note or `via` (implementer project id or "external:<name>") so the matrix can say where it lives.
- Add one validator rule: a capability in both `provides` (built/partial) and `presents` of the same project is an error.
- Candidate entries, **unverified, capant has read none of these projects**: ollama and lightbulb currently `claimed` run-model; if either wraps another engine, the honest entry may be `presents` rather than `provides`. Each owner would have to say; capant should not guess.

## Cost and owner
Tool change (rule plus `via`) is in `tools/capability_graph.py` and `tools/tests/test_capability_graph.py`, which capant owns: small. Data edits to `CAPABILITY-GRAPH.json` need the PM's acceptance, as for batch 2. Blocked on: the PM accepting the definition above, and on the owners of ollama and lightbulb for the candidates.
