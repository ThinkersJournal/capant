# `capant check` in CI

A consumer commits a matrix file, one row per provider and capability, saying what each backend was **shown** to do. CI re-runs the probes and fails when the file no longer says what the probes find, or when a capability the project declares has no probe. This is the `--check` of `docs/DESIGN.md` section 5; the command line is the consumer's, and `examples/capant_check/main.rs` is a complete one (it ships in the published crate, and is at `https://github.com/ThinkersJournal/capant/blob/main/examples/capant_check/main.rs`; the README shows the same calls in miniature).

## Layout

```text
my-app/
  Cargo.toml                      # capant = "0.0"
  capabilities/matrix.tsv         # committed; written by `--write`, read by CI
  examples/capant_check/main.rs   # CAPS, PROVIDERS, a Prober (copy and edit)
  .github/workflows/capant-check.yml
```

To adopt it, copy `examples/capant_check/main.rs` and replace `CAPS`, `PROVIDERS` and the `Store`/`Probes` section with your own. What must stay is the use of `capant::{generate, check, parse, write}` in `main` and an `impl Prober` whose `probe` returns an `Outcome` (`Pass`, `Fail { reason: Reason, .. }` or `Unverified { why: Why, .. }`). An example in `examples/` is found by Cargo without a `Cargo.toml` entry; drop the imports your probes no longer use. The matrix path, and any extra capability, go after `--`.

The sample in this repo is `examples/capant_check/`: two stores (`mem`, `files`), three capabilities (`kv.get`, `kv.put`, `kv.durable`), real probes, and `matrix.tsv` committed beside it.

## The workflow

`examples/capant_check/capant-check.yml`, copied here byte for byte (`tests/ci_check.rs` fails if this block and that file differ):

```yaml
name: capant check

on:
  push:
    branches: [main]
  pull_request:

permissions:
  contents: read

jobs:
  capant-check:
    name: capant check
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@11d5960a326750d5838078e36cf38b85af677262 # v4
      - uses: dtolnay/rust-toolchain@686976e191b89faba57d3206551f0f330d8cb249 # stable
        with:
          toolchain: stable
      # Exit 1 when the committed matrix differs from what the probes find, or when a declared
      # capability has no probe.
      - run: cargo run --example capant_check -- capabilities/matrix.tsv
```

## What fails it

Run `cargo run --example capant_check -- examples/capant_check/matrix.tsv` in this repo. Exit 0: `6 rows match the probes`. Then change one thing:

| Change | Exit | Message starts |
|---|---|---|
| delete a row from `matrix.tsv` | 1 | `... lacks what the probes found:` |
| flip a cell (`no` to `yes`) | 1 | `... lacks what the probes found:` and `... claims what the probes do not find:` |
| add `kv.watch` to the command line (a declared capability nobody probes) | 1 | `unprobed: kv.watch on mem` |
| edit only `probed_at` | 0 | - |

A declared capability that is also missing from the file gets both the `lacks` lines and the `unprobed` lines. A declared capability with no probe fails even when the committed file already records it as unverified: a file that agrees with the run is still a hole. Declaring one in `CAPS` is how a project says "I rely on this"; the check is what keeps that sentence true.

## Regenerating

`cargo run --example capant_check -- --write capabilities/matrix.tsv` rewrites the file from the probes. Review the diff: a `yes` that turned `no` is a regression, not noise.

## What it does not do

Rows carried over for a provider this machine cannot probe are trusted, not verified (`Report::carried`); this example probes every provider, so it has none. Record no environment (as here) and one file holds on every runner; record one and a row from another environment is not accepted unless the need allows inherited rows.

## Tested

`tests/ci_check.rs` runs the example as a process: it passes on the committed file and fails on each row of the table above, each case differing from the passing one in one thing. The repo's own `ci.yml` runs the same command in a `capant check` job.
