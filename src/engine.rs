// SPDX-License-Identifier: MIT OR Apache-2.0
//! Resolving capabilities and ranking providers (`docs/DESIGN.md` sections 2 and 3).

use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::fmt;

use crate::{CapabilityId, Config, Evidence, Matrix, Reason, Row, State, Timestamp, Why};

/// One place the matrix is asked about a provider: a `(backend, version, config)` triple. A
/// provider built on another layer has one surface per layer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Surface {
    /// Backend name as in the matrix.
    pub backend: String,
    /// Backend version as in the matrix.
    pub version: String,
    /// The configuration facts the rows are matched against.
    pub config: Config,
}

impl Surface {
    /// A surface for `backend` at `version` with `config`.
    pub fn new(backend: impl Into<String>, version: impl Into<String>, config: Config) -> Self {
        Surface {
            backend: backend.into(),
            version: version.into(),
            config,
        }
    }
}

/// A provider has no surface, so nothing could ever be looked up for it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoSurface(pub String);

impl fmt::Display for NoSurface {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "provider `{}` has no surface", self.0)
    }
}

impl std::error::Error for NoSurface {}

/// Something that could satisfy needs. Plain data: the consumer keeps the executable next to it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Provider {
    id: String,
    surfaces: Vec<Surface>,
}

impl Provider {
    /// A provider called `id` (unique in its table). The first surface is the primary one.
    pub fn new(id: impl Into<String>, surfaces: Vec<Surface>) -> Result<Self, NoSurface> {
        let id = id.into();
        if surfaces.is_empty() {
            return Err(NoSurface(id));
        }
        Ok(Provider { id, surfaces })
    }

    /// The provider's id.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Where the matrix is asked, primary first.
    pub fn surfaces(&self) -> &[Surface] {
        &self.surfaces
    }
}

/// How old a `yes` may be, and whether the probing tool's version must match.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Policy {
    /// A `yes` older than this many seconds is stale. `None`: rows never age by time. With a
    /// window set, a `yes` that records no time cannot show it is fresh and is stale.
    pub max_age_secs: Option<i64>,
    /// A `yes` whose recorded tool version differs from the version in [`Context::tool_versions`]
    /// for its probe is stale. A version that is recorded on only one side cannot be compared and
    /// is not stale.
    pub require_tool_version_match: bool,
}

impl Default for Policy {
    fn default() -> Self {
        Policy {
            max_age_secs: None,
            require_tool_version_match: true,
        }
    }
}

/// Everything about "here and now" that the engine is told rather than reads.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Context {
    /// The current time.
    pub now: Timestamp,
    /// The environment this process is in.
    pub environment: Config,
    /// The current version of each probing tool, by probe id.
    pub tool_versions: BTreeMap<String, String>,
    /// Staleness rules.
    pub policy: Policy,
}

impl Context {
    /// A context at `now` in `environment`, default policy.
    pub fn new(now: Timestamp, environment: Config) -> Self {
        Context {
            now,
            environment,
            ..Context::default()
        }
    }
}

/// What the developer needs.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Need {
    /// Every one of these must be `Yes`.
    pub required: Vec<CapabilityId>,
    /// What matters most, most important first. Orders providers; never excludes one.
    pub preferred: Vec<CapabilityId>,
    /// Provider ids in order of preference, after the capability preferences (AntSpawn's `prefer`).
    pub preferred_providers: Vec<String>,
    /// Accept a `yes` carried over from another environment (it is flagged in the outcome).
    pub allow_inherited: bool,
}

impl Need {
    /// Needs with these required capabilities.
    pub fn new(required: Vec<CapabilityId>) -> Self {
        Need {
            required,
            ..Need::default()
        }
    }

    /// With ordered preferred capabilities.
    #[must_use]
    pub fn prefer(mut self, preferred: Vec<CapabilityId>) -> Self {
        self.preferred = preferred;
        self
    }

    /// With ordered preferred providers.
    #[must_use]
    pub fn prefer_providers(mut self, ids: Vec<String>) -> Self {
        self.preferred_providers = ids;
        self
    }

    /// Accept inherited rows.
    #[must_use]
    pub fn allow_inherited(mut self) -> Self {
        self.allow_inherited = true;
        self
    }
}

