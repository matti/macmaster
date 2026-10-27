# macmaster

- Rust CLI, edition 2024. `macmaster` applies built-in macOS user defaults; `-n/--dry-run` writes nothing.
- NEVER run `macmaster` without `--dry-run` on a dev machine: it changes live settings.
- Tests must never touch NSGlobalDomain or real settings; real backend test uses `dev.triac.macmaster.test` only.
- Zero config: no config files, no prompts, settings live in `src/settings.rs` (`SETTINGS`, one line each).
- Prefs via CFPreferences (`objc2-core-foundation`), never shell out to `defaults`.
- Comparison via `Value::satisfies`: numbers by value across int/float, other types strict (`Int(0)` does not satisfy `Bool(false)`).
- User-facing output in Finnish (correct ä/ö). No em/en dashes anywhere.
- Gate: `cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test`.
- Layout: `src/settings.rs` table, `src/apply.rs` diff/apply, `src/prefs/` backend trait + CF + fake, `src/report.rs` output.
- Docs: `docs/README.md` (usage), `docs/settings.md` (what and why, Tahoe findings), `docs/adding-a-setting.md`, `docs/distribution.md`, `docs/alternatives.md`.
