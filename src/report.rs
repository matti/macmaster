//! Finnish, human-readable output.

use crate::apply::{Entry, Mode, Outcome, Status, Target};
use crate::prefs::Domain;
use crate::value::Value;

fn target(e: &Entry) -> String {
    match e.target {
        Target::Pref(Domain::Global, key) => key.to_owned(),
        Target::Pref(Domain::App(id), key) => format!("{id} {key}"),
        Target::Hid(key) => format!("{key} (ns)"),
    }
}

fn description(e: &Entry) -> String {
    match e.target {
        Target::Pref(..) => e.setting.description_fi.to_owned(),
        Target::Hid(_) => format!("{} heti voimaan ilman uloskirjautumista", e.setting.key),
    }
}

/// Shows types only when a change also changes the type.
fn transition(old: Option<&Value>, new: &Value) -> String {
    match old {
        None => format!("ei asetettu -> {new}"),
        Some(old) if old.same_type(new) => format!("{old} -> {new}"),
        Some(old) => format!(
            "{old} ({}) -> {new} ({})",
            old.type_name_fi(),
            new.type_name_fi()
        ),
    }
}

pub fn line(e: &Entry) -> String {
    let (label, detail) = match &e.status {
        Status::Unchanged => ("ennallaan", format!("{} = {}", target(e), e.desired)),
        Status::WouldChange => (
            "muuttuisi",
            format!("{}: {}", target(e), transition(e.old.as_ref(), &e.desired)),
        ),
        Status::Changed => (
            "muutettu",
            format!("{}: {}", target(e), transition(e.old.as_ref(), &e.desired)),
        ),
        Status::Failed(err) => ("VIRHE", format!("{}: {err}", target(e))),
    };
    format!("{label:<10} {detail}  - {}", description(e))
}

pub fn summary(o: &Outcome) -> Vec<String> {
    let dry = o.mode == Mode::DryRun;
    let differ = o.count(|s| matches!(s, Status::Changed | Status::WouldChange));
    let same = o.count(|s| *s == Status::Unchanged);
    let failed = o.count(|s| matches!(s, Status::Failed(_)));
    let mut out = vec![if dry {
        format!(
            "Kuivaharjoitus, mitään ei kirjoitettu. Muuttuisi: {differ}, ennallaan: {same}, virheitä: {failed}."
        )
    } else {
        format!("Valmis. Muutettu: {differ}, ennallaan: {same}, virheitä: {failed}.")
    }];
    if o.needs_logout() {
        out.push(
            "Näppäintoistoa ei saatu voimaan heti: se tulee voimaan, kun kirjaudut ulos ja takaisin sisään."
                .into(),
        );
    }
    if o.needs_relaunch() {
        out.push(
            "Jo auki olevat ohjelmat näkevät muutokset vasta, kun ne käynnistetään uudelleen."
                .into(),
        );
    }
    let procs = o.processes_to_restart();
    if !procs.is_empty() {
        let verb = if dry {
            "Käynnistettäisiin uudelleen"
        } else {
            "Käynnistetään uudelleen"
        };
        out.push(format!("{verb}: {}", procs.join(", ")));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::apply::apply;
    use crate::hid::memory::MemoryHid;
    use crate::prefs::memory::MemoryPrefs;
    use crate::settings::{Effect, Setting};

    const G: Domain = Domain::Global;

    const TABLE: &[Setting] = &[Setting {
        domain: G,
        key: "KeyRepeat",
        value: Value::Int(2),
        effect: Effect::LiveHid("HIDKeyRepeat"),
        description_fi: "Näppäimen toistonopeus",
    }];

    fn notes(o: &Outcome) -> Vec<String> {
        summary(o).split_off(1)
    }

    #[test]
    fn dry_run_lines_show_type_change_and_live_line_without_logout_note() {
        let mut p = MemoryPrefs::default().with(G, "KeyRepeat", Value::Float(5.0));
        let mut h = MemoryHid::default().with("HIDKeyRepeat", Value::Int(83_333_333));
        let o = apply(&mut p, &mut h, TABLE, Mode::DryRun);
        assert_eq!(
            line(&o.entries[0]),
            "muuttuisi  KeyRepeat: 5.0 (liukuluku) -> 2 (kokonaisluku)  - Näppäimen toistonopeus"
        );
        assert_eq!(
            line(&o.entries[1]),
            "muuttuisi  HIDKeyRepeat (ns): 83333333 -> 33333333  - KeyRepeat heti voimaan ilman uloskirjautumista"
        );
        assert!(summary(&o)[0].starts_with("Kuivaharjoitus"));
        assert!(notes(&o).is_empty());
    }

    #[test]
    fn failed_live_apply_asks_for_logout() {
        let mut p = MemoryPrefs::default();
        let mut h = MemoryHid::default();
        h.failing_keys.insert("HIDKeyRepeat".into());
        let o = apply(&mut p, &mut h, TABLE, Mode::Apply);
        assert_eq!(
            line(&o.entries[1]),
            "VIRHE      HIDKeyRepeat (ns): simuloitu HID-virhe: HIDKeyRepeat  - KeyRepeat heti voimaan ilman uloskirjautumista"
        );
        assert!(notes(&o).iter().any(|l| l.contains("kirjaudut ulos")));
    }

    #[test]
    fn unchanged_has_no_notes() {
        let mut p = MemoryPrefs::default().with(G, "KeyRepeat", Value::Int(2));
        let mut h = MemoryHid::default().with("HIDKeyRepeat", Value::Int(33_333_333));
        let o = apply(&mut p, &mut h, TABLE, Mode::Apply);
        assert_eq!(
            line(&o.entries[0]),
            "ennallaan  KeyRepeat = 2  - Näppäimen toistonopeus"
        );
        assert_eq!(
            summary(&o),
            vec!["Valmis. Muutettu: 0, ennallaan: 2, virheitä: 0."]
        );
    }
}
