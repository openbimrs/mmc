# Roadmap

## Implemented: MMC 2.0 core

- MMC 2.0 archive envelope used by DIN 18290 workflows
- `MultiModel.xml`
- LinkModel XML
- opaque embedded and external application resources
- structural/referential validation
- deterministic writing and safe extraction

## Next: domain layers

DIN 18290 Parts 2–4 belong above this crate:

- BIM-LV profile using `openbim-gaeb`;
- BIM cost profile using `openbim-gaeb`;
- BIM invoicing profile using `openbim-gaeb`;
- optional semantic IFC validation using the canonical IFC repository.

A future MMC↔ICDD converter must depend on both cores. Neither format core will depend on the other.

Supporting a newer MMC/DIN schema revision requires a separately reviewed,
versioned compatibility layer. Public MMC 2.0 schemas alone are not evidence of
complete conformance with current normative DIN editions.
