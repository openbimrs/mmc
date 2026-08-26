#!/usr/bin/env bash
set -euo pipefail

export CARGO_TERM_COLOR=always
TOOLCHAIN="${RUST_TOOLCHAIN:-1.88.0}"

cargo "+${TOOLCHAIN}" fmt --all -- --check
cargo "+${TOOLCHAIN}" check --workspace --all-targets --locked
cargo "+${TOOLCHAIN}" test --workspace --all-targets --locked
cargo "+${TOOLCHAIN}" clippy --workspace --all-targets --locked -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo "+${TOOLCHAIN}" doc --workspace --no-deps --locked
cargo "+${TOOLCHAIN}" check -p openbim-mmc --no-default-features --locked
python3 scripts/verify-package.py
cargo "+${TOOLCHAIN}" package -p openbim-mmc --allow-dirty --locked
