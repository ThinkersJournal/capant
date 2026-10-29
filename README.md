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

Matrices also live in a plain text file (`parse`, `write`, `generate`, `check`), so a reviewer can read what each backend is claimed to do and CI can fail when the file drifts. See `docs/DESIGN.md`.

Status: early release, API may still change before 0.1. The newest version on crates.io may be older than this README; the example works on it either way. See `ROADMAP.md`.

Part of the ant family of shared building blocks (names and rulings: PORTFOLIO-ROADMAP.md in the PM's tree). Licensed under MIT OR Apache-2.0.
