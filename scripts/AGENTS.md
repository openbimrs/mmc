# Verification scripts

- `gate.sh` is the required Rust 1.88.0 format/build/test/Clippy/docs/package gate.
- `verify-package.py` enforces the crate allowlist and excludes standards/reference artifacts.
- `mutation-probes.py` weakens critical guards in disposable copies and requires the relevant tests to fail.
