//! Diff the settings table against the backends and (unless dry-run) write the difference.
//! Each setting is one stored preference; a `LiveHid` setting is also a live HID property.

use crate::error::{Error, Result};
use crate::hid::{Hid, ticks_to_ns};
use crate::prefs::{Domain, Prefs};
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
    Failed(Error),
}

/// Where a value lives.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    /// Stored preference: persists, read at login or app launch.
    Pref(Domain, &'static str),
    /// Live HID event system property: what the running session uses now.
    Hid(&'static str),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Entry<'a> {
    pub setting: &'a Setting,
    pub target: Target,
    pub desired: Value,
    /// Value before macmaster ran; `None` if not set.
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

    /// True if a setting read at app launch differs: running apps need a relaunch.
    pub fn needs_relaunch(&self) -> bool {
        self.entries
            .iter()
            .any(|e| e.differs() && e.setting.effect == Effect::RelaunchApps)
    }

    /// True if a live HID property could not be read or set: only a new login applies it.
    pub fn needs_logout(&self) -> bool {
        self.entries
            .iter()
            .any(|e| matches!(e.target, Target::Hid(_)) && matches!(e.status, Status::Failed(_)))
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

/// Compares `read` with `desired` and, when applying, writes the difference.
fn reconcile(
    read: Result<Option<Value>>,
    desired: &Value,
    mode: Mode,
    write: impl FnOnce() -> Result<()>,
) -> (Option<Value>, Status) {
    match read {
        Err(err) => (None, Status::Failed(err)),
        Ok(old) if old.as_ref().is_some_and(|v| v.satisfies(desired)) => (old, Status::Unchanged),
        Ok(old) if mode == Mode::DryRun => (old, Status::WouldChange),
        Ok(old) => match write() {
            Ok(()) => (old, Status::Changed),
            Err(err) => (old, Status::Failed(err)),
        },
    }
}

fn pref_entry<'a>(prefs: &mut impl Prefs, setting: &'a Setting, mode: Mode) -> Entry<'a> {
    let (domain, key, desired) = (setting.domain, setting.key, &setting.value);
    let (old, status) = reconcile(prefs.read(domain, key), desired, mode, || {
        prefs.write(domain, key, desired)
    });
    Entry {
        setting,
        target: Target::Pref(domain, key),
        desired: desired.clone(),
        old,
        status,
    }
}

fn hid_entry<'a>(
    hid: &mut impl Hid,
    setting: &'a Setting,
    key: &'static str,
    mode: Mode,
) -> Entry<'a> {
    let (desired, (old, status)) = match ticks_to_ns(&setting.value) {
        Some(ns) => {
            let result = reconcile(hid.read(key), &ns, mode, || hid.write(key, &ns));
            (ns, result)
        }
        None => {
            let err = Error(format!("{} ei ole luku", setting.key));
            (setting.value.clone(), (None, Status::Failed(err)))
        }
    };
    Entry {
        setting,
        target: Target::Hid(key),
        desired,
        old,
        status,
    }
}

