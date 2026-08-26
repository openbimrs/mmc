use std::collections::HashMap;

use quick_xml::{
    encoding::Decoder,
    events::{BytesRef, BytesStart, Event},
    name::ResolveResult,
    NsReader,
};
use uuid::Uuid;

use crate::{
    ApplicationModel, ContainerMetadata, DataResource, Limits, Link, LinkModel, LinkModelReference,
    MetadataEntry, MmcError, ModelData, MultiModel, Rate, Relatum, ResourceLocation,
    LINK_MODEL_NAMESPACE, MMC_NAMESPACE,
};

type Attributes = HashMap<String, String>;

pub(crate) fn parse_multimodel(source: &[u8], limits: Limits) -> Result<MultiModel, MmcError> {
    check_xml_size("MultiModel.xml", source, limits)?;
    let mut reader = NsReader::from_reader(source);
    reader.config_mut().trim_text(false);
    let mut buffer = Vec::new();
    let mut stack = Vec::<String>::new();
    let mut events = 0usize;
    let mut root_seen = false;
    let mut metadata: Option<ContainerMetadata> = None;
    let mut models = Vec::<ApplicationModel>::new();
    let mut link_models = Vec::<LinkModelReference>::new();
    let mut linked_model_text: Option<String> = None;

    loop {
        let (namespace, event) = reader
            .read_resolved_event_into(&mut buffer)
            .map_err(|error| xml_error("MultiModel.xml", error))?;
        events = events.saturating_add(1);
        check_count("XML events", events, limits.max_xml_events)?;
        match event {
            Event::Start(element) => {
                let local = local_name(&element)?;
                let standard = resolved_namespace(namespace, MMC_NAMESPACE)?;
                if stack.is_empty() {
                    ensure_root("MultiModel.xml", &local, standard, "MultiModel")?;
                    if root_seen {
                        return Err(xml_message("MultiModel.xml", "multiple root elements"));
                    }
                    root_seen = true;
                    metadata = Some(parse_container_attributes(&attributes(
                        &element,
                        reader.decoder(),
                        "MultiModel.xml",
                    )?)?);
                }
                stack.push(stack_name(local, standard));
                if standard {
                    if !valid_multimodel_path(&stack) {
                        return Err(xml_message(
                            "MultiModel.xml",
                            "unexpected MMC element position",
                        ));
                    }
                    handle_multimodel_start(
                        &stack,
                        &element,
                        reader.decoder(),
                        &mut models,
                        &mut link_models,
                        &mut metadata,
                        limits,
                    )?;
                    if path_ends(&stack, &["LinkModel", "LinkedModel"]) {
                        linked_model_text = Some(String::new());
                    }
                }
            }
            Event::Empty(element) => {
                let local = local_name(&element)?;
                let standard = resolved_namespace(namespace, MMC_NAMESPACE)?;
                if stack.is_empty() {
                    ensure_root("MultiModel.xml", &local, standard, "MultiModel")?;
                    if root_seen {
                        return Err(xml_message("MultiModel.xml", "multiple root elements"));
                    }
                    root_seen = true;
                    metadata = Some(parse_container_attributes(&attributes(
                        &element,
                        reader.decoder(),
                        "MultiModel.xml",
                    )?)?);
                }
                stack.push(stack_name(local, standard));
                if standard {
                    if !valid_multimodel_path(&stack) {
                        return Err(xml_message(
                            "MultiModel.xml",
                            "unexpected MMC element position",
                        ));
                    }
                    handle_multimodel_start(
                        &stack,
                        &element,
                        reader.decoder(),
                        &mut models,
                        &mut link_models,
                        &mut metadata,
                        limits,
                    )?;
                }
                stack.pop();
            }
            Event::Text(text) => {
                let decoded = text
                    .xml_content()
                    .map_err(|error| xml_error("MultiModel.xml", error))?;
                if decoded.chars().any(|character| !is_xml_char(character)) {
                    return Err(xml_message(
                        "MultiModel.xml",
                        "text resolves to a character forbidden by XML 1.0",
                    ));
                }
                if stack.is_empty() && !decoded.trim().is_empty() {
                    return Err(xml_message(
                        "MultiModel.xml",
                        "character content outside the document root",
                    ));
                }
                if let Some(value) = &mut linked_model_text {
                    value.push_str(&decoded);
                }
            }
            Event::CData(text) => {
                if stack.is_empty() {
                    return Err(xml_message(
                        "MultiModel.xml",
                        "CDATA outside the document root",
                    ));
                }
                if let Some(value) = &mut linked_model_text {
                    value.push_str(
                        std::str::from_utf8(text.as_ref())
                            .map_err(|error| xml_error("MultiModel.xml", error))?,
                    );
                }
            }
            Event::End(_) => {
                if path_ends(&stack, &["LinkModel", "LinkedModel"]) {
                    let value = linked_model_text.take().unwrap_or_default();
                    if let Some(reference) = link_models.last_mut() {
                        reference.linked_models.push(value);
                    }
                }
                stack.pop();
            }
            Event::Decl(_) if events != 1 => {
                return Err(xml_message(
                    "MultiModel.xml",
                    "XML declaration is only allowed at document start",
                ));
            }
            Event::DocType(_) => {
                return Err(xml_message("MultiModel.xml", "DOCTYPE is prohibited"));
            }
            Event::GeneralRef(reference) => {
                if stack.is_empty() {
                    return Err(xml_message(
                        "MultiModel.xml",
                        "entity reference outside the document root",
                    ));
                }
                let character = resolve_xml_reference(&reference, "MultiModel.xml")?;
                if let Some(value) = &mut linked_model_text {
                    value.push(character);
                }
            }
            Event::Eof => {
                if !stack.is_empty() {
                    return Err(xml_message("MultiModel.xml", "truncated XML document"));
                }
                break;
            }
            _ => {}
        }
        buffer.clear();
    }

    if !root_seen {
        return Err(xml_message("MultiModel.xml", "missing MultiModel root"));
    }
    Ok(MultiModel {
        metadata: metadata.ok_or_else(|| xml_message("MultiModel.xml", "missing root metadata"))?,
        models,
        link_models,
        source: source.to_vec(),
    })
}

