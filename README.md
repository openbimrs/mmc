# openbim-mmc

[![CI](https://github.com/openbimrs/mmc/actions/workflows/ci.yml/badge.svg)](https://github.com/openbimrs/mmc/actions/workflows/ci.yml)
[![Documentation](https://github.com/openbimrs/mmc/actions/workflows/pages.yml/badge.svg)](https://openbimrs.github.io/mmc/)
[![docs.rs](https://docs.rs/openbim-mmc/badge.svg)](https://docs.rs/openbim-mmc)
[![MSRV](https://img.shields.io/badge/MSRV-1.88-blue)](https://www.rust-lang.org)

Pure-Rust Multi-Model Container (MMC) 2.0 mechanics for DIN 18290 workflows.

**[Read the documentation →](https://openbimrs.github.io/mmc/)**

## Compatibility status

The typed XML projection and writer target the public buildingSMART MMC `2.0.0`
schemas. The crate does **not** claim complete conformance with every edition of
DIN 18290-1; normative DIN documents and current digital annexes are not
redistributed. `validate()` checks the implemented structural and referential
contract, not full XSD conformance.

## Implemented

- Bounded `.mmc` ZIP parsing.
- Namespace-aware `MultiModel.xml` and LinkModel XML projections.
- Exact access to original archive, XML, and opaque entry bytes.
- Deterministic re-emission that preserves each entry's uncompressed bytes.
- Stable structural and referential-integrity reports.
- Safe extraction with path, collision, symlink, and overwrite protection.
- Deterministic archive construction and writing.

`original_bytes()` provides an exact no-op copy of the input archive. Rebuilding an
archive preserves every uncompressed entry byte, including unknown entries and
source XML, but intentionally does not reproduce ZIP compression streams,
timestamps, or central-directory layout byte-for-byte.

## Boundary

This repository owns only MMC 2.0 container mechanics. It does not implement ISO 21597 ICDD or parse IFC and GAEB payloads. DIN 18290 Parts 2–4 for LV, cost, and invoicing belong in optional domain crates built on `openbim-mmc`, `openbim-gaeb`, and canonical IFC.

The crates.io name `mmc` is owned by an unrelated Minimal Model Collection crate. The canonical package is therefore `openbim-mmc`; this repository does not include a misleading alias crate.

## Use

```rust
use openbim_mmc::MmcArchive;

let bytes = std::fs::read("exchange.mmc")?;
let archive = MmcArchive::parse(&bytes)?;
let report = archive.validate();
if !report.is_valid() {
    for issue in report.issues() {
        eprintln!("{:?}: {}", issue.code, issue.message);
    }
}
archive.extract_to("out")?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

## Verification

Run `scripts/gate.sh` from a clean Git tree. The gate pins Rust `1.88.0`, tests all targets, denies Clippy and rustdoc warnings, verifies the minimal feature build, and audits package contents. `scripts/mutation-probes.py` proves critical tests detect weakened guards.

See [ADR 0001](docs/adr/0001-format-boundary.md), the [roadmap](docs/ROADMAP.md), and the [changelog](CHANGELOG.md).

## License

AGPL-3.0-or-later. Normative DIN documents and schemas are not redistributed.
