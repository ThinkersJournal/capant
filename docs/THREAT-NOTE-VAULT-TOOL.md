# Threat note: an agent-facing vault tool (draft)

Docs only; nothing built. Item 3 of `SCOPING-D2-VAULT-FILE-TOOLS.md`, drafted 2026-11-24 on a PM [TASK] relayed by pm-ants. A proposal for the PM and CireSnave to decide before any owner starts; not a ruling. Facts from the SecretAnt README at secretant `origin/main` f81a0a8 (read, not run). Graph id: `tool-provider-vault` (needs `permission-policy`; no provider).

## Why decide first
SecretAnt states a same-user limit: another program running as the same user can do what its own read API does. An MCP face that an agent calls moves that reach from "any local program" to "any prompt the agent reads", so a prompt injection becomes a path to a secret. SecretAnt also keeps no audit of who read a secret, so a leak through this face would be unattributable.

## Operations an agent may ask for
| Operation | Verdict | Why |
|---|---|---|
| List secret names (`SecretReader` list) | Allow, behind `permission-policy` | Names are not values; still disclosure of what exists, so scope by prefix per role. |
| Deliver a named secret to a named file or environment file (delivery plan, apply, verify) | Allow only with `human-approval` per target path, never to a path outside an allow-list | The value lands where a program needs it and never enters the agent's context. |
| Run a program with a secret in its environment (`run`) | Ask each time; child output must not be returned unfiltered | Child output can echo the value. |
| Delivery ledger read (what was placed where) | Allow | Holds no values. |
| Verify a placement matches | Allow | Returns yes/no. |

## Operations never offered
- **Return a value** (`get`, `SecretReader::get`) to the agent, in any form: plain, encoded, hashed prefix or length-revealing partial.
- Put, rotate, import, export, backup, restore: owner actions at a terminal.
- Unlock material: passphrase, recovery key, OS wrap management (`init`, `--add-os`).
- Deliver to Cloudflare Workers: not proven against the live service (README), and it reaches off the machine.
- Any call that widens the allow-lists above.

## Rules the face must keep
1. Responses carry names, paths and status only; a value never appears in a result, error text or log (`SecretValue` has no `Display`/`Serialize`; the face must not add one).
2. Every call is gated by `permission-policy` and, for delivery, `human-approval`; a denial is a result the agent can read, not a retry loop.
3. The face keeps its own record of who asked for what, since SecretAnt keeps none; it holds no values.
4. A delivered file's mode is owner-only; delivery fails closed if it cannot set it.
5. The agent never supplies the unlock: the host opens the store, not a tool argument.

## Open
- Owner of the face: SecretAnt's lane or Synapse's (neither named; carried from the scoping note).
- Whether delivery to an allow-listed path is itself acceptable when the agent can later read that file (it can, via the file tool): the pair `tool-provider-vault` + `tool-provider-files` needs a path rule that keeps delivered secrets out of the file tool's reach. Not tested.
- Whether the same-user limit makes any agent-facing face worth building; "no vault tool, delivery done by the human or a host step" is a valid outcome of this note.

— lane: capant
