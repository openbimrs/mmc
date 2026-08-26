# Roadmap

## Implemented: MMC 2.0 core

- DIN 18290-1 archive envelope
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
