// SPDX-License-Identifier: MIT OR Apache-2.0
//! The matrix file, `generate` and `check` (`docs/DESIGN.md` sections 5 and 9). Each rule has a
//! test and a control: the same input varied in the one way that must change the answer.

use capant::*;

const ANTSPAWN: &str = include_str!("fixtures/antspawn-launchers.tsv");
const PERSISTANT: &str = include_str!("fixtures/persistant-sqlx-sqlite.tsv");
const H9: &str =
    "subject_kind\tsubject\tbackend\tversion\tconfig\tenabled\treason\tmetadata\tnote\n";

fn id(s: &str) -> CapabilityId {
    CapabilityId::new(s).unwrap()
}

fn lf(s: &str) -> String {
    s.replace("\r\n", "\n")
}

fn ok(text: &str) -> Parsed {
    parse(text).unwrap_or_else(|e| panic!("{e}"))
}

// Compatibility: the two consumers' checked-in files.
#[test]
fn antspawns_checked_in_file_parses_and_rewrites_byte_for_byte() {
    let text = lf(ANTSPAWN);
    let p = ok(&text);
    assert!(p.warnings.is_empty());
    assert_eq!(p.matrix.rows().len(), 24);
    assert_eq!(
        write(&p.matrix),
        text,
        "the canonical writer reproduces AntSpawn's file"
    );
    let account_read = p
        .matrix
        .rows()
        .iter()
        .find(|r| r.backend == "account" && r.subject.as_str() == "network.off")
        .unwrap();
    assert_eq!(account_read.state, State::Yes);
    assert_eq!(account_read.evidence.probe.as_deref(), Some("connect"));
    let sandbox = p
        .matrix
        .rows()
        .iter()
        .find(|r| r.backend == "windows-sandbox")
        .unwrap();
    assert_eq!(sandbox.state, State::Unverified(Why::NotSetUp));
    let untested = p
        .matrix
        .rows()
        .iter()
        .find(|r| r.backend == "naive" && r.subject.as_str() == "gpu.cuda")
        .unwrap();
    assert_eq!(untested.state, State::Unverified(Why::Untested));
}

#[test]
fn persistants_checked_in_file_parses_and_round_trips_by_content() {
    let p = ok(&lf(PERSISTANT));
    assert!(p.warnings.is_empty());
    assert_eq!(p.matrix.rows().len(), 61);
    let again = ok(&write(&p.matrix));
    assert_eq!(again.matrix, p.matrix);
    let get = p
        .matrix
        .rows()
        .iter()
        .find(|r| r.subject.as_str() == "kv.ttl")
        .unwrap();
    assert_eq!(get.state, State::Yes);
    assert_eq!(
        get.evidence.probe.as_deref(),
        Some("kv::kv.ttl"),
        "`test=` is the probe id"
    );
    assert_eq!(
        get.metadata.get("sleep_ms"),
        Some("1200"),
        "other metadata is kept"
    );
    assert_eq!(get.config.get("dialect"), Some("sqlite"));
    assert!(
        get.evidence.environment.is_empty(),
        "os/arch in old metadata stay informational; they are not a claim about where the row holds"
    );
    let off = p
        .matrix
        .rows()
        .iter()
        .find(|r| r.state != State::Yes)
        .unwrap();
    assert_eq!(off.state, State::No(Reason::Missing));
}

#[test]
fn old_files_certify_through_the_engine() {
    let m = ok(&lf(ANTSPAWN)).matrix;
    let provider = |n: &str| Provider::new(n, vec![Surface::new(n, "1", Config::new())]).unwrap();
    let table = [
        provider("naive"),
        provider("account"),
        provider("windows-sandbox"),
    ];
    let need = Need::new(vec![id("network.off"), id("isolation.no-read-outside")]);
    let e = evaluate(&table, &need, &m, &Context::new(0, Config::new())).unwrap();
    let names: Vec<&str> = e.satisfying().map(|e| e.provider.as_str()).collect();
    assert_eq!(names, ["account"]);
    let need = Need::new(vec![id("gpu.cuda")]);
    let e = evaluate(&table, &need, &m, &Context::new(0, Config::new())).unwrap();
    assert!(
        e.satisfying().next().is_none(),
        "no provider is certified for an untested capability"
    );
}

