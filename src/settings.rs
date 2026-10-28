//! The opinionated settings table. Adding a setting is one line in `SETTINGS`.

use crate::prefs::Domain;
use crate::value::Value;

/// What has to happen before a changed setting takes effect.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Effect {
    /// Key repeat (1/60 s ticks): macmaster also sets this HID event system property, in ns,
    /// so the running session uses it at once. The plist value is what each login restores.
    LiveHid(&'static str),
    /// Read by each app at launch: relaunch running apps.
    RelaunchApps,
    /// macmaster restarts this process (`killall <name>`) after a change, e.g. "Dock".
    Restart(&'static str),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Setting {
    pub domain: Domain,
    pub key: &'static str,
    pub value: Value,
    pub effect: Effect,
    pub description_fi: &'static str,
}

const fn global(
    key: &'static str,
    value: Value,
    effect: Effect,
    description_fi: &'static str,
) -> Setting {
    Setting {
        domain: Domain::Global,
        key,
        value,
        effect,
        description_fi,
    }
}

use Effect::{LiveHid, RelaunchApps};
use Value::{Bool, Float, Int};

#[rustfmt::skip]
pub const SETTINGS: &[Setting] = &[
    // Keyboard: the fastest repeat System Settings allows, not below it.
    global("InitialKeyRepeat", Int(12), LiveHid("HIDInitialKeyRepeat"), "Viive ennen toistoa: System Settingsin minimiä lyhyempi (12 = 200 ms)"),
    global("KeyRepeat", Float(1.5), LiveHid("HIDKeyRepeat"), "Näppäimen toistonopeus: System Settingsin minimiä nopeampi (1.5 = 25 ms)"),
    global("ApplePressAndHoldEnabled", Bool(false), RelaunchApps, "Pohjassa pidetty näppäin toistaa merkkiä, ei avaa aksenttivalikkoa"),
    // Text input: no automatic corrections or substitutions.
    global("NSAutomaticSpellingCorrectionEnabled", Bool(false), RelaunchApps, "Oikeinkirjoituksen automaattinen korjaus pois"),
    global("WebAutomaticSpellingCorrectionEnabled", Bool(false), RelaunchApps, "Automaattinen korjaus pois myös WebKit-näkymissä (Safari, Mail)"),
    global("NSAutomaticCapitalizationEnabled", Bool(false), RelaunchApps, "Automaattiset isot alkukirjaimet pois"),
    global("NSAutomaticPeriodSubstitutionEnabled", Bool(false), RelaunchApps, "Piste kahdella välilyönnillä pois"),
    global("NSAutomaticDashSubstitutionEnabled", Bool(false), RelaunchApps, "Älykkäät ajatusviivat pois"),
    global("NSAutomaticQuoteSubstitutionEnabled", Bool(false), RelaunchApps, "Kaarevat lainausmerkit pois"),
    global("NSAutomaticInlinePredictionEnabled", Bool(false), RelaunchApps, "Tekstin mukana näkyvä ennakoiva teksti pois"),
];

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn keys_are_unique_per_domain() {
        let mut seen = HashSet::new();
        for s in SETTINGS {
            assert!(
                seen.insert((s.domain, s.key)),
                "duplicate setting {}",
                s.key
            );
        }
    }

    #[test]
    fn live_hid_values_are_numeric() {
        for s in SETTINGS {
            if let Effect::LiveHid(key) = s.effect {
                assert!(crate::hid::ticks_to_ns(&s.value).is_some(), "{key}");
            }
        }
    }

    #[test]
    fn key_repeat_not_below_gui_minimum() {
        let get = |k| SETTINGS.iter().find(|s| s.key == k).map(|s| &s.value);
        assert_eq!(get("InitialKeyRepeat"), Some(&Int(12)));
        assert_eq!(get("KeyRepeat"), Some(&Float(1.5)));
    }
}
