mod common;

use openbim_mmc::MmcArchive;

#[test]
fn malformed_xml_bytes_never_panic() {
    let index = common::valid_multimodel("links/elements.xml", "models/model.ifc");
    for offset in (0..index.len()).step_by(7) {
        let mut mutated = index.clone();
        mutated[offset] ^= 0xff;
        let bytes = common::zip(&[("MultiModel.xml", mutated.as_slice())]);
        let _ = MmcArchive::parse(&bytes);
    }

    let valid_index = common::valid_multimodel("links/elements.xml", "models/model.ifc");
    let link = common::valid_link_model();
    for offset in (0..link.len()).step_by(5) {
        let mut mutated = link.clone();
        mutated[offset] ^= 0xff;
        let bytes = common::zip(&[
            ("MultiModel.xml", valid_index.as_slice()),
            ("models/model.ifc", b"ifc"),
            ("links/elements.xml", mutated.as_slice()),
        ]);
        let _ = MmcArchive::parse(&bytes);
    }
}
