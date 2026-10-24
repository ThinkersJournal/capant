// SPDX-License-Identifier: MIT OR Apache-2.0
//! Capability ids.

use std::fmt;

/// Why a string is not a capability id.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum IdError {
    /// The string is empty.
    Empty,
    /// A character outside `a-z 0-9 _ . : -` at this byte offset.
    BadChar {
        /// Byte offset of the first offending character.
        at: usize,
    },
    /// A leading, trailing or doubled `.`.
    BadDot,
}

impl fmt::Display for IdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IdError::Empty => f.write_str("capability id is empty"),
            IdError::BadChar { at } => write!(
                f,
                "capability id has a character outside a-z 0-9 _ . : - at byte {at}"
            ),
            IdError::BadDot => f.write_str("capability id has a leading, trailing or doubled '.'"),
        }
    }
}

impl std::error::Error for IdError {}

/// A capability id such as `kv.prefix_list`, `fn:max` or `isolation.no-read-outside`.
///
/// Lower-case ASCII, digits, `_`, `-`, `:` and `.` as the namespace separator. This is PersistAnt's
/// grammar plus `-`, which AntSpawn's ids use (`clean-slate-next-run`), so both consumers' ids are
/// valid. The set of ids is open: the engine never lists them.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CapabilityId(String);

impl CapabilityId {
    /// Validate and wrap an id.
    pub fn new(s: impl Into<String>) -> Result<Self, IdError> {
        let s = s.into();
        if s.is_empty() {
            return Err(IdError::Empty);
        }
        let ok = |b: u8| {
            b.is_ascii_lowercase() || b.is_ascii_digit() || matches!(b, b'_' | b'.' | b':' | b'-')
        };
        if let Some(at) = s.bytes().position(|b| !ok(b)) {
            return Err(IdError::BadChar { at });
        }
        if s.starts_with('.') || s.ends_with('.') || s.contains("..") {
            return Err(IdError::BadDot);
        }
        Ok(CapabilityId(s))
    }

    /// The id as a string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for CapabilityId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::str::FromStr for CapabilityId {
    type Err = IdError;
    fn from_str(s: &str) -> Result<Self, IdError> {
        CapabilityId::new(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_both_consumers_ids() {
        for ok in [
            "kv.get",
            "fn:max",
            "cast.to_uuid",
            "isolation.no-read-outside",
            "clean-slate-next-run",
            "gpu.cuda",
        ] {
            assert_eq!(CapabilityId::new(ok).unwrap().as_str(), ok);
        }
    }

    #[test]
    fn rejects_malformed_ids() {
        assert_eq!(CapabilityId::new(""), Err(IdError::Empty));
        assert_eq!(
            CapabilityId::new("Kv.get"),
            Err(IdError::BadChar { at: 0 }),
            "control: upper case is not an id"
        );
        assert_eq!(CapabilityId::new("kv get"), Err(IdError::BadChar { at: 2 }));
        for bad in [".kv", "kv.", "kv..get"] {
            assert_eq!(CapabilityId::new(bad), Err(IdError::BadDot), "{bad}");
        }
    }
}
