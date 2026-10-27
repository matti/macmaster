# Settings

All settings are in NSGlobalDomain (kCFPreferencesAnyApplication), current user, any host.

## Keyboard

| Key | Value | What and why |
|---|---|---|
| `InitialKeyRepeat` | int 15 | Delay before a held key starts repeating, in 15 ms ticks: 225 ms. The shortest the System Settings slider allows; we deliberately do not go below the GUI minimum. |
| `KeyRepeat` | int 2 | Repeat interval, 30 ms. Fastest GUI setting. |
| `ApplePressAndHoldEnabled` | bool false | A held key repeats instead of opening the accent popup. Accents remain available through dead keys / the Finnish layout. |

Key repeat is read at login, so a change needs logout and login. Press-and-hold is read by each app
at launch.

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
- Type finding: on the build machine `InitialKeyRepeat` and `KeyRepeat` are stored as float
  (`defaults read-type -g KeyRepeat` -> float), most likely written by the Tahoe System Settings.
  macmaster writes int; the reader (HIToolbox) accepts any CFNumber. Numbers compare by value across
  int and float (`Value::satisfies`), so a stored 15.0 counts as unchanged.
- Not included, but toggles that exist on Tahoe: "Show typing suggestions"
  (`NSAutomaticTextCompletionEnabled`) and "Show suggested replies". Add them with one line if wanted.
