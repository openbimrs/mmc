# openbim-mmc crate

Entry point: `src/lib.rs`.

Modules:

- `archive.rs` — bounded ZIP ingestion, original/deterministic writing, extraction;
- `xml.rs` — namespace-aware typed projections while retaining source bytes;
- `model.rs` — public MMC and LinkModel data types;
- `validation.rs` — stable conformance findings;
- `builder.rs` — construction of new deterministic archives;
- `path.rs` — cross-platform path ambiguity rejection.

Do not add GAEB, IFC, ICDD, or DIN 18290 Parts 2–4 behavior here.
