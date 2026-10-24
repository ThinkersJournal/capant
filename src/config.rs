// SPDX-License-Identifier: MIT OR Apache-2.0
//! `k=v;k=v` fact maps (a surface's configuration, a row's environment).

use std::collections::BTreeMap;
use std::fmt;

/// Why a string is not a [`Config`].
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ConfigError {
    /// A piece that is not `key=value` with a non-empty key.
    BadPair(String),
    /// A key given twice.
    DuplicateKey(String),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::BadPair(p) => write!(f, "config piece {p:?} is not key=value"),
            ConfigError::DuplicateKey(k) => write!(f, "config key {k:?} appears twice"),
        }
    }
}

impl std::error::Error for ConfigError {}

/// An ordered map of facts. Displays sorted by key, so files diff stably.
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Config(BTreeMap<String, String>);

impl Config {
    /// No facts.
    pub fn new() -> Self {
        Config::default()
    }

    /// Parse `k=v;k=v` (empty string is no facts).
    pub fn parse(s: &str) -> Result<Self, ConfigError> {
        let mut map = BTreeMap::new();
        if s.is_empty() {
            return Ok(Config(map));
        }
        for piece in s.split(';') {
            match piece.split_once('=') {
                Some((k, v)) if !k.is_empty() => {
                    if map.insert(k.to_owned(), v.to_owned()).is_some() {
                        return Err(ConfigError::DuplicateKey(k.to_owned()));
                    }
                }
                _ => return Err(ConfigError::BadPair(piece.to_owned())),
            }
        }
        Ok(Config(map))
    }

    /// This config plus one fact (replacing an existing value for the key).
    #[must_use]
    pub fn with(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.0.insert(key.into(), value.into());
        self
    }

    /// Whether every fact named here is also in `actual` with the same value. The empty config
    /// holds in everything.
    pub fn holds_in(&self, actual: &Config) -> bool {
        self.0.iter().all(|(k, v)| actual.0.get(k) == Some(v))
    }

    /// The value of a key.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).map(String::as_str)
    }

    /// Whether there are no facts.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, (k, v)) in self.0.iter().enumerate() {
            if i > 0 {
                f.write_str(";")?;
            }
            write!(f, "{k}={v}")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_and_sorts() {
        let c = Config::parse("schema=none;collation=bin").unwrap();
        assert_eq!(c.to_string(), "collation=bin;schema=none");
        assert_eq!(Config::parse(&c.to_string()).unwrap(), c);
        assert_eq!(Config::parse("").unwrap(), Config::new());
    }

    #[test]
    fn rejects_malformed_pieces() {
        assert_eq!(Config::parse("a"), Err(ConfigError::BadPair("a".into())));
        assert_eq!(Config::parse("=v"), Err(ConfigError::BadPair("=v".into())));
        assert_eq!(
            Config::parse("a=1;a=2"),
            Err(ConfigError::DuplicateKey("a".into()))
        );
    }

    #[test]
    fn holds_in_is_a_subset_test_with_negative_controls() {
        let row = Config::parse("os=linux").unwrap();
        let actual = Config::parse("os=linux;arch=x86_64").unwrap();
        assert!(row.holds_in(&actual));
        assert!(Config::new().holds_in(&actual));
        assert!(
            !row.holds_in(&Config::parse("os=windows").unwrap()),
            "control: a different value must not match"
        );
        assert!(
            !row.holds_in(&Config::new()),
            "control: an absent fact must not match"
        );
    }
}
