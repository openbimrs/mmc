mod common;

use openbim_mmc::{EntryKind, MmcArchive, ValidationCode, LINK_MODEL_NAMESPACE, MMC_NAMESPACE};

#[test]
fn opens_and_inspects_a_conformant_mmc_2_archive() {
    let bytes = common::valid_archive();
    let archive = MmcArchive::parse(&bytes).unwrap();

    assert_eq!(MMC_NAMESPACE, common::MMC_NS);
    assert_eq!(LINK_MODEL_NAMESPACE, common::LINK_NS);
    assert_eq!(archive.container().metadata.uuid.to_string(), common::UUID);
    assert_eq!(archive.container().metadata.format_version, "2.0.0");
    assert_eq!(archive.container().models.len(), 2);
    assert_eq!(archive.container().link_models.len(), 1);

    let model = &archive.container().models[0];
    assert_eq!(model.id, "model-ifc");
    assert_eq!(
        model.representations[0].resources[0]
            .location
            .embedded_path(),
        Some("models/model.ifc")
    );
    assert_eq!(
        archive.entry("models/model.ifc").unwrap().kind(),
        EntryKind::Payload
    );

    let links = archive.parsed_link_models();
    assert_eq!(links.len(), 1);
    assert_eq!(links[0].path(), "links/elements.xml");
    assert_eq!(links[0].model().format_version, "2.0.0");
    assert_eq!(links[0].model().links[0].relata.len(), 2);
    assert_eq!(links[0].model().links[0].relata[0].model_id, "model-ifc");
    assert_eq!(
        archive.entry("links/elements.xml").unwrap().kind(),
        EntryKind::LinkModel
    );
    assert!(
        archive.validate().is_valid(),
        "{:?}",
        archive.validate().issues()
    );
}

#[test]
fn namespace_resolution_accepts_arbitrary_prefixes() {
    let index = common::valid_multimodel("links/elements.xml", "models/model.ifc");
    let index = String::from_utf8(index)
        .unwrap()
        .replace("mmc:", "x:")
        .replace("xmlns:mmc=", "xmlns:x=");
    let links = String::from_utf8(common::valid_link_model())
        .unwrap()
        .replace("l:", "different:")
        .replace("xmlns:l=", "xmlns:different=");
    let bytes = common::zip(&[
        ("MultiModel.xml", index.as_bytes()),
        ("models/model.ifc", b"ifc"),
        ("links/elements.xml", links.as_bytes()),
    ]);

    let archive = MmcArchive::parse(&bytes).unwrap();
    assert!(
        archive.validate().is_valid(),
        "{:?}",
        archive.validate().issues()
    );
}

#[test]
fn preserves_unknown_xml_and_payload_bytes() {
    let index = format!(
        r#"<m:MultiModel xmlns:m="{}" xmlns:v="urn:vendor" uuid="{}" formatVersion="2.0.0" mmDomain="urn:vendor:test" v:flag="keep"><m:ApplicationModels/><m:LinkModels/><v:Private answer="42"/></m:MultiModel>"#,
        common::MMC_NS,
        common::UUID,
    );
    let original = common::zip(&[
        ("MultiModel.xml", index.as_bytes()),
        ("vendor/opaque.bin", b"\x00\xffprivate"),
    ]);

    let archive = MmcArchive::parse(&original).unwrap();
    assert_eq!(archive.original_bytes(), original);
    assert_eq!(
        archive.entry("MultiModel.xml").unwrap().bytes(),
        index.as_bytes()
    );
    assert_eq!(
        archive.entry("vendor/opaque.bin").unwrap().bytes(),
        b"\x00\xffprivate"
    );
}

#[test]
fn deterministic_writer_reopens_to_the_same_semantics() {
    let archive = MmcArchive::parse(common::valid_archive()).unwrap();
    let first = archive.to_deterministic_bytes().unwrap();
    let second = archive.to_deterministic_bytes().unwrap();
    assert_eq!(first, second);

    let reopened = MmcArchive::parse(&first).unwrap();
    assert!(reopened.validate().is_valid());
    assert_eq!(
        reopened.entry("models/model.ifc").unwrap().bytes(),
        b"ISO-10303-21;ENDSEC;END-ISO-10303-21;"
    );
}

#[test]
fn semantic_validation_reports_missing_payloads_unknown_models_and_short_links() {
    let index = common::valid_multimodel("links/elements.xml", "models/missing.ifc");
    let links = format!(
        r#"<LinkModel xmlns="{}" formatVersion="2.0.0"><Link><Relatum m="not-declared" id="x"/></Link></LinkModel>"#,
        common::LINK_NS,
    );
    let bytes = common::zip(&[
        ("MultiModel.xml", index.as_slice()),
        ("links/elements.xml", links.as_bytes()),
    ]);
    let archive = MmcArchive::parse(&bytes).unwrap();
    let report = archive.validate();

    assert!(!report.is_valid());
    assert!(report.contains(ValidationCode::MissingEmbeddedResource));
    assert!(report.contains(ValidationCode::UnknownModelReference));
    assert!(report.contains(ValidationCode::LinkHasTooFewRelata));
}

#[test]
fn rejects_wrong_root_namespace_and_doctype() {
    let wrong = common::zip(&[("MultiModel.xml", br#"<MultiModel xmlns="urn:not-mmc"/>"#)]);
    assert!(MmcArchive::parse(&wrong).is_err());

    let doctype = common::zip(&[(
        "MultiModel.xml",
        br#"<!DOCTYPE x [<!ENTITY boom "x">]><x:MultiModel xmlns:x="http://www.buildingsmart.org/multi-model/MMContainer/2.0.0" uuid="4d69a342-31b6-4e80-9d05-83a28754c84d" formatVersion="2.0.0"><x:ApplicationModels/><x:LinkModels/></x:MultiModel>"#,
    )]);
    assert!(MmcArchive::parse(&doctype).is_err());
}
