# Adding a setting

Add one line to `SETTINGS` in `src/settings.rs`:

```rust
global("AppleShowAllExtensions", Bool(true), RelaunchApps, "Näytä kaikki tiedostopäätteet"),
```

For an app domain, construct `Setting` directly (or add a small `const fn` helper next to `global`
if several settings share a domain):

```rust
Setting { domain: Domain::App("com.apple.dock"), key: "autohide", value: Bool(true),
          effect: Restart("Dock"), description_fi: "Dock piiloutuu automaattisesti" },
```

Fields:

- `domain`: `Domain::Global` (NSGlobalDomain) or `Domain::App("bundle.id")`.
- `value`: `Bool`, `Int`, `Float` or `Str(Cow::Borrowed("..."))`. Use the type macOS itself reads;
  check with `defaults read-type <domain> <key>` on a Mac where the setting was changed in System Settings.
- `effect`: what makes it take effect.
  - `LiveHid("HIDKeyRepeat")`: key repeat style value in 1/60 s ticks that the running session
    reads from the HID event system. macmaster also diffs and sets that live property (in ns), so
    no logout is needed; the summary asks for a logout only if the live set fails.
  - `RelaunchApps`: read by each app at launch. Summary says running apps need a relaunch.
  - `Restart("Dock")`: macmaster runs `killall Dock` after a change (launchd restarts it).
- `description_fi`: short Finnish description shown on every output line.

Then document the setting in `settings.md` (what it does and why) and run the test gate.
