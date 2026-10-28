//! In-memory fake HID backend for unit tests.

use super::Hid;
use crate::error::{Error, Result};
use crate::value::Value;
use std::collections::{HashMap, HashSet};

#[derive(Default)]
pub struct MemoryHid {
    pub values: HashMap<String, Value>,
    /// Keys whose writes fail, to exercise error paths.
    pub failing_keys: HashSet<String>,
    pub writes: usize,
}

impl MemoryHid {
    pub fn with(mut self, key: &str, value: Value) -> Self {
        self.values.insert(key.to_owned(), value);
        self
    }

    pub fn get(&self, key: &str) -> Option<&Value> {
        self.values.get(key)
    }
}

impl Hid for MemoryHid {
    fn read(&self, key: &str) -> Result<Option<Value>> {
        Ok(self.get(key).cloned())
    }

    fn write(&mut self, key: &str, value: &Value) -> Result<()> {
        if self.failing_keys.contains(key) {
            return Err(Error(format!("simuloitu HID-virhe: {key}")));
        }
        self.writes += 1;
        self.values.insert(key.to_owned(), value.clone());
        Ok(())
    }
}