#[allow(clippy::too_many_arguments)]
fn handle_multimodel_start(
    stack: &[String],
    element: &BytesStart<'_>,
    decoder: Decoder,
    models: &mut Vec<ApplicationModel>,
    link_models: &mut Vec<LinkModelReference>,
    container: &mut Option<ContainerMetadata>,
    limits: Limits,
) -> Result<(), MmcError> {
    let attrs = attributes(element, decoder, "MultiModel.xml")?;
    if path_ends(stack, &["ApplicationModels", "ApplicationModel"]) {
        models.push(ApplicationModel {
            id: required(&attrs, "id", "ApplicationModel")?,
            model_type: required(&attrs, "modelType", "ApplicationModel")?,
            ..ApplicationModel::default()
        });
        check_count("application models", models.len(), limits.max_models)?;
    } else if path_ends(stack, &["ApplicationModel", "ModelData"]) {
        let model = models
            .last_mut()
            .ok_or_else(|| xml_message("MultiModel.xml", "ModelData outside ApplicationModel"))?;
        model.representations.push(ModelData {
            id: required(&attrs, "id", "ModelData")?,
            format_type: required(&attrs, "formatType", "ModelData")?,
            format_version: attrs.get("formatVersion").cloned(),
            ..ModelData::default()
        });
    } else if path_ends(stack, &["ModelData", "DataRessource"]) {
        let model = models
            .last_mut()
            .ok_or_else(|| xml_message("MultiModel.xml", "DataRessource outside model"))?;
        let representation = model
            .representations
            .last_mut()
            .ok_or_else(|| xml_message("MultiModel.xml", "DataRessource outside ModelData"))?;
        representation.resources.push(DataResource {
            id: required(&attrs, "id", "DataRessource")?,
            location: classify_location(required(&attrs, "location", "DataRessource")?),
            ..DataResource::default()
        });
    } else if path_ends(stack, &["LinkModels", "LinkModel"]) {
        link_models.push(LinkModelReference {
            location: classify_location(required(&attrs, "location", "LinkModel")?),
            ..LinkModelReference::default()
        });
        check_count("link models", link_models.len(), limits.max_link_models)?;
    } else if stack.last().is_some_and(|name| name == "Meta") {
        let item = parse_mmc_meta(&attrs)?;
        if stack
            .iter()
            .rev()
            .nth(2)
            .is_some_and(|name| name == "ApplicationModel")
        {
            let model = models
                .last_mut()
                .ok_or_else(|| xml_message("MultiModel.xml", "Meta outside ApplicationModel"))?;
            model.metadata.push(item);
        } else if stack
            .iter()
            .rev()
            .nth(2)
            .is_some_and(|name| name == "ModelData")
        {
            let representation = models
                .last_mut()
                .and_then(|model| model.representations.last_mut())
                .ok_or_else(|| xml_message("MultiModel.xml", "Meta outside ModelData"))?;
            representation.metadata.push(item);
        } else if stack
            .iter()
            .rev()
            .nth(2)
            .is_some_and(|name| name == "DataRessource")
        {
            let resource = models
                .last_mut()
                .and_then(|model| model.representations.last_mut())
                .and_then(|representation| representation.resources.last_mut())
                .ok_or_else(|| xml_message("MultiModel.xml", "Meta outside DataRessource"))?;
            resource.metadata.push(item);
        } else if stack
            .iter()
            .rev()
            .nth(2)
            .is_some_and(|name| name == "LinkModel")
        {
            let reference = link_models
                .last_mut()
                .ok_or_else(|| xml_message("MultiModel.xml", "Meta outside LinkModel"))?;
            reference.metadata.push(item);
        } else if stack == ["MultiModel", "MetaData", "Meta"] {
            let root = container
                .as_mut()
                .ok_or_else(|| xml_message("MultiModel.xml", "Meta outside MultiModel"))?;
            root.metadata.push(item);
        }
    }
    Ok(())
}

