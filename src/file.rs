// SPDX-License-Identifier: MIT OR Apache-2.0
//! The matrix file: tab-separated, header read by column name (`docs/DESIGN.md` section 5).
//!
//! Required columns: `subject_kind subject backend version config enabled reason metadata note`
//! (PersistAnt's and AntSpawn's nine). Optional: `probed_at tool_version environment
//! inherited_from`. Unknown columns are kept in [`Row::extra`]. Blank lines and lines starting
//! with `#` are skipped on read and dropped on write.
//!
//! How the old nine columns map onto [`State`]:
//! - `enabled=yes`, empty `reason`: [`State::Yes`] (it still needs a probe id to count).
//! - `enabled=no`, `reason` is `missing` / `wrong`: [`State::No`]; any other text is
//!   `No(Other(text))`.
//! - `reason` starting `UNVERIFIED: ` (AntSpawn's spelling), or exactly `no probe for this
//!   capability yet`: [`State::Unverified`].
//!
//! `inherited_from` is `?` when the origin is unknown.
//! The probe id is `metadata`'s `probe` key, else its `test` key; it is written back as `probe`.
//! The other metadata keys are kept. `os`/`arch` in old metadata stay informational: only the
//! `environment` column makes a claim about where a row holds.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use crate::{
    CapabilityId, Config, ConfigError, Evidence, IdError, Matrix, Reason, Row, State, Why, time,
};

const REQUIRED: [&str; 9] = [
    "subject_kind",
    "subject",
    "backend",
    "version",
    "config",
    "enabled",
    "reason",
    "metadata",
    "note",
];
const OPTIONAL: [&str; 4] = ["probed_at", "tool_version", "environment", "inherited_from"];
const UNVERIFIED_PREFIX: &str = "UNVERIFIED: ";
const UNTESTED_TEXT: &str = "no probe for this capability yet";

/// Why a matrix file was refused. A malformed row is an error, never a skipped row: a skipped
/// row silently turns a capability off, or hides that it was asserted.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ParseError {
    /// No header line.
    NoHeader,
    /// A required column is missing.
    MissingColumn(&'static str),
    /// A column named twice.
    DuplicateColumn(String),
    /// A row with a different number of fields than the header.
    FieldCount {
        /// 1-based line.
        line: usize,
        /// Header width.
        want: usize,
        /// Row width.
        got: usize,
    },
    /// `subject` is not a capability id.
    BadId {
        /// 1-based line.
        line: usize,
        /// Why.
        error: IdError,
    },
    /// `enabled` is neither `yes` nor `no`.
    BadEnabled {
        /// 1-based line.
        line: usize,
        /// What was found.
        value: String,
    },
    /// `yes` with a reason, or `no` without one.
    ReasonMismatch {
        /// 1-based line.
        line: usize,
    },
    /// A `k=v;k=v` column that is not.
    BadConfig {
        /// 1-based line.
        line: usize,
        /// The column.
        column: &'static str,
        /// Why.
        error: ConfigError,
    },
    /// `probed_at` is not `YYYY-MM-DDTHH:MM:SSZ`.
    BadTimestamp {
        /// 1-based line.
        line: usize,
        /// What was found.
        value: String,
    },
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::NoHeader => f.write_str("matrix has no header line"),
            ParseError::MissingColumn(c) => write!(f, "matrix header lacks column {c:?}"),
            ParseError::DuplicateColumn(c) => write!(f, "matrix header names column {c:?} twice"),
            ParseError::FieldCount { line, want, got } => {
                write!(f, "line {line}: expected {want} fields, found {got}")
            }
            ParseError::BadId { line, error } => write!(f, "line {line}: subject: {error}"),
            ParseError::BadEnabled { line, value } => {
                write!(
                    f,
                    "line {line}: `enabled` must be yes or no, found {value:?}"
                )
            }
            ParseError::ReasonMismatch { line } => write!(
                f,
                "line {line}: a `no` row needs a reason and a `yes` row must not have one"
            ),
            ParseError::BadConfig {
                line,
                column,
                error,
            } => write!(f, "line {line}: {column}: {error}"),
            ParseError::BadTimestamp { line, value } => write!(
                f,
                "line {line}: probed_at {value:?} is not YYYY-MM-DDTHH:MM:SSZ"
            ),
        }
    }
}

impl std::error::Error for ParseError {}

