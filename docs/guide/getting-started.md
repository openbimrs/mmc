# Getting started

## Add the crate

```bash
cargo add openbim-mmc
```

The unprefixed package name `mmc` belongs to an unrelated project. Use `openbim-mmc` explicitly.

## Inspect an archive

```rust
use openbim_mmc::MmcArchive;

let bytes = std::fs::read("exchange.mmc")?;
let archive = MmcArchive::parse(&bytes)?;

println!("version: {}", archive.container().metadata.format_version);
for model in &archive.container().models {
    println!("model {} ({})", model.id, model.model_type);
}

let report = archive.validate();
for issue in report.issues() {
    eprintln!("{:?}: {}", issue.code, issue.message);
}
# Ok::<(), Box<dyn std::error::Error>>(())
```

Parsing and validation are separate by design. Parsing establishes a safe, well-formed typed projection; `validate()` reports structural and referential problems without claiming complete normative DIN certification.

## Extract safely

```rust
# use openbim_mmc::MmcArchive;
# let bytes = std::fs::read("exchange.mmc")?;
# let archive = MmcArchive::parse(&bytes)?;
archive.extract_to("out")?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

Extraction rejects absolute paths, traversal, platform-ambiguous names, symlink ancestors, collisions, and overwrites. The caller must prevent concurrent hostile mutation of the destination tree while extraction runs.

## Work with IFC or GAEB

The MMC core identifies models, representations, resources, and links but does not parse payload formats. Add the canonical IFC or GAEB crate in a profile or application layer when you need semantic payload validation.