pub(crate) fn parse_link_model(
    path: &str,
    source: &[u8],
    limits: Limits,
) -> Result<LinkModel, MmcError> {
    check_xml_size(path, source, limits)?;
    let mut reader = NsReader::from_reader(source);
    reader.config_mut().trim_text(false);
    let mut buffer = Vec::new();
    let mut stack = Vec::<String>::new();
    let mut events = 0usize;
    let mut root_seen = false;
    let mut model: Option<LinkModel> = None;
    let mut relata = 0usize;

    loop {
        let (namespace, event) = reader
            .read_resolved_event_into(&mut buffer)
            .map_err(|error| xml_error(path, error))?;
        events = events.saturating_add(1);
        check_count("XML events", events, limits.max_xml_events)?;
        match event {
            Event::Start(ref element) | Event::Empty(ref element) => {
                let is_empty = matches!(&event, Event::Empty(_));
                let local = local_name(element)?;
                let standard = resolved_namespace(namespace, LINK_MODEL_NAMESPACE)?;
                if stack.is_empty() {
                    ensure_root(path, &local, standard, "LinkModel")?;
                    if root_seen {
                        return Err(xml_message(path, "multiple root elements"));
                    }
                    root_seen = true;
                    let attrs = attributes(element, reader.decoder(), path)?;
                    model = Some(LinkModel {
                        format_version: required_at(&attrs, "formatVersion", "LinkModel", path)?,
                        ..LinkModel::default()
                    });
                }
                stack.push(stack_name(local, standard));
                if standard {
                    if !valid_link_path(&stack) {
                        return Err(xml_message(path, "unexpected LinkModel element position"));
                    }
                    let attrs = attributes(element, reader.decoder(), path)?;
                    handle_link_start(&stack, &attrs, model.as_mut(), &mut relata, limits, path)?;
                }
                if is_empty {
                    stack.pop();
                }
            }
            Event::End(_) => {
                stack.pop();
            }
            Event::Text(text) => {
                let decoded = text.xml_content().map_err(|error| xml_error(path, error))?;
                if decoded.chars().any(|character| !is_xml_char(character)) {
                    return Err(xml_message(
                        path,
                        "text resolves to a character forbidden by XML 1.0",
                    ));
                }
                if stack.is_empty() && !decoded.trim().is_empty() {
                    return Err(xml_message(
                        path,
                        "character content outside the document root",
                    ));
                }
            }
            Event::CData(_) if stack.is_empty() => {
                return Err(xml_message(path, "CDATA outside the document root"));
            }
            Event::Decl(_) if events != 1 => {
                return Err(xml_message(
                    path,
                    "XML declaration is only allowed at document start",
                ));
            }
            Event::DocType(_) => return Err(xml_message(path, "DOCTYPE is prohibited")),
            Event::GeneralRef(reference) => {
                if stack.is_empty() {
                    return Err(xml_message(
                        path,
                        "entity reference outside the document root",
                    ));
                }
                resolve_xml_reference(&reference, path)?;
            }
            Event::Eof => {
                if !stack.is_empty() {
                    return Err(xml_message(path, "truncated XML document"));
                }
                break;
            }
            _ => {}
        }
        buffer.clear();
    }
    if !root_seen {
        return Err(xml_message(path, "missing LinkModel root"));
    }
    model.ok_or_else(|| xml_message(path, "missing LinkModel"))
}

