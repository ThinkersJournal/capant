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
| Events in Synapse (non-lane publishers, topic subscriptions) | Candidate with a CireSnave 2026-11-01 source; no capability id. Rooms (built, fan-out to joined members) are not events: no non-lane publisher, no typed events, no delivery to an offline subscriber. Scoped in `SCOPING-SYNAPSE-EVENTS.md`. Board 359 design call. |
| Hierarchical-file edit tool (MCP) | Owner: EditAnt (editant confirmed 2026-11-22, relayed by pm-ants). Its MVP is Markdown heading-sections only (editant SPEC 2); JSON/TOML/code are a non-goal for now. So the gap is "other formats", not "the tool". No capability id yet. |
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
- Owner now known: budget-governor (agent-loop, per the ruling); harness-adapter (agent-loop, row A); rig as agent-loop's engine (agent-loop, C); eval-harness (agent-loop, named with the harness adapter in the PM ruling, relayed by pm-ants 2026-11-22; CireSnave's 2026-11-01 "not ready to start" still stands, so owner is not a start date).
- Hierarchical-file edit tool now has an owner: EditAnt (confirmed by editant 2026-11-22 via pm-ants; MVP Markdown heading-sections only, other formats a non-goal for now; SPEC 2 not read by capant). Four rows are unowned, ranked below owned rows (PM ruling 2026-11-21, relayed by pm-ants; demand orders work, never whether). When a lane has free capacity the highest is picked first. pm-ants has put them to the PM. Ranking below is capant's proposal, not a ruling:
  1. D2 links for vault and file tools: smallest and concrete (`tool-providers` lists synapse only; add secretant and file-tool links as providers). Scoped in `SCOPING-D2-VAULT-FILE-TOOLS.md` (capant took it 2026-11-22, pm-ants [TASK]).
  2. Events in Synapse: board 359 design round; rooms give fan-out, the gap is non-lane publishers, typed events and offline delivery. Scoped in `SCOPING-SYNAPSE-EVENTS.md` (capant took it 2026-11-22, pm-ants [TASK]).
  3. `presents` field beyond claude-code: graph schema question, cheap but touches PM-owned graph. Scoped in `SCOPING-PRESENTS-FIELD.md` (no schema change needed: the tool already validates and draws it; data and a definition are missing).
  4. Named kv-cache, Fuel partial (pm-ants 2026-11-23). Scoped in `SCOPING-MODEL-SIDE-CACHE.md` (2026-11-22): Fuel's rows already split mechanism (Fuel) from policy (Lightbulb); what is missing is only a graph id. Blocked on the PM for the id and the Fuel lane to confirm `partial` and its test path (asked via pm-kernels 2026-11-23).

## Review of agent-loop's budget-governor note (2026-11-22)
Reviewed `docs/BUDGET-GOVERNOR.md` at agent-loop branch `budget-governor`, commit 4946e7f (note only, no PR yet). Its source citations were spot-checked at agent-loop `1a8c892`: `max_turns` (`src/run.rs:35`), `Event::Usage` (`src/events.rs:24`), and the two `Usage` emits in `src/driver.rs` (compaction and the main call) all say what the note says. Not run; nothing in it is built.

What it enforces, and where: an in-run cap (layer C) as a `BudgetPolicy` evaluated at turn boundaries, stopping through the existing soft stop; the fleet cap (layer H) is left to AgentLife, fed by the G `cost` report. That split is sound and creates no standing work.

Points for agent-loop and the PM:
1. **Fleet policy input is THROTTLE.md, not a limit the governor invents.** Portfolio CLAUDE.md says cost is "a routing constraint, not a stop-work order" and that throttling is read only from `THROTTLE.md`. A governor that stops runs on its own number would act as a stop-work order. The note cites the cost rule as the reason a fleet cap is wanted; it should say the cap is opt-in and its source is the PM-set status, with the in-run cap a per-run safety, not a fleet policy.
2. **Token fields are `Option<u64>`** (`src/events.rs`), so an unreported counter is `None`. The note's "unknown is not zero" rule for cost should say the same for tokens: a `None` must not add 0 to the total. A cache-read or cache-write count is not in `Usage` (grep of `src/events.rs` for "cache": no hit), so a token cap over-counts cached input; say whether that is accepted.
3. **Overshoot is bounded only by a turn plus a batch.** The policy runs after each response and a soft stop finishes the batch, so one parallel tool batch can run past the cap. State the bound, or note that it is accepted.
4. **Budget scope across a resume.** The note makes stop-save-resume the path, but does not say whether cumulative usage carries over a resume. If it does, a resumed run stops again at once; if it does not, the cap is per-segment. Pick one (run, session or day) before decision 1.
5. **Synapse "has no subscribe or topic" is stale.** Rooms are built at synapse 3f5726e (see `SCOPING-SYNAPSE-EVENTS.md`). The fleet-cap blocker is narrower: no non-lane publisher and no typed event, so a G `cost` report has no tool identity to post as. Decision 3 (Claude Code cost source, unowned) is untouched by this.
6. **Eval-harness row:** consistent with this list: not started, CireSnave 2026-11-01 "not ready", owner agent-loop is not a start date.
7. **Harness-adapter row:** consistent; its Synapse-transport blocker should cite point 5.

Verdict: the note can go to the PM gate as a draft after points 1 and 4 are answered; 2, 3 and 5 are edits, not blockers. This is a review, not a ruling.

### Closure of points 1 and 4 (2026-11-22)
Read at agent-loop `origin/main` 9b82f7a (#51 merged), `docs/BUDGET-GOVERNOR.md`: point 1 is answered (lines 52-54: the fleet cap is opt-in and reads `THROTTLE.md`, "nothing here defaults to stopping a lane") and point 4 is answered (line 45: usage is not carried across a resume today; the host keeps the running total or saved state gains it, left as a decision). Both blockers in the verdict are cleared. Points 2, 3 and 5 were not re-read, so they stay open as edits. The note's status table still marks the in-run cap "designed, not built" and the fleet cap "not designed past section 3"; this list keeps `budget-governor` as owned by agent-loop, no provider.
