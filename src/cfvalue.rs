//! Conversions between [`Value`] and Core Foundation objects, shared by the real backends.

use crate::error::{Error, Result};
use crate::value::Value;
use objc2_core_foundation::{CFBoolean, CFGetTypeID, CFNumber, CFString, CFType};

pub fn to_value(v: &CFType) -> Value {
    if let Some(b) = v.downcast_ref::<CFBoolean>() {
        Value::Bool(b.as_bool())
    } else if let Some(n) = v.downcast_ref::<CFNumber>() {
        match (n.is_float_type(), n.as_i64(), n.as_f64()) {
            (false, Some(i), _) => Value::Int(i),
            (_, _, Some(x)) => Value::Float(x),
            _ => Value::Unsupported("CFNumber".into()),
        }
    } else if let Some(s) = v.downcast_ref::<CFString>() {
        Value::Str(s.to_string().into())
    } else {
        Value::Unsupported(format!("CFTypeID {}", CFGetTypeID(Some(v))))
    }
}

/// Calls `f` with `value` as a Core Foundation object.
pub fn with_cf<R>(value: &Value, f: impl FnOnce(&CFType) -> R) -> Result<R> {
    Ok(match value {
        Value::Bool(b) => f(CFBoolean::new(*b)),
        Value::Int(i) => f(&CFNumber::new_i64(*i)),
        Value::Float(x) => f(&CFNumber::new_f64(*x)),
        Value::Str(s) => f(&CFString::from_str(s)),
        Value::Unsupported(t) => return Err(Error(format!("ei voi kirjoittaa tyyppiä {t}"))),
    })
}