// A malformed row is an error, never a skipped row.
#[test]
fn malformed_files_are_errors_not_silence() {
    assert_eq!(parse(""), Err(ParseError::NoHeader));
    assert_eq!(parse("# only a comment\n"), Err(ParseError::NoHeader));
    assert_eq!(
        parse("subject\tbackend\n"),
        Err(ParseError::MissingColumn("subject_kind"))
    );
    let dup = H9.replace("note\n", "note\tbackend\n");
    assert!(matches!(parse(&dup), Err(ParseError::DuplicateColumn(c)) if c == "backend"));
    let row = |cells: &str| format!("{H9}{cells}\n");
    assert!(matches!(
        parse(&row("capability\tk\tb")),
        Err(ParseError::FieldCount {
            line: 2,
            want: 9,
            got: 3
        })
    ));
    assert!(matches!(
        parse(&row("capability\tk\tb\t1\t\tmaybe\t\t\t")),
        Err(ParseError::BadEnabled { line: 2, .. })
    ));
    assert!(matches!(
        parse(&row("capability\tK\tb\t1\t\tyes\t\t\t")),
        Err(ParseError::BadId { line: 2, .. })
    ));
    assert_eq!(
        parse(&row("capability\tk\tb\t1\t\tno\t\t\t")),
        Err(ParseError::ReasonMismatch { line: 2 })
    );
    assert_eq!(
        parse(&row("capability\tk\tb\t1\t\tyes\twrong\t\t")),
        Err(ParseError::ReasonMismatch { line: 2 })
    );
    assert!(matches!(
        parse(&row("capability\tk\tb\t1\tnot-a-pair\tyes\t\t\t")),
        Err(ParseError::BadConfig {
            column: "config",
            ..
        })
    ));
    assert!(
        parse(&row("capability\tk\tb\t1\t\tyes\t\tprobe=p\t")).is_ok(),
        "control: a good row parses"
    );
}

#[test]
fn optional_and_unknown_columns_are_read_by_name_and_kept() {
    let text = "# note\nsubject_kind\tsubject\tbackend\tversion\tconfig\tenabled\treason\tmetadata\tnote\tfrom\tprobed_at\ttool_version\tenvironment\tinherited_from\n\
                capability\tk\tb\t1\t\tyes\t\tprobe=p\t\tpostgres\t2023-11-14T22:13:20Z\t1.2\tos=linux\t?\n";
    let p = ok(text);
    let r = &p.matrix.rows()[0];
    assert_eq!(r.extra.get("from").map(String::as_str), Some("postgres"));
    assert_eq!(r.evidence.probed_at, Some(1_700_000_000));
    assert_eq!(r.evidence.tool_version.as_deref(), Some("1.2"));
    assert_eq!(r.evidence.environment.get("os"), Some("linux"));
    assert_eq!(
        r.evidence.inherited_from,
        Some(Config::new()),
        "`?` is an unknown origin"
    );
    assert_eq!(ok(&write(&p.matrix)).matrix, p.matrix);

    let bad = text.replace("2023-11-14T22:13:20Z", "yesterday");
    assert!(matches!(
        parse(&bad),
        Err(ParseError::BadTimestamp { line: 3, .. })
    ));
}

#[test]
fn every_state_round_trips() {
    let states = [
        State::Yes,
        State::No(Reason::Missing),
        State::No(Reason::Wrong),
        State::No(Reason::Other("job read a host file".into())),
        State::Unverified(Why::NoRow),
        State::Unverified(Why::NoEvidence),
        State::Unverified(Why::Stale),
        State::Unverified(Why::OtherEnvironment),
        State::Unverified(Why::NoControl),
        State::Unverified(Why::NotSetUp),
        State::Unverified(Why::Untested),
        State::Unverified(Why::Other("this host cannot reach the address".into())),
    ];
    let rows: Vec<Row> = states
        .iter()
        .enumerate()
        .map(|(i, s)| Row::capability(id(&format!("c{i}")), "b", "1", s.clone()))
        .collect();
    let m = Matrix::from_rows(rows);
    assert_eq!(ok(&write(&m)).matrix, m);
}

#[test]
fn text_with_tabs_and_newlines_cannot_break_a_row() {
    let mut r = Row::capability(
        id("k"),
        "b",
        "1",
        State::No(Reason::Other("a\tb\nc".into())),
    );
    r.note = "x\ty".into();
    let text = write(&Matrix::from_rows(vec![r]));
    let p = ok(&text);
    assert_eq!(p.matrix.rows().len(), 1);
    assert_eq!(p.matrix.rows()[0].note, "x y");
}

