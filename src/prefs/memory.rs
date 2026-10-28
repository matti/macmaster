//! In-memory fake backend for unit tests.

use super::{Domain, Prefs};
use crate::error::{Error, Result};
use crate::value::Value;
use std::collections::{HashMap, HashSet};

#[derive(Default)]
pub struct MemoryPrefs {
    pub values: HashMap<(Domain, String), Value>,
    /// Keys whose writes fail, to exercise error paths.
    pub failing_keys: HashSet<String>,
    pub writes: usize,
    pub syncs: Vec<Domain>,
}

impl MemoryPrefs {
    pub fn with(mut self, domain: Domain, key: &str, value: Value) -> Self {
        self.values.insert((domain, key.to_owned()), value);
        self
    }

    pub fn get(&self, domain: Domain, key: &str) -> Option<&Value> {
        self.values.get(&(domain, key.to_owned()))
    }
}

impl Prefs for MemoryPrefs {
    fn read(&self, domain: Domain, key: &str) -> Result<Option<Value>> {
        Ok(self.get(domain, key).cloned())
    }

    fn write(&mut self, domain: Domain, key: &str, value: &Value) -> Result<()> {
        if self.failing_keys.contains(key) {
            return Err(Error(format!("simuloitu virhe: {key}")));
        }
        self.writes += 1;
        self.values.insert((domain, key.to_owned()), value.clone());
        Ok(())
    }

    fn sync(&mut self, domain: Domain) -> Result<()> {
        self.syncs.push(domain);
        Ok(())
    }
}
