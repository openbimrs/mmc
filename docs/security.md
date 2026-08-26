# Security model

MMC archives, entry names, XML, IRIs, declared locations, and payload bytes are untrusted input.

## Admission limits

Configurable limits bound:

- compressed archive bytes;
- entry count;
- actual decompressed bytes per entry and in total;
- actual compression ratio;
- XML bytes, models, resources, links, and relata.

ZIP metadata is not trusted as proof of decompressed size. Reads are budgeted before decompression can complete beyond the configured boundary, and declared/actual size inconsistencies are rejected.

## Path policy

Archive names and embedded resource locations reject:

- absolute paths and parent traversal;
- empty, dot, or backslash-separated components;
- Windows drive/device ambiguities;
- Unicode-normalization and case-fold collisions;
- duplicate names and symbolic-link entries.

Extraction creates new files only, refuses symlink roots or ancestors, and never overwrites existing files. Portable filesystem APIs cannot prevent another process from mutating the destination concurrently; callers must coordinate exclusive access.

## XML policy

The parsers require one complete root document, exact standard namespaces, balanced elements, and correctly placed declarations. DOCTYPE is prohibited. Unknown named entities and XML-invalid characters fail closed. Schema-significant `xs:string` whitespace is preserved.

## Reporting vulnerabilities

Do not include private archives, licensed standards material, or credentials in a public issue. Report a minimal synthetic reproducer through the repository’s [security advisory interface](https://github.com/openbimrs/mmc/security/advisories/new).
