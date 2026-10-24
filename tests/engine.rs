// SPDX-License-Identifier: MIT OR Apache-2.0
//! One test per rule of `docs/SCOPE.md` and per consumer behaviour of `docs/DESIGN.md` section 7,
//! each with a negative control (the same input varied in one way that must change the answer).

use capant::*;

fn id(s: &str) -> CapabilityId {
    CapabilityId::new(s).unwrap()
}

fn provider(name: &str) -> Provider {
    Provider::new(name, vec![Surface::new(name, "1", Config::new())]).unwrap()
}

fn linux() -> Config {
    Config::new().with("os", "linux")
}

fn yes(cap: &str, backend: &str, ev: Evidence) -> Row {
    Row::capability(id(cap), backend, "1", State::Yes).evidence(ev)
}

fn no(cap: &str, backend: &str, why: Reason) -> Row {
    Row::capability(id(cap), backend, "1", State::No(why))
}

fn ctx() -> Context {
    Context::new(1_000, linux())
}

fn names(e: &Evaluation) -> Vec<&str> {
    e.entries().iter().map(|e| e.provider.as_str()).collect()
}

fn satisfying(e: &Evaluation) -> Vec<&str> {
    e.satisfying().map(|e| e.provider.as_str()).collect()
}

fn run(table: &[Provider], need: &Need, rows: Vec<Row>, ctx: &Context) -> Evaluation {
    evaluate(table, need, &Matrix::from_rows(rows), ctx).unwrap()
}

fn state_of(rows: Vec<Row>, cap: &str, ctx: &Context) -> State {
    resolve(
        &Matrix::from_rows(rows),
        &provider("a"),
        &id(cap),
        ctx,
        false,
    )
    .state
}

// SCOPE 1: unverified is the default; `yes` needs evidence.
#[test]
fn rule1_yes_requires_recorded_evidence() {
    let need = Need::new(vec![id("x")]);
    let t = [provider("a")];
    let with = run(&t, &need, vec![yes("x", "a", Evidence::probe("p"))], &ctx());
    assert_eq!(
        satisfying(&with),
        ["a"],
        "control: a yes with a probe satisfies"
    );

    let without = run(&t, &need, vec![yes("x", "a", Evidence::default())], &ctx());
    assert!(satisfying(&without).is_empty());
    assert_eq!(
        state_of(vec![yes("x", "a", Evidence::default())], "x", &ctx()),
        State::Unverified(Why::NoEvidence)
    );
    assert_eq!(
        state_of(vec![], "x", &ctx()),
        State::Unverified(Why::NoRow),
        "no row is unverified, not no"
    );
    assert_eq!(
        state_of(vec![no("x", "a", Reason::Missing)], "x", &ctx()),
        State::No(Reason::Missing),
        "control: a recorded no stays a no"
    );
}

// SCOPE 2: stale rows degrade to unverified.
#[test]
fn rule2_stale_rows_degrade_and_only_yes_degrades() {
    let ev = Evidence::probe("p").at(100).tool_version("1.0");
    let mut c = ctx();
    c.policy.max_age_secs = Some(500);
    assert_eq!(
        state_of(vec![yes("x", "a", ev.clone())], "x", &c),
        State::Unverified(Why::Stale),
        "age 900 > 500"
    );
    c.now = 400;
    assert_eq!(
        state_of(vec![yes("x", "a", ev.clone())], "x", &c),
        State::Yes,
        "control: only the clock changed, age 300 <= 500"
    );
    c.now = 1_000_000;
    assert_eq!(
        state_of(vec![no("x", "a", Reason::Wrong)], "x", &c),
        State::No(Reason::Wrong),
        "an old known-bad is still known-bad"
    );
    assert_eq!(
        state_of(vec![yes("x", "a", Evidence::probe("p"))], "x", &c),
        State::Unverified(Why::Stale),
        "a window is set and the row records no time"
    );
    let mut c = ctx();
    assert_eq!(
        state_of(vec![yes("x", "a", ev.clone())], "x", &c),
        State::Yes
    );
    c.tool_versions.insert("p".into(), "2.0".into());
    assert_eq!(
        state_of(vec![yes("x", "a", ev.clone())], "x", &c),
        State::Unverified(Why::Stale),
        "probe tool version changed"
    );
    c.policy.require_tool_version_match = false;
    assert_eq!(
        state_of(vec![yes("x", "a", ev)], "x", &c),
        State::Yes,
        "control: the version rule is switchable"
    );
}

