---
layout: home

hero:
  name: openbim-mmc
  text: Safe MMC 2.0 mechanics for Rust
  tagline: Bounded archives, namespace-aware XML, deterministic construction, and an explicit DIN 18290 scope boundary.
  image:
    src: /logo.svg
    alt: openbim-mmc logo
  actions:
    - theme: brand
      text: Get started
      link: /guide/getting-started
    - theme: alt
      text: Capability contract
      link: /capabilities
    - theme: alt
      text: View on GitHub
      link: https://github.com/openbimrs/mmc

features:
  - title: Defensive archive handling
    details: Bound actual decompressed output, reject ambiguous or hostile paths, and extract without following symlinks or overwriting files.
  - title: Typed without discarding source
    details: Inspect MultiModel and LinkModel structures while retaining exact source XML, opaque payload entries, and original archive bytes.
  - title: Deterministic construction
    details: Build public MMC 2.0 schema-shaped documents and reproducible ZIP archives through a typed Rust API.
  - title: Honest standards boundary
    details: Public MMC 2.0 mechanics are implemented; complete conformance with every current licensed DIN edition is not claimed.
---

## Container mechanics, not payload semantics

`openbim-mmc` reads, validates, extracts, and constructs Multi-Model Container archives used in DIN 18290 workflows. IFC, GAEB, PDF, and other application resources remain opaque. Domain profiles combine this crate with their own maintained format libraries rather than coupling every payload parser into the container core.

Start with the [getting-started guide](/guide/getting-started), then read the [capability and preservation contract](/capabilities).
