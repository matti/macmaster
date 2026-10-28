# macmaster

One command that configures a fresh Mac: fastest key repeat allowed by System Settings, and
autocorrect, auto-capitalization and other text substitutions off. No config file, no questions.

```sh
curl -fsSL https://raw.githubusercontent.com/matti/macmaster/main/install.sh | sh
```

This installs `macmaster` to `~/.local/bin` and runs it. Afterwards, run `macmaster` again any time
(it is idempotent) or `macmaster --dry-run` to see what would change. Log out and back in for key
repeat to take effect.

Details: [docs/README.md](docs/README.md), the setting list: [docs/settings.md](docs/settings.md).
