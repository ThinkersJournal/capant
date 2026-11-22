# Agent-stack gap list

Docs-only note for pm-ants' [TASK] (2026-11-21). Compares `C:\Projects\AGENT-STACK.md` (DRAFT, PM, 2026-10-31) with `C:\Projects\CAPABILITY-GRAPH.json` as read on 2026-11-21. Not checked against any project's code; a row is a reading of the graph, not a probe. Nothing here is decided.

Method: every capability id in the graph, listed with its providers and states (`provides` entries per project). "No provider" = zero `provides` rows. "Claude-only" = the only providers are Claude Code rows, which are `claimed` and not ours.

## A. No provider in the graph
| Capability | Stack layer | Note |
|---|---|---|
| harness-adapter | G | AGENT-STACK: "does not exist". Graph `gaps_noted_by_pm` agrees. Belongs to the agent-loop lane (designed with C). |
| budget-governor | H | Only `tools/lane_models.py` exists (outside the graph's providers). |
| eval-harness | candidate | Source CireSnave 2026-11-01: "we're not ready to even start that yet". What Verify scores is undefined. |

## B. Provider exists but is Claude Code only, or only `claimed`/`none`
| Capability | Layer | Providers |
|---|---|---|
| agent-loop | C | claude-code `claimed`, overmind `claimed` |
| context-window, session-store | C/F | claude-code `claimed` only (durable-storage is covered by persistant `built`) |
| tool-host | D | claude-code `claimed` only; "none of ours" |
| permission-policy | E | claude-code `claimed` only (sandbox has antspawn `built`; human-approval has consent `built`) |
| secrets | cross-cutting | secretant `none` |
| expose-program | not in stack | surfaceant `none` |
| model-files, run-model | A | mlmf `partial`; run-model fuel `partial`, lightbulb/ollama `claimed` |

## C. Covered but only `partial` / `planned`
route-models (overmind partial), tool-providers (synapse partial), agent-lifecycle (agentlife partial), work-dispatch, merge-gate, observability (pm-tools partial), host-serve (webhostant partial), alert-human (alarmant partial), rank-signals (lightbulb partial, sortant planned), token-tools (tokenant planned), knowledge-store (sageant planned), wait-supervision (agentlife planned).

## D. Named in AGENT-STACK.md but absent from the graph as a capability id
| AGENT-STACK item | Why it is a gap |
|---|---|
| Events in Synapse (non-lane publishers, topic subscriptions) | Candidate with a CireSnave 2026-11-01 source; no capability id. Synapse is point-to-point only today (per AGENT-STACK). Board 359 design call. |
| Hierarchical-file edit tool (MCP) | "A project for later", no owner, no capability id. |
| Model-side cache (KV, prefix) as its own layer | Folded into `run-model`; AGENT-STACK says the boundary with Fuel's scheduler is blurry. |
| Split of A (model files / library / runner / server) | Graph already splits kernels, ml-library, model-files, run-model, serve-openai-api; matches the "A is at least three layers" revision. |
| Split of H (lifecycle, messaging/identity, dispatch, governor, gates, observability) | Graph has these as separate ids; only `budget-governor` and `observability` lack a real provider. |
| Presents vs implements | Graph has a `presents` field on claude-code only; no other project records it. |
| D2 tool providers for vault and file tools | `tool-providers` lists synapse only; vault (secretant) and file tools are not linked. |

## E. Graph-side notes
- Unplaced projects (no capability row): cpufish, capant, bonefish, rockfish, openclownfish, easyants, licence-tools, community, ThinkersFoundation, Humboldt, Thinker's Journal.
- Claude Code rows are all `claimed` (PM claims); no lane can confirm them, so C-F cannot reach `built` for a project of ours until a replacement exists.

## F. What the agent-loop notes add (read 2026-11-21, first 4000 bytes of each of docs/LOOP-CORE, HARNESS-ADAPTER, TOOLS, SANDBOX, SESSION; ARCHITECTURE seen but not relevant here)
Source: `ThinkersJournal/agent-loop` main via `gh api`; notes are DRAFT, nothing built there. Only the opening of each was read; sections past that are not covered here.
- C, D, E, F, G now have design notes, so those layers are no longer undesigned; they remain unbuilt, so B-section rows stay unprovided by anything of ours.
- The notes name adoption candidates that the graph has no project for: goose-agent (C), codex-rs (C, E), rmcp (D, MIT/Apache-2.0 MSRV 1.88), rig (F, read at `4908f428`). A capability graph row needs a `provides` project for them, or a rule that outside crates are not graph entries. Question for agent-loop.
- G (harness-adapter) is designed in `HARNESS-ADAPTER.md` as a Synapse-carried face; the notes mark Claude Code's `cost` as UNVERIFIED, so `budget-governor` and `observability` have no cost source today.
- E1/E2 split: AntSpawn (`sandbox`, built) covers only E2 for one job; no E1 policy provider of ours exists. Consent (`human-approval`, built) is the asker.
- `agent-lifecycle` needs `harness-adapter` in the graph (`needs`), so it cannot be fully `built` until G exists.

## Open
- agent-loop design-round date: pm-ants 2026-11-21 says no round is scheduled.
- Answered by agent-loop 2026-11-21 (its message, read from its own docs on main): rig (rig-agent/rig-core 0.46.0) is its adopted engine and a Cargo.toml dependency; goose-agent alpha.13 and rmcp 3.8.0 are assessed only (docs/GOOSE-ASSESSMENT.md, docs/LOOP-CORE.md), neither a direct dependency; codex-rs is read only for E and G, not a dependency. So only rig is a candidate graph entry (provider for agent-loop, C); the others are not providers. Not re-verified here.
- budget-governor: nothing of ours enforces one. rig-agent has max_turns and cumulative usage(); no spend cap or policy. agent-loop's view: the cap belongs at the Driver layer as a policy over that usage; the owner was a PM ruling.
- PM ruling (relayed by pm-ants 2026-11-21, not re-read from the PM): budget-governor goes to agent-loop as a design note, no code yet. capant keeps this list current and reviews that note when it lands. Row A (budget-governor) owner: agent-loop (design note pending).

## Owner check (2026-11-22)
Each row above against agent-loop's answers (F and Open). Not checked against code.
- Owner now known: budget-governor (agent-loop, per the ruling); harness-adapter (agent-loop, row A); rig as agent-loop's engine (agent-loop, C).
- Still no owner named anywhere I have read: eval-harness (CireSnave: not started); hierarchical-file edit tool (AGENT-STACK: no owner); Events in Synapse (board 359 design round, no lane); model-side cache layer (boundary with Fuel's scheduler unsettled); `presents` field beyond claude-code; D2 links for vault and file tools. I add no owner for these: assigning one is a PM/pm-ants call. Question to pm-ants.