pub fn apply<'a>(
    prefs: &mut impl Prefs,
    hid: &mut impl Hid,
    settings: &'a [Setting],
    mode: Mode,
) -> Outcome<'a> {
    let mut entries = Vec::new();
    for setting in settings {
        entries.push(pref_entry(prefs, setting, mode));
        if let Effect::LiveHid(key) = setting.effect {
            entries.push(hid_entry(hid, setting, key, mode));
        }
    }

    // Flush each written domain once; a failed flush fails every write in it.
    let mut written: Vec<Domain> = Vec::new();
    for e in &entries {
        if let (Target::Pref(domain, _), Status::Changed) = (e.target, &e.status)
            && !written.contains(&domain)
        {
            written.push(domain);
        }
    }
    for domain in written {
        if let Err(err) = prefs.sync(domain) {
            for e in entries.iter_mut().filter(|e| {
                matches!(e.target, Target::Pref(d, _) if d == domain) && e.status == Status::Changed
            }) {
                e.status = Status::Failed(err.clone());
            }
        }
    }

    Outcome { mode, entries }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hid::memory::MemoryHid;
    use crate::prefs::memory::MemoryPrefs;
    use crate::settings::Effect::{LiveHid, RelaunchApps, Restart};
    use Value::{Bool, Float, Int};

    const G: Domain = Domain::Global;
    const DOCK: Domain = Domain::App("com.apple.dock");
    const HID_KR: &str = "HIDKeyRepeat";
    const NS_2: Value = Int(33_333_333);

    const TABLE: &[Setting] = &[
        Setting {
            domain: G,
            key: "KeyRepeat",
            value: Int(2),
            effect: LiveHid(HID_KR),
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

    // Entry indices: the live HID entry follows its preference entry.
    const KR: usize = 0;
    const KR_LIVE: usize = 1;
    const FLAG: usize = 2;

    fn all_set() -> MemoryPrefs {
        MemoryPrefs::default()
            .with(G, "KeyRepeat", Int(2))
            .with(G, "Flag", Bool(false))
            .with(DOCK, "autohide", Bool(true))
            .with(DOCK, "tilesize", Int(40))
    }

    fn live_set() -> MemoryHid {
        MemoryHid::default().with(HID_KR, NS_2)
    }

    fn statuses(o: &Outcome) -> Vec<Status> {
        o.entries.iter().map(|e| e.status.clone()).collect()
    }

    #[test]
    fn unchanged_writes_and_syncs_nothing() {
        let (mut p, mut h) = (all_set(), live_set());
        let o = apply(&mut p, &mut h, TABLE, Mode::Apply);
        assert_eq!(o.entries.len(), 5);
        assert!(statuses(&o).iter().all(|s| *s == Status::Unchanged));
        assert_eq!((p.writes, h.writes), (0, 0));
        assert!(p.syncs.is_empty());
        assert!(!o.needs_logout() && !o.needs_relaunch());
        assert!(o.processes_to_restart().is_empty());
    }

    #[test]
    fn changed_value_is_written_and_domain_synced_once() {
        let mut p = all_set()
            .with(G, "KeyRepeat", Int(5))
            .with(DOCK, "autohide", Bool(false))
            .with(DOCK, "tilesize", Int(64));
        let mut h = MemoryHid::default().with(HID_KR, Int(83_333_333));
        let o = apply(&mut p, &mut h, TABLE, Mode::Apply);
        assert_eq!(o.entries[KR].status, Status::Changed);
        assert_eq!(o.entries[KR].old, Some(Int(5)));
        assert_eq!(o.entries[KR_LIVE].target, Target::Hid(HID_KR));
        assert_eq!(o.entries[KR_LIVE].status, Status::Changed);
        assert_eq!(o.entries[KR_LIVE].old, Some(Int(83_333_333)));
        assert_eq!(o.entries[FLAG].status, Status::Unchanged);
        assert_eq!(p.get(G, "KeyRepeat"), Some(&Int(2)));
        assert_eq!(h.get(HID_KR), Some(&NS_2));
        assert_eq!(p.writes, 3);
        assert_eq!(p.syncs, vec![G, DOCK]);
        assert!(!o.needs_logout());
        assert!(!o.needs_relaunch());
        assert_eq!(o.processes_to_restart(), vec!["Dock"]);
    }

    #[test]
    fn stale_live_value_is_set_even_when_plist_is_correct() {
        let mut p = all_set();
        let mut h = MemoryHid::default().with(HID_KR, Int(83_333_333));
        let o = apply(&mut p, &mut h, TABLE, Mode::Apply);
        assert_eq!(o.entries[KR].status, Status::Unchanged);
        assert_eq!(o.entries[KR_LIVE].status, Status::Changed);
        assert_eq!(h.get(HID_KR), Some(&NS_2));
        assert_eq!((p.writes, h.writes), (0, 1));
        assert!(p.syncs.is_empty());
    }

    #[test]
    fn missing_key_is_a_change() {
        let (mut p, mut h) = (MemoryPrefs::default(), MemoryHid::default());
        let o = apply(&mut p, &mut h, TABLE, Mode::Apply);
        assert!(
            o.entries
                .iter()
                .all(|e| e.old.is_none() && e.status == Status::Changed)
        );
        assert_eq!(p.get(DOCK, "tilesize"), Some(&Int(40)));
        assert_eq!(h.get(HID_KR), Some(&NS_2));
        assert!(o.needs_relaunch());
    }

    #[test]
    fn bool_type_change_counts_but_int_float_does_not() {
        let mut p = all_set()
            .with(G, "Flag", Int(0))
            .with(G, "KeyRepeat", Float(2.0));
        let o = apply(&mut p, &mut live_set(), TABLE, Mode::Apply);
        assert_eq!(o.entries[KR].status, Status::Unchanged);
        assert_eq!(o.entries[FLAG].status, Status::Changed);
        assert_eq!(p.get(G, "Flag"), Some(&Bool(false)));
        assert_eq!(p.get(G, "KeyRepeat"), Some(&Float(2.0)));
    }

    #[test]
    fn dry_run_writes_nothing() {
        let mut p = all_set().with(G, "KeyRepeat", Int(5));
        let mut h = MemoryHid::default().with(HID_KR, Int(83_333_333));
        let (before, live_before) = (p.values.clone(), h.values.clone());
        let o = apply(&mut p, &mut h, TABLE, Mode::DryRun);
        assert_eq!(o.entries[KR].status, Status::WouldChange);
        assert_eq!(o.entries[KR_LIVE].status, Status::WouldChange);
        assert_eq!(p.values, before);
        assert_eq!(h.values, live_before);
        assert_eq!((p.writes, h.writes), (0, 0));
        assert!(p.syncs.is_empty());
        assert!(!o.needs_logout());
    }

    #[test]
    fn write_failure_is_reported_and_others_continue() {
        let mut p = MemoryPrefs::default();
        p.failing_keys.insert("Flag".into());
        let o = apply(&mut p, &mut MemoryHid::default(), TABLE, Mode::Apply);
        assert!(matches!(o.entries[FLAG].status, Status::Failed(_)));
        assert_eq!(o.entries[KR].status, Status::Changed);
        assert!(o.has_failures());
        assert!(!o.needs_logout());
    }

    #[test]
    fn live_failure_keeps_plist_write_and_needs_logout() {
        let mut p = MemoryPrefs::default();
        let mut h = MemoryHid::default();
        h.failing_keys.insert(HID_KR.into());
        let o = apply(&mut p, &mut h, TABLE, Mode::Apply);
        assert_eq!(o.entries[KR].status, Status::Changed);
        assert!(matches!(o.entries[KR_LIVE].status, Status::Failed(_)));
        assert_eq!(p.get(G, "KeyRepeat"), Some(&Int(2)));
        assert!(o.needs_logout());
    }
}