fn handle_link_start(
    stack: &[String],
    attrs: &Attributes,
    model: Option<&mut LinkModel>,
    relata_count: &mut usize,
    limits: Limits,
    path: &str,
) -> Result<(), MmcError> {
    let model = model.ok_or_else(|| xml_message(path, "content outside LinkModel"))?;
    if path_ends(stack, &["LinkModel", "Link"]) {
        model.links.push(Link::default());
        check_count("links", model.links.len(), limits.max_links)?;
    } else if path_ends(stack, &["Link", "Relatum"]) {
        let link = model
            .links
            .last_mut()
            .ok_or_else(|| xml_message(path, "Relatum outside Link"))?;
        link.relata.push(Relatum {
            element_id: required_at(attrs, "id", "Relatum", path)?,
            model_id: required_at(attrs, "m", "Relatum", path)?,
            format_id: attrs.get("f").cloned(),
            resource_id: attrs.get("r").cloned(),
            ..Relatum::default()
        });
        *relata_count = relata_count.saturating_add(1);
        check_count("relata", *relata_count, limits.max_linked_elements)?;
    } else if path_ends(stack, &["Relatum", "Rate"]) {
        let relatum = model
            .links
            .last_mut()
            .and_then(|link| link.relata.last_mut())
            .ok_or_else(|| xml_message(path, "Rate outside Relatum"))?;
        relatum.rates.push(Rate {
            rate_type: required_at(attrs, "t", "Rate", path)?,
            value: required_at(attrs, "v", "Rate", path)?,
            target_model: required_at(attrs, "m", "Rate", path)?,
        });
    } else if stack.last().is_some_and(|name| name == "Meta") {
        let item = parse_link_meta(attrs, path)?;
        if stack
            .iter()
            .rev()
            .nth(2)
            .is_some_and(|name| name == "Relatum")
        {
            let relatum = model
                .links
                .last_mut()
                .and_then(|link| link.relata.last_mut())
                .ok_or_else(|| xml_message(path, "Meta outside Relatum"))?;
            relatum.metadata.push(item);
        } else if stack.iter().rev().nth(2).is_some_and(|name| name == "Link") {
            let link = model
                .links
                .last_mut()
                .ok_or_else(|| xml_message(path, "Meta outside Link"))?;
            link.metadata.push(item);
        } else if stack == ["LinkModel", "MetaData", "Meta"] {
            model.metadata.push(item);
        }
    }
    Ok(())
}

fn parse_container_attributes(attrs: &Attributes) -> Result<ContainerMetadata, MmcError> {
    let raw_uuid = required(attrs, "uuid", "MultiModel")?;
    let uuid = Uuid::parse_str(&raw_uuid)
        .map_err(|error| xml_message("MultiModel.xml", format!("invalid uuid: {error}")))?;
    Ok(ContainerMetadata {
        uuid,
        format_version: required(attrs, "formatVersion", "MultiModel")?,
        mm_domain: attrs.get("mmDomain").cloned(),
        metadata: Vec::new(),
    })
}

fn parse_mmc_meta(attrs: &Attributes) -> Result<MetadataEntry, MmcError> {
    Ok(MetadataEntry {
        key: required(attrs, "key", "Meta")?,
        value: required(attrs, "value", "Meta")?,
        value_type: attrs.get("type").cloned(),
        category: attrs.get("category").cloned(),
    })
}

fn parse_link_meta(attrs: &Attributes, path: &str) -> Result<MetadataEntry, MmcError> {
    Ok(MetadataEntry {
        key: required_at(attrs, "k", "Meta", path)?,
        value: required_at(attrs, "v", "Meta", path)?,
        value_type: attrs.get("t").cloned(),
        category: attrs.get("c").cloned(),
    })
}

fn classify_location(value: String) -> ResourceLocation {
    if has_uri_scheme(&value) {
        ResourceLocation::External(value)
    } else {
        ResourceLocation::Embedded(value)
    }
}

