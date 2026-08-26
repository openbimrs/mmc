# openbim-mmc

Pure-Rust DIN 18290-1 / Multi-Model Container 2.0 mechanics.

## Implemented

- Bounded `.mmc` ZIP parsing.
- Namespace-aware `MultiModel.xml` and LinkModel XML projections.
- Original XML, payload, and archive byte preservation.
- Stable structural and referential-integrity reports.
- Safe extraction with path, collision, symlink, and overwrite protection.
- Deterministic archive construction and writing.

## Boundary

This repository owns only MMC 2.0 container mechanics. It does not implement ISO 21597 ICDD or parse IFC and GAEB payloads. DIN 18290 Parts 2–4 for LV, cost, and invoicing belong in optional domain crates built on `openbim-mmc`, `openbim-gaeb`, and canonical IFC.

The crates.io name `mmc` is owned by an unrelated Minimal Model Collection crate, so this repository publishes only `openbim-mmc`. It does not ship a misleading alias.

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

Run `scripts/gate.sh`. The gate pins Rust `1.88.0`, tests all targets, denies Clippy and rustdoc warnings, verifies the minimal feature build, and audits package contents. `scripts/mutation-probes.sh` proves critical tests detect weakened guards.

See [ADR 0001](docs/adr/0001-format-boundary.md), the [roadmap](docs/ROADMAP.md), and the [changelog](CHANGELOG.md).

## License

MIT. Normative DIN documents and schemas are not redistributed.
