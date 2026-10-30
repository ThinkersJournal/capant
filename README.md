# CapAnt

Pick the backend that can really do the job, and know why. You list what you NEED; providers record what they can do, with evidence; CapAnt returns the providers that qualify, ranked, and says why each other one did not. No evidence means `unverified`, never "probably yes".

```rust
use capant::*;

let id = |s: &str| CapabilityId::new(s).unwrap();
// A provider is a named backend; its surface says which version of it the matrix rows are about.
let provider = |name: &str| Provider::new(name, vec![Surface::new(name, "1", Config::new())]).unwrap();

// What each backend can do. A `Yes` only counts with the probe that showed it.
let mut matrix = Matrix::new();
matrix.push(Row::capability(id("kv.get"), "mem", "1", State::Yes).evidence(Evidence::probe("kv::get")));

// What I need, then ask. `Context::new(now, environment)`: `now` is seconds since the epoch (0 = rows never age).
let need = Need::new(vec![id("kv.get")]);
let eval = evaluate(&[provider("mem"), provider("fs")], &need, &matrix, &Context::new(0, Config::new())).unwrap();

assert_eq!(eval.satisfying().map(|e| e.provider.as_str()).collect::<Vec<_>>(), ["mem"]);
// `fs` has no row, so it is rejected, and the error says why (`provider `fs` is not certified for: kv.get [unverified (no row)]`):
let why = eval.check("fs").unwrap_err().to_string();
assert!(why.contains("kv.get [unverified (no row)]"), "{why}");
```

Paste it inside a `fn main()` or a `#[test]`. Add the crate with `cargo add capant`. This block is compiled and run by `cargo test`.

A row only counts when it can prove itself. A `Yes` with no probe, or older than your window, is `unverified`, never a quiet pass:

```rust
use capant::*;

let id = |s: &str| CapabilityId::new(s).unwrap();
let fs = Provider::new("fs", vec![Surface::new("fs", "1", Config::new())]).unwrap();
let state = |row: Row, now| {
    let ctx = Context {
        policy: Policy { max_age_secs: Some(3600), ..Policy::default() }, // evidence older than an hour is stale
        ..Context::new(now, Config::new())
    };
    resolve(&Matrix::from_rows(vec![row]), &fs, &id("kv.get"), &ctx, false).state.to_string()
};

let claimed = Row::capability(id("kv.get"), "fs", "1", State::Yes); // says yes, shows nothing
assert_eq!(state(claimed, 10_000), "unverified (no evidence)");

let probe = || Evidence::probe("kv::get").at(1_000);
let old = Row::capability(id("kv.get"), "fs", "1", State::Yes).evidence(probe());
assert_eq!(state(old, 10_000), "unverified (stale)"); // probed 9000 s ago
let fresh = Row::capability(id("kv.get"), "fs", "1", State::Yes).evidence(probe());
assert_eq!(state(fresh, 2_000), "yes");
```

Needs come in two kinds. `Need::new(required)` must all be `yes`; `.prefer(preferred)` only orders the providers that qualify, most important first, and never excludes one. `satisfying()` lists the qualifying providers best first, and `check(name)` explains a rejected one:

```rust
use capant::*;

let id = |s: &str| CapabilityId::new(s).unwrap();
let provider = |name: &str| Provider::new(name, vec![Surface::new(name, "1", Config::new())]).unwrap();
let proven = |cap: &str, backend: &str| {
    Row::capability(id(cap), backend, "1", State::Yes).evidence(Evidence::probe("probe"))
};

let matrix = Matrix::from_rows(vec![
    proven("kv.get", "files"), proven("kv.atomic_replace", "files"),
    proven("kv.get", "sqlite"), proven("kv.atomic_replace", "sqlite"), proven("kv.fast_scan", "sqlite"),
    proven("kv.get", "mem"),
    Row::capability(id("kv.atomic_replace"), "mem", "1", State::Yes), // claimed, never probed
]);
let need = Need::new(vec![id("kv.get"), id("kv.atomic_replace")]).prefer(vec![id("kv.fast_scan")]);
let providers = [provider("files"), provider("sqlite"), provider("mem")];
let eval = evaluate(&providers, &need, &matrix, &Context::new(0, Config::new())).unwrap();

// `sqlite` is listed after `files` in the table, but it has the preferred capability, so it ranks first.
assert_eq!(eval.satisfying().map(|e| e.provider.as_str()).collect::<Vec<_>>(), ["sqlite", "files"]);
let why = eval.check("mem").unwrap_err().to_string();
assert!(why.contains("kv.atomic_replace [unverified (no evidence)]"), "{why}");
```

Matrices also live in a plain text file, so a reviewer can read what each backend is claimed to do and CI can fail when the file drifts. You supply a `Prober` that runs one real check per provider and capability; `generate` runs it, `write` makes the file, and `check` compares a committed file with a fresh run:

```rust
use capant::*;

struct Probes;
impl Prober for Probes {
    fn can_probe(&self, _p: &Provider) -> bool { true }
    fn probe(&self, p: &Provider, cap: &CapabilityId, _ctx: &Context) -> Outcome {
        let proof = Evidence::probe(format!("{}::{}", p.id(), cap.as_str()));
        match cap.as_str() {
            "kv.get" => Outcome::Pass(proof),
            "kv.ttl" => Outcome::Fail { reason: Reason::Missing, evidence: proof },
            // Nothing to try yet: written as `enabled=no`, reason `no probe for this capability yet`.
            _ => Outcome::Unverified { why: Why::Untested, evidence: Evidence::default() },
        }
    }
}

let id = |s: &str| CapabilityId::new(s).unwrap();
let providers = [Provider::new("mem", vec![Surface::new("mem", "1", Config::new())]).unwrap()];
let caps = [id("kv.get"), id("kv.ttl"), id("kv.watch")];
let policy = Policy { max_age_secs: Some(3600), ..Policy::default() };
let at = |now| Context { policy: policy.clone(), ..Context::new(now, Config::new()) };

// Commit this text (`std::fs::write("capabilities/matrix.tsv", &committed)`).
let committed = write(&generate(&providers, &caps, &Probes, &Matrix::new(), &at(1_000)));

// CI: probe again and compare. Same moment: ok.
let fresh = generate(&providers, &caps, &Probes, &Matrix::new(), &at(1_000));
assert!(check(&committed, &fresh, &at(1_000)).is_ok());

// Rows older than `max_age_secs` are listed in `stale` and fail the check, so re-probe and re-commit.
let later = at(1_000 + 7_200);
let fresh = generate(&providers, &caps, &Probes, &Matrix::new(), &later);
let report = check(&committed, &fresh, &later);
assert!(!report.is_ok() && !report.stale.is_empty());
```

`docs/CI-CHECK.md` turns this into a runnable CI check with its GitHub Actions workflow; `docs/DESIGN.md` has the file format and the reasoning.

Status: early release, API may still change before 0.1. The newest version on crates.io may be older than this README; the example works on it either way. See `ROADMAP.md`.

Part of the ant family of shared building blocks (names and rulings: PORTFOLIO-ROADMAP.md in the PM's tree). Licensed under MIT OR Apache-2.0.