fn has_uri_scheme(value: &str) -> bool {
    let Some(colon) = value.find(':') else {
        return false;
    };
    let boundary = value.find(['/', '?', '#']).unwrap_or(value.len());
    colon < boundary
        && value[..colon].chars().enumerate().all(|(index, ch)| {
            ch.is_ascii_alphanumeric() || (index > 0 && matches!(ch, '+' | '-' | '.'))
        })
}

fn attributes(
    element: &BytesStart<'_>,
    decoder: Decoder,
    path: &str,
) -> Result<Attributes, MmcError> {
    let mut result = HashMap::new();
    for attribute in element.attributes().with_checks(true) {
        let attribute = attribute.map_err(|error| xml_error(path, error))?;
        if attribute.key.prefix().is_some() {
            continue;
        }
        let key = std::str::from_utf8(attribute.key.local_name().as_ref())
            .map_err(|error| xml_error(path, error))?
            .to_owned();
        let value = attribute
            .decode_and_unescape_value(decoder)
            .map_err(|error| xml_error(path, error))?
            .into_owned();
        if value.chars().any(|character| !is_xml_char(character)) {
            return Err(xml_message(
                path,
                format!("attribute {key} resolves to a character forbidden by XML 1.0"),
            ));
        }
        if result.insert(key.clone(), value).is_some() {
            return Err(xml_message(path, format!("duplicate attribute {key}")));
        }
    }
    Ok(result)
}

fn required(attrs: &Attributes, name: &str, element: &str) -> Result<String, MmcError> {
    required_at(attrs, name, element, "MultiModel.xml")
}

fn required_at(
    attrs: &Attributes,
    name: &str,
    element: &str,
    path: &str,
) -> Result<String, MmcError> {
    attrs
        .get(name)
        .filter(|value| !value.is_empty())
        .cloned()
        .ok_or_else(|| xml_message(path, format!("{element} requires @{name}")))
}

fn local_name(element: &BytesStart<'_>) -> Result<String, MmcError> {
    std::str::from_utf8(element.local_name().as_ref())
        .map(str::to_owned)
        .map_err(|error| xml_error("XML", error))
}

fn resolved_namespace(namespace: ResolveResult<'_>, expected: &str) -> Result<bool, MmcError> {
    match namespace {
        ResolveResult::Bound(namespace) => Ok(namespace.as_ref() == expected.as_bytes()),
        ResolveResult::Unbound => Err(xml_message("XML", "unbound namespace prefix")),
        ResolveResult::Unknown(_) => Ok(false),
    }
}

fn ensure_root(
    path: &str,
    local: &str,
    standard_namespace: bool,
    expected: &str,
) -> Result<(), MmcError> {
    if local != expected || !standard_namespace {
        return Err(xml_message(
            path,
            format!(
                "expected {{{}}}{expected} root",
                if expected == "MultiModel" {
                    MMC_NAMESPACE
                } else {
                    LINK_MODEL_NAMESPACE
                }
            ),
        ));
    }
    Ok(())
}

fn valid_multimodel_path(stack: &[String]) -> bool {
    let path = stack.iter().map(String::as_str).collect::<Vec<_>>();
    matches!(
        path.as_slice(),
        ["MultiModel"]
            | ["MultiModel", "MetaData"]
            | ["MultiModel", "MetaData", "Meta"]
            | ["MultiModel", "ApplicationModels"]
            | ["MultiModel", "ApplicationModels", "ApplicationModel"]
            | [
                "MultiModel",
                "ApplicationModels",
                "ApplicationModel",
                "MetaData"
            ]
            | [
                "MultiModel",
                "ApplicationModels",
                "ApplicationModel",
                "MetaData",
                "Meta"
            ]
            | [
                "MultiModel",
                "ApplicationModels",
                "ApplicationModel",
                "ModelData"
            ]
            | [
                "MultiModel",
                "ApplicationModels",
                "ApplicationModel",
                "ModelData",
                "MetaData"
            ]
            | [
                "MultiModel",
                "ApplicationModels",
                "ApplicationModel",
                "ModelData",
                "MetaData",
                "Meta"
            ]
            | [
                "MultiModel",
                "ApplicationModels",
                "ApplicationModel",
                "ModelData",
                "DataRessource"
            ]
            | [
                "MultiModel",
                "ApplicationModels",
                "ApplicationModel",
                "ModelData",
                "DataRessource",
                "MetaData"
            ]
            | [
                "MultiModel",
                "ApplicationModels",
                "ApplicationModel",
                "ModelData",
                "DataRessource",
                "MetaData",
                "Meta"
            ]
            | ["MultiModel", "LinkModels"]
            | ["MultiModel", "LinkModels", "LinkModel"]
            | ["MultiModel", "LinkModels", "LinkModel", "MetaData"]
            | ["MultiModel", "LinkModels", "LinkModel", "MetaData", "Meta"]
            | ["MultiModel", "LinkModels", "LinkModel", "LinkedModel"]
    )
}

