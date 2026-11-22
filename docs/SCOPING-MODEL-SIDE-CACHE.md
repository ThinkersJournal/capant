# Scoping: the model-side cache (KV, prefix) as a layer

Row 4 of the unowned stack-gap rows in `AGENT-STACK-GAPS.md`. A scoping note, no code, nothing run. Written 2026-11-22. Facts read from Fuel `origin/main` f747717f4 (`ROADMAP.md` rows 32, 132, 149; `docs/design/paged-attention-agnostic-seam.md`) after `git fetch`. Fuel's own claims are quoted as Fuel's; capant did not read `fuel-kv-pool` source.

## What Fuel's roadmap says already exists or is asked
| Piece | Where it stands (Fuel ROADMAP) |
|---|---|
| Block pool with prefix sharing | `fuel-kv-pool`, driven by Lightbulb's `model_fuel::policies` (row 32) |
| Hit, miss, commit, eviction counters | done, `CacheStats` / `KvBlockPool::cache_stats()` (row 32, 0.20.64); ruling there: **mechanism in Fuel, policy in Lightbulb** |
| Export and restore a session's KV state | **not started**, low priority, asked by lightbulb#275 (row 132): versioned byte form, restore refuses a model/geometry/dtype mismatch |
| Paged attention on GPU | not started (row 149); a performance row, not a layer boundary |
| Forkable copy-on-write cache | listed as future substrate (ROADMAP "Batched multi-sequence decode"), not a row with an owner read here |

## Where the boundary already falls
The blur in AGENT-STACK is between three things, and Fuel's rows separate them:
1. **Mechanism of the cache** (blocks, sharing, eviction, counters, export/restore): Fuel. PM ruling 2026-10-22.
2. **Policy** (what to share, when to evict, what `cache_stats` is shown to whom): the serving host, Lightbulb.
3. **What the agent puts in the context window** (compaction, which messages): agent-loop, layer C. This is not the model-side cache; a provider-side prompt cache is a prefix property of how the loop orders messages, which agent-loop can exploit without owning the cache.

So the cache is not a missing capability that needs a new project: it has an owner (Fuel) and a consumer (Lightbulb). What is missing is only that the capability graph has no id for it.

## Proposal (for the PM, not decided)
- Add one capability, e.g. `kv-cache`, "Hold and reuse a model's attention state (blocks, prefix sharing, export/restore)", needs `run-model`. Fuel `partial` (evidence: Fuel row 32 counters and `fuel-kv-pool`; row 132 for export is open, so not `built`). Lightbulb `claimed` as the policy consumer only if its owner says so.
- Leave claude-code and hosted APIs out: their cache is the provider's and sits behind `presents`/`run-model`, which `SCOPING-PRESENTS-FIELD.md` is about.
- Do not build a cross-project cache layer: the one consumer that needs a durable form (checkpoints, row 132) is already a Fuel row.

## Blocked on
- PM: whether to add the graph id (the graph is PM-owned).
- Fuel lane: confirm `fuel-kv-pool` is `partial` with a test path; capant has only read the roadmap rows. Row 132 stays Fuel's to start.
- Not checked: Lightbulb's side (policies), and whether agent-loop's compaction plans to depend on a provider prefix cache.
