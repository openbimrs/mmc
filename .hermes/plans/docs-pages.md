# Documentation and capability-boundary plan

## Goal

Publish a maintained VitePress site for `openbim-mmc`, and make the ICDD README describe only enduring library capabilities.

## Decisions

- MMC Pages uses the OpenBIM.rs VitePress pattern at `https://openbimrs.github.io/mmc/`.
- The site documents implemented mechanics, safety limits, API entry points, and standards boundaries without redistributing schemas or normative text.
- ICDD will not carry an open-ended “complete normative validation” roadmap row. The README will state the implemented structural checks and explicitly disclaim ISO certification.
- Neither core promises byte-identical rewriting of an arbitrary reconstructed ZIP/XML archive. MMC exposes exact original archive/XML/payload bytes and deterministic entry-preserving re-emission; ICDD preserves payload bytes and unknown RDF triples semantically while its serializer intentionally canonicalizes lexical XML.
- Application-specific Poing state does not belong in the root ICDD README.

## Validation

1. Build VitePress from a clean dependency install.
2. Scan the generated site for restricted artifact types and machine-local paths.
3. Run MMC and ICDD repository gates.
4. Review exact committed trees, push, verify hosted CI and the public Pages URL.
