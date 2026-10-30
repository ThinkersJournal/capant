// SPDX-License-Identifier: MIT OR Apache-2.0
//! Generating a matrix from probes, and checking a committed file against a fresh run
//! (`docs/DESIGN.md` section 5).

use std::collections::BTreeSet;

use crate::{
    CapabilityId, Context, Evidence, Matrix, ParseError, Provider, Reason, Row, State, Surface,
    Warning, Why, file, resolve,
};

/// What one probe found.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Outcome {
    /// The capability holds. Without a probe id in the evidence it is recorded as unverified.
    Pass(Evidence),
    /// The probe ran and the capability does not hold.
    Fail {
        /// Why.
        reason: Reason,
        /// What found it.
        evidence: Evidence,
    },
    /// The probe could not conclude (its own positive control failed, no probe exists, a timeout):
    /// never `Yes` and never `No`.
    Unverified {
        /// Why.
        why: Why,
        /// What was tried, if anything.
        evidence: Evidence,
    },
}

/// Runs probes. The engine knows nothing about what they do.
pub trait Prober {
    /// Whether this machine can probe `provider` at all. A provider it cannot reach has its
    /// committed rows carried over (or rebuilt as unverified).
    fn can_probe(&self, provider: &Provider) -> bool;

    /// Probe one capability of a provider that [`Prober::can_probe`] accepted.
    fn probe(&self, provider: &Provider, cap: &CapabilityId, ctx: &Context) -> Outcome;
}

fn stamp(mut e: Evidence, ctx: &Context) -> Evidence {
    if e.environment.is_empty() {
        e.environment = ctx.environment.clone();
    }
    if e.probed_at.is_none() {
        e.probed_at = Some(ctx.now);
    }
    if e.tool_version.is_none()
        && let Some(p) = &e.probe
    {
        e.tool_version = ctx.tool_versions.get(p).cloned();
    }
    e
}

fn row_for(provider: &Provider, cap: &CapabilityId, state: State, evidence: Evidence) -> Row {
    let s = &provider.surfaces()[0];
    Row::capability(cap.clone(), s.backend.clone(), s.version.clone(), state)
        .config(s.config.clone())
        .evidence(evidence)
}

/// The committed rows of `provider` if they can be trusted as they are: exactly one per
/// capability, and every `yes` names the probe that earned it. A carried `yes` is marked
/// inherited from where it was probed (`?` if the row does not say), so it counts only where that
/// is this environment or the need allows inherited rows.
fn carry_over(provider: &Provider, caps: &[CapabilityId], committed: &Matrix) -> Option<Vec<Row>> {
    let s = &provider.surfaces()[0];
    let mine: Vec<&Row> = committed
        .rows()
        .iter()
        .filter(|r| {
            r.subject_kind == "capability"
                && r.backend == s.backend
                && r.version == s.version
                && r.config == s.config
        })
        .collect();
    let complete = mine.len() == caps.len()
        && caps
            .iter()
            .all(|c| mine.iter().filter(|r| &r.subject == c).count() == 1);
    let sound = mine
        .iter()
        .all(|r| r.state != State::Yes || r.evidence.probe.is_some());
    if !(complete && sound) {
        return None;
    }
    // In the declared capability order, so the output does not depend on the committed order.
    Some(
        caps.iter()
            .filter_map(|c| mine.iter().find(|r| &r.subject == c))
            .map(|r| {
                let mut r = (*r).clone();
                if r.state == State::Yes && r.evidence.inherited_from.is_none() {
                    r.evidence.inherited_from = Some(r.evidence.environment.clone());
                }
                r
            })
            .collect(),
    )
}

/// Build the matrix for `table` and `caps`: one `capability` row per provider and capability, on
/// the provider's primary surface.
///
/// - A provider the prober can reach is probed for every capability.
/// - Any other keeps its committed rows if they are whole and sound, else all its rows are
///   `Unverified(NotSetUp)`.
///
/// **Nothing is `Yes` without a probe that ran, passed and named itself.** A passing probe is
/// stamped with the context's environment and time, and the tool version if it gave none.
pub fn generate(
    table: &[Provider],
    caps: &[CapabilityId],
    prober: &dyn Prober,
    committed: &Matrix,
    ctx: &Context,
) -> Matrix {
    let mut rows = Vec::new();
    for p in table {
        if prober.can_probe(p) {
            for c in caps {
                rows.push(match prober.probe(p, c, ctx) {
                    Outcome::Pass(e) if e.probe.is_some() => {
                        row_for(p, c, State::Yes, stamp(e, ctx))
                    }
                    Outcome::Pass(e) => row_for(p, c, State::Unverified(Why::NoEvidence), e),
                    Outcome::Fail { reason, evidence } => {
                        row_for(p, c, State::No(reason), stamp(evidence, ctx))
                    }
                    Outcome::Unverified { why, evidence } => {
                        row_for(p, c, State::Unverified(why), evidence)
                    }
                });
            }
        } else if let Some(kept) = carry_over(p, caps, committed) {
            rows.extend(kept);
        } else {
            for c in caps {
                rows.push(row_for(
                    p,
                    c,
                    State::Unverified(Why::NotSetUp),
                    Evidence::default(),
                ));
            }
        }
    }
    Matrix::from_rows(rows)
}