#[test]
fn duplicate_rows_are_kept_with_a_warning_and_no_wins() {
    let text = format!(
        "{H9}capability\tk\tb\t1\t\tyes\t\tprobe=p\t\ncapability\tk\tb\t1\t\tno\twrong\t\t\n"
    );
    let p = ok(&text);
    assert_eq!(p.matrix.rows().len(), 2);
    assert_eq!(p.warnings.len(), 1);
    let prov = Provider::new("b", vec![Surface::new("b", "1", Config::new())]).unwrap();
    let r = resolve(&p.matrix, &prov, &id("k"), &Context::default(), false);
    assert_eq!(r.state, State::No(Reason::Wrong));
    let single = ok(&format!("{H9}capability\tk\tb\t1\t\tyes\t\tprobe=p\t\n"));
    assert!(
        single.warnings.is_empty(),
        "control: no duplicate, no warning"
    );
}

// Generation and --check (SCOPE 4).

struct Fake {
    reachable: Vec<&'static str>,
    broken: Vec<&'static str>,
    no_control: bool,
}

impl Prober for Fake {
    fn can_probe(&self, p: &Provider) -> bool {
        self.reachable.contains(&p.id())
    }
    fn probe(&self, p: &Provider, cap: &CapabilityId, _: &Context) -> Outcome {
        match cap.as_str() {
            "net" if self.no_control => Outcome::Unverified {
                why: Why::NoControl,
                evidence: Evidence::probe("connect"),
            },
            "none" => Outcome::Unverified {
                why: Why::Untested,
                evidence: Evidence::default(),
            },
            "anon" => Outcome::Pass(Evidence::default()),
            c if self.broken.contains(&p.id()) => Outcome::Fail {
                reason: Reason::Other(format!("{c} broke")),
                evidence: Evidence::probe(c),
            },
            c => Outcome::Pass(Evidence::probe(c)),
        }
    }
}

fn table() -> Vec<Provider> {
    ["naive", "account"]
        .iter()
        .map(|n| Provider::new(*n, vec![Surface::new(*n, "1", Config::new())]).unwrap())
        .collect()
}

fn caps() -> Vec<CapabilityId> {
    ["read", "net", "none", "anon"]
        .iter()
        .map(|c| id(c))
        .collect()
}

fn ctx(now: i64) -> Context {
    let mut c = Context::new(now, Config::new().with("os", "linux"));
    c.tool_versions.insert("read".into(), "9".into());
    c
}

fn gen_with(f: &Fake, committed: &Matrix, c: &Context) -> Matrix {
    generate(&table(), &caps(), f, committed, c)
}

fn state(m: &Matrix, backend: &str, cap: &str) -> State {
    m.rows()
        .iter()
        .find(|r| r.backend == backend && r.subject.as_str() == cap)
        .unwrap()
        .state
        .clone()
}

#[test]
fn rule1_generate_never_says_yes_without_a_probe_that_named_itself() {
    let f = Fake {
        reachable: vec!["naive"],
        broken: vec![],
        no_control: false,
    };
    let m = gen_with(&f, &Matrix::new(), &ctx(100));
    assert_eq!(
        state(&m, "naive", "read"),
        State::Yes,
        "control: a passing named probe is yes"
    );
    assert_eq!(
        state(&m, "naive", "anon"),
        State::Unverified(Why::NoEvidence)
    );
    assert_eq!(state(&m, "naive", "none"), State::Unverified(Why::Untested));
    let row = m
        .rows()
        .iter()
        .find(|r| r.subject.as_str() == "read" && r.backend == "naive")
        .unwrap();
    assert_eq!(row.evidence.probed_at, Some(100));
    assert_eq!(
        row.evidence.tool_version.as_deref(),
        Some("9"),
        "stamped from the context"
    );
    assert_eq!(row.evidence.environment.get("os"), Some("linux"));
}

#[test]
fn a_failed_control_is_never_yes_and_never_no() {
    let with = Fake {
        reachable: vec!["naive"],
        broken: vec![],
        no_control: true,
    };
    let m = gen_with(&with, &Matrix::new(), &ctx(0));
    assert_eq!(state(&m, "naive", "net"), State::Unverified(Why::NoControl));
    let without = Fake {
        reachable: vec!["naive"],
        broken: vec![],
        no_control: false,
    };
    let m = gen_with(&without, &Matrix::new(), &ctx(0));
    assert_eq!(
        state(&m, "naive", "net"),
        State::Yes,
        "control: with a working control it passes"
    );
    let broken = Fake {
        reachable: vec!["naive"],
        broken: vec!["naive"],
        no_control: false,
    };
    let m = gen_with(&broken, &Matrix::new(), &ctx(0));
    assert_eq!(
        state(&m, "naive", "read"),
        State::No(Reason::Other("read broke".into())),
        "control: a failing probe is no, so the mapping is not constant"
    );
}