/// What the matrix says about one capability of one provider, after degrading.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Resolved {
    /// The combined state.
    pub state: State,
    /// Whether a `Yes` rests only on carried-over rows.
    pub inherited: bool,
    /// The evidence of the row that decided the state, if any row did.
    pub evidence: Option<Evidence>,
}

fn is_stale(ev: &Evidence, ctx: &Context) -> bool {
    if let Some(max) = ctx.policy.max_age_secs {
        match ev.probed_at {
            None => return true,
            Some(t) if ctx.now.saturating_sub(t) > max => return true,
            Some(_) => {}
        }
    }
    if ctx.policy.require_tool_version_match
        && let (Some(recorded), Some(probe)) = (&ev.tool_version, &ev.probe)
        && let Some(current) = ctx.tool_versions.get(probe)
        && current != recorded
    {
        return true;
    }
    false
}

/// One row's state on its own, before rows are combined. Returns the state and whether it is
/// inherited.
fn row_state(row: &Row, ctx: &Context, allow_inherited: bool) -> (State, bool) {
    match &row.state {
        State::No(_) | State::Unverified(_) => (row.state.clone(), false),
        State::Yes => {
            let ev = &row.evidence;
            if ev.probe.is_none() {
                return (State::Unverified(Why::NoEvidence), false);
            }
            let inherited = ev.inherited_from.is_some();
            let here = if let Some(origin) = &ev.inherited_from {
                // Carried over from an environment that is this one is not foreign; an origin
                // that names nothing is unknown, so it needs consent.
                allow_inherited || (!origin.is_empty() && origin.holds_in(&ctx.environment))
            } else {
                ev.environment.holds_in(&ctx.environment)
            };
            if !here {
                return (State::Unverified(Why::OtherEnvironment), false);
            }
            if is_stale(ev, ctx) {
                return (State::Unverified(Why::Stale), false);
            }
            (State::Yes, inherited)
        }
    }
}

/// What the matrix says about `id` for `provider`: rows are matched on each surface (exact
/// backend and version, config facts held in the surface's), degraded one by one, then combined:
/// **`No` beats everything, then `Yes`, then `Unverified`** (the strongest cause).
pub fn resolve(
    matrix: &Matrix,
    provider: &Provider,
    id: &CapabilityId,
    ctx: &Context,
    allow_inherited: bool,
) -> Resolved {
    let mut no: Option<(Reason, &Row)> = None;
    let mut yes: Option<(bool, &Row)> = None;
    let mut unverified: Option<(Why, &Row)> = None;
    for row in matrix.rows() {
        if row.subject_kind != "capability" || &row.subject != id {
            continue;
        }
        let matches = provider.surfaces.iter().any(|s| {
            row.backend == s.backend && row.version == s.version && row.config.holds_in(&s.config)
        });
        if !matches {
            continue;
        }
        match row_state(row, ctx, allow_inherited) {
            (State::No(r), _) => {
                if no.is_none() {
                    no = Some((r, row));
                }
            }
            (State::Yes, inherited) => match yes {
                // A fresh local `yes` is better evidence than an inherited one.
                Some((false, _)) => {}
                Some((true, _)) | None => yes = Some((inherited, row)),
            },
            (State::Unverified(w), _) => {
                if unverified.as_ref().is_none_or(|(old, _)| w.outranks(old)) {
                    unverified = Some((w, row));
                }
            }
        }
    }
    if let Some((r, row)) = no {
        return Resolved {
            state: State::No(r),
            inherited: false,
            evidence: Some(row.evidence.clone()),
        };
    }
    if let Some((inherited, row)) = yes {
        return Resolved {
            state: State::Yes,
            inherited,
            evidence: Some(row.evidence.clone()),
        };
    }
    match unverified {
        Some((w, row)) => Resolved {
            state: State::Unverified(w),
            inherited: false,
            evidence: Some(row.evidence.clone()),
        },
        None => Resolved {
            state: State::Unverified(Why::NoRow),
            inherited: false,
            evidence: None,
        },
    }
}

/// Whether an outcome is for a required or a preferred capability.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Role {
    /// Must be `Yes`.
    Required,
    /// Preferred, at this position (0 = most important).
    Preferred(usize),
}