/// The result of comparing a committed file with a fresh run.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct Report {
    /// The committed file was refused by the reader.
    pub parse_error: Option<ParseError>,
    /// Duplicate rows in the committed file.
    pub warnings: Vec<Warning>,
    /// Lines the fresh run has and the committed file lacks (`probed_at` ignored).
    pub missing: Vec<String>,
    /// Lines the committed file has and the fresh run lacks.
    pub unexpected: Vec<String>,
    /// Same lines, different order.
    pub reordered: bool,
    /// Committed `yes` rows that are stale under the context's policy.
    pub stale: Vec<String>,
    /// Committed rows marked inherited: carried over, so **not verified by this run**. For review
    /// only; it never fails [`Report::is_ok`] (see [`check`]).
    pub carried: Vec<String>,
}

impl Report {
    /// Whether the committed file is exactly what the probes produce and nothing in it is stale.
    /// [`Report::carried`] is informational and does not count.
    pub fn is_ok(&self) -> bool {
        self.parse_error.is_none()
            && self.warnings.is_empty()
            && self.missing.is_empty()
            && self.unexpected.is_empty()
            && !self.reordered
            && self.stale.is_empty()
    }
}

fn lines_without_time(m: &Matrix) -> Vec<String> {
    let rows: Vec<Row> = m
        .rows()
        .iter()
        .cloned()
        .map(|mut r| {
            r.evidence.probed_at = None;
            r
        })
        .collect();
    file::write(&Matrix::from_rows(rows))
        .lines()
        .map(str::to_owned)
        .collect()
}

fn is_stale(row: &Row, ctx: &Context) -> bool {
    let Ok(p) = Provider::new(
        "check",
        vec![Surface::new(
            row.backend.clone(),
            row.version.clone(),
            row.config.clone(),
        )],
    ) else {
        return false;
    };
    let one = Matrix::from_rows(vec![row.clone()]);
    resolve(&one, &p, &row.subject, ctx, true).state == State::Unverified(Why::Stale)
}

/// Compare `committed` (file text) with `generated`. **Only `probed_at` is ignored** (it changes
/// every run); every other cell must match, in order, and a committed `yes` that is stale under
/// `ctx.policy` fails too. `generated` is a [`generate`] result; the command line is the
/// consumer's.
///
/// **Carried-over rows are not verified.** [`generate`] copies them from the committed file, so
/// this comparison cannot tell a genuine one from a hand-edited one, and a machine that did not
/// probe them has no way to. They are listed in [`Report::carried`] so a reviewer sees which rows
/// rest on trust; they do not fail the check.
pub fn check(committed: &str, generated: &Matrix, ctx: &Context) -> Report {
    let parsed = match file::parse(committed) {
        Ok(p) => p,
        Err(e) => {
            return Report {
                parse_error: Some(e),
                ..Report::default()
            };
        }
    };
    let have = lines_without_time(&parsed.matrix);
    let want = lines_without_time(generated);
    let have_set: BTreeSet<&String> = have.iter().collect();
    let want_set: BTreeSet<&String> = want.iter().collect();
    let missing: Vec<String> = want
        .iter()
        .filter(|l| !have_set.contains(l))
        .cloned()
        .collect();
    let unexpected: Vec<String> = have
        .iter()
        .filter(|l| !want_set.contains(l))
        .cloned()
        .collect();
    let reordered = missing.is_empty() && unexpected.is_empty() && have != want;
    let stale = parsed
        .matrix
        .rows()
        .iter()
        .filter(|r| r.state == State::Yes && is_stale(r, ctx))
        .map(|r| format!("{} on {}", r.subject, r.backend))
        .collect();
    let carried = parsed
        .matrix
        .rows()
        .iter()
        .filter(|r| r.evidence.inherited_from.is_some())
        .map(|r| format!("{} on {}", r.subject, r.backend))
        .collect();
    Report {
        parse_error: None,
        warnings: parsed.warnings,
        missing,
        unexpected,
        reordered,
        stale,
        carried,
    }
}