/// Something odd that is not an error.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Warning {
    /// Two rows for the same `(kind, subject, backend, version, config)`. They are resolved when
    /// read (a `no` wins), but a generated file has none, so `--check` fails on them.
    Duplicate {
        /// 1-based line of the second row.
        line: usize,
        /// The key, rendered.
        key: String,
    },
}

/// A parsed file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Parsed {
    /// The rows.
    pub matrix: Matrix,
    /// Anything odd that was kept.
    pub warnings: Vec<Warning>,
}

fn why_from_text(t: &str) -> Why {
    match t {
        "no row" => Why::NoRow,
        "no evidence" => Why::NoEvidence,
        "stale" => Why::Stale,
        "other environment" => Why::OtherEnvironment,
        "no control" => Why::NoControl,
        "not set up" => Why::NotSetUp,
        "untested" => Why::Untested,
        other => Why::Other(other.to_owned()),
    }
}

fn state_from(enabled: bool, reason: &str, line: usize) -> Result<State, ParseError> {
    if enabled {
        return if reason.is_empty() {
            Ok(State::Yes)
        } else {
            Err(ParseError::ReasonMismatch { line })
        };
    }
    if reason.is_empty() {
        return Err(ParseError::ReasonMismatch { line });
    }
    Ok(if reason == UNTESTED_TEXT {
        State::Unverified(Why::Untested)
    } else if let Some(rest) = reason.strip_prefix(UNVERIFIED_PREFIX) {
        State::Unverified(why_from_text(rest))
    } else {
        State::No(match reason {
            "missing" => Reason::Missing,
            "wrong" => Reason::Wrong,
            other => Reason::Other(other.to_owned()),
        })
    })
}

fn state_to(state: &State) -> (&'static str, String) {
    match state {
        State::Yes => ("yes", String::new()),
        State::No(r) => ("no", r.to_string()),
        State::Unverified(Why::Untested) => ("no", UNTESTED_TEXT.to_owned()),
        State::Unverified(w) => ("no", format!("{UNVERIFIED_PREFIX}{w}")),
    }
}

/// Read a matrix file.
pub fn parse(text: &str) -> Result<Parsed, ParseError> {
    let mut header: Option<Vec<&str>> = None;
    let mut rows = Vec::new();
    let mut warnings = Vec::new();
    let mut seen: BTreeSet<(String, String, String, String, String)> = BTreeSet::new();
    for (i, raw) in text.lines().enumerate() {
        let line = i + 1;
        let l = raw.trim_end_matches('\r');
        if l.is_empty() || l.starts_with('#') {
            continue;
        }
        let fields: Vec<&str> = l.split('\t').collect();
        let Some(h) = &header else {
            for c in REQUIRED {
                if !fields.contains(&c) {
                    return Err(ParseError::MissingColumn(c));
                }
            }
            for (j, c) in fields.iter().enumerate() {
                if fields[..j].contains(c) {
                    return Err(ParseError::DuplicateColumn((*c).to_owned()));
                }
            }
            header = Some(fields);
            continue;
        };
        if fields.len() != h.len() {
            return Err(ParseError::FieldCount {
                line,
                want: h.len(),
                got: fields.len(),
            });
        }
        let row = parse_row(h, &fields, line)?;
        let key = (
            row.subject_kind.clone(),
            row.subject.to_string(),
            row.backend.clone(),
            row.version.clone(),
            row.config.to_string(),
        );
        if !seen.insert(key) {
            warnings.push(Warning::Duplicate {
                line,
                key: format!(
                    "{} `{}` on `{}` {} [{}]",
                    row.subject_kind, row.subject, row.backend, row.version, row.config
                ),
            });
        }
        rows.push(row);
    }
    if header.is_none() {
        return Err(ParseError::NoHeader);
    }
    Ok(Parsed {
        matrix: Matrix::from_rows(rows),
        warnings,
    })
}

