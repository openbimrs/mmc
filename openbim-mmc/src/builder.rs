use std::{collections::BTreeMap, io::Cursor};

use quick_xml::{
    events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event},
    Writer,
};
use zip::{write::SimpleFileOptions, CompressionMethod, ZipWriter};

use crate::{
    path::{collision_key, validate_archive_path},
    ApplicationModel, ContainerMetadata, LinkModel, LinkModelReference, MetadataEntry, MmcArchive,
    MmcError, ResourceLocation, LINK_MODEL_NAMESPACE, MMC_NAMESPACE,
};

/// Builder for new deterministic MMC 2.0 archives.
#[derive(Debug)]
pub struct MmcArchiveBuilder {
    metadata: ContainerMetadata,
    models: Vec<ApplicationModel>,
    link_models: Vec<(String, Vec<String>, LinkModel)>,
    payloads: BTreeMap<String, Vec<u8>>,
    collisions: BTreeMap<String, String>,
}

impl MmcArchiveBuilder {
    #[must_use]
    pub fn new(metadata: ContainerMetadata) -> Self {
        let mut collisions = BTreeMap::new();
        collisions.insert(collision_key("MultiModel.xml"), "MultiModel.xml".to_owned());
        Self {
            metadata,
            models: Vec::new(),
            link_models: Vec::new(),
            payloads: BTreeMap::new(),
            collisions,
        }
    }

    pub fn add_application_model(
        &mut self,
        model: ApplicationModel,
    ) -> Result<&mut Self, MmcError> {
        if model.id.is_empty() || model.model_type.is_empty() {
            return Err(MmcError::InvalidBuilder(
                "application models require non-empty id and model_type".to_owned(),
            ));
        }
        self.models.push(model);
        Ok(self)
    }

    pub fn add_embedded_resource(
        &mut self,
        path: impl Into<String>,
        bytes: Vec<u8>,
    ) -> Result<&mut Self, MmcError> {
        let path = path.into();
        self.reserve_path(&path)?;
        self.payloads.insert(path, bytes);
        Ok(self)
    }

    pub fn add_link_model(
        &mut self,
        path: impl Into<String>,
        linked_models: Vec<String>,
        model: LinkModel,
    ) -> Result<&mut Self, MmcError> {
        let path = path.into();
        self.reserve_path(&path)?;
        if linked_models.iter().any(String::is_empty) {
            return Err(MmcError::InvalidBuilder(
                "linked model identifiers must not be empty".to_owned(),
            ));
        }
        self.link_models.push((path, linked_models, model));
        Ok(self)
    }

    pub fn build(self) -> Result<MmcArchive, MmcError> {
        let references = self
            .link_models
            .iter()
            .map(|(path, linked_models, _)| LinkModelReference {
                location: ResourceLocation::Embedded(path.clone()),
                linked_models: linked_models.clone(),
                metadata: Vec::new(),
            })
            .collect::<Vec<_>>();
        let index = serialize_multimodel(&self.metadata, &self.models, &references)?;
        let mut entries = self.payloads;
        entries.insert("MultiModel.xml".to_owned(), index);
        for (path, _, model) in self.link_models {
            entries.insert(path, serialize_link_model(&model)?);
        }

        let bytes = deterministic_zip(&entries)?;
        let archive = MmcArchive::parse(bytes)?;
        let report = archive.validate();
        if !report.is_valid() {
            return Err(MmcError::InvalidBuilder(format!(
                "built archive is not conformant: {:?}",
                report.issues()
            )));
        }
        Ok(archive)
    }

    fn reserve_path(&mut self, path: &str) -> Result<(), MmcError> {
        validate_archive_path(path)?;
        let key = collision_key(path);
        if let Some(first) = self.collisions.insert(key, path.to_owned()) {
            return Err(MmcError::DuplicateArchivePath {
                first,
                second: path.to_owned(),
            });
        }
        Ok(())
    }
}

