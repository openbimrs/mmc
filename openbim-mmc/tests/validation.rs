mod common;

use openbim_mmc::{MmcArchive, ValidationCode};

#[test]
fn reports_duplicate_ids_and_unsafe_or_invalid_locations() {
    let raw = common::valid_multimodel("links/elements.xml", "models/model.ifc");
    let index = String::from_utf8(raw)
        .unwrap()
        .replace("id=\"model-gaeb\"", "id=\"model-ifc\"")
        .replace(
            "https://example.test/bill.xml",
            "https://exa mple.test/bill.xml",
        )
        .replace("models/model.ifc", "../model.ifc");
    let links = common::valid_link_model();
    let bytes = common::zip(&[
        ("MultiModel.xml", index.as_bytes()),
        ("links/elements.xml", links.as_slice()),
    ]);
    let archive = MmcArchive::parse(&bytes).unwrap();
    let report = archive.validate();
    assert!(report.contains(ValidationCode::DuplicateModelId));
    assert!(
        report
            .issues()
            .iter()
            .filter(|issue| issue.code == ValidationCode::InvalidResourceLocation)
            .count()
            >= 2
    );
}

#[test]
fn reports_unknown_representation_resource_and_rate_targets() {
    let index = common::valid_multimodel("links/elements.xml", "models/model.ifc");
    let links = String::from_utf8(common::valid_link_model())
        .unwrap()
        .replace("f=\"ifc-spf\"", "f=\"missing-format\"")
        .replace("r=\"ifc-file\"", "r=\"missing-resource\"")
        .replace(
            "<l:Relatum m=\"model-gaeb\" id=\"item-1\"/>",
            "<l:Relatum m=\"model-gaeb\" id=\"item-1\"><l:Rate t=\"weight\" v=\"1\" m=\"missing-model\"/></l:Relatum>",
        );
    let bytes = common::zip(&[
        ("MultiModel.xml", index.as_slice()),
        ("models/model.ifc", b"ifc"),
        ("links/elements.xml", links.as_bytes()),
    ]);
    let archive = MmcArchive::parse(&bytes).unwrap();
    let report = archive.validate();
    assert!(report.contains(ValidationCode::UnknownRepresentationReference));
    assert!(report.contains(ValidationCode::RateTargetsUnknownModel));
}

#[test]
fn reports_duplicate_link_locations() {
    let raw = String::from_utf8(common::valid_multimodel(
        "links/elements.xml",
        "models/model.ifc",
    ))
    .unwrap();
    let invalid = raw
        .replace("id=\"model-ifc\"", "id=\"bad:id\"")
        .replace(
            "</mmc:LinkModels>",
            "<mmc:LinkModel location=\"links/elements.xml\"><mmc:LinkedModel>bad:id</mmc:LinkedModel><mmc:LinkedModel>model-gaeb</mmc:LinkedModel></mmc:LinkModel></mmc:LinkModels>",
        );
    let links = common::valid_link_model();
    let bytes = common::zip(&[
        ("MultiModel.xml", invalid.as_bytes()),
        ("links/elements.xml", links.as_slice()),
    ]);
    let report = MmcArchive::parse(&bytes).unwrap().validate();
    assert!(report.contains(ValidationCode::DuplicateLinkModelLocation));
}
