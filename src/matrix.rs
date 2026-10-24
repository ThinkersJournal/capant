// SPDX-License-Identifier: MIT OR Apache-2.0
//! The recorded facts: rows, in memory. (Reading and writing the file is R3.)

use std::collections::BTreeMap;

use crate::{CapabilityId, Config, Evidence, State};

/// One recorded fact: for this subject, on this backend, version and config, this state, earned
/// by this evidence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    /// `capability` for rows that can grant anything; other kinds (`measurement`) are kept and
    /// never consulted.
    pub subject_kind: String,
    /// The capability id.
    pub subject: CapabilityId,
    /// The provider's backend name.
    pub backend: String,
    /// The backend version.
    pub version: String,
    /// Configuration facts the row holds under (empty = any).
    pub config: Config,
    /// What the row says.
    pub state: State,
    /// What earned it.
    pub evidence: Evidence,
    /// Free text.
    pub note: String,
    /// `metadata` facts that are not evidence (anything but the probe id), kept as read.
    pub metadata: Config,
    /// Columns the engine does not know, kept as read and written back sorted.
    pub extra: BTreeMap<String, String>,
}

impl Row {
    /// A `capability` row.
    pub fn capability(
        subject: CapabilityId,
        backend: impl Into<String>,
        version: impl Into<String>,
        state: State,
    ) -> Self {
        Row {
            subject_kind: "capability".to_owned(),
            subject,
            backend: backend.into(),
            version: version.into(),
            config: Config::new(),
            state,
            evidence: Evidence::default(),
            note: String::new(),
            metadata: Config::new(),
            extra: BTreeMap::new(),
        }
    }

    /// With the configuration the row holds under.
    #[must_use]
    pub fn config(mut self, c: Config) -> Self {
        self.config = c;
        self
    }

    /// With evidence.
    #[must_use]
    pub fn evidence(mut self, e: Evidence) -> Self {
        self.evidence = e;
        self
    }
}

/// A set of rows. Nothing is deduplicated: conflicting rows are resolved when read, not when added.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Matrix {
    rows: Vec<Row>,
}

impl Matrix {
    /// No rows: every capability is unverified.
    pub fn new() -> Self {
        Matrix::default()
    }

    /// A matrix of exactly these rows.
    pub fn from_rows(rows: Vec<Row>) -> Self {
        Matrix { rows }
    }

    /// Add a row.
    pub fn push(&mut self, row: Row) {
        self.rows.push(row);
    }

    /// Add the rows of another matrix.
    pub fn merge(&mut self, other: Matrix) {
        self.rows.extend(other.rows);
    }

    /// The rows, in insertion order.
    pub fn rows(&self) -> &[Row] {
        &self.rows
    }
}
