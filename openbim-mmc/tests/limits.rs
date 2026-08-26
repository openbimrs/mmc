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

#[test]
fn rejects_forged_uncompressed_sizes_using_actual_output_budgets() {
    let index = common::valid_multimodel("links/elements.xml", "models/model.ifc");
    let repeated = vec![b'x'; 100_000];
    let forged = common::zip_with_forged_uncompressed_size(
        &[
            ("MultiModel.xml", index.as_slice()),
            ("models/model.ifc", repeated.as_slice()),
        ],
        "models/model.ifc",
        1,
    );

    assert!(matches!(
        MmcArchive::parse(&forged),
        Err(MmcError::InvalidZipMetadata { .. })
    ));
    is_limit(MmcArchive::parse_with_limits(
        &forged,
        Limits {
            max_total_uncompressed_bytes: index.len() + 1_000,
            max_compression_ratio: usize::MAX,
            ..Limits::default()
        },
    ));
    is_limit(MmcArchive::parse_with_limits(
        &forged,
        Limits {
            max_total_uncompressed_bytes: usize::MAX,
            max_compression_ratio: 10,
            ..Limits::default()
        },
    ));
}

#[test]
fn reading_from_a_stream_never_overflows_at_the_max_archive_bytes_limit() {
    let bytes = common::valid_archive();
    let result = MmcArchive::read_from_with_limits(
        bytes.as_slice(),
        Limits {
            max_archive_bytes: usize::MAX,
            ..Limits::default()
        },
    );
    assert!(result.is_ok());
}
