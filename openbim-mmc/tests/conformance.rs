mod common;

use openbim_mmc::MmcArchive;

#[test]
fn required_root_attributes_are_strict() {
    let raw = common::valid_multimodel("links/elements.xml", "models/model.ifc");
    let valid = String::from_utf8(raw).unwrap();
    for malformed in [
        valid.replace(&format!(" uuid=\"{}\"", common::UUID), ""),
        valid.replace(" formatVersion=\"2.0.0\"", ""),
        valid.replace(" mmDomain=\"urn:din:18290:test\"", ""),
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
    let archive = MmcArchive::parse(common::zip(&[("MultiModel.xml", spoof.as_bytes())])).unwrap();
    assert_eq!(archive.container().models.len(), 2);
    assert!(archive
        .container()
        .models
        .iter()
        .all(|model| model.id != "evil"));
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
