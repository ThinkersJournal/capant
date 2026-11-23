# Scoping note: events in Synapse

Docs only; no code, no new standing work (pm-ants [TASK] 2026-11-22, authorised by the PM notice of the same day: take rank 2 of the unowned rows in `AGENT-STACK-GAPS.md`). Proposal, not a decision; the design call is board 359 (CireSnave's). Synapse read at `origin/main` 3f5726e (ROADMAP.md, `docs/ROOMS_DESIGN.md`, `docs/superpowers/specs/2026-10-23-shared-state-design.md`, `crates/synapse-mcp-server/src/lib.rs`, `crates/synapsed/src/lib.rs`) on 2026-11-22. Read, not run: nothing here was exercised against a live daemon.

## The ask (CireSnave 2026-11-01, verbatim from `AGENT-STACK.md`)
*"Events don't necessarily need a lane as a source.  Should Synapse distribute event notifications?  If it did, wouldn't that be a logical place for tools to share results?"*

## Correction to the gap row first
`AGENT-STACK-GAPS.md` (0.1.9) says Synapse is "point-to-point only today". That is stale at 3f5726e. Rooms are built (ROOMS_DESIGN slices R1-R5, rc.214-221; MCP tools `join`, `leave`, `rooms`, `members`; `send` to a room name fans out with delivery `room_broadcast`, `room_any` or `room_all`). So a topic-like fan-out exists; what does not is the part CireSnave's question is about. The row is corrected in this PR.

## What exists, what is missing
| Piece of "events" | State at 3f5726e | Evidence |
|---|---|---|
| Fan-out to many subscribers | **Built** as rooms: members who joined and are online | ROOMS_DESIGN section 7, R3 |
| Publish without being a member | **Designed**, grant `synapse:post` held without `synapse:join` | ROOMS_DESIGN 6a; whether built not separately read |
| Non-lane source | **Partly**: the daemon itself sends as `daemon:alerts@<acct>` and `daemon:state@<acct>` (hard-coded constants); no tool identity (`gate`, `publish`, `ci`) can send | `crates/synapsed/src/lib.rs` ALERT_SENDER, STATE_SENDER |
| Ids-only change notices | **Built** for shared state (S3, `daemon:state`) | shared-state spec section 6 |
| Typed events (`merged`, `ci-finished`, `wait-due`) | **Missing**: no event type or schema; `MESSAGE-FORMATS` headers are conventions on mail text | ROADMAP row 41 |
| Late joiner or offline subscriber gets the event | **Missing by design**: offline members get nothing, no history (bounded replay "a later slice, not designed") | ROOMS_DESIGN 7 and 11 |
| Durable "tools share results" | **Missing** as events; shared state (rows 30, 50) stores records, with CAS and signed writes | shared-state spec |

So the real gap is three things, not "pub/sub": (1) **non-lane identities that may publish**, (2) **typed, keyed events** a consumer can subscribe to by kind and repo/PR/crate rather than reading free text in a room, (3) **delivery that survives the subscriber being offline** (a wait wakes on an event that happened while the lane was down).

## Who would consume it
Evidence is from CIRESNAVE-DECISIONS (PM reading, 2026-11), not re-measured here:
- the **merge gate**: merges announced by hand and blocking when not (Unpopped#220/#221, aocl#8);
- **lanes with a WAIT** in HANDOFF (`lane-restart waits`): today a session timer, with no wake on publish-finished, CI-finished or a deadline; agentlife's "Waits v2" is named as the first piece;
- **tools that keep results in files** (sweep log, publish ledger, durability run) and could publish a result instead;
- the planned **budget-governor** and observability rows (agent-loop), which are event sinks and sources.

## Smallest first slice (design, then the least code)
1. **Decide the model before code** (board 359). Three candidates, in rising cost:
   - **A. Rooms plus tool identities.** Rooms already fan out; give tool roles (`gate`, `publish`, `ci`) a keystore identity and `synapse:post` on a room per event family. No new daemon concept; events are typed only by the `MESSAGE-FORMATS` header.
   - **B. A typed event envelope on top of A.** Add `[EVENT] kind= key=` as a registered format, with a closed kind registry (as the shared-state spec does for forge facts), so a subscriber can filter by kind and key. Free text stays out (channel events are untrusted).
   - **C. Durable events via shared state.** Publish as a state record plus an ids-only `daemon:state` notice (already built), so an offline subscriber reads the record on return. Reuses CAS, signed writes and staleness; no new store.
2. **My recommendation: A then B, with C only for the "offline waiter" case.** A and B cost the least and use built parts. The offline case (a WAIT that must wake on an event that fired while the lane was down) is exactly what rooms refuse by design, and shared state already answers it, so route that case through C instead of adding room history.
3. **First concrete step:** one event family end to end, `merged`, from the gate. Success test: a lane whose HANDOFF waits on a merge wakes on the event with no hand announcement. That tests identity, post-without-join, the envelope and the offline case together.

## Not in scope
Building any of it; cross-account or federated events (ROOMS_DESIGN 11a, "routing between daemons not built"); external channels (email, Slack; board 131); replacing the `daemon:alerts` transport; claude-peers lanes (still on it, per the PM reading).

## Open (for the Synapse lane and the PM)
- Does a keystore identity for a non-lane tool exist, or does it need role registration that requires a claimed session? Not read (`synapse claim` and `id init` are outside the capant lane).
- Is `synapse:post`-without-join built, or only designed? 6a is the design; R3 says only that `send` resolves rooms. Not read in the code.
- Who owns it: Synapse's lane (build) with agentlife (Waits v2). Neither confirmed by me.
- The model choice (A, B or C) is CireSnave's call under board 359; this note ranks, it does not decide.
