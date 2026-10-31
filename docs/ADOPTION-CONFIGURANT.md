# CapAnt in ConfigurAnt: adoption review

Docs only. A follow-up to `docs/ADOPTION.md` (R6), which left ConfigurAnt as "its owner's call". This note reads ConfigurAnt's own capability and fit code and says what CapAnt would replace there. Nothing here edits ConfigurAnt; every step below would be that repo's own PR, decided by its lane.

**Refs read** (`git show origin/main:<path>` after `git fetch`, 2026-10-31): ConfigurAnt `59a88db` (0.5.39), PersistAnt `c4ed00e`, CapAnt `9301776` (0.1.2; crates.io latest is 0.1.0). Read: `src/module.rs`, `src/persistant.rs`, `src/exec.rs`, `src/exec/run.rs`, `src/modules/ports.rs`, `docs/DESIGN.md` item 12, `docs/MODULES.md`, `ROADMAP.md`, `Cargo.toml`, `Cargo.lock`. Not read: `edit.rs`, the TUI and GUI crates' transfer screens, `configurant-scan` internals. The configurant lane confirmed (peers, 2026-10-31) that `src/persistant.rs` is the only place that picks among options by capability, and that it knows of no other.

## 1. What ConfigurAnt has that looks like capability or fit

| Code | What it decides | Capability-shaped? |
|---|---|---|
| `Needs { required, order }` (`module.rs`) | what a project asks of a module; strings whose meaning each module defines | Input only; no evidence |
| `PersistantModule::try_list_choices` (`persistant.rs`) | parses the strings to PersistAnt `CapabilityId`s, builds `ProviderNeeds` with the app's minimum durability, calls `registry::providers`, joins `l.why()` into `Choice.reason` | **Yes, but delegated**: the ranking and the matrix are PersistAnt's |
| `ChoiceList` (`module.rs`) | a fixed list; `list_choices` ignores `Needs` | No: it ranks nothing |
| `Host` and `cfg!(windows)` in the plan executor (`exec.rs:556`, `exec/run.rs:532`) | File modes are advisory on Windows (a note); a secret file needs `Host::owner_only_acl` there | No: a fixed rule for the host it runs on, asked at the moment of use |
| `paths::Platform`, `ports` rules P1 to P4 | what text is a valid path or port | No: pure functions of the text; nothing is probed |

Only one row is a choice among providers by capability, and ConfigurAnt owns neither its providers nor its matrix.

## 2. What CapAnt would replace

**Nothing in ConfigurAnt's own code.**

- The ranking and explanation of backends are already CapAnt's, through PersistAnt (`provider.rs` calls `capant::evaluate` since persistant 0.12.0). ConfigurAnt gets them by calling `persistant::registry`.
- `ChoiceList` has no capabilities to rank. Making it rank would need a matrix and a provider table that the caller does not have.
- The host and platform rows are not claims proven by a test. They are compile-time facts (`cfg!(windows)`) or answers that must be asked at the moment of use (`owner_only_acl`). CapAnt's engine has no clock and no I/O (DESIGN D4) and reads a committed matrix, so it cannot answer "is this path protected right now".
- `Needs` is opaque on purpose (`module.rs`: "a module defines the meaning of the strings"); ROADMAP row 5 ruled flags only, no `Needs` schema field.

## 3. One finding that is not CapAnt adoption

ConfigurAnt pins `persistant = "0.11.7"` (`Cargo.toml:34`; `Cargo.lock` shows `persistant`, `persistant-core` 0.11.7 and `capant` 0.0.7). crates.io's latest `persistant` is 0.12.3 (read back 2026-10-31). Two things follow:

1. The dependency rule (every project on the newest versions) is not met for `persistant`.
2. 0.12.0 is where selection went through `capant::evaluate`. Moving up changes the unmet text: `no-row` becomes `unverified(<why>)` (DESIGN Q3), and `l.why()` feeds `Choice.reason`, so what a user sees in the wizard's backend list changes in wording. PersistAnt's 0.12.5 added `registry::explain` for exactly this caller. Also, a `yes` row with no probe id no longer counts, so a backend that was offered before may now be listed as unverified.

This is a ConfigurAnt PR (a bump, plus a look at the tests that pin reason text), not a CapAnt change. I have not built it; whether the tests need changes is unknown.

## 4. Options

| Option | What | For | Against |
|---|---|---|---|
| A | Keep ConfigurAnt as it is; no CapAnt dependency | Nothing in its own code needs the engine; no break to the `Module` trait; the rest of the portfolio's choices already reach it through PersistAnt | `Needs` stays untyped (a typo is a module-time error, which it already reports) |
| B | A, and take the `persistant` bump (section 3) | Meets the dependency rule; gets `explain` and the current selection | A visible wording change in `Choice.reason`; tests that pin text may need updates |
| C | Typed needs inside a module only: `persistant.rs` already builds `CapabilityId`s; a new module for another consumer could wrap `capant::CapabilityId` itself | No trait break (the configurant lane's own suggestion: wrap in the module, not the trait) | Nothing needs it today |
| D | Typed `Needs` on the `Module` trait | One type for every module | Breaking for every module author; a new CapAnt dependency in the core crate; contradicts row 5 |
| E | A generic capability-ranked `Module` in ConfigurAnt (providers and matrix passed in) | Another consumer could offer its choices without writing a module | No customer asks; AntSpawn can write its own module over its registry exactly as `persistant.rs` does for PersistAnt (ROADMAP row 12) |

**Recommendation: B.** It is the only step with a gain (the dependency rule, and the CapAnt-backed explanation), and it is not CapAnt adoption in ConfigurAnt's code: it needs no `capant` entry in ConfigurAnt's `Cargo.toml`. C, D and E wait for a consumer to ask. If one does, take C first.

## 5. Risks

- B changes wording users see. A ConfigurAnt test or screenshot that pins `no-row` text will fail; that is the intended signal, not a regression.
- Adding a direct `capant` dependency later (option C or D) means one `capant` version across the build: two versions in one build give two incompatible sets of types (PersistAnt's 0.1.0 migration note). ConfigurAnt would have to track the same line as `persistant`.
- Nothing here was run: the claims in section 3 are from reading `Cargo.toml`, `Cargo.lock`, the PersistAnt changelog and crates.io.

## 6. What is blocked

- B: nothing on CapAnt. It waits on the configurant lane (its #64 ports slice and any PRs in its gate) and on the PM allocating the version.
- C, D, E: a consumer that asks, and for D a ConfigurAnt interface version change.