// SCOPE 3: needs are ordered; the result says why, by need.
#[test]
fn rule3_order_of_preference_decides_and_explanations_name_the_need() {
    let t = [provider("a"), provider("b")];
    let rows = vec![
        yes("req", "a", Evidence::probe("p")),
        yes("req", "b", Evidence::probe("p")),
        yes("fast", "a", Evidence::probe("p")),
        yes("small", "b", Evidence::probe("p")),
    ];
    let fast_first = Need::new(vec![id("req")]).prefer(vec![id("fast"), id("small")]);
    let small_first = Need::new(vec![id("req")]).prefer(vec![id("small"), id("fast")]);
    assert_eq!(
        names(&run(&t, &fast_first, rows.clone(), &ctx())),
        ["a", "b"]
    );
    assert_eq!(
        names(&run(&t, &small_first, rows.clone(), &ctx())),
        ["b", "a"],
        "control: swapping the importance order flips the ranking"
    );

    let e = run(&t, &fast_first, rows, &ctx());
    let lines = e.explain("b").unwrap();
    assert!(
        lines
            .iter()
            .any(|l| l.starts_with("fast: unverified (no row)")),
        "{lines:?}"
    );
    assert!(
        lines.iter().any(|l| l.starts_with("small: yes (probe p)")),
        "{lines:?}"
    );
}

#[test]
fn a_preferred_capability_never_excludes() {
    let t = [provider("a")];
    let need = Need::new(vec![id("req")]).prefer(vec![id("nice")]);
    let e = run(
        &t,
        &need,
        vec![yes("req", "a", Evidence::probe("p"))],
        &ctx(),
    );
    assert_eq!(satisfying(&e), ["a"]);
    let need = Need::new(vec![id("req"), id("nice")]);
    let e = run(
        &t,
        &need,
        vec![yes("req", "a", Evidence::probe("p"))],
        &ctx(),
    );
    assert!(
        satisfying(&e).is_empty(),
        "control: the same capability required does exclude"
    );
}

// SCOPE 5: another environment does not count unless marked inherited, and then only on request.
#[test]
fn rule5_other_environment_needs_an_inherited_mark_and_consent() {
    let win = Config::new().with("os", "windows");
    let foreign = Evidence::probe("p").environment(win.clone());
    assert_eq!(
        state_of(vec![yes("x", "a", foreign)], "x", &ctx()),
        State::Unverified(Why::OtherEnvironment)
    );
    let local = Evidence::probe("p").environment(linux());
    assert_eq!(
        state_of(vec![yes("x", "a", local)], "x", &ctx()),
        State::Yes,
        "control: the same row probed here counts"
    );

    let inherited = Evidence::probe("p")
        .environment(win.clone())
        .inherited_from(win);
    let t = [provider("a")];
    let rows = vec![yes("x", "a", inherited)];
    let need = Need::new(vec![id("x")]);
    let refused = run(&t, &need, rows.clone(), &ctx());
    assert!(satisfying(&refused).is_empty());
    let accepted = run(&t, &need.clone().allow_inherited(), rows, &ctx());
    assert_eq!(satisfying(&accepted), ["a"]);
    assert!(
        accepted.entries()[0].outcomes[0].inherited
            && accepted.entries()[0].outcomes[0]
                .explain()
                .contains("[inherited]"),
        "an accepted inherited yes is marked as such"
    );
}

// SCOPE 6: the engine knows nothing about what capabilities mean.
#[test]
fn rule6_unknown_kinds_of_capability_resolve_the_same_way() {
    let t = [provider("a")];
    let need = Need::new(vec![id("bananas.ripe")]);
    let e = run(
        &t,
        &need,
        vec![yes("bananas.ripe", "a", Evidence::probe("p"))],
        &ctx(),
    );
    assert_eq!(satisfying(&e), ["a"]);
    let e = run(
        &t,
        &Need::new(vec![id("bananas.ripe_")]),
        vec![yes("bananas.ripe", "a", Evidence::probe("p"))],
        &ctx(),
    );
    assert!(
        satisfying(&e).is_empty(),
        "control: a typo'd id is unverified, not a panic"
    );
}

// DESIGN 2: off wins, surfaces, config.
#[test]
fn no_beats_yes_across_rows_and_surfaces() {
    let both = vec![
        yes("x", "a", Evidence::probe("p")),
        no("x", "a", Reason::Wrong),
    ];
    assert_eq!(state_of(both, "x", &ctx()), State::No(Reason::Wrong));
    assert_eq!(
        state_of(vec![yes("x", "a", Evidence::probe("p"))], "x", &ctx()),
        State::Yes,
        "control: remove the no row"
    );

    let layered = Provider::new(
        "kv",
        vec![
            Surface::new("kv-on-sql", "1", Config::new()),
            Surface::new("sql", "1", Config::new()),
        ],
    )
    .unwrap();
    let m = Matrix::from_rows(vec![
        yes("x", "kv-on-sql", Evidence::probe("p")),
        no("x", "sql", Reason::Missing),
    ]);
    assert_eq!(
        resolve(&m, &layered, &id("x"), &ctx(), false).state,
        State::No(Reason::Missing),
        "a no on any surface wins"
    );
}

