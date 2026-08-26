# Roadmap

## Core status

The MMC 2.0 archive/XML core is implemented, tested, mutation-verified, and independently reviewed. Further core work should be driven by concrete interoperability cases rather than speculative format coupling.

## Domain profiles

DIN 18290 Parts 2–4 belong above the container:

- BIM-LV profile using `openbim-gaeb`;
- BIM cost profile using `openbim-gaeb`;
- BIM invoicing profile using `openbim-gaeb`;
- optional semantic IFC validation using the canonical IFC repository.

## Cross-format integration

A future MMC↔ICDD converter should depend on both format cores and expose conversion losses explicitly. `openbim-mmc` and `openbim-icdd` remain independent.

## New schema revisions

A newer MMC or DIN schema revision requires an explicit, versioned compatibility layer backed by redistributable test material or caller-supplied local schemas. Public MMC 2.0 prior art is not evidence for every current DIN edition.

The repository-maintained roadmap is also available as [`docs/ROADMAP.md`](https://github.com/openbimrs/mmc/blob/main/docs/ROADMAP.md).
