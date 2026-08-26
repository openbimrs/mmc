use openbim_mmc::{
    ApplicationModel, ContainerMetadata, DataResource, Link, LinkModel, MetadataEntry,
    MmcArchiveBuilder, ModelData, Rate, Relatum, ResourceLocation,
};
use uuid::Uuid;

#[test]
fn builds_a_new_mmc_with_escaped_metadata_and_round_trips() {
    let metadata = ContainerMetadata {
        uuid: Uuid::parse_str("4d69a342-31b6-4e80-9d05-83a28754c84d").unwrap(),
        format_version: "2.0.0".to_owned(),
        mm_domain: Some("urn:din:18290:lv".to_owned()),
        metadata: vec![MetadataEntry {
            key: "project".to_owned(),
            value: "P<&\"1".to_owned(),
            ..MetadataEntry::default()
        }],
    };
    let model = ApplicationModel {
        id: "model-ifc".to_owned(),
        model_type: "IFC".to_owned(),
        metadata: vec![],
        representations: vec![ModelData {
            id: "ifc-spf".to_owned(),
            format_type: "IFC".to_owned(),
            format_version: Some("IFC4".to_owned()),
            metadata: vec![],
            resources: vec![DataResource {
                id: "ifc-file".to_owned(),
                location: ResourceLocation::Embedded("models/model.ifc".to_owned()),
                metadata: vec![],
            }],
        }],
    };
    let link_model = LinkModel {
        format_version: "2.0.0".to_owned(),
        metadata: vec![MetadataEntry {
            key: "role".to_owned(),
            value: "Links & relations".to_owned(),
            ..MetadataEntry::default()
        }],
        links: vec![Link {
            metadata: vec![],
            relata: vec![
                Relatum {
                    model_id: "model-ifc".to_owned(),
                    element_id: "2AbC".to_owned(),
                    format_id: Some("ifc-spf".to_owned()),
                    resource_id: Some("ifc-file".to_owned()),
                    metadata: vec![],
                    rates: vec![Rate {
                        rate_type: "confidence".to_owned(),
                        value: "1.0".to_owned(),
                        target_model: "model-ifc".to_owned(),
                    }],
                },
                Relatum {
                    model_id: "model-gaeb".to_owned(),
                    element_id: "item-1".to_owned(),
                    ..Relatum::default()
                },
            ],
        }],
    };

    let mut builder = MmcArchiveBuilder::new(metadata);
    builder.add_application_model(model).unwrap();
    builder
        .add_application_model(ApplicationModel {
            id: "model-gaeb".to_owned(),
            model_type: "GAEB".to_owned(),
            representations: vec![ModelData {
                id: "gaeb-xml".to_owned(),
                format_type: "GAEB DA XML".to_owned(),
                format_version: Some("3.3".to_owned()),
                resources: vec![DataResource {
                    id: "gaeb-file".to_owned(),
                    location: ResourceLocation::External(
                        "https://example.test/bill.xml".to_owned(),
                    ),
                    ..DataResource::default()
                }],
                ..ModelData::default()
            }],
            ..ApplicationModel::default()
        })
        .unwrap();
    builder
        .add_embedded_resource("models/model.ifc", b"IFC bytes".to_vec())
        .unwrap();
    builder
        .add_link_model(
            "links/relations.xml",
            vec!["model-ifc".to_owned(), "model-gaeb".to_owned()],
            link_model,
        )
        .unwrap();
    let archive = builder.build().unwrap();

    assert!(
        archive.validate().is_valid(),
        "{:?}",
        archive.validate().issues()
    );
    let link_xml = std::str::from_utf8(archive.parsed_link_models()[0].source_bytes()).unwrap();
    assert!(link_xml.contains("<link:Rate t=\"confidence\" v=\"1.0\" m=\"model-ifc\"/>"));
    assert!(!link_xml.contains("<link:Rates"));
    assert_eq!(archive.container().metadata.metadata[0].value, "P<&\"1");
    assert_eq!(
        archive.parsed_link_models()[0].model().metadata[0].value,
        "Links & relations"
    );
    assert_eq!(
        archive.parsed_link_models()[0].model().links[0].relata[0].rates[0].value,
        "1.0"
    );
}

#[test]
fn builder_rejects_collisions_and_missing_payloads() {
    let mut builder = MmcArchiveBuilder::new(ContainerMetadata::default());
    builder
        .add_embedded_resource("MultiModel.xml", vec![])
        .unwrap_err();
    builder.add_embedded_resource("a.bin", vec![1]).unwrap();
    builder.add_embedded_resource("A.BIN", vec![2]).unwrap_err();

    let model = ApplicationModel {
        id: "missing".to_owned(),
        model_type: "IFC".to_owned(),
        representations: vec![ModelData {
            id: "r".to_owned(),
            format_type: "IFC".to_owned(),
            resources: vec![DataResource {
                id: "f".to_owned(),
                location: ResourceLocation::Embedded("missing.ifc".to_owned()),
                ..DataResource::default()
            }],
            ..ModelData::default()
        }],
        ..ApplicationModel::default()
    };
    builder.add_application_model(model).unwrap();
    assert!(builder.build().is_err());
}