fn serialize_multimodel(
    metadata: &ContainerMetadata,
    models: &[ApplicationModel],
    link_models: &[LinkModelReference],
) -> Result<Vec<u8>, MmcError> {
    let mut writer = Writer::new_with_indent(Vec::new(), b' ', 2);
    writer.write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))?;
    let mut root = BytesStart::new("mmc:MultiModel");
    root.push_attribute(("xmlns:mmc", MMC_NAMESPACE));
    let uuid = metadata.uuid.to_string();
    root.push_attribute(("uuid", uuid.as_str()));
    root.push_attribute(("formatVersion", metadata.format_version.as_str()));
    if let Some(domain) = &metadata.mm_domain {
        root.push_attribute(("mmDomain", domain.as_str()));
    }
    writer.write_event(Event::Start(root))?;
    write_metadata(&mut writer, "mmc", &metadata.metadata, false)?;

    writer.write_event(Event::Start(BytesStart::new("mmc:ApplicationModels")))?;
    for model in models {
        let mut element = BytesStart::new("mmc:ApplicationModel");
        element.push_attribute(("id", model.id.as_str()));
        element.push_attribute(("modelType", model.model_type.as_str()));
        writer.write_event(Event::Start(element))?;
        write_metadata(&mut writer, "mmc", &model.metadata, false)?;
        for representation in &model.representations {
            let mut element = BytesStart::new("mmc:ModelData");
            element.push_attribute(("id", representation.id.as_str()));
            element.push_attribute(("formatType", representation.format_type.as_str()));
            if let Some(version) = &representation.format_version {
                element.push_attribute(("formatVersion", version.as_str()));
            }
            writer.write_event(Event::Start(element))?;
            write_metadata(&mut writer, "mmc", &representation.metadata, false)?;
            for resource in &representation.resources {
                let mut element = BytesStart::new("mmc:DataRessource");
                element.push_attribute(("id", resource.id.as_str()));
                match &resource.location {
                    ResourceLocation::Embedded(path) | ResourceLocation::External(path) => {
                        element.push_attribute(("location", path.as_str()));
                    }
                }
                if resource.metadata.is_empty() {
                    writer.write_event(Event::Empty(element))?;
                } else {
                    writer.write_event(Event::Start(element))?;
                    write_metadata(&mut writer, "mmc", &resource.metadata, false)?;
                    writer.write_event(Event::End(BytesEnd::new("mmc:DataRessource")))?;
                }
            }
            writer.write_event(Event::End(BytesEnd::new("mmc:ModelData")))?;
        }
        writer.write_event(Event::End(BytesEnd::new("mmc:ApplicationModel")))?;
    }
    writer.write_event(Event::End(BytesEnd::new("mmc:ApplicationModels")))?;

    if !link_models.is_empty() {
        writer.write_event(Event::Start(BytesStart::new("mmc:LinkModels")))?;
        for reference in link_models {
            let mut element = BytesStart::new("mmc:LinkModel");
            match &reference.location {
                ResourceLocation::Embedded(path) | ResourceLocation::External(path) => {
                    element.push_attribute(("location", path.as_str()));
                }
            }
            writer.write_event(Event::Start(element))?;
            write_metadata(&mut writer, "mmc", &reference.metadata, false)?;
            for model in &reference.linked_models {
                writer.write_event(Event::Start(BytesStart::new("mmc:LinkedModel")))?;
                writer.write_event(Event::Text(BytesText::new(model)))?;
                writer.write_event(Event::End(BytesEnd::new("mmc:LinkedModel")))?;
            }
            writer.write_event(Event::End(BytesEnd::new("mmc:LinkModel")))?;
        }
        writer.write_event(Event::End(BytesEnd::new("mmc:LinkModels")))?;
    }
    writer.write_event(Event::End(BytesEnd::new("mmc:MultiModel")))?;
    Ok(writer.into_inner())
}

