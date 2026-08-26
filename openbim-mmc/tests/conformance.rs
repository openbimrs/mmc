mod common;

use openbim_mmc::{MmcArchive, MmcError};

#[test]
fn required_root_attributes_are_strict() {
    let raw = common::valid_multimodel("links/elements.xml", "models/model.ifc");
    let valid = String::from_utf8(raw).unwrap();
    for malformed in [
        valid.replace(&format!(" uuid=\"{}\"", common::UUID), ""),
        valid.replace(" formatVersion=\"2.0.0\"", ""),
        valid.replace(common::UUID, "not-a-uuid"),
    ] {
        let bytes = common::zip(&[("MultiModel.xml", malformed.as_bytes())]);
        assert!(MmcArchive::parse(&bytes).is_err());
    }
}

#[test]
fn extension_elements_cannot_impersonate_standard_ancestors() {
    let raw = String::from_utf8(common::valid_multimodel(
        "links/elements.xml",
        "models/model.ifc",
    ))
    .unwrap();
    let spoof = raw
        .replace("xmlns:mmc=", "xmlns:x=\"urn:extension\" xmlns:mmc=")
        .replace(
            "<mmc:ApplicationModels>",
            "<x:ApplicationModels><mmc:ApplicationModel id=\"evil\" modelType=\"spoof\"/></x:ApplicationModels><mmc:ApplicationModels>",
        );
    assert!(MmcArchive::parse(common::zip(&[("MultiModel.xml", spoof.as_bytes(),)])).is_err());
}

#[test]
fn required_nested_attributes_are_strict() {
    let raw = common::valid_multimodel("links/elements.xml", "models/model.ifc");
    let valid = String::from_utf8(raw).unwrap();
    for malformed in [
        valid.replace(" id=\"model-ifc\"", ""),
        valid.replace(" modelType=\"IFC\"", ""),
        valid.replace(" id=\"ifc-spf\"", ""),
        valid.replace(" formatType=\"IFC\"", ""),
        valid.replace(" id=\"ifc-file\"", ""),
        valid.replace(" location=\"models/model.ifc\"", ""),
    ] {
        let bytes = common::zip(&[("MultiModel.xml", malformed.as_bytes())]);
        assert!(MmcArchive::parse(&bytes).is_err());
    }
}

#[test]
fn malformed_standard_nesting_returns_an_error_instead_of_panicking() {
    let index = format!(
        r#"<m:MultiModel xmlns:m="{}" uuid="11111111-1111-4111-8111-111111111111" formatVersion="2.0.0" mmDomain="urn:test"><m:ModelData><m:MetaData><m:Meta key="k" value="v"/></m:MetaData></m:ModelData></m:MultiModel>"#,
        common::MMC_NS,
    );
    let bytes = common::zip(&[("MultiModel.xml", index.as_bytes())]);
    assert!(MmcArchive::parse(&bytes).is_err());
}

#[test]
fn link_model_errors_report_the_embedded_document_path() {
    let index = common::valid_multimodel("links/broken.xml", "models/model.ifc");
    let broken = format!(
        r#"<l:LinkModel xmlns:l="{}"><l:Link><l:Relatum m="a" id="x"/><l:Relatum m="b" id="y"/></l:Link></l:LinkModel>"#,
        common::LINK_NS,
    );
    let bytes = common::zip(&[
        ("MultiModel.xml", index.as_slice()),
        ("links/broken.xml", broken.as_bytes()),
    ]);
    let error = MmcArchive::parse(&bytes).unwrap_err();
    assert!(matches!(error, MmcError::Xml { path, .. } if path == "links/broken.xml"));
}
