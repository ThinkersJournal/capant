// SPDX-License-Identifier: MIT OR Apache-2.0
//! CapAnt: a capability engine. Developers declare NEEDS in order of importance, providers have
//! recorded capabilities with evidence, and CapAnt returns every provider ranked, each with why.
//!
//! A capability is [`State::Yes`] only with recorded evidence; no row, a `yes` without a probe, a
//! stale row, or a row from another environment is [`State::Unverified`], never `Yes`. There is no
//! closest match: only providers whose required capabilities are all `Yes` may be opened from an
//! [`Evaluation`]. See `docs/DESIGN.md`.
//!
//! ```
//! use capant::*;
//! let id = |s: &str| CapabilityId::new(s).unwrap();
//! let provider = |name: &str| {
//!     Provider::new(name, vec![Surface::new(name, "1", Config::new())]).unwrap()
//! };
//! let mut matrix = Matrix::new();
//! matrix.push(
//!     Row::capability(id("kv.get"), "mem", "1", State::Yes).evidence(Evidence::probe("kv::get")),
//! );
//! let table = [provider("mem"), provider("fs")];
//! let ctx = Context::new(0, Config::new());
//! let eval = evaluate(&table, &Need::new(vec![id("kv.get")]), &matrix, &ctx).unwrap();
//! assert_eq!(eval.satisfying().map(|e| e.provider.as_str()).collect::<Vec<_>>(), ["mem"]);
//! assert!(eval.check("fs").is_err()); // `fs` has no row: unverified, not "probably yes"
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod config;
mod engine;
mod file;
mod generate;
mod id;
mod matrix;
mod state;
pub mod time;

pub use config::{Config, ConfigError};
pub use engine::{
    CheckError, Choice, Context, DuplicateProvider, Entry, Evaluation, Need, NeedOutcome,
    NoSurface, Policy, Provider, ProviderUnmet, Resolved, Role, Surface, Unmet, evaluate, resolve,
};
pub use file::{ParseError, Parsed, Warning, parse, write};
pub use generate::{Outcome, Prober, Report, check, generate};
pub use id::{CapabilityId, IdError};
pub use matrix::{Matrix, Row};
pub use state::{Evidence, Reason, State, Timestamp, Why};
