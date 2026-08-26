mod common;

use openbim_mmc::{MmcArchive, ValidationCode};

fn report(index: String, links: String) -> openbim_mmc::ValidationReport {
    let bytes = common::zip(&[
        ("MultiModel.xml", index.as_bytes()),
        ("models/model.ifc", b"ifc"),
        ("links/elements.xml", links.as_bytes()),
    ]);
    MmcArchive::parse(&bytes).unwrap().validate()
}

#[test]
fn requires_format_id_when_a_model_has_multiple_representations() {
    let index = String::from_utf8(common::valid_multimodel(
        "links/elements.xml",
        "models/model.ifc",
    ))
    .unwrap()
    .replacen(
        "</mmc:ApplicationModel>",
        "<mmc:ModelData id=\"ifc-alt\" formatType=\"IFCXML\"><mmc:DataRessource id=\"ifc-alt-file\" location=\"https://example.test/alt.xml\"/></mmc:ModelData></mmc:ApplicationModel>",
        1,
    );
    let links = String::from_utf8(common::valid_link_model())
        .unwrap()
        .replacen(" f=\"ifc-spf\" r=\"ifc-file\"", "", 1);
    assert!(report(index, links).contains(ValidationCode::MissingRepresentationDisambiguator));
}

#[test]
fn requires_resource_id_when_a_representation_has_multiple_resources() {
    let index = String::from_utf8(common::valid_multimodel(
        "links/elements.xml",
        "models/model.ifc",
    ))
    .unwrap()
    .replacen(
        "</mmc:ModelData>",
        "<mmc:DataRessource id=\"ifc-alt-file\" location=\"https://example.test/alt.ifc\"/></mmc:ModelData>",
        1,
    );
    let links = String::from_utf8(common::valid_link_model())
        .unwrap()
        .replacen(" r=\"ifc-file\"", "", 1);
    assert!(report(index, links).contains(ValidationCode::MissingResourceDisambiguator));
}
