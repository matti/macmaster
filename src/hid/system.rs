//! Real backend: the HID event system via IOHIDEventSystemClient (what `hidutil property` uses).
//! Setting key repeat here needs no privileges and applies to the running session at once.

use super::Hid;
use crate::cfvalue::{to_value, with_cf};
use crate::error::{Error, Result};
use crate::value::Value;
use objc2_core_foundation::{CFRetained, CFString};
use objc2_io_kit::IOHIDEventSystemClient;

pub struct HidSystem {
    client: CFRetained<IOHIDEventSystemClient>,
}

impl HidSystem {
    pub fn new() -> Self {
        Self {
            client: IOHIDEventSystemClient::new_simple_client(None),
        }
    }
}

impl Default for HidSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl Hid for HidSystem {
    fn read(&self, key: &str) -> Result<Option<Value>> {
        Ok(self
            .client
            .property(&CFString::from_str(key))
            .as_deref()
            .map(to_value))
    }

    fn write(&mut self, key: &str, value: &Value) -> Result<()> {
        let key_cf = CFString::from_str(key);
        // SAFETY: the HID properties macmaster sets (key repeat) take a CFNumber, and
        // `ticks_to_ns` only produces numbers.
        let ok = with_cf(value, |cf| unsafe { self.client.set_property(&key_cf, cf) })?;
        if !ok {
            return Err(Error(format!("HID-ominaisuuden {key} asetus epäonnistui")));
        }
        // Read back so a silently ignored set is reported, not assumed.
        match self.read(key)? {
            Some(ref live) if live.satisfies(value) => Ok(()),
            other => Err(Error(format!(
                "HID-ominaisuus {key} ei muuttunut: {}",
                other.map_or_else(|| "ei saatavilla".into(), |v| v.to_string())
            ))),
        }
    }
}