fn parse_row(h: &[&str], f: &[&str], line: usize) -> Result<Row, ParseError> {
    let get = |name: &str| -> &str { h.iter().position(|c| *c == name).map_or("", |i| f[i]) };
    let cfg = |name: &'static str| {
        Config::parse(get(name)).map_err(|error| ParseError::BadConfig {
            line,
            column: name,
            error,
        })
    };
    let subject =
        CapabilityId::new(get("subject")).map_err(|error| ParseError::BadId { line, error })?;
    let enabled = match get("enabled") {
        "yes" => true,
        "no" => false,
        other => {
            return Err(ParseError::BadEnabled {
                line,
                value: other.to_owned(),
            });
        }
    };
    let state = state_from(enabled, get("reason"), line)?;

    let mut metadata = cfg("metadata")?;
    let mut probe = None;
    for key in ["probe", "test"] {
        if let Some(v) = metadata.get(key).map(str::to_owned) {
            metadata = metadata.without(key);
            if probe.is_none() {
                probe = Some(v);
            } else {
                // Both keys named: `probe` is the id, keep `test` as an ordinary fact.
                metadata = metadata.with(key, v);
            }
        }
    }
    let probed_at = match get("probed_at") {
        "" => None,
        v => Some(time::parse(v).ok_or_else(|| ParseError::BadTimestamp {
            line,
            value: v.to_owned(),
        })?),
    };
    let evidence = Evidence {
        probe,
        tool_version: Some(get("tool_version"))
            .filter(|v| !v.is_empty())
            .map(str::to_owned),
        environment: cfg("environment")?,
        probed_at,
        inherited_from: match get("inherited_from") {
            "" => None,
            "?" => Some(Config::new()),
            _ => Some(cfg("inherited_from")?),
        },
    };
    let extra: BTreeMap<String, String> = h
        .iter()
        .zip(f)
        .filter(|(c, _)| !REQUIRED.contains(c) && !OPTIONAL.contains(c))
        .map(|(c, v)| ((*c).to_owned(), (*v).to_owned()))
        .collect();
    Ok(Row {
        subject_kind: get("subject_kind").to_owned(),
        subject,
        backend: get("backend").to_owned(),
        version: get("version").to_owned(),
        config: cfg("config")?,
        state,
        evidence,
        note: get("note").to_owned(),
        metadata,
        extra,
    })
}

/// A cell that cannot break the row: tabs and line breaks become spaces.
fn cell(s: &str) -> String {
    s.replace(['\t', '\n', '\r'], " ")
}

fn metadata_of(r: &Row) -> Config {
    match &r.evidence.probe {
        Some(p) => r.metadata.clone().with("probe", p.clone()),
        None => r.metadata.clone(),
    }
}

/// The canonical text of `matrix`: LF line ends, the nine columns, then any optional column some
/// row uses (in the order `probed_at tool_version environment inherited_from`), then unknown
/// columns sorted. Deterministic: `parse(write(m)) == m` for rows whose text has no tabs or line
/// breaks, and a reason that does not itself spell another state (see the module docs).
pub fn write(matrix: &Matrix) -> String {
    let rows = matrix.rows();
    let use_col = |name: &str| match name {
        "probed_at" => rows.iter().any(|r| r.evidence.probed_at.is_some()),
        "tool_version" => rows.iter().any(|r| r.evidence.tool_version.is_some()),
        "environment" => rows.iter().any(|r| !r.evidence.environment.is_empty()),
        _ => rows.iter().any(|r| r.evidence.inherited_from.is_some()),
    };
    let extra: BTreeSet<&str> = rows
        .iter()
        .flat_map(|r| r.extra.keys().map(String::as_str))
        .collect();
    let columns: Vec<&str> = REQUIRED
        .iter()
        .copied()
        .chain(OPTIONAL.iter().copied().filter(|c| use_col(c)))
        .chain(extra)
        .collect();
    let mut out = columns.join("\t");
    out.push('\n');
    for r in rows {
        let (enabled, reason) = state_to(&r.state);
        let cells: Vec<String> = columns
            .iter()
            .map(|c| match *c {
                "subject_kind" => cell(&r.subject_kind),
                "subject" => r.subject.to_string(),
                "backend" => cell(&r.backend),
                "version" => cell(&r.version),
                "config" => r.config.to_string(),
                "enabled" => enabled.to_owned(),
                "reason" => cell(&reason),
                "metadata" => metadata_of(r).to_string(),
                "note" => cell(&r.note),
                "probed_at" => r.evidence.probed_at.map(time::format).unwrap_or_default(),
                "tool_version" => cell(r.evidence.tool_version.as_deref().unwrap_or("")),
                "environment" => r.evidence.environment.to_string(),
                "inherited_from" => r
                    .evidence
                    .inherited_from
                    .as_ref()
                    .map(|o| {
                        if o.is_empty() {
                            "?".to_owned()
                        } else {
                            o.to_string()
                        }
                    })
                    .unwrap_or_default(),
                other => cell(r.extra.get(other).map_or("", String::as_str)),
            })
            .collect();
        out.push_str(&cells.join("\t"));
        out.push('\n');
    }
    out
}
