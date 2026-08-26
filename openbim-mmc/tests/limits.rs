mod common;

use openbim_mmc::{Limits, MmcArchive, MmcError};

fn is_limit(result: Result<MmcArchive, MmcError>) {
    assert!(matches!(result, Err(MmcError::LimitExceeded { .. })));
}

#[test]
fn enforces_model_link_model_link_and_relatum_budgets() {
    let bytes = common::valid_archive();
    is_limit(MmcArchive::parse_with_limits(
        &bytes,
        Limits {
            max_models: 1,
            ..Limits::default()
        },
    ));
    is_limit(MmcArchive::parse_with_limits(
        &bytes,
        Limits {
            max_link_models: 0,
            ..Limits::default()
        },
    ));
    is_limit(MmcArchive::parse_with_limits(
        &bytes,
        Limits {
            max_links: 0,
            ..Limits::default()
        },
    ));
    is_limit(MmcArchive::parse_with_limits(
        &bytes,
        Limits {
            max_linked_elements: 1,
            ..Limits::default()
        },
    ));
}

#[test]
fn enforces_total_size_and_compression_ratio_budgets() {
    let bytes = common::valid_archive();
    is_limit(MmcArchive::parse_with_limits(
        &bytes,
        Limits {
            max_total_uncompressed_bytes: 32,
            ..Limits::default()
        },
    ));

    let index = common::valid_multimodel("links/elements.xml", "models/model.ifc");
    let repeated = vec![b'x'; 100_000];
    let compressed = common::zip(&[
        ("MultiModel.xml", index.as_slice()),
        ("models/model.ifc", repeated.as_slice()),
    ]);
    is_limit(MmcArchive::parse_with_limits(
        &compressed,
        Limits {
            max_compression_ratio: 10,
            ..Limits::default()
        },
    ));
}