/// One capability of one provider, with why.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct NeedOutcome {
    /// The capability.
    pub id: CapabilityId,
    /// Required or preferred.
    pub role: Role,
    /// What is known.
    pub state: State,
    /// Whether a `Yes` rests on carried-over rows.
    pub inherited: bool,
    /// The deciding row's evidence.
    pub evidence: Option<Evidence>,
}

impl NeedOutcome {
    /// One line: `kv.ttl: yes (probe kv::kv.ttl, at 1700000000)`.
    pub fn explain(&self) -> String {
        let mut s = format!("{}: {}", self.id, self.state);
        if self.inherited {
            s.push_str(" [inherited]");
        }
        if let Some(e) = &self.evidence {
            let e = e.to_string();
            if !e.is_empty() {
                s.push_str(&format!(" ({e})"));
            }
        }
        s
    }
}

/// One provider's result.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Entry {
    /// The provider's id.
    pub provider: String,
    /// Whether every required capability is `Yes`.
    pub satisfies: bool,
    /// Required outcomes first (need order), then preferred (importance order).
    pub outcomes: Vec<NeedOutcome>,
}

impl Entry {
    /// The required capabilities that are not `Yes`.
    pub fn unmet(&self) -> impl Iterator<Item = &NeedOutcome> {
        self.outcomes
            .iter()
            .filter(|o| o.role == Role::Required && o.state != State::Yes)
    }

    /// One line per outcome.
    pub fn explain(&self) -> Vec<String> {
        self.outcomes.iter().map(NeedOutcome::explain).collect()
    }
}

/// What a provider lacks. Never contains a `Yes`.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct ProviderUnmet {
    /// The provider's id.
    pub provider: String,
    /// Each required capability that is not `Yes`, with the state.
    pub missing: Vec<NeedOutcome>,
}

/// No provider satisfies the whole need; lists every provider and what it lacks.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Unmet {
    /// One entry per provider considered, in ranked order.
    pub providers: Vec<ProviderUnmet>,
}

impl fmt::Display for Unmet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.providers.is_empty() {
            return f.write_str("no provider is registered");
        }
        f.write_str("no provider satisfies the need:")?;
        for p in &self.providers {
            write!(f, " [{}:", p.provider)?;
            for o in &p.missing {
                match &o.state {
                    State::No(r) => write!(f, " {} off({r})", o.id)?,
                    State::Unverified(w) => write!(f, " {} unverified({w})", o.id)?,
                    State::Yes => {}
                }
            }
            f.write_str("]")?;
        }
        Ok(())
    }
}

impl std::error::Error for Unmet {}

/// Why a provider could not be opened from an [`Evaluation`].
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum CheckError {
    /// No provider has this id.
    Unknown(String),
    /// The provider does not satisfy the need.
    Rejected(ProviderUnmet),
}

impl fmt::Display for CheckError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CheckError::Unknown(id) => write!(f, "no provider named `{id}` is registered"),
            CheckError::Rejected(u) => {
                write!(f, "provider `{}` is not certified for:", u.provider)?;
                for o in &u.missing {
                    write!(f, " {} [{}]", o.id, o.state)?;
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for CheckError {}

/// The table has two providers with this id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DuplicateProvider(pub String);

impl fmt::Display for DuplicateProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "two providers are named `{}`", self.0)
    }
}

impl std::error::Error for DuplicateProvider {}

/// A choice shown to a person: an id and why it fits.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Choice {
    /// The provider's id.
    pub id: String,
    /// The explanation, outcomes joined by `; `.
    pub reason: String,
}

/// Every provider, ranked, each with its outcomes. There is no closest match: only
/// [`Evaluation::satisfying`] may be opened from.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Evaluation {
    entries: Vec<Entry>,
}

