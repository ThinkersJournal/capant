# Scoping note: D2 links for vault and file tools

Docs only; no code, no new standing work (pm-ants [TASK] 2026-11-22: take the top-ranked unowned row of `AGENT-STACK-GAPS.md`). Proposal, not a decision. Read from `AGENT-STACK.md` lines 57 and 73 and `CAPABILITY-GRAPH.json` lines 20, 34, 62, 68 on 2026-11-22; secretant README read at origin/main. Not probed against any running tool.

## What it is
D2 is the tool-provider half of layer D: MCP servers an agent calls. AGENT-STACK says Synapse's agent-facing side is a provider and "same for the vault and file tools". The graph has one capability, `tool-providers` ("messaging, vault, files"), with one provider, `synapse` (`partial`: send, fetch, ack, list, set_summary, whoami; "Messaging only: no vault or files tool provider read"). So two of the three things the capability names have no provider link at all.

## Two different things behind it
1. **Vault tool provider.** An MCP face over SecretAnt (the vault). Graph row for `secretant` says `secrets: none` as of 2026-11-01 (placeholder); its current README describes a working store, read API, CLI and delivery, so that row is probably stale. Re-read before use. Whether an MCP face should exist at all is a security question first: SecretAnt states a same-user limit, and an agent-reachable tool that returns secret values widens it. An approval or no-values (names and delivery only) design must precede any code.
2. **File tool provider.** AGENT-STACK's hierarchical-file edit tool (read or write sections of markdown/TOML/JSON by heading or key path). Proposed owner: editant, UNCONFIRMED (pm-ants is asking). It has no repo or graph row read by me.

## Who would consume it
- `tool-host` (D1) in any harness, including the planned agent-loop (its `TOOLS.md` is the consumer-side note; not re-read for this scoping).
- Lanes editing large files such as PORTFOLIO-ROADMAP.md (the CireSnave 2026-11-01 motivation).
- `permission-policy` (E) must gate the vault tool; `human-approval` (consent, built) is the asker.

## Smallest first slice (all docs or graph rows, nothing built)
1. Split the graph capability: `tool-providers` stays for messaging; add `tool-provider-vault` and `tool-provider-files`, each `needs: [permission-policy]`, each initially with no `provides` row (a PM edit to `CAPABILITY-GRAPH.json`; capant proposes, does not make it).
2. Refresh the `secretant` row from its current README (capant owns the evidence tool; a read, not a run).
3. One-page threat note for the vault tool: which operations an agent may ask for (list names, deliver to file) versus never (return a value), decided before an owner starts.
4. File tool: wait for editant's answer; if confirmed, link the `provides` row with `planned`.

## Not in scope
Building either server; choosing MCP library (rmcp is assessed, not adopted, per agent-loop); ranking against other rows.

## Open
- Who owns the vault MCP face: SecretAnt's lane or Synapse's? Neither named.
- Does the existing vault ("the working one", graph note) already have an agent-facing surface? Not read.
