# Can CapAnt be the matching engine over KISS capabilities?

Status: **UNVERIFIED design note, docs only, no code.** Question (CireSnave 2026-10-28, first-hand, relayed by the PM): *"Kernel needs are by capability. Should CapAnt be involved in the kernel capability interface?"*
Read at `KISS` `origin/main` on 2026-10-28: `spec/announce.md` (§6.4 decline codes, §7.1-§7.3 capability bitset, profiles, role-aware live word) and `spec/classify.md` (§6.8 `target_capability`). Nothing else of KISS was read; every claim about KISS below is a reading of those two files, not a test.

## Intended hello-world (the shape a Fuel or baracuda developer would copy)

```rust
// UNVERIFIED sketch: `kiss_matrix` is an adapter in FireLine, not part of CapAnt.
let offers = kiss_matrix::from_announce(&peer_envelope);          // rows: one per known bit, evidence = this handshake
let need = Need::new(vec![id("kiss.feat.contract_query"), id("cuda:sm89")]);
let eval = evaluate(&providers, &need, &offers, &ctx)?;
let kernel = eval.satisfying().next();                             // or Err(unmet) saying which need failed and why
```

## Verdict

CapAnt can be the engine; **KISS stays free of it** (KISS cannot depend on a Rust crate). The glue is an adapter in FireLine (the owner of the KISS Rust types) that turns an announce envelope into a `Matrix` and turns CapAnt's answer back into KISS's own decline. CapAnt itself learns nothing about KISS. Two small engine questions below are the only possible changes.

## What maps

| KISS | CapAnt | Notes |
|---|---|---|
| A set bit in the 64-bit `capabilities` word (announce §7.2) | `Row ... State::Yes` for a capability id such as `kiss.ext.dlpack_v1`, `kiss.feat.contract_query`, `kiss.sub.speaks_classify` | One id per assigned bit. The axis (EXT/FEAT/SUB) goes in the id so a bit number never leaks into a need. |
| A required bit | `Need.required` | Order of importance = `Need.preferred` order. |
| Unknown bit set by the peer: "ignore, never reject" (§7.2-0007) | the adapter creates no row for it | CapAnt never lists ids, so an unknown bit costs nothing and can never fail a need. Matches the rule. |
| Bit clear (known bit) | `No(Missing)` row | KISS says clear = absent. Reserved-but-zero bits must not be turned into `No`; only bits the adapter recognises. |
| `target_capability` `cuda:sm89` (classify §6.8) | the id `cuda:sm89` | `:` is already in `CapabilityId`'s alphabet. The rule "byte-exact, no subset or implication" (§6.8-0002) is exactly CapAnt's exact id equality: `sm90` never satisfies a need for `sm89`, with no extra code. |
| Kernel availability for a target | a `Provider` (the kernel source) with a `Yes` row per `target_capability` it has a kernel for | A need for a target the provider lacks is `Unverified(NoRow)`, and the explanation says so. |
| Experimental and vendor ranges (EXT 16-23, FEAT 40-47, SUB 60-63) | ids under `kiss.exp.<axis>.<bit>` and `kiss.vendor.<axis>.<bit>`; the vendor's own registry owns the meaning | Experimental bits have no registered meaning, so a need on one is the consumer's own declaration. |
| "Why was I refused" | `Evaluation::explain`, `Unmet`, `State::No(Reason)` / `Unverified(Why)` | One line per need. |

## What does not map (or only partly)

1. **Role-aware live word** (§7.3-0003: `(local & remote & !FEAT) | (provider & FEAT)`). CapAnt asks *"does this provider give X?"*, not *"do both sides speak X?"*. It reduces cleanly if the adapter treats the consumer's own word as the set of ids it is willing to ask about: a need is only created for bits the local side speaks, then the provider's word decides. The adapter must do that masking; CapAnt will not notice a need the local side could not itself honour. **Gap: documentation, not code.**
2. **Profile negotiation** (`max(L ∩ R)`, §7.1). A number picked from two integer sets is not a yes/no capability. It can be faked (`profile.1`, `profile.2`, ... as preferred needs in descending order picks the highest the provider has) but the intersection with the *local* set is again the adapter's job, and KISS's own ten-line `max` is simpler. **Recommend: leave it in KISS.**
3. **Decline codes** (§6.4-0009: numeric `u32`, core 1-6, experimental `0x40000000..`, vendor `0x80000000..`, unknown code = generic decline). CapAnt's reason is `Reason::Other(String)` / `Why::Other(String)`: the number would survive only as text (`kiss.decline.0x00000004 QUERY_NOT_SUPPORTED`). Those *standing* declines (`QUERY_NOT_SUPPORTED`, because FEAT bit 33 is clear) do come out of CapAnt for free: the engine rejects first, and the adapter maps `Unmet` to the code. **Per-request declines** (`UNKNOWN_STRUCTURE_KEY`, `UNKNOWN_REVISION`, `MALFORMED_REQUEST`, `CANNOT_PROVISION`) are answers to one request, not facts about a provider, and do not belong in a matrix. **Gap: possibly a structured reason; see Q-K2.**
4. **Handshake evidence.** CapAnt's rule 1: `Yes` needs recorded evidence, and a row is `Unverified` without it. A peer's announce is a *claim*, not a probe. If the adapter records the handshake as probe `announce:handshake` with the peer and time, CapAnt will call it `Yes`, which is exactly "the peer says so". That is honest only if the explanation shows the evidence kind. **Gap: Q-K1.**
5. **Lifetime.** A handshake is per connection; the file-backed matrix (`parse`/`write`/`generate`) and staleness machinery are for probe results that outlive a process. For KISS the `Matrix` is built in memory and thrown away; `Policy::max_age = None`.
6. **Namespace registration** (§6.8-0003) is a producer-side rule; CapAnt does not check it and should not.
7. **Case.** `target_capability` is case-sensitive ASCII (§6.8-0005) and may contain upper-case; `CapabilityId` is lower-case only. Every example in classify §6.8 is lower-case (`cuda:sm80`, `rocm:gfx942`, `metal:apple9`), but no clause forbids upper case. UNVERIFIED whether any namespace uses it. **Gap: Q-K3.**

## What CapAnt would need (all optional, none blocks the adapter)

- **Q-K1** Distinguish a *claimed* `Yes` (peer announce) from a *probed* `Yes` in `Evidence` and in `explain` output. Small: an `Evidence` field or a probe-id convention.
- **Q-K2** A structured, comparable reason (a code plus text) instead of free text only, if consumers need to branch on it. Without it the adapter keeps its own table from `Unmet` to decline code.
- **Q-K3** Case-sensitive ids, or an adapter-side escaping of upper case (reversible, must not collide). The escape is the cheaper choice; widening `CapabilityId` changes a published rule.

## Who owns what

CapAnt: the engine, unchanged. FireLine: the adapter crate (KISS types in, `Matrix` and `Need` out; `Unmet` in, KISS decline out). Fuel, baracuda, Unpopped: depend on FireLine's adapter and on capant by crates.io version. KISS: the spec, with no Rust dependency; the conformance crate stays std-only.

## Not done

No adapter was written or run. The sketch was not compiled. No KISS-Conform vector was fed through CapAnt; the first honest test is the §7.3-0003 worked vector (consumer `0x0031_0000_0000_0001`, provider `0x0091_0003_0000_0003`, live `0x0011_0003_0000_0001`) reproduced by the adapter plus `evaluate`, with a negative control that flips one FEAT bit on the consumer side and shows the live answer does not change.
