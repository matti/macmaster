# Alternatives considered

| Tool | What it is | Why not |
|---|---|---|
| [cutler](https://github.com/machlit/cutler) | Rust, declarative TOML config for macOS defaults, GPL-3, v0.19.8 | Needs a config file to write and maintain; general-purpose rather than opinionated. |
| [dsully/macos-defaults](https://github.com/dsully/macos-defaults) | Rust, applies defaults from YAML files | Same: you bring the YAML. |
| [nix-darwin](https://github.com/nix-darwin/nix-darwin) | Full declarative system config via Nix (`system.defaults.*`) | Requires installing Nix and learning it; far more than needed. |
| Dotfiles `.macos` scripts (e.g. mathiasbynens), OmacOS | Shell scripts running `defaults write` | No dry-run or change report, untyped, one `defaults` process per key, drift between machines. |

Why macmaster exists: zero learning and zero config. Type one command on a fresh Mac, get the
settings. The opinions are compiled in, the output says exactly what changed, and `--dry-run`
shows it first.
