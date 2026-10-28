# CapAnt

Pick the backend that can really do the job, and know why. You list what you NEED; providers record what they can do, with evidence; CapAnt returns the providers that qualify, ranked, and says why each other one did not. No evidence means `unverified`, never "probably yes".

```rust
use capant::*;

let id = |s: &str| CapabilityId::new(s).unwrap();
let provider = |name: &str| Provider::new(name, vec![Surface::new(name, "1", Config::new())]).unwrap();

// What each backend can do, and the probe that showed it.
let mut matrix = Matrix::new();
matrix.push(Row::capability(id("kv.get"), "mem", "1", State::Yes).evidence(Evidence::probe("kv::get")));

// What I need, then ask.
let need = Need::new(vec![id("kv.get")]);
let eval = evaluate(&[provider("mem"), provider("fs")], &need, &matrix, &Context::new(0, Config::new())).unwrap();

assert_eq!(eval.satisfying().map(|e| e.provider.as_str()).collect::<Vec<_>>(), ["mem"]);
assert!(eval.check("fs").is_err()); // `fs` has no evidence, so it is not a candidate
```

Add it with `cargo add capant`. This block is compiled and run by `cargo test`.

Matrices also live in a plain text file (`parse`, `write`, `generate`, `check`), so a reviewer can read what each backend is claimed to do and CI can fail when the file drifts. See `docs/DESIGN.md`.

Status: early release (version 0.0.9), API may still change before 0.1. See `ROADMAP.md`.

Part of the ant family of shared building blocks (names and rulings: PORTFOLIO-ROADMAP.md in the PM's tree). Licensed under MIT OR Apache-2.0.
