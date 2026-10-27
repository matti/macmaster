# macmaster

One command that configures a Mac the way we like it. No config file, no questions.

## Usage

```sh
macmaster            # apply every setting for the current user
macmaster --dry-run  # (-n) show what would change, write nothing
macmaster --help
```

Running it again is safe: settings already in place are reported as `ennallaan`.

## Output

One line per setting, in Finnish:

```
ennallaan  NSAutomaticCapitalizationEnabled = epätosi  - Automaattiset isot alkukirjaimet pois
muutettu   KeyRepeat: 5.0 (liukuluku) -> 2 (kokonaisluku)  - Näppäimen toistonopeus: ...
muuttuisi  ApplePressAndHoldEnabled: ei asetettu -> epätosi  - ...   (dry-run)
VIRHE      <key>: <reason>
```

Types are shown only when a change also changes the stored type (for example a float written by
System Settings replaced by the integer macmaster wants). After the lines comes a summary and the
notes that apply:

- Key repeat changed: log out and back in (the value is read at login).
- Text input or press-and-hold changed: already running apps see it after they are relaunched.
- A setting with a process to restart (for example Dock) changed: macmaster runs `killall` on it.

## Exit status

- `0`: everything applied (or dry-run finished).
- `1`: at least one read, write or sync failed, or a process restart could not be started.

## How it works

- Settings are a static table in `src/settings.rs` (see `adding-a-setting.md`).
- Each value is read with `CFPreferencesCopyAppValue` (current user, any host), compared with `Value::satisfies` (numbers by value, other types strict),
  and written with `CFPreferencesSetAppValue` only if it differs. Each write is read back to
  catch silently rejected writes, then `CFPreferencesAppSynchronize` flushes each touched domain once.
- No `defaults` subprocess, no `killall cfprefsd`. Writing through CFPreferences goes through
  cfprefsd, so its cache is already correct. Killing cfprefsd is only needed after editing plist
  files directly; here it would just throw away its in-memory state.

## Development

```sh
cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test
cargo install --path .
```

Unit tests run against an in-memory fake (`src/prefs/memory.rs`). The single integration test
(`tests/cf_backend.rs`) round-trips through real CFPreferences in the throwaway domain
`dev.triac.macmaster.test` and deletes it afterwards. No test touches NSGlobalDomain.