fn serialize_link_model(model: &LinkModel) -> Result<Vec<u8>, MmcError> {
    let mut writer = Writer::new_with_indent(Vec::new(), b' ', 2);
    writer.write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))?;
    let mut root = BytesStart::new("link:LinkModel");
    root.push_attribute(("xmlns:link", LINK_MODEL_NAMESPACE));
    root.push_attribute(("formatVersion", model.format_version.as_str()));
    writer.write_event(Event::Start(root))?;
    write_metadata(&mut writer, "link", &model.metadata, true)?;
    for link in &model.links {
        writer.write_event(Event::Start(BytesStart::new("link:Link")))?;
        write_metadata(&mut writer, "link", &link.metadata, true)?;
        for relatum in &link.relata {
            let mut element = BytesStart::new("link:Relatum");
            element.push_attribute(("id", relatum.element_id.as_str()));
            element.push_attribute(("m", relatum.model_id.as_str()));
            if let Some(format_id) = &relatum.format_id {
                element.push_attribute(("f", format_id.as_str()));
            }
            if let Some(resource_id) = &relatum.resource_id {
                element.push_attribute(("r", resource_id.as_str()));
            }
            if relatum.metadata.is_empty() && relatum.rates.is_empty() {
                writer.write_event(Event::Empty(element))?;
            } else {
                writer.write_event(Event::Start(element))?;
                write_metadata(&mut writer, "link", &relatum.metadata, true)?;
                for rate in &relatum.rates {
                    let mut rate_element = BytesStart::new("link:Rate");
                    rate_element.push_attribute(("t", rate.rate_type.as_str()));
                    rate_element.push_attribute(("v", rate.value.as_str()));
                    rate_element.push_attribute(("m", rate.target_model.as_str()));
                    writer.write_event(Event::Empty(rate_element))?;
                }
                writer.write_event(Event::End(BytesEnd::new("link:Relatum")))?;
            }
        }
        writer.write_event(Event::End(BytesEnd::new("link:Link")))?;
    }
    writer.write_event(Event::End(BytesEnd::new("link:LinkModel")))?;
    Ok(writer.into_inner())
}

fn write_metadata(
    writer: &mut Writer<Vec<u8>>,
    prefix: &str,
    metadata: &[MetadataEntry],
    compact_attributes: bool,
) -> Result<(), MmcError> {
    if metadata.is_empty() {
        return Ok(());
    }
    let collection = format!("{prefix}:MetaData");
    let item_name = format!("{prefix}:Meta");
    writer.write_event(Event::Start(BytesStart::new(collection.as_str())))?;
    for item in metadata {
        let mut element = BytesStart::new(item_name.as_str());
        let (key, value, value_type, category) = if compact_attributes {
            ("k", "v", "t", "c")
        } else {
            ("key", "value", "type", "category")
        };
        element.push_attribute((key, item.key.as_str()));
        element.push_attribute((value, item.value.as_str()));
        if let Some(kind) = &item.value_type {
            element.push_attribute((value_type, kind.as_str()));
        }
        if let Some(group) = &item.category {
            element.push_attribute((category, group.as_str()));
        }
        writer.write_event(Event::Empty(element))?;
    }
    writer.write_event(Event::End(BytesEnd::new(collection.as_str())))?;
    Ok(())
}

fn deterministic_zip(entries: &BTreeMap<String, Vec<u8>>) -> Result<Vec<u8>, MmcError> {
    let mut output = Cursor::new(Vec::new());
    {
        let mut writer = ZipWriter::new(&mut output);
        let options = SimpleFileOptions::default()
            .compression_method(CompressionMethod::Deflated)
            .unix_permissions(0o100644);
        for (path, bytes) in entries {
            writer.start_file(path, options)?;
            std::io::Write::write_all(&mut writer, bytes)?;
        }
        writer.finish()?;
    }
    Ok(output.into_inner())
}
