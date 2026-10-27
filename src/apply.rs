//! Diff the settings table against the backend and (unless dry-run) write the difference.

use crate::prefs::{Domain, Prefs, PrefsError};
use crate::settings::{Effect, Setting};
use crate::value::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Apply,
    DryRun,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Status {
    Unchanged,
    /// Dry-run: differs, nothing written.
    WouldChange,
    Changed,
    Failed(PrefsError),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Entry<'a> {
    pub setting: &'a Setting,
    /// Value before macmaster ran; `None` if the key was not set.
    pub old: Option<Value>,
    pub status: Status,
}

impl Entry<'_> {
    /// Differs from the desired value (changed, or would change in a dry-run).
    pub fn differs(&self) -> bool {
        matches!(self.status, Status::Changed | Status::WouldChange)
    }
}

#[derive(Debug)]
pub struct Outcome<'a> {
    pub mode: Mode,
    pub entries: Vec<Entry<'a>>,
}

impl Outcome<'_> {
    pub fn count(&self, pred: impl Fn(&Status) -> bool) -> usize {
        self.entries.iter().filter(|e| pred(&e.status)).count()
    }

    pub fn has_failures(&self) -> bool {
        self.count(|s| matches!(s, Status::Failed(_))) > 0
    }

    /// True if any setting with this effect differs.
    pub fn needs(&self, effect: Effect) -> bool {
        self.entries
            .iter()
            .any(|e| e.differs() && e.setting.effect == effect)
    }

    /// Processes to restart, deduplicated, in table order.
    pub fn processes_to_restart(&self) -> Vec<&'static str> {
        let mut out = Vec::new();
        for e in self.entries.iter().filter(|e| e.differs()) {
            if let Effect::Restart(name) = e.setting.effect
                && !out.contains(&name)
            {
                out.push(name);
            }
        }
        out
    }
}

pub fn apply<'a>(prefs: &mut impl Prefs, settings: &'a [Setting], mode: Mode) -> Outcome<'a> {
    let mut dirty: Vec<Domain> = Vec::new();
    let mut entries: Vec<Entry<'a>> = settings
        .iter()
        .map(|setting| {
            let (old, status) = match prefs.read(setting.domain, setting.key) {
                Err(err) => (None, Status::Failed(err)),
                Ok(old) if old.as_ref().is_some_and(|v| v.satisfies(&setting.value)) => {
                    (old, Status::Unchanged)
                }
                Ok(old) if mode == Mode::DryRun => (old, Status::WouldChange),
                Ok(old) => match prefs.write(setting.domain, setting.key, &setting.value) {
                    Ok(()) => {
                        if !dirty.contains(&setting.domain) {
                            dirty.push(setting.domain);
                        }
                        (old, Status::Changed)
                    }
                    Err(err) => (old, Status::Failed(err)),
                },
            };
            Entry {
                setting,
                old,
                status,
            }
        })
        .collect();

    for domain in dirty {
        if let Err(err) = prefs.sync(domain) {
            for e in entries
                .iter_mut()
                .filter(|e| e.setting.domain == domain && e.differs())
            {
                e.status = Status::Failed(err.clone());
            }
        }
    }

    Outcome { mode, entries }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prefs::memory::MemoryPrefs;
    use crate::settings::Effect::{Logout, RelaunchApps, Restart};
    use Value::{Bool, Float, Int};

    const G: Domain = Domain::Global;
    const DOCK: Domain = Domain::App("com.apple.dock");

    const TABLE: &[Setting] = &[
        Setting {
            domain: G,
            key: "KeyRepeat",
            value: Int(2),
            effect: Logout,
            description_fi: "a",
        },
        Setting {
            domain: G,
            key: "Flag",
            value: Bool(false),
            effect: RelaunchApps,
            description_fi: "b",
        },
        Setting {
            domain: DOCK,
            key: "autohide",
            value: Bool(true),
            effect: Restart("Dock"),
            description_fi: "c",
        },
        Setting {
            domain: DOCK,
            key: "tilesize",
            value: Int(40),
            effect: Restart("Dock"),
            description_fi: "d",
        },
    ];

    fn all_set() -> MemoryPrefs {
        MemoryPrefs::default()
            .with(G, "KeyRepeat", Int(2))
            .with(G, "Flag", Bool(false))
            .with(DOCK, "autohide", Bool(true))
            .with(DOCK, "tilesize", Int(40))
    }

    fn statuses(o: &Outcome) -> Vec<Status> {
        o.entries.iter().map(|e| e.status.clone()).collect()
    }

    #[test]
    fn unchanged_writes_and_syncs_nothing() {
        let mut p = all_set();
        let o = apply(&mut p, TABLE, Mode::Apply);
        assert!(statuses(&o).iter().all(|s| *s == Status::Unchanged));
        assert_eq!(p.writes, 0);
        assert!(p.syncs.is_empty());
        assert!(!o.needs(Logout));
        assert!(o.processes_to_restart().is_empty());
    }

    #[test]
    fn changed_value_is_written_and_domain_synced_once() {
        let mut p = all_set()
            .with(G, "KeyRepeat", Int(5))
            .with(DOCK, "autohide", Bool(false))
            .with(DOCK, "tilesize", Int(64));
        let o = apply(&mut p, TABLE, Mode::Apply);
        assert_eq!(o.entries[0].status, Status::Changed);
        assert_eq!(o.entries[0].old, Some(Int(5)));
        assert_eq!(o.entries[1].status, Status::Unchanged);
        assert_eq!(p.get(G, "KeyRepeat"), Some(&Int(2)));
        assert_eq!(p.writes, 3);
        assert_eq!(p.syncs, vec![G, DOCK]);
        assert!(o.needs(Logout));
        assert!(!o.needs(RelaunchApps));
        assert_eq!(o.processes_to_restart(), vec!["Dock"]);
    }

    #[test]
    fn missing_key_is_a_change() {
        let mut p = MemoryPrefs::default();
        let o = apply(&mut p, TABLE, Mode::Apply);
        assert!(
            o.entries
                .iter()
                .all(|e| e.old.is_none() && e.status == Status::Changed)
        );
        assert_eq!(p.get(DOCK, "tilesize"), Some(&Int(40)));
    }

    #[test]
    fn bool_type_change_counts_but_int_float_does_not() {
        let mut p = all_set()
            .with(G, "Flag", Int(0))
            .with(G, "KeyRepeat", Float(2.0));
        let o = apply(&mut p, TABLE, Mode::Apply);
        assert_eq!(o.entries[0].status, Status::Unchanged);
        assert_eq!(o.entries[1].status, Status::Changed);
        assert_eq!(p.get(G, "Flag"), Some(&Bool(false)));
        assert_eq!(p.get(G, "KeyRepeat"), Some(&Float(2.0)));
    }

    #[test]
    fn dry_run_writes_nothing() {
        let mut p = all_set().with(G, "KeyRepeat", Int(5));
        let before = p.values.clone();
        let o = apply(&mut p, TABLE, Mode::DryRun);
        assert_eq!(o.entries[0].status, Status::WouldChange);
        assert_eq!(p.values, before);
        assert_eq!(p.writes, 0);
        assert!(p.syncs.is_empty());
        assert!(o.needs(Logout));
    }

    #[test]
    fn write_failure_is_reported_and_others_continue() {
        let mut p = MemoryPrefs::default();
        p.failing_keys.insert("Flag".into());
        let o = apply(&mut p, TABLE, Mode::Apply);
        assert!(matches!(o.entries[1].status, Status::Failed(_)));
        assert_eq!(o.entries[0].status, Status::Changed);
        assert!(o.has_failures());
    }
}
