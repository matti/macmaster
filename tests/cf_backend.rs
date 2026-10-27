//! Round-trip through the real CFPreferences backend.
//! Uses a throwaway domain only; never touches NSGlobalDomain or real settings.

use macmaster::prefs::{CfPrefs, Domain, Prefs};
use macmaster::value::Value;

const DOMAIN: Domain = Domain::App("dev.triac.macmaster.test");

#[test]
fn round_trip_and_cleanup() {
    let mut p = CfPrefs;
    let cases = [
        ("bool", Value::Bool(true)),
        ("int", Value::Int(15)),
        ("float", Value::Float(2.5)),
        ("str", Value::Str("hei".into())),
    ];

    for (key, value) in &cases {
        p.write(DOMAIN, key, value).unwrap();
    }
    p.sync(DOMAIN).unwrap();
    for (key, value) in &cases {
        assert_eq!(p.read(DOMAIN, key).unwrap().as_ref(), Some(value), "{key}");
    }

    // Type change: int stored, bool wanted.
    p.write(DOMAIN, "int", &Value::Bool(false)).unwrap();
    assert_eq!(p.read(DOMAIN, "int").unwrap(), Some(Value::Bool(false)));

    for (key, _) in &cases {
        p.delete(DOMAIN, key);
    }
    p.sync(DOMAIN).unwrap();
    for (key, _) in &cases {
        assert_eq!(p.read(DOMAIN, key).unwrap(), None, "{key} not deleted");
    }

    // cfprefsd leaves an empty plist behind for the throwaway domain.
    if let Some(home) = std::env::var_os("HOME") {
        let plist =
            std::path::Path::new(&home).join("Library/Preferences/dev.triac.macmaster.test.plist");
        let _ = std::fs::remove_file(plist);
    }
}
