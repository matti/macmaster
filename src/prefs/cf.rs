//! Real backend: CFPreferences for the current user on any host
//! (the same store `defaults` and System Settings use, via cfprefsd).

use super::{Domain, Prefs, PrefsError, Result};
use crate::value::Value;
use objc2_core_foundation::{
    CFBoolean, CFGetTypeID, CFNumber, CFPreferencesAppSynchronize, CFPreferencesCopyAppValue,
    CFPreferencesSetAppValue, CFRetained, CFString, CFType, Type, kCFPreferencesAnyApplication,
};

#[derive(Debug, Default)]
pub struct CfPrefs;

fn app_id(domain: Domain) -> CFRetained<CFString> {
    match domain {
        // SAFETY: immutable CFString constant exported by CoreFoundation.
        Domain::Global => unsafe { kCFPreferencesAnyApplication }.retain(),
        Domain::App(id) => CFString::from_str(id),
    }
}

fn to_value(v: &CFType) -> Value {
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

fn set(domain: Domain, key: &str, value: Option<&CFType>) {
    // SAFETY: key, value and app id are valid CF objects that outlive the call;
    // CFPreferencesSetAppValue retains what it stores.
    unsafe { CFPreferencesSetAppValue(&CFString::from_str(key), value, &app_id(domain)) }
}

impl CfPrefs {
    /// Remove `key` from `domain` (writes are flushed by `sync`).
    pub fn delete(&mut self, domain: Domain, key: &str) {
        set(domain, key, None);
    }
}

impl Prefs for CfPrefs {
    fn read(&self, domain: Domain, key: &str) -> Result<Option<Value>> {
        let v = CFPreferencesCopyAppValue(&CFString::from_str(key), &app_id(domain));
        Ok(v.as_deref().map(to_value))
    }

    fn write(&mut self, domain: Domain, key: &str, value: &Value) -> Result<()> {
        match value {
            Value::Bool(b) => set(domain, key, Some(CFBoolean::new(*b))),
            Value::Int(i) => set(domain, key, Some(&CFNumber::new_i64(*i))),
            Value::Float(x) => set(domain, key, Some(&CFNumber::new_f64(*x))),
            Value::Str(s) => set(domain, key, Some(&CFString::from_str(s))),
            Value::Unsupported(t) => {
                return Err(PrefsError(format!("ei voi kirjoittaa tyyppiä {t}")));
            }
        }
        // CFPreferencesSetAppValue has no error result; read back to detect a rejected write.
        match self.read(domain, key)? {
            Some(ref stored) if stored == value => Ok(()),
            other => Err(PrefsError(format!(
                "kirjoitus ei tallentunut: {domain} {key} = {}",
                other.map_or_else(|| "ei asetettu".into(), |v| v.to_string())
            ))),
        }
    }

    fn sync(&mut self, domain: Domain) -> Result<()> {
        if CFPreferencesAppSynchronize(&app_id(domain)) {
            Ok(())
        } else {
            Err(PrefsError(format!(
                "CFPreferencesAppSynchronize epäonnistui: {domain}"
            )))
        }
    }
}
