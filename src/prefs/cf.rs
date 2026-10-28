//! Real backend: CFPreferences for the current user on any host
//! (the same store `defaults` and System Settings use, via cfprefsd).

use super::{Domain, Prefs};
use crate::cfvalue::{to_value, with_cf};
use crate::error::{Error, Result};
use crate::value::Value;
use objc2_core_foundation::{
    CFPreferencesAppSynchronize, CFPreferencesCopyAppValue, CFPreferencesSetAppValue, CFRetained,
    CFString, CFType, Type, kCFPreferencesAnyApplication,
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
        with_cf(value, |cf| set(domain, key, Some(cf)))?;
        // CFPreferencesSetAppValue has no error result; read back to detect a rejected write.
        match self.read(domain, key)? {
            Some(ref stored) if stored == value => Ok(()),
            other => Err(Error(format!(
                "kirjoitus ei tallentunut: {domain} {key} = {}",
                other.map_or_else(|| "ei asetettu".into(), |v| v.to_string())
            ))),
        }
    }

    fn sync(&mut self, domain: Domain) -> Result<()> {
        if CFPreferencesAppSynchronize(&app_id(domain)) {
            Ok(())
        } else {
            Err(Error(format!(
                "CFPreferencesAppSynchronize epäonnistui: {domain}"
            )))
        }
    }
}
