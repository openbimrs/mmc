mod common;

use openbim_mmc::{MmcArchive, ValidationCode};

#[test]
fn accepts_schema_minimum_without_domain_or_link_models() {
    let xml = format!(
        r#"<m:MultiModel xmlns:m="{}" uuid="11111111-1111-4111-8111-111111111111" formatVersion="2.0.0"><m:ApplicationModels><m:ApplicationModel id="one:model" modelType="custom"><m:ModelData id="format one" formatType="opaque"><m:DataRessource id="resource one" location="https://example.test/model.bin"/></m:ModelData></m:ApplicationModel></m:ApplicationModels></m:MultiModel>"#,
        common::MMC_NS,
    );
    let archive = MmcArchive::parse(common::zip(&[("MultiModel.xml", xml.as_bytes())])).unwrap();
    assert!(archive.validate().is_valid());
    assert!(archive.container().metadata.mm_domain.is_none());
}

#[test]
fn reports_schema_version_mismatches() {
    let xml = String::from_utf8(common::valid_multimodel(
        "links/elements.xml",
        "models/model.ifc",
    ))
    .unwrap()
    .replace("formatVersion=\"2.0.0\"", "formatVersion=\"9.0\"");
    let links = common::valid_link_model();
    let bytes = common::zip(&[
        ("MultiModel.xml", xml.as_bytes()),
        ("links/elements.xml", links.as_slice()),
    ]);
    let report = MmcArchive::parse(&bytes).unwrap().validate();
    assert!(report.contains(ValidationCode::UnsupportedFormatVersion));
}

#[test]
fn reports_duplicate_metadata_keys_within_a_category() {
    let xml = String::from_utf8(common::valid_multimodel(
        "links/elements.xml",
        "models/model.ifc",
    ))
    .unwrap()
    .replace(
        "<mmc:ApplicationModels>",
        "<mmc:MetaData><mmc:Meta key=\"k\" value=\"1\" category=\"c\"/><mmc:Meta key=\"k\" value=\"2\" category=\"c\"/></mmc:MetaData><mmc:ApplicationModels>",
    );
    let links = common::valid_link_model();
    let bytes = common::zip(&[
        ("MultiModel.xml", xml.as_bytes()),
        ("links/elements.xml", links.as_slice()),
    ]);
    let report = MmcArchive::parse(&bytes).unwrap().validate();
    assert!(report.contains(ValidationCode::DuplicateMetadataKey));
}
