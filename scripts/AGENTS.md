# Verification scripts

- `gate.sh` is the required Rust 1.88.0 format/build/test/Clippy/docs/package gate.
- `verify-package.py` enforces the crate allowlist and excludes standards/reference artifacts.
- `mutation-probes.py` weakens critical guards in disposable copies and requires the relevant tests to fail.
- `build-docs.sh` installs locked Node dependencies, scans documentation source, builds VitePress, and scans the final Pages tree.
- `check-docs-leakage.py` rejects standards/reference artifacts and private filesystem paths in documentation artifacts.
