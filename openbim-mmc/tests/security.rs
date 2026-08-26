mod common;

use std::fs;

use openbim_mmc::{Limits, MmcArchive, MmcError};

#[test]
fn rejects_unsafe_archive_names_and_ambiguous_normalization() {
    for name in [
        "/absolute.ifc",
        "../escape.ifc",
        "models/../escape.ifc",
        "models\\escape.ifc",
        "a:b.ifc",
        "NUL",
        "dir/COM1.txt",
        "trail.",
        "trail ",
        "models//escape.ifc",
        "models/",
        "./models/a.ifc",
    ] {
        let index = common::valid_multimodel("links/elements.xml", "models/model.ifc");
        let bytes = common::zip(&[("MultiModel.xml", index.as_slice()), (name, b"x")]);
        let error = MmcArchive::parse(&bytes).unwrap_err();
        assert!(
            matches!(error, MmcError::UnsafeArchivePath { .. }),
            "{name}: {error}"
        );
    }
}

#[test]
fn rejects_duplicate_and_case_folded_archive_names() {
    let index = common::valid_multimodel("links/elements.xml", "models/model.ifc");
    let exact = common::zip_with_exact_duplicate_root(&index);
    assert!(MmcArchive::parse(&exact).is_err());

    for duplicate in ["multimodel.xml", "MULTIMODEL.XML"] {
        let bytes = common::zip(&[("MultiModel.xml", index.as_slice()), (duplicate, b"other")]);
        let error = MmcArchive::parse(&bytes).unwrap_err();
        assert!(matches!(error, MmcError::DuplicateArchivePath { .. }));
    }
}

#[test]
fn requires_one_exact_root_multimodel_xml() {
    for path in ["nested/MultiModel.xml", "MultiModel.XML", "Index.rdf"] {
        let bytes = common::zip(&[(path, b"<x/>")]);
        let error = MmcArchive::parse(&bytes).unwrap_err();
        assert!(matches!(error, MmcError::MissingRoot));
    }
}

#[test]
fn rejects_xml_1_0_forbidden_characters() {
    let valid = String::from_utf8(common::valid_multimodel(
        "links/elements.xml",
        "models/model.ifc",
    ))
    .unwrap();
    for value in ["IF\0C", "IF&#0;C"] {
        let xml = valid.replace("modelType=\"IFC\"", &format!("modelType=\"{value}\""));
        let bytes = common::zip(&[("MultiModel.xml", xml.as_bytes())]);
        assert!(matches!(
            MmcArchive::parse(&bytes),
            Err(MmcError::Xml { .. })
        ));
    }
}

#[test]
fn rejects_symbolic_link_zip_entries() {
    let bytes = common::zip_with_mode("link", b"MultiModel.xml", 0o120777);
    assert!(matches!(
        MmcArchive::parse(&bytes),
        Err(MmcError::UnsafeArchivePath { .. })
    ));
}

#[test]
fn rejects_wrong_xml_root_namespace_and_doctype() {
    for xml in [
        b"<MultiModel xmlns=\"urn:not:mmc\" uuid=\"11111111-1111-4111-8111-111111111111\" formatVersion=\"2.0.0\" mmDomain=\"urn:test\"><ApplicationModels/><LinkModels/></MultiModel>".as_slice(),
        b"<!DOCTYPE MultiModel [<!ENTITY x \"boom\">]><MultiModel xmlns=\"http://www.buildingsmart.org/multi-model/MMContainer/2.0.0\" uuid=\"11111111-1111-4111-8111-111111111111\" formatVersion=\"2.0.0\" mmDomain=\"urn:test\"><ApplicationModels/><LinkModels/></MultiModel>".as_slice(),
    ] {
        let bytes = common::zip(&[("MultiModel.xml", xml)]);
        let error = MmcArchive::parse(&bytes).unwrap_err();
        assert!(matches!(error, MmcError::Xml { .. }));
    }
}

#[test]
fn enforces_entry_archive_and_xml_budgets() {
    let valid = common::valid_archive();

    let limits = Limits {
        max_archive_bytes: valid.len() - 1,
        ..Limits::default()
    };
    assert!(matches!(
        MmcArchive::parse_with_limits(&valid, limits),
        Err(MmcError::LimitExceeded { .. })
    ));

    let limits = Limits {
        max_entries: 2,
        ..Limits::default()
    };
    assert!(matches!(
        MmcArchive::parse_with_limits(&valid, limits),
        Err(MmcError::LimitExceeded { .. })
    ));

    let limits = Limits {
        max_entry_bytes: 8,
        ..Limits::default()
    };
    assert!(matches!(
        MmcArchive::parse_with_limits(&valid, limits),
        Err(MmcError::LimitExceeded { .. })
    ));

    let limits = Limits {
        max_xml_events: 2,
        ..Limits::default()
    };
    assert!(matches!(
        MmcArchive::parse_with_limits(&valid, limits),
        Err(MmcError::LimitExceeded { .. })
    ));
}

#[cfg(unix)]
#[test]
fn extraction_refuses_symlink_roots_and_ancestors() {
    use std::os::unix::fs::symlink;

    let archive = MmcArchive::parse(common::valid_archive()).unwrap();
    let temp = tempfile::tempdir().unwrap();
    let real = temp.path().join("real");
    fs::create_dir(&real).unwrap();
    let root_link = temp.path().join("root-link");
    symlink(&real, &root_link).unwrap();
    assert!(matches!(
        archive.extract_to(&root_link),
        Err(MmcError::UnsafeExtractionPath { .. })
    ));

    let root = temp.path().join("root");
    fs::create_dir(&root).unwrap();
    symlink(&real, root.join("models")).unwrap();
    assert!(matches!(
        archive.extract_to(&root),
        Err(MmcError::UnsafeExtractionPath { .. })
    ));
    assert!(!real.join("model.ifc").exists());
}

#[test]
fn extraction_uses_new_file_semantics() {
    let archive = MmcArchive::parse(common::valid_archive()).unwrap();
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("out");
    fs::create_dir_all(root.join("models")).unwrap();
    fs::write(root.join("models/model.ifc"), b"keep").unwrap();

    let error = archive.extract_to(&root).unwrap_err();
    assert!(matches!(error, MmcError::UnsafeExtractionPath { .. }));
    assert_eq!(fs::read(root.join("models/model.ifc")).unwrap(), b"keep");
}

#[test]
fn successful_extraction_writes_exact_payloads() {
    let archive = MmcArchive::parse(common::valid_archive()).unwrap();
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("out");
    archive.extract_to(&root).unwrap();

    assert_eq!(
        fs::read(root.join("MultiModel.xml")).unwrap(),
        archive.entry("MultiModel.xml").unwrap().bytes()
    );
    assert_eq!(
        fs::read(root.join("models/model.ifc")).unwrap(),
        b"ISO-10303-21;ENDSEC;END-ISO-10303-21;"
    );
}