#[test]
fn rows_match_exact_version_and_config_facts() {
    let p = Provider::new(
        "a",
        vec![Surface::new(
            "a",
            "2",
            Config::new().with("collation", "bin"),
        )],
    )
    .unwrap();
    let ev = Evidence::probe("p");
    let m = |version: &str, config: Config| {
        Matrix::from_rows(vec![
            Row::capability(id("x"), "a", version, State::Yes)
                .config(config)
                .evidence(ev.clone()),
        ])
    };
    let r = |m: &Matrix| resolve(m, &p, &id("x"), &ctx(), false).state;
    assert_eq!(
        r(&m("2", Config::new().with("collation", "bin"))),
        State::Yes
    );
    assert_eq!(
        r(&m("2", Config::new())),
        State::Yes,
        "a row naming no facts holds anywhere"
    );
    assert_eq!(
        r(&m("3", Config::new())),
        State::Unverified(Why::NoRow),
        "other version"
    );
    assert_eq!(
        r(&m("2", Config::new().with("collation", "ci"))),
        State::Unverified(Why::NoRow),
        "control: a different config fact"
    );
}

#[test]
fn rows_of_other_kinds_grant_nothing() {
    let mut row = yes("x", "a", Evidence::probe("p"));
    row.subject_kind = "measurement".into();
    assert_eq!(
        state_of(vec![row], "x", &ctx()),
        State::Unverified(Why::NoRow)
    );
}

#[test]
fn the_strongest_unverified_cause_is_reported() {
    let rows = vec![
        Row::capability(id("x"), "a", "1", State::Unverified(Why::Untested)),
        yes("x", "a", Evidence::default()),
    ];
    assert_eq!(
        state_of(rows, "x", &ctx()),
        State::Unverified(Why::NoEvidence)
    );
}

// DESIGN 3: no closest match.
#[test]
fn there_is_no_closest_match() {
    let t = [provider("a"), provider("b")];
    let need = Need::new(vec![id("x"), id("y")]);
    let rows = vec![
        yes("x", "a", Evidence::probe("p")),
        yes("y", "a", Evidence::probe("p")),
        yes("x", "b", Evidence::probe("p")),
    ];
    let e = run(&t, &need, rows.clone()[..1].to_vec(), &ctx());
    assert!(satisfying(&e).is_empty());
    for p in ["a", "b"] {
        assert!(matches!(e.check(p), Err(CheckError::Rejected(_))), "{p}");
    }
    assert!(matches!(e.check("zzz"), Err(CheckError::Unknown(_))));

    let e = run(&t, &need, rows, &ctx());
    assert!(
        e.check("a").is_ok(),
        "control: one satisfier makes check Ok for it"
    );
    assert!(e.check("b").is_err(), "and only for it");
}

#[test]
fn duplicate_provider_ids_are_refused() {
    let t = [provider("a"), provider("a")];
    assert_eq!(
        evaluate(&t, &Need::default(), &Matrix::new(), &ctx()),
        Err(DuplicateProvider("a".into()))
    );
    assert!(Provider::new("a", vec![]).is_err());
}

// Consumer behaviours (DESIGN section 7), reproduced from each consumer's origin/main code.

/// PersistAnt `provider::providers`: `Err(Unmet)` names every provider and what it lacks, with the
/// matrix word; never a partial list.
#[test]
fn persistant_shape_unmet_lists_every_provider_and_never_a_partial_list() {
    let t = [provider("mem"), provider("fs")];
    let need = Need::new(vec![id("kv.get"), id("kv.ttl")]);
    let rows = vec![
        yes("kv.get", "mem", Evidence::probe("p")),
        yes("kv.get", "fs", Evidence::probe("p")),
        no("kv.ttl", "fs", Reason::Missing),
    ];
    let e = run(&t, &need, rows.clone(), &ctx());
    let unmet = e.satisfying_or_unmet().unwrap_err();
    assert_eq!(
        unmet.to_string(),
        "no provider satisfies the need: [mem: kv.ttl unverified(no row)] [fs: kv.ttl off(missing)]"
    );

    let mut with_ttl = rows;
    with_ttl.push(yes("kv.ttl", "mem", Evidence::probe("p")));
    let e = run(&t, &need, with_ttl, &ctx());
    let ok = e.satisfying_or_unmet().unwrap();
    assert_eq!(
        ok.iter().map(|e| e.provider.as_str()).collect::<Vec<_>>(),
        ["mem"],
        "control: with a satisfier the answer is Ok and holds only the satisfier"
    );

    let empty = evaluate(&[], &need, &Matrix::new(), &ctx()).unwrap();
    assert_eq!(
        empty.satisfying_or_unmet().unwrap_err().to_string(),
        "no provider is registered"
    );
}

