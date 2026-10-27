//! Typed preference values.

use std::borrow::Cow;
use std::fmt;

/// A preference value. Equality is type-aware: `Int(1)` is not `Bool(true)`
/// and `Int(15)` is not `Float(15.0)`, so a value stored with the wrong type
/// counts as a change.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(Cow<'static, str>),
    /// A stored value of a type macmaster does not write (data, array, dict, date).
    /// Carries the Core Foundation type name for display. Never equal to a desired value.
    Unsupported(String),
}

impl Value {
    /// Finnish name of the value's type, used when a change also changes the type.
    pub fn type_name_fi(&self) -> &str {
        match self {
            Value::Bool(_) => "totuusarvo",
            Value::Int(_) => "kokonaisluku",
            Value::Float(_) => "liukuluku",
            Value::Str(_) => "merkkijono",
            Value::Unsupported(name) => name,
        }
    }

    pub fn same_type(&self, other: &Value) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Bool(true) => f.write_str("tosi"),
            Value::Bool(false) => f.write_str("epätosi"),
            Value::Int(i) => write!(f, "{i}"),
            Value::Float(x) => write!(f, "{x:?}"),
            Value::Str(s) => write!(f, "\"{s}\""),
            Value::Unsupported(name) => write!(f, "<{name}>"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equality_is_type_aware() {
        assert_ne!(Value::Int(1), Value::Bool(true));
        assert_ne!(Value::Int(15), Value::Float(15.0));
        assert_eq!(Value::Int(2), Value::Int(2));
    }

    #[test]
    fn display_is_finnish() {
        assert_eq!(Value::Bool(false).to_string(), "epätosi");
        assert_eq!(Value::Float(15.0).to_string(), "15.0");
    }
}
