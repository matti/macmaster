//! Preference storage abstraction. The real backend talks to CFPreferences;
//! tests use the in-memory fake.

mod cf;
#[cfg(test)]
pub mod memory;

pub use cf::CfPrefs;

use crate::value::Value;
use std::fmt;

/// A preferences domain for the current user on any host.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Domain {
    /// NSGlobalDomain (kCFPreferencesAnyApplication).
    Global,
    /// An application domain such as `com.apple.dock`.
    App(&'static str),
}

impl fmt::Display for Domain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Domain::Global => f.write_str("NSGlobalDomain"),
            Domain::App(id) => f.write_str(id),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrefsError(pub String);

impl fmt::Display for PrefsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for PrefsError {}

pub type Result<T> = std::result::Result<T, PrefsError>;

/// Minimal preferences backend.
pub trait Prefs {
    /// Current value, or `None` if the key is not set.
    fn read(&self, domain: Domain, key: &str) -> Result<Option<Value>>;
    fn write(&mut self, domain: Domain, key: &str, value: &Value) -> Result<()>;
    /// Flush pending writes for `domain` to disk.
    fn sync(&mut self, domain: Domain) -> Result<()>;
}