/// AntSpawn `Registry::providers`: every launcher, satisfying first, then `prefer` position, then
/// registry order; `unmet` lists the needed capabilities that are not certified.
#[test]
fn antspawn_shape_ranks_every_launcher_with_unmet() {
    let t = [
        provider("naive"),
        provider("account"),
        provider("windows-sandbox"),
        provider("zzz"),
    ];
    let caps = ["network.off", "isolation.no-read-outside"];
    let mut rows = Vec::new();
    for c in caps {
        rows.push(no(
            c,
            "naive",
            Reason::Other("job read a host file outside its environment".into()),
        ));
        rows.push(yes(c, "account", Evidence::probe("read")));
        rows.push(Row::capability(
            id(c),
            "windows-sandbox",
            "1",
            State::Unverified(Why::NotSetUp),
        ));
    }
    rows.push(yes("network.off", "zzz", Evidence::probe("connect")));
    rows.push(yes(
        "isolation.no-read-outside",
        "zzz",
        Evidence::probe("read"),
    ));

    let need = Need::new(caps.iter().map(|c| id(c)).collect());
    let e = run(&t, &need, rows.clone(), &ctx());
    assert_eq!(
        names(&e),
        ["account", "zzz", "naive", "windows-sandbox"],
        "satisfying first, then registry order; rejected keep registry order"
    );

    let prefer = need
        .clone()
        .prefer_providers(vec!["zzz".into(), "windows-sandbox".into()]);
    let e = run(&t, &prefer, rows.clone(), &ctx());
    assert_eq!(
        names(&e),
        ["zzz", "account", "windows-sandbox", "naive"],
        "prefer order inside each group (AntSpawn sorts by (!satisfies, prefer rank, registry))"
    );

    let unmet: Vec<String> = e.entries()[3].unmet().map(|o| o.id.to_string()).collect();
    assert_eq!(unmet, ["network.off", "isolation.no-read-outside"]);
    assert_eq!(e.entries()[0].unmet().count(), 0);

    assert!(
        matches!(e.check("naive"), Err(CheckError::Rejected(_))),
        "open_named re-checks"
    );
    assert!(matches!(e.check("nope"), Err(CheckError::Unknown(_))));
    // Control: certify naive and it ranks into the satisfying group.
    let mut better = rows;
    better.retain(|r| r.backend != "naive");
    for c in caps {
        better.push(yes(c, "naive", Evidence::probe("read")));
    }
    assert_eq!(
        satisfying(&run(&t, &need, better, &ctx())),
        ["naive", "account", "zzz"]
    );
}

/// ConfigurAnt `Module::list_choices`: best first, each with a reason.
#[test]
fn configurant_shape_choices_best_first_with_reasons() {
    let t = [provider("sqlite"), provider("postgres"), provider("mem")];
    let need = Need::new(vec![id("kv.get")]).prefer(vec![id("kv.ttl")]);
    let rows = vec![
        yes("kv.get", "sqlite", Evidence::probe("p")),
        yes("kv.get", "postgres", Evidence::probe("p")),
        yes("kv.ttl", "postgres", Evidence::probe("t")),
    ];
    let choices = run(&t, &need, rows, &ctx()).choices();
    assert_eq!(
        choices.iter().map(|c| c.id.as_str()).collect::<Vec<_>>(),
        ["postgres", "sqlite"],
        "mem lacks kv.get and is not offered; the one with the preferred capability is first"
    );
    assert_eq!(
        choices[0].reason,
        "kv.get: yes (probe p); kv.ttl: yes (probe t)"
    );
    assert!(choices[1].reason.contains("kv.ttl: unverified (no row)"));
}

#[test]
fn consumers_reorder_satisfiers_with_their_own_tie_break() {
    let t = [provider("a"), provider("b"), provider("c")];
    let rows = vec![
        yes("x", "a", Evidence::probe("p")),
        yes("x", "b", Evidence::probe("p")),
    ];
    let mut e = run(&t, &Need::new(vec![id("x")]), rows, &ctx());
    assert_eq!(names(&e), ["a", "b", "c"]);
    e.reorder_satisfiers(|a, b| b.provider.cmp(&a.provider));
    assert_eq!(
        names(&e),
        ["b", "a", "c"],
        "satisfiers reordered, the rejected one stays last"
    );
}
