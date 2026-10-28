# Getting macmaster onto a fresh Mac

A fresh Mac has no Rust toolchain, so users install a prebuilt binary:

```sh
curl -fsSL https://raw.githubusercontent.com/matti/macmaster/main/install.sh | sh
```

`install.sh` downloads the latest release binary and its SHA-256 file, verifies the checksum,
installs to `~/.local/bin/macmaster` (no sudo), adds that directory to `PATH` in `~/.zprofile` if
missing, and runs `macmaster` with any arguments passed (`| sh -s -- --dry-run`).

## Releasing

`.github/workflows/ci.yml` runs fmt, clippy and tests on every push. On a `v*` tag it also builds
`aarch64-apple-darwin` and `x86_64-apple-darwin`, merges them with `lipo` into one universal
binary, and attaches `macmaster` and `macmaster.sha256` to a GitHub release.

```sh
# bump version in Cargo.toml, commit, then:
git tag v0.x.y && git push origin main v0.x.y
```

## Signing

Release binaries carry only the ad-hoc signature the linker adds (required on Apple Silicon). This is
enough because `curl` does not set the quarantine attribute, so Gatekeeper does not check it. A
binary downloaded with a browser would be blocked; Developer ID signing and notarization would fix
that if it ever matters.

## Developer machine

```sh
cargo install --path .
```
