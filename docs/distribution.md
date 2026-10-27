# Getting macmaster onto a fresh Mac

A fresh Mac has no Rust toolchain, so `cargo install` is a developer path only.

## Today (developer machine)

```sh
cargo install --path .
```

## TODO: intended path for a fresh Mac (nothing published yet)

1. CI on a macOS runner builds `aarch64-apple-darwin` and `x86_64-apple-darwin` release binaries and
   merges them with `lipo -create` into one universal binary.
2. Sign with a Developer ID and notarize, so Gatekeeper does not block it (a `curl` download has no
   quarantine flag, but notarization keeps it future-proof and allows browser downloads).
3. Attach the binary plus a SHA-256 checksum to a GitHub release.
4. One-line installer in the repo (`install.sh`):

   ```sh
   curl -fsSL https://github.com/<owner>/macmaster/releases/latest/download/install.sh | sh
   ```

   The script downloads the universal binary, verifies the checksum, installs to
   `/usr/local/bin/macmaster` (or `~/.local/bin`), and runs `macmaster`.

Open decisions: repository owner/name, signing identity, whether the installer runs macmaster
automatically or only installs it.
