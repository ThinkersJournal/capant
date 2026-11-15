// SPDX-License-Identifier: MIT OR Apache-2.0
//! The consumer's CI check (`examples/capant_check`, `docs/CI-CHECK.md`) run as a process: it
//! passes on the committed matrix and fails on each way the file or the probes can go wrong.
//! Each failure case differs from the passing one in one thing.

use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::OnceLock;

const COMMITTED: &str = include_str!("../examples/capant_check/matrix.tsv");

fn example() -> PathBuf {
    // Ask cargo where the example went: the build directory layout is cargo's to change (1.100
    // moved it), and building here also means the binary is never older than its source.
    static EXE: OnceLock<PathBuf> = OnceLock::new();
    EXE.get_or_init(|| {
        let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
        let out = Command::new(cargo)
            .args([
                "build",
                "--example",
                "capant_check",
                "--message-format=json",
            ])
            .output()
            .expect("run cargo build --example");
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        let text = String::from_utf8_lossy(&out.stdout).into_owned();
        let exe = text
            .lines()
            .filter(|l| l.contains("capant_check") && l.contains("\"executable\":\""))
            .filter_map(|l| {
                let rest = l.split("\"executable\":\"").nth(1)?;
                Some(rest.split('"').next()?.replace("\\\\", "\\"))
            })
            .next_back()
            .unwrap_or_else(|| panic!("no executable in cargo output: {text}"));
        let p = PathBuf::from(exe);
        assert!(p.exists(), "example not built: {}", p.display());
        p
    })
    .clone()
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
