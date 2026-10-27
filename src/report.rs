//! Finnish, human-readable output.

use crate::apply::{Entry, Mode, Outcome, Status};
use crate::prefs::Domain;
use crate::settings::Effect;
use crate::value::Value;

fn target(e: &Entry) -> String {
    match e.setting.domain {
        Domain::Global => e.setting.key.to_owned(),
        Domain::App(id) => format!("{id} {}", e.setting.key),
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
    let desired = &e.setting.value;
    let (label, detail) = match &e.status {
        Status::Unchanged => ("ennallaan", format!("{} = {desired}", target(e))),
        Status::WouldChange => (
            "muuttuisi",
            format!("{}: {}", target(e), transition(e.old.as_ref(), desired)),
        ),
        Status::Changed => (
            "muutettu",
            format!("{}: {}", target(e), transition(e.old.as_ref(), desired)),
        ),
        Status::Failed(err) => ("VIRHE", format!("{}: {err}", target(e))),
    };
    format!("{label:<10} {detail}  - {}", e.setting.description_fi)
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
    if o.needs(Effect::Logout) {
        out.push(if dry {
            "Näppäimistöasetukset muuttuisivat: näppäintoisto tulisi voimaan vasta, kun kirjaudut ulos ja takaisin sisään.".into()
        } else {
            "Näppäimistöasetukset muuttuivat: kirjaudu ulos ja takaisin sisään, jotta näppäintoisto tulee voimaan.".into()
        });
    }
    if o.needs(Effect::RelaunchApps) {
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
    use crate::prefs::memory::MemoryPrefs;
    use crate::settings::Setting;

    const TABLE: &[Setting] = &[Setting {
        domain: Domain::Global,
        key: "KeyRepeat",
        value: Value::Int(2),
        effect: Effect::Logout,
        description_fi: "Näppäimen toistonopeus",
    }];

    #[test]
    fn dry_run_lines_show_type_change_and_logout_note() {
        let mut p = MemoryPrefs::default().with(Domain::Global, "KeyRepeat", Value::Float(5.0));
        let o = apply(&mut p, TABLE, Mode::DryRun);
        assert_eq!(
            line(&o.entries[0]),
            "muuttuisi  KeyRepeat: 5.0 (liukuluku) -> 2 (kokonaisluku)  - Näppäimen toistonopeus"
        );
        let s = summary(&o);
        assert!(s[0].starts_with("Kuivaharjoitus"));
        assert!(s.iter().any(|l| l.contains("kirjaudut ulos")));
    }

    #[test]
    fn unchanged_has_no_notes() {
        let mut p = MemoryPrefs::default().with(Domain::Global, "KeyRepeat", Value::Int(2));
        let o = apply(&mut p, TABLE, Mode::Apply);
        assert_eq!(
            line(&o.entries[0]),
            "ennallaan  KeyRepeat = 2  - Näppäimen toistonopeus"
        );
        assert_eq!(summary(&o).len(), 1);
    }
}
