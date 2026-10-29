// SPDX-License-Identifier: MIT OR Apache-2.0
//! The consumer's CI check (`examples/capant_check`, `docs/CI-CHECK.md`) run as a process: it
//! passes on the committed matrix and fails on each way the file or the probes can go wrong.
//! Each failure case differs from the passing one in one thing.

use std::path::PathBuf;
use std::process::{Command, Output};

const COMMITTED: &str = include_str!("../examples/capant_check/matrix.tsv");

fn example() -> PathBuf {
    // tests run from target/<profile>/deps; `cargo test` builds the examples into ../examples.
    let mut p = std::env::current_exe().unwrap();
    p.pop();
    p.pop();
    p.push("examples");
    p.push(format!("capant_check{}", std::env::consts::EXE_SUFFIX));
    assert!(p.exists(), "example not built: {}", p.display());
    // `cargo test --test ci_check` alone does not rebuild examples: refuse a stale binary.
    let built = std::fs::metadata(&p).and_then(|m| m.modified()).unwrap();
    let edited = std::fs::metadata("examples/capant_check/main.rs")
        .and_then(|m| m.modified())
        .unwrap();
    assert!(
        built >= edited,
        "example is older than its source: run `cargo test`"
    );
    p
}

/// Run the check on `file` text; `extra` capabilities are declared for the run.
fn run(name: &str, file: &str, extra: &[&str]) -> Output {
    let path = std::env::temp_dir().join(format!("capant-ci-{}-{name}.tsv", std::process::id()));
    std::fs::write(&path, file).unwrap();
    let out = Command::new(example())
        .arg(&path)
        .args(extra)
        .output()
        .unwrap();
    let _ = std::fs::remove_file(&path);
    out
}

fn stderr(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).into_owned()
}

#[test]
fn committed_matrix_passes_and_only_probed_at_may_differ() {
    let ok = run("ok", COMMITTED, &[]);
    assert_eq!(ok.status.code(), Some(0), "{}", stderr(&ok));
    let later = COMMITTED.replace("2026-10-29T20:07:00Z", "2030-01-01T00:00:00Z");
    assert_ne!(later, COMMITTED, "the mutation must apply");
    let o = run("later", &later, &[]);
    assert_eq!(o.status.code(), Some(0), "{}", stderr(&o));
}

#[test]
fn a_row_removed_from_the_file_fails() {
    let without: Vec<&str> = COMMITTED
        .lines()
        .filter(|l| !l.contains("files\t1") || !l.contains("kv.durable"))
        .collect();
    assert_eq!(without.len(), COMMITTED.lines().count() - 1);
    let o = run("removed", &without.join("\n"), &[]);
    assert_eq!(o.status.code(), Some(1));
    assert!(stderr(&o).contains("kv.durable"), "{}", stderr(&o));
}

#[test]
fn a_cell_flipped_in_the_file_fails() {
    let flipped = COMMITTED.replace(
        "kv.durable\tmem\t1\t\tno\tmissing",
        "kv.durable\tmem\t1\t\tyes\t",
    );
    assert_ne!(flipped, COMMITTED, "the mutation must apply");
    let o = run("flipped", &flipped, &[]);
    assert_eq!(o.status.code(), Some(1));
}

#[test]
fn a_declared_capability_nobody_probes_fails() {
    let o = run("unprobed", COMMITTED, &["kv.watch"]);
    assert_eq!(o.status.code(), Some(1));
    assert!(
        stderr(&o).contains("unprobed: kv.watch on mem"),
        "{}",
        stderr(&o)
    );
}

#[test]
fn a_file_that_records_the_hole_still_fails() {
    // Regenerate the file with the unprobed capability declared: the file now agrees with the
    // run, and the check must still refuse it.
    let path = std::env::temp_dir().join(format!("capant-ci-{}-hole.tsv", std::process::id()));
    let w = Command::new(example())
        .arg("--write")
        .arg(&path)
        .arg("kv.watch")
        .output()
        .unwrap();
    assert_eq!(w.status.code(), Some(0), "{}", stderr(&w));
    let text = std::fs::read_to_string(&path).unwrap();
    let _ = std::fs::remove_file(&path);
    assert!(text.contains("kv.watch"));
    let o = run("hole", &text, &["kv.watch"]);
    assert_eq!(o.status.code(), Some(1));
    assert!(!stderr(&o).contains("lacks what"), "{}", stderr(&o));
    assert!(stderr(&o).contains("unprobed"), "{}", stderr(&o));
}

#[test]
fn bad_usage_is_exit_2() {
    let o = Command::new(example()).output().unwrap();
    assert_eq!(o.status.code(), Some(2));
}

#[test]
fn the_doc_embeds_the_workflow_file_verbatim() {
    // A Windows checkout may turn both files into CRLF.
    let lf = |s: &str| s.replace("\r\n", "\n");
    let yml = lf(include_str!("../examples/capant_check/capant-check.yml"));
    let doc = lf(include_str!("../docs/CI-CHECK.md"));
    assert!(doc.contains(&format!("```yaml\n{yml}```")));
    // control: the doc does not contain a changed workflow
    assert!(!doc.contains(&format!("```yaml\n{}```", yml.replace("stable", "nightly"))));
}