fn valid_link_path(stack: &[String]) -> bool {
    let path = stack.iter().map(String::as_str).collect::<Vec<_>>();
    matches!(
        path.as_slice(),
        ["LinkModel"]
            | ["LinkModel", "MetaData"]
            | ["LinkModel", "MetaData", "Meta"]
            | ["LinkModel", "Link"]
            | ["LinkModel", "Link", "MetaData"]
            | ["LinkModel", "Link", "MetaData", "Meta"]
            | ["LinkModel", "Link", "Relatum"]
            | ["LinkModel", "Link", "Relatum", "MetaData"]
            | ["LinkModel", "Link", "Relatum", "MetaData", "Meta"]
            | ["LinkModel", "Link", "Relatum", "Rate"]
    )
}

fn path_ends(stack: &[String], suffix: &[&str]) -> bool {
    stack.len() >= suffix.len()
        && stack[stack.len() - suffix.len()..]
            .iter()
            .map(String::as_str)
            .eq(suffix.iter().copied())
}

fn stack_name(local: String, standard_namespace: bool) -> String {
    if standard_namespace {
        local
    } else {
        // Extension elements stay lossless in `source` but cannot impersonate
        // a standard ancestor in the typed projection.
        format!("\0{local}")
    }
}

fn check_xml_size(path: &str, source: &[u8], limits: Limits) -> Result<(), MmcError> {
    if source.len() > limits.max_xml_bytes {
        return Err(MmcError::LimitExceeded {
            resource: "XML bytes",
            actual: source.len() as u64,
            maximum: limits.max_xml_bytes as u64,
        });
    }
    let text = std::str::from_utf8(source).map_err(|_| xml_message(path, "XML is not UTF-8"))?;
    if text.chars().any(|character| !is_xml_char(character)) {
        return Err(xml_message(
            path,
            "XML contains a character forbidden by XML 1.0",
        ));
    }
    Ok(())
}

fn is_xml_char(character: char) -> bool {
    matches!(
        character,
        '\u{9}' | '\u{A}' | '\u{D}' | '\u{20}'..='\u{D7FF}' | '\u{E000}'..='\u{FFFD}' | '\u{10000}'..='\u{10FFFF}'
    )
}

fn resolve_xml_reference(reference: &BytesRef<'_>, path: &str) -> Result<char, MmcError> {
    let character = match reference
        .resolve_char_ref()
        .map_err(|error| xml_error(path, error))?
    {
        Some(character) => character,
        None => match reference
            .decode()
            .map_err(|error| xml_error(path, error))?
            .as_ref()
        {
            "lt" => '<',
            "gt" => '>',
            "amp" => '&',
            "apos" => '\'',
            "quot" => '"',
            name => {
                return Err(xml_message(
                    path,
                    format!("undeclared entity reference &{name};"),
                ));
            }
        },
    };
    if !is_xml_char(character) {
        return Err(xml_message(
            path,
            "entity reference resolves to a character forbidden by XML 1.0",
        ));
    }
    Ok(character)
}

fn check_count(resource: &'static str, actual: usize, maximum: usize) -> Result<(), MmcError> {
    if actual > maximum {
        return Err(MmcError::LimitExceeded {
            resource,
            actual: actual as u64,
            maximum: maximum as u64,
        });
    }
    Ok(())
}

fn xml_error(path: &str, error: impl std::fmt::Display) -> MmcError {
    MmcError::Xml {
        path: path.to_owned(),
        message: error.to_string(),
    }
}

fn xml_message(path: &str, message: impl Into<String>) -> MmcError {
    MmcError::Xml {
        path: path.to_owned(),
        message: message.into(),
    }
}
