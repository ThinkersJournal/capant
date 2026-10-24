// SPDX-License-Identifier: MIT OR Apache-2.0
//! Capability states and the evidence behind them.

use std::fmt;

use crate::Config;

/// Why a capability is `No`: the provider was tested and the claim failed.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Reason {
    /// The provider does not give it.
    Missing,
    /// The provider gives something else: the result broke the contract.
    Wrong,
    /// Any other recorded reason (free text, as AntSpawn writes it).
    Other(String),
}

impl fmt::Display for Reason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Reason::Missing => f.write_str("missing"),
            Reason::Wrong => f.write_str("wrong"),
            Reason::Other(t) => f.write_str(t),
        }
    }
}

/// Why a capability is `Unverified`: nobody has shown it either way, here and now.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Why {
    /// No row matches this provider and capability.
    NoRow,
    /// A `yes` row names no probe.
    NoEvidence,
    /// A `yes` row is older than the policy window, or from another tool version.
    Stale,
    /// A `yes` row was probed in another environment and is not accepted as inherited.
    OtherEnvironment,
    /// The probe's own positive control failed, so its answer proves nothing.
    NoControl,
    /// The provider could not be set up to be probed.
    NotSetUp,
    /// No probe exists for this capability yet.
    Untested,
    /// Any other recorded cause (free text).
    Other(String),
}

impl Why {
    /// How actionable the cause is; the strongest one is reported when several rows disagree.
    fn rank(&self) -> u8 {
        match self {
            Why::Stale => 7,
            Why::OtherEnvironment => 6,
            Why::NoEvidence => 5,
            Why::NoControl => 4,
            Why::NotSetUp => 3,
            Why::Untested => 2,
            Why::Other(_) => 1,
            Why::NoRow => 0,
        }
    }

    pub(crate) fn outranks(&self, other: &Why) -> bool {
        self.rank() > other.rank()
    }
}

impl fmt::Display for Why {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Why::NoRow => f.write_str("no row"),
            Why::NoEvidence => f.write_str("no evidence"),
            Why::Stale => f.write_str("stale"),
            Why::OtherEnvironment => f.write_str("other environment"),
            Why::NoControl => f.write_str("no control"),
            Why::NotSetUp => f.write_str("not set up"),
            Why::Untested => f.write_str("untested"),
            Why::Other(t) => f.write_str(t),
        }
    }
}

/// What is known about one capability of one provider. Only `Yes` satisfies a need.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum State {
    /// Shown to hold, with evidence.
    Yes,
    /// Tested, and it does not hold.
    No(Reason),
    /// Not shown either way (the default).
    Unverified(Why),
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            State::Yes => f.write_str("yes"),
            State::No(r) => write!(f, "no ({r})"),
            State::Unverified(w) => write!(f, "unverified ({w})"),
        }
    }
}

/// Seconds since the Unix epoch, UTC. The engine has no clock: callers pass "now".
pub type Timestamp = i64;

/// The record that earns a `yes`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Evidence {
    /// The probe that produced the state (`read`, `kv::kv.get`). A `yes` without one is not `Yes`.
    pub probe: Option<String>,
    /// The version of the tool or driver that ran the probe.
    pub tool_version: Option<String>,
    /// Where it was probed (`os`, `arch`). A row that names facts holds only where they hold;
    /// a row that names none makes no claim about where, and is accepted anywhere.
    pub environment: Config,
    /// When it was probed.
    pub probed_at: Option<Timestamp>,
    /// Set on a row carried over from the environment it names: the probe did not run here.
    pub inherited_from: Option<Config>,
}

impl Evidence {
    /// Evidence naming just the probe.
    pub fn probe(id: impl Into<String>) -> Self {
        Evidence {
            probe: Some(id.into()),
            ..Evidence::default()
        }
    }

    /// With the probing tool's version.
    #[must_use]
    pub fn tool_version(mut self, v: impl Into<String>) -> Self {
        self.tool_version = Some(v.into());
        self
    }

    /// With the environment it ran in.
    #[must_use]
    pub fn environment(mut self, e: Config) -> Self {
        self.environment = e;
        self
    }

    /// With the time it ran.
    #[must_use]
    pub fn at(mut self, t: Timestamp) -> Self {
        self.probed_at = Some(t);
        self
    }

    /// Marked as carried over from `origin`.
    #[must_use]
    pub fn inherited_from(mut self, origin: Config) -> Self {
        self.inherited_from = Some(origin);
        self
    }
}

impl fmt::Display for Evidence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut parts = Vec::new();
        if let Some(p) = &self.probe {
            parts.push(format!("probe {p}"));
        }
        if let Some(v) = &self.tool_version {
            parts.push(format!("tool {v}"));
        }
        if let Some(t) = self.probed_at {
            parts.push(format!("at {t}"));
        }
        if !self.environment.is_empty() {
            parts.push(format!("env {}", self.environment));
        }
        if let Some(o) = &self.inherited_from {
            parts.push(format!("inherited from {o}"));
        }
        f.write_str(&parts.join(", "))
    }
}