#[test]
fn rule4_generate_write_parse_write_is_byte_identical_and_check_passes() {
    let f = Fake {
        reachable: vec!["naive", "account"],
        broken: vec!["naive"],
        no_control: false,
    };
    let m = gen_with(&f, &Matrix::new(), &ctx(1_700_000_000));
    let text = write(&m);
    let again = write(&ok(&text).matrix);
    assert_eq!(again, text);
    assert!(check(&text, &m, &ctx(1_700_000_000)).is_ok());
}

#[test]
fn rule4_check_fails_on_a_flipped_cell_and_ignores_only_probed_at() {
    let f = Fake {
        reachable: vec!["naive", "account"],
        broken: vec![],
        no_control: false,
    };
    let c = ctx(1_700_000_000);
    let m = gen_with(&f, &Matrix::new(), &c);
    let text = write(&m);

    let later = gen_with(&f, &Matrix::new(), &ctx(1_700_086_400));
    assert_ne!(write(&later), text, "the later run differs in probed_at");
    assert!(
        check(&text, &later, &c).is_ok(),
        "probed_at alone does not fail --check"
    );

    let flipped = text.replacen("\tyes\t\t", "\tno\twrong\t", 1);
    let report = check(&flipped, &m, &c);
    assert!(!report.is_ok());
    assert_eq!(report.missing.len(), 1);
    assert_eq!(report.unexpected.len(), 1);

    let other_probe = text.replacen("probe=read", "probe=readx", 1);
    assert!(
        !check(&other_probe, &m, &c).is_ok(),
        "the probe id is compared"
    );
    let other_tool = text.replacen("\t9\t", "\t8\t", 1);
    assert!(
        !check(&other_tool, &m, &c).is_ok(),
        "the tool version is compared"
    );

    let mut lines: Vec<&str> = text.lines().collect();
    lines.swap(1, 2);
    let report = check(&(lines.join("\n") + "\n"), &m, &c);
    assert!(report.reordered && report.missing.is_empty() && report.unexpected.is_empty());

    assert!(check("garbage", &m, &c).parse_error.is_some());
}

#[test]
fn check_fails_on_duplicates_and_on_stale_yes_rows() {
    let f = Fake {
        reachable: vec!["naive", "account"],
        broken: vec![],
        no_control: false,
    };
    let c = ctx(1_700_000_000);
    let m = gen_with(&f, &Matrix::new(), &c);
    let text = write(&m);

    let mut dup: Vec<&str> = text.lines().collect();
    dup.push(dup[1]);
    let report = check(&(dup.join("\n") + "\n"), &m, &c);
    assert_eq!(report.warnings.len(), 1);
    assert!(!report.is_ok());

    let mut policy = ctx(1_700_000_000 + 10_000);
    policy.policy.max_age_secs = Some(5_000);
    let stale = check(&text, &m, &policy);
    assert!(
        !stale.stale.is_empty() && !stale.is_ok(),
        "committed yes rows older than the window"
    );
    let mut fresh = ctx(1_700_000_000 + 10_000);
    fresh.policy.max_age_secs = Some(50_000);
    assert!(
        check(&text, &m, &fresh).is_ok(),
        "control: only the window changed"
    );
}

