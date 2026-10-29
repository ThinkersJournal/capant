// SPDX-License-Identifier: MIT OR Apache-2.0
//! A consumer's `capant check`, for CI (`docs/CI-CHECK.md`).
//!
//! ```text
//! capant_check [--write] <matrix.tsv> [extra-capability ...]
//! ```
//!
//! Probes every provider for every capability in [`CAPS`], then compares the result with the
//! committed matrix file. Exit 0: the file is exactly what the probes produce. Exit 1: it
//! drifted, or a declared capability has no probe. Exit 2: usage. `--write` rewrites the file.
//! Extra capabilities on the command line are declared for this run only; it is the quick way to
//! see the check fail.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::{SystemTime, UNIX_EPOCH};

use capant::*;

/// The capabilities this project relies on. Add one here and the check fails until a probe
/// covers it and the matrix file records it.
const CAPS: [&str; 3] = ["kv.get", "kv.put", "kv.durable"];

/// The providers this project can use.
const PROVIDERS: [&str; 2] = ["mem", "files"];

/// A key-value store: in memory, or one file per key in a directory.
enum Store {
    Mem(HashMap<String, String>),
    Files(PathBuf),
}

impl Store {
    fn open(provider: &str, dir: &Path) -> Store {
        match provider {
            "files" => Store::Files(dir.to_path_buf()),
            _ => Store::Mem(HashMap::new()),
        }
    }

    fn put(&mut self, key: &str, value: &str) {
        match self {
            Store::Mem(m) => {
                m.insert(key.into(), value.into());
            }
            Store::Files(dir) => {
                let _ = fs::create_dir_all(&*dir);
                let _ = fs::write(dir.join(key), value);
            }
        }
    }

    fn get(&self, key: &str) -> Option<String> {
        match self {
            Store::Mem(m) => m.get(key).cloned(),
            Store::Files(dir) => fs::read_to_string(dir.join(key)).ok(),
        }
    }
}

struct Probes {
    dir: PathBuf,
}

impl Prober for Probes {
    fn can_probe(&self, _: &Provider) -> bool {
        true
    }

    fn probe(&self, p: &Provider, cap: &CapabilityId, _: &Context) -> Outcome {
        let evidence = Evidence::probe(format!("{}::{}", p.id(), cap));
        let dir = self.dir.join(p.id());
        let mut store = Store::open(p.id(), &dir);
        // The control for every probe below: a key nobody wrote reads back as nothing.
        let control = store.get("absent").is_none();
        store.put("k", "v");
        let outcome = match cap.as_str() {
            "kv.get" | "kv.put" => store.get("k").as_deref() == Some("v"),
            // A second handle on the same provider still sees the value.
            "kv.durable" => Store::open(p.id(), &dir).get("k").as_deref() == Some("v"),
            _ => {
                return Outcome::Unverified {
                    why: Why::Untested,
                    evidence: Evidence::default(),
                };
            }
        };
        let _ = fs::remove_dir_all(&dir);
        match (control, outcome) {
            (false, _) => Outcome::Unverified {
                why: Why::NoControl,
                evidence,
            },
            (true, true) => Outcome::Pass(evidence),
            (true, false) => Outcome::Fail {
                reason: Reason::Missing,
                evidence,
            },
        }
    }
}

fn main() -> ExitCode {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    let rewrite = args.first().is_some_and(|a| a == "--write");
    if rewrite {
        args.remove(0);
    }
    let Some(path) = args.first() else {
        eprintln!("usage: capant_check [--write] <matrix.tsv> [extra-capability ...]");
        return ExitCode::from(2);
    };
    let mut caps = Vec::new();
    for name in CAPS
        .iter()
        .copied()
        .chain(args[1..].iter().map(String::as_str))
    {
        match CapabilityId::new(name) {
            Ok(c) => caps.push(c),
            Err(e) => {
                eprintln!("capability {name:?}: {e}");
                return ExitCode::from(2);
            }
        }
    }
    let table: Vec<Provider> = PROVIDERS
        .iter()
        .map(|n| Provider::new(*n, vec![Surface::new(*n, "1", Config::new())]).unwrap())
        .collect();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs() as i64);
    // No environment is recorded, so one committed file holds on every CI runner.
    let ctx = Context::new(now, Config::new());
    let probes = Probes {
        dir: std::env::temp_dir().join(format!("capant-check-{}", std::process::id())),
    };

    let text = fs::read_to_string(path).unwrap_or_default();
    let committed = parse(&text).map(|p| p.matrix).unwrap_or_default();
    let generated = generate(&table, &caps, &probes, &committed, &ctx);
    if rewrite {
        if let Err(e) = fs::write(path, capant::write(&generated)) {
            eprintln!("{path}: {e}");
            return ExitCode::from(2);
        }
        println!("wrote {path}");
        return ExitCode::SUCCESS;
    }

    let report = check(&text, &generated, &ctx);
    let mut problems = Vec::new();
    if let Some(e) = &report.parse_error {
        problems.push(format!("{path}: {e}"));
    }
    problems.extend(report.warnings.iter().map(|w| format!("{path}: {w:?}")));
    problems.extend(
        report
            .missing
            .iter()
            .map(|l| format!("{path} lacks what the probes found: {l}")),
    );
    problems.extend(
        report
            .unexpected
            .iter()
            .map(|l| format!("{path} claims what the probes do not find: {l}")),
    );
    problems.extend(report.stale.iter().map(|s| format!("stale: {s}")));
    // A declared capability whose probe could not conclude is a hole, even if the file agrees.
    for r in generated.rows() {
        if matches!(r.state, State::Unverified(_)) {
            problems.push(format!(
                "unprobed: {} on {} ({})",
                r.subject, r.backend, r.state
            ));
        }
    }
    for p in &problems {
        eprintln!("{p}");
    }
    if problems.is_empty() {
        println!("{path}: {} rows match the probes", generated.rows().len());
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    }
}
