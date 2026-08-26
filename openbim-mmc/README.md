# openbim-mmc

Safe, lossless Rust mechanics for Multi-Model Container 2.0 archives used in DIN 18290 workflows.

The typed projection and writer target the public buildingSMART MMC `2.0.0`
schema shape. This crate does not claim complete conformance with every DIN
18290-1 edition, and `validate()` is a structural/referential check rather than
a complete XSD validator.

```rust
use openbim_mmc::MmcArchive;

let bytes = std::fs::read("exchange.mmc")?;
let archive = MmcArchive::parse(&bytes)?;
assert_eq!(archive.container().metadata.format_version, "2.0.0");
assert!(archive.validate().is_valid());
# Ok::<(), Box<dyn std::error::Error>>(())
```

The crate provides:

- bounded ZIP ingestion and decompression-ratio limits;
- exact root `MultiModel.xml` handling;
- namespace-aware MMC and LinkModel typed projections;
- opaque application resources with original-byte access;
- deterministic writing and a typed archive builder;
- extraction that rejects unsafe names, symlinks, collisions, and overwrites;
- stable validation codes for structural and referential-integrity findings.

`openbim-mmc` intentionally does not parse IFC or GAEB payloads and does not depend on ISO 21597 ICDD. DIN 18290 Parts 2–4 are domain profiles above this crate.

The unprefixed crates.io name `mmc` belongs to an unrelated project. No alias package is provided.

Rust `1.88.0` is the supported baseline. License: MIT. Normative standards and schemas are not included in the package.