#[test]
fn carry_over_keeps_whole_sound_rows_marked_inherited_and_rebuilds_the_rest() {
    let win = Config::new().with("os", "windows");
    let probed_on_windows = Fake {
        reachable: vec!["naive", "account"],
        broken: vec![],
        no_control: false,
    };
    let mut win_ctx = ctx(500);
    win_ctx.environment = win.clone();
    let committed = gen_with(&probed_on_windows, &Matrix::new(), &win_ctx);

    // On Linux `account` cannot be probed: its committed rows are carried over, marked inherited.
    let linux_only = Fake {
        reachable: vec!["naive"],
        broken: vec![],
        no_control: false,
    };
    let regen = gen_with(&linux_only, &committed, &ctx(900));
    let carried = regen
        .rows()
        .iter()
        .find(|r| r.backend == "account" && r.subject.as_str() == "read")
        .unwrap();
    assert_eq!(carried.state, State::Yes);
    assert_eq!(carried.evidence.inherited_from, Some(win.clone()));
    assert_eq!(
        carried.evidence.probed_at,
        Some(500),
        "never re-stamped: it was not measured here"
    );

    let prov = |n: &str| Provider::new(n, vec![Surface::new(n, "1", Config::new())]).unwrap();
    let need = Need::new(vec![id("read")]);
    let on_linux = evaluate(&[prov("account")], &need, &regen, &ctx(900)).unwrap();
    assert!(
        on_linux.satisfying().next().is_none(),
        "a Windows yes does not count on Linux"
    );
    let consent = evaluate(
        &[prov("account")],
        &need.clone().allow_inherited(),
        &regen,
        &ctx(900),
    )
    .unwrap();
    assert!(
        consent.satisfying().next().is_some(),
        "control: with consent it does, marked inherited"
    );
    let mut win_now = ctx(900);
    win_now.environment = win;
    let at_home = evaluate(&[prov("account")], &need, &regen, &win_now).unwrap();
    assert!(
        at_home.satisfying().next().is_some(),
        "and it counts without consent on its own environment"
    );

    // Incomplete committed rows are not carried: rebuilt as not set up.
    let mut partial = committed.clone();
    let rows: Vec<Row> = partial
        .rows()
        .iter()
        .filter(|r| r.subject.as_str() != "net")
        .cloned()
        .collect();
    partial = Matrix::from_rows(rows);
    let regen = gen_with(&linux_only, &partial, &ctx(900));
    assert_eq!(
        state(&regen, "account", "read"),
        State::Unverified(Why::NotSetUp)
    );

    // A committed yes without a probe makes the whole set unsound.
    let mut unsound = committed.rows().to_vec();
    for r in &mut unsound {
        if r.backend == "account" && r.subject.as_str() == "read" {
            r.evidence.probe = None;
        }
    }
    let regen = gen_with(&linux_only, &Matrix::from_rows(unsound), &ctx(900));
    assert_eq!(
        state(&regen, "account", "net"),
        State::Unverified(Why::NotSetUp)
    );

    // Nothing committed at all.
    let regen = gen_with(&linux_only, &Matrix::new(), &ctx(900));
    assert_eq!(
        state(&regen, "account", "read"),
        State::Unverified(Why::NotSetUp)
    );
}

#[test]
fn carried_rows_are_listed_but_a_tampered_one_does_not_fail_check() {
    let win = Config::new().with("os", "windows");
    let probed = Fake {
        reachable: vec!["naive", "account"],
        broken: vec![],
        no_control: false,
    };
    let mut win_ctx = ctx(500);
    win_ctx.environment = win;
    let committed = gen_with(&probed, &Matrix::new(), &win_ctx);

    let linux_only = Fake {
        reachable: vec!["naive"],
        broken: vec![],
        no_control: false,
    };
    let c = ctx(900);
    let regen = gen_with(&linux_only, &committed, &c);
    let text = write(&regen);
    let report = check(&text, &regen, &c);
    assert!(report.is_ok());
    assert!(
        report.carried.contains(&"read on account".to_owned()),
        "{:?}",
        report.carried
    );
    assert!(
        report.carried.iter().all(|l| l.ends_with("on account")),
        "only the carried provider is listed (control: naive was probed here)"
    );

    // Known limit: the carried rows are copied from the committed file, so editing one of them
    // there cannot be seen from here. It is listed, not failed.
    let tampered: String = text
        .lines()
        .map(|l| {
            if l.contains("	account	") && l.contains("probe=read") {
                format!(
                    "{}
",
                    l.replace("probe=read", "probe=forged")
                )
            } else {
                format!(
                    "{l}
"
                )
            }
        })
        .collect();
    assert_ne!(tampered, text);
    let regen_from_tampered = gen_with(&linux_only, &ok(&tampered).matrix, &c);
    let report = check(&tampered, &regen_from_tampered, &c);
    assert!(report.is_ok(), "{report:?}");
    assert!(!report.carried.is_empty());

    // Control: the same edit to a row that WAS probed here is caught.
    let naive_edit = text.replacen(
        "naive\t1\t\tyes\t\tprobe=read",
        "naive\t1\t\tyes\t\tprobe=forged",
        1,
    );
    assert_ne!(naive_edit, text, "the edit applied");
    assert!(!check(&naive_edit, &regen, &c).is_ok());
}
