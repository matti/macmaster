# Settings

All settings are in NSGlobalDomain (kCFPreferencesAnyApplication), current user, any host. Key
repeat is additionally set live in the HID event system (see below).

## Keyboard

| Key | Value | What and why |
|---|---|---|
| `InitialKeyRepeat` | int 15 | Delay before a held key starts repeating, in 1/60 s ticks: 250 ms. The shortest the System Settings slider allows; we deliberately do not go below the GUI minimum. |
| `KeyRepeat` | int 2 | Repeat interval, 2/60 s = 33 ms. Fastest GUI setting. |
| `ApplePressAndHoldEnabled` | bool false | A held key repeats instead of opening the accent popup. Accents remain available through dead keys / the Finnish layout. |

Press-and-hold is read by each app at launch.

### Key repeat applies live, no logout

The plist values are only read at login (that is what persists). The running session uses the
values held by the HID event system, in nanoseconds. macmaster treats those as their own diff
targets, `HIDInitialKeyRepeat` and `HIDKeyRepeat`, shown as separate output lines:

- Read and set through `IOHIDEventSystemClientCreateSimpleClient` +
  `IOHIDEventSystemClientCopyProperty` / `IOHIDEventSystemClientSetProperty` (crate
  `objc2-io-kit`, feature `hidsystem`). This is what `hidutil property` and System Settings do;
  no sudo needed, effective immediately.
- Value: ticks * 1e9 / 60, truncated: 2 -> 33333333, 15 -> 250000000 (matches what System Settings
  sets). Each set is read back to catch silent rejection.
- The live value is compared even when the plist is already correct, so a session still running
  with old values is fixed without logout. Only if the live set fails does the summary say to log
  out and back in.

## Text input (Keyboard > Text Input > Edit)

| Key | Value | System Settings toggle |
|---|---|---|
| `NSAutomaticSpellingCorrectionEnabled` | false | Correct spelling automatically |
| `WebAutomaticSpellingCorrectionEnabled` | false | Same, for WebKit text views (Safari, Mail) |
| `NSAutomaticCapitalizationEnabled` | false | Capitalize words automatically |
| `NSAutomaticPeriodSubstitutionEnabled` | false | Add period with double-space |
| `NSAutomaticDashSubstitutionEnabled` | false | Use smart quotes and dashes (dashes) |
| `NSAutomaticQuoteSubstitutionEnabled` | false | Use smart quotes and dashes (quotes) |
| `NSAutomaticInlinePredictionEnabled` | false | Show inline predictive text |

Why: autocorrect and smart substitutions silently rewrite code, commands, Finnish words and
straight quotes. Spell checking (red underline) stays on; only automatic rewriting is turned off.
Running apps pick the change up after relaunch.

## Verification on macOS 26.4.1 (Tahoe), 2026-10

Checked read-only on the build machine, not from memory:

- All ten key names are present as strings in the Tahoe system frameworks (dyld shared cache:
  AppKit/WebKit), so none are obsolete.
- The Keyboard settings extension (`/System/Library/ExtensionKit/Extensions/KeyboardSettings.appex`)
  does not write these keys itself: it calls `NSSpellChecker setAutomatic...Enabled:` (spelling
  correction, capitalization, period, dash, quote, inline completion, text completion), and AppKit
  persists them under the `NSAutomatic...` keys above. No additional key for "Correct spelling
  automatically" was found. Whether AppKit also writes `WebAutomaticSpellingCorrectionEnabled` when
  the toggle is flipped could not be confirmed without changing live settings; we set it explicitly.
- Units: `hidutil property --get HIDKeyRepeat` gave 83333333 with `KeyRepeat` 5, and
  `HIDInitialKeyRepeat` 250000000 with `InitialKeyRepeat` 15, so one tick is 1/60 s (not 15 ms).
  `hidutil property --set` as a normal user changed the live repeat rate at once.
- Type finding: on the build machine `InitialKeyRepeat` and `KeyRepeat` are stored as float
  (`defaults read-type -g KeyRepeat` -> float), most likely written by the Tahoe System Settings.
  macmaster writes int; the reader (HIToolbox) accepts any CFNumber. Numbers compare by value across
  int and float (`Value::satisfies`), so a stored 15.0 counts as unchanged.
- Not included, but toggles that exist on Tahoe: "Show typing suggestions"
  (`NSAutomaticTextCompletionEnabled`) and "Show suggested replies". Add them with one line if wanted.