impl Evaluation {
    /// All entries, ranked: satisfying providers first.
    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }

    /// The providers that satisfy the whole need, best first.
    pub fn satisfying(&self) -> impl Iterator<Item = &Entry> {
        self.entries.iter().filter(|e| e.satisfies)
    }

    /// Re-check one provider (what `open_named` does): `Ok` only if it satisfies.
    pub fn check(&self, id: &str) -> Result<&Entry, CheckError> {
        let e = self
            .entries
            .iter()
            .find(|e| e.provider == id)
            .ok_or_else(|| CheckError::Unknown(id.to_owned()))?;
        if e.satisfies {
            Ok(e)
        } else {
            Err(CheckError::Rejected(unmet_of(e)))
        }
    }

    /// One line per need for one provider.
    pub fn explain(&self, id: &str) -> Option<Vec<String>> {
        self.entries
            .iter()
            .find(|e| e.provider == id)
            .map(Entry::explain)
    }

    /// Re-order the satisfying providers (stable) with a consumer's own tie-break, such as a
    /// measured speed. Providers that do not satisfy keep their place after them.
    pub fn reorder_satisfiers(&mut self, mut cmp: impl FnMut(&Entry, &Entry) -> Ordering) {
        let n = self.entries.iter().take_while(|e| e.satisfies).count();
        self.entries[..n].sort_by(|a, b| cmp(a, b));
    }

    /// The satisfying providers, or `Unmet` naming what every provider lacks (PersistAnt's shape).
    pub fn satisfying_or_unmet(&self) -> Result<Vec<&Entry>, Unmet> {
        let ok: Vec<&Entry> = self.satisfying().collect();
        if !ok.is_empty() {
            return Ok(ok);
        }
        Err(Unmet {
            providers: self.entries.iter().map(unmet_of).collect(),
        })
    }

    /// The satisfying providers as choices, best first, each with its reasons (ConfigurAnt's shape).
    pub fn choices(&self) -> Vec<Choice> {
        self.satisfying()
            .map(|e| Choice {
                id: e.provider.clone(),
                reason: e.explain().join("; "),
            })
            .collect()
    }
}

fn unmet_of(e: &Entry) -> ProviderUnmet {
    ProviderUnmet {
        provider: e.provider.clone(),
        missing: e.unmet().cloned().collect(),
    }
}

/// Rank `table` for `need`. Sort key, stable: satisfying first; then the preferred capabilities as
/// a vector in importance order, `Yes` before not-`Yes`, compared lexicographically; then the
/// position in `need.preferred_providers`; then table order.
pub fn evaluate(
    table: &[Provider],
    need: &Need,
    matrix: &Matrix,
    ctx: &Context,
) -> Result<Evaluation, DuplicateProvider> {
    for (i, p) in table.iter().enumerate() {
        if table[..i].iter().any(|q| q.id == p.id) {
            return Err(DuplicateProvider(p.id.clone()));
        }
    }
    let mut ranked: Vec<(Vec<bool>, usize, usize, Entry)> = table
        .iter()
        .enumerate()
        .map(|(pos, p)| {
            let outcome = |id: &CapabilityId, role| {
                let r = resolve(matrix, p, id, ctx, need.allow_inherited);
                NeedOutcome {
                    id: id.clone(),
                    role,
                    state: r.state,
                    inherited: r.inherited,
                    evidence: r.evidence,
                }
            };
            let mut outcomes: Vec<NeedOutcome> = need
                .required
                .iter()
                .map(|id| outcome(id, Role::Required))
                .collect();
            let satisfies = outcomes.iter().all(|o| o.state == State::Yes);
            let prefs: Vec<NeedOutcome> = need
                .preferred
                .iter()
                .enumerate()
                .map(|(i, id)| outcome(id, Role::Preferred(i)))
                .collect();
            let pref_vec: Vec<bool> = prefs.iter().map(|o| o.state != State::Yes).collect();
            outcomes.extend(prefs);
            let provider_rank = need
                .preferred_providers
                .iter()
                .position(|q| *q == p.id)
                .unwrap_or(usize::MAX);
            (
                pref_vec,
                provider_rank,
                pos,
                Entry {
                    provider: p.id.clone(),
                    satisfies,
                    outcomes,
                },
            )
        })
        .collect();
    ranked.sort_by(|a, b| (!a.3.satisfies, &a.0, a.1, a.2).cmp(&(!b.3.satisfies, &b.0, b.1, b.2)));
    Ok(Evaluation {
        entries: ranked.into_iter().map(|r| r.3).collect(),
    })
}
