//! Live HID event system properties (the values the running session uses right now).
//! The real backend talks to IOHIDEventSystemClient; tests use the in-memory fake.

#[cfg(test)]
pub mod memory;
mod system;

pub use system::HidSystem;

use crate::error::Result;
use crate::value::Value;

/// Minimal live HID properties backend.
pub trait Hid {
    /// Current value, or `None` if the property is not available.
    fn read(&self, key: &str) -> Result<Option<Value>>;
    /// Sets the property; takes effect immediately.
    fn write(&mut self, key: &str, value: &Value) -> Result<()>;
}

/// Key repeat defaults are in ticks of 1/60 s; HID wants nanoseconds.
const NS_PER_SECOND: i64 = 1_000_000_000;
const TICKS_PER_SECOND: i64 = 60;

/// Converts a key repeat defaults value (1/60 s ticks) into the HID value in nanoseconds,
/// truncating like System Settings does (2 -> 33333333, 15 -> 250000000).
/// `None` for non-numeric values.
pub fn ticks_to_ns(ticks: &Value) -> Option<Value> {
    match ticks {
        Value::Int(i) => Some(Value::Int(i * NS_PER_SECOND / TICKS_PER_SECOND)),
        Value::Float(x) => Some(Value::Int(
            (x * NS_PER_SECOND as f64 / TICKS_PER_SECOND as f64) as i64,
        )),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ticks_convert_to_ns_like_system_settings() {
        assert_eq!(ticks_to_ns(&Value::Int(2)), Some(Value::Int(33_333_333)));
        assert_eq!(ticks_to_ns(&Value::Int(5)), Some(Value::Int(83_333_333)));
        assert_eq!(ticks_to_ns(&Value::Int(15)), Some(Value::Int(250_000_000)));
        assert_eq!(
            ticks_to_ns(&Value::Float(2.0)),
            Some(Value::Int(33_333_333))
        );
        assert_eq!(ticks_to_ns(&Value::Bool(true)), None);
    }
}
