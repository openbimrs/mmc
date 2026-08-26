use std::collections::{HashMap, HashSet};

use iri_string::types::IriReferenceStr;

use crate::{path::validate_archive_path, MmcArchive, ResourceLocation};

/// Stable machine-readable conformance issue codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ValidationCode {
    TooFewApplicationModels,
    MissingLinkModels,
    ApplicationModelMissingRepresentation,
    ModelDataMissingResource,
    LinkModelHasNoLinks,
    DuplicateModelId,
    DuplicateRepresentationId,
    DuplicateResourceId,
    InvalidXmlId,
    DuplicateXmlId,
    DuplicateLinkModelLocation,
    InvalidResourceLocation,
    MissingEmbeddedResource,
    MissingEmbeddedLinkModel,
    TooFewLinkedModels,
    UnknownModelReference,
    UnknownRepresentationReference,
    UnknownResourceReference,
    LinkHasTooFewRelata,
    RateTargetsUnknownModel,
}

/// One deterministic conformance finding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationIssue {
    pub code: ValidationCode,
    pub location: String,
    pub message: String,
}

/// Complete structural and referential-integrity report for an opened archive.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ValidationReport {
    issues: Vec<ValidationIssue>,
}

impl ValidationReport {
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.issues.is_empty()
    }

    #[must_use]
    pub fn issues(&self) -> &[ValidationIssue] {
        &self.issues
    }

    #[must_use]
    pub fn contains(&self, code: ValidationCode) -> bool {
        self.issues.iter().any(|issue| issue.code == code)
    }

    fn push(
        &mut self,
        code: ValidationCode,
        location: impl Into<String>,
        message: impl Into<String>,
    ) {
        self.issues.push(ValidationIssue {
            code,
            location: location.into(),
            message: message.into(),
        });
    }
}

pub(crate) fn validate(archive: &MmcArchive) -> ValidationReport {
    let container = archive.container();
    let mut report = ValidationReport::default();
    if IriReferenceStr::new(&container.metadata.mm_domain).is_err() {
        report.push(
            ValidationCode::InvalidResourceLocation,
            "MultiModel.xml@mmDomain",
            "mmDomain is not a valid IRI reference",
        );
    }
    if container.models.len() < 2 {
        report.push(
            ValidationCode::TooFewApplicationModels,
            "MultiModel.xml",
            "MMC 2.0 requires at least two ApplicationModel declarations",
        );
    }
    if container.link_models.is_empty() {
        report.push(
            ValidationCode::MissingLinkModels,
            "MultiModel.xml",
            "MMC 2.0 requires at least one LinkModel declaration",
        );
    }

    let mut model_ids = HashSet::new();
    let mut xml_ids = HashSet::new();
    let mut representations = HashMap::<&str, HashSet<&str>>::new();
    let mut resources = HashMap::<(&str, &str), HashSet<&str>>::new();
    for (model_index, model) in container.models.iter().enumerate() {
        let model_location = format!("ApplicationModel[{model_index}]");
        validate_xml_id(&model.id, &model_location, &mut xml_ids, &mut report);
        if !model_ids.insert(model.id.as_str()) {
            report.push(
                ValidationCode::DuplicateModelId,
                &model_location,
                format!("duplicate application-model id {}", model.id),
            );
        }
        if model.representations.is_empty() {
            report.push(
                ValidationCode::ApplicationModelMissingRepresentation,
                &model_location,
                "ApplicationModel requires at least one ModelData",
            );
        }
        let mut representation_ids = HashSet::new();
        for (data_index, data) in model.representations.iter().enumerate() {
            let data_location = format!("{model_location}/ModelData[{data_index}]");
            validate_xml_id(&data.id, &data_location, &mut xml_ids, &mut report);
            if !representation_ids.insert(data.id.as_str()) {
                report.push(
                    ValidationCode::DuplicateRepresentationId,
                    &data_location,
                    format!(
                        "duplicate representation id {} in model {}",
                        data.id, model.id
                    ),
                );
            }
            if data.resources.is_empty() {
                report.push(
                    ValidationCode::ModelDataMissingResource,
                    &data_location,
                    "ModelData requires at least one DataRessource",
                );
            }
            let mut resource_ids = HashSet::new();
            for (resource_index, resource) in data.resources.iter().enumerate() {
                let resource_location = format!("{data_location}/DataRessource[{resource_index}]");
                validate_xml_id(&resource.id, &resource_location, &mut xml_ids, &mut report);
                if !resource_ids.insert(resource.id.as_str()) {
                    report.push(
                        ValidationCode::DuplicateResourceId,
                        &resource_location,
                        format!(
                            "duplicate resource id {} in representation {}",
                            resource.id, data.id
                        ),
                    );
                }
                validate_location(
                    archive,
                    &resource.location,
                    &resource_location,
                    ValidationCode::MissingEmbeddedResource,
                    &mut report,
                );
            }
            resources.insert((model.id.as_str(), data.id.as_str()), resource_ids);
        }
        representations.insert(model.id.as_str(), representation_ids);
    }

    let mut link_locations = HashSet::new();
    for (reference_index, reference) in container.link_models.iter().enumerate() {
        let location = format!("LinkModel[{reference_index}]");
        let location_key = match &reference.location {
            ResourceLocation::Embedded(path) => format!("embedded:{path}"),
            ResourceLocation::External(uri) => format!("external:{uri}"),
        };
        if !link_locations.insert(location_key) {
            report.push(
                ValidationCode::DuplicateLinkModelLocation,
                &location,
                "duplicate LinkModel location",
            );
        }
        validate_location(
            archive,
            &reference.location,
            &location,
            ValidationCode::MissingEmbeddedLinkModel,
            &mut report,
        );
        let distinct_linked_models = reference
            .linked_models
            .iter()
            .map(String::as_str)
            .collect::<HashSet<_>>();
        if distinct_linked_models.len() < 2 {
            report.push(
                ValidationCode::TooFewLinkedModels,
                &location,
                "LinkModel requires at least two LinkedModel references",
            );
        }
        for model_id in &reference.linked_models {
            validate_idref(model_id, &location, &mut report);
            if !model_ids.contains(model_id.as_str()) {
                report.push(
                    ValidationCode::UnknownModelReference,
                    &location,
                    format!("LinkedModel references undeclared model {model_id}"),
                );
            }
        }
    }

    for document in archive.parsed_link_models() {
        let declared = container
            .link_models
            .iter()
            .find(|reference| reference.location.embedded_path() == Some(document.path()))
            .map(|reference| {
                reference
                    .linked_models
                    .iter()
                    .map(String::as_str)
                    .collect::<HashSet<_>>()
            })
            .unwrap_or_default();
        if document.model().links.is_empty() {
            report.push(
                ValidationCode::LinkModelHasNoLinks,
                document.path(),
                "LinkModel requires at least one Link",
            );
        }
        for (link_index, link) in document.model().links.iter().enumerate() {
            let location = format!("{}:Link[{link_index}]", document.path());
            if link.relata.len() < 2 {
                report.push(
                    ValidationCode::LinkHasTooFewRelata,
                    &location,
                    "Link requires at least two Relatum elements",
                );
            }
            for (relatum_index, relatum) in link.relata.iter().enumerate() {
                let relatum_location = format!("{location}/Relatum[{relatum_index}]");
                validate_idref(&relatum.model_id, &relatum_location, &mut report);
                if let Some(format_id) = &relatum.format_id {
                    validate_idref(format_id, &relatum_location, &mut report);
                }
                if let Some(resource_id) = &relatum.resource_id {
                    validate_idref(resource_id, &relatum_location, &mut report);
                }
                if !model_ids.contains(relatum.model_id.as_str()) {
                    report.push(
                        ValidationCode::UnknownModelReference,
                        &relatum_location,
                        format!("Relatum references undeclared model {}", relatum.model_id),
                    );
                    continue;
                }
                if !declared.contains(relatum.model_id.as_str()) {
                    report.push(
                        ValidationCode::UnknownModelReference,
                        &relatum_location,
                        format!(
                            "Relatum model {} is not declared by the containing LinkModel",
                            relatum.model_id
                        ),
                    );
                }
                if let Some(format_id) = &relatum.format_id {
                    if !representations
                        .get(relatum.model_id.as_str())
                        .is_some_and(|ids| ids.contains(format_id.as_str()))
                    {
                        report.push(
                            ValidationCode::UnknownRepresentationReference,
                            &relatum_location,
                            format!(
                                "unknown representation {format_id} in model {}",
                                relatum.model_id
                            ),
                        );
                    }
                    if let Some(resource_id) = &relatum.resource_id {
                        if !resources
                            .get(&(relatum.model_id.as_str(), format_id.as_str()))
                            .is_some_and(|ids| ids.contains(resource_id.as_str()))
                        {
                            report.push(
                                ValidationCode::UnknownResourceReference,
                                &relatum_location,
                                format!(
                                    "unknown resource {resource_id} in representation {format_id}"
                                ),
                            );
                        }
                    }
                } else if relatum.resource_id.is_some() {
                    report.push(
                        ValidationCode::UnknownRepresentationReference,
                        &relatum_location,
                        "resource reference requires a representation reference",
                    );
                }
                for rate in &relatum.rates {
                    validate_idref(&rate.target_model, &relatum_location, &mut report);
                    if !model_ids.contains(rate.target_model.as_str()) {
                        report.push(
                            ValidationCode::RateTargetsUnknownModel,
                            &relatum_location,
                            format!("Rate targets undeclared model {}", rate.target_model),
                        );
                    }
                }
            }
        }
    }

    report.issues.sort_by(|left, right| {
        left.location
            .cmp(&right.location)
            .then_with(|| format!("{:?}", left.code).cmp(&format!("{:?}", right.code)))
            .then_with(|| left.message.cmp(&right.message))
    });
    report
}

fn validate_xml_id<'a>(
    id: &'a str,
    location: &str,
    seen: &mut HashSet<&'a str>,
    report: &mut ValidationReport,
) {
    validate_idref(id, location, report);
    if !seen.insert(id) {
        report.push(
            ValidationCode::DuplicateXmlId,
            location,
            format!("XML ID is not globally unique: {id}"),
        );
    }
}

fn validate_idref(id: &str, location: &str, report: &mut ValidationReport) {
    if !is_ncname(id) {
        report.push(
            ValidationCode::InvalidXmlId,
            location,
            format!("not a valid XML NCName/ID value: {id}"),
        );
    }
}

fn is_ncname(value: &str) -> bool {
    let mut chars = value.chars();
    chars.next().is_some_and(is_ncname_start) && chars.all(is_ncname_char)
}

fn is_ncname_start(ch: char) -> bool {
    matches!(
        ch,
        'A'..='Z'
            | '_'
            | 'a'..='z'
            | '\u{00C0}'..='\u{00D6}'
            | '\u{00D8}'..='\u{00F6}'
            | '\u{00F8}'..='\u{02FF}'
            | '\u{0370}'..='\u{037D}'
            | '\u{037F}'..='\u{1FFF}'
            | '\u{200C}'..='\u{200D}'
            | '\u{2070}'..='\u{218F}'
            | '\u{2C00}'..='\u{2FEF}'
            | '\u{3001}'..='\u{D7FF}'
            | '\u{F900}'..='\u{FDCF}'
            | '\u{FDF0}'..='\u{FFFD}'
            | '\u{10000}'..='\u{EFFFF}'
    )
}

fn is_ncname_char(ch: char) -> bool {
    is_ncname_start(ch)
        || matches!(
            ch,
            '-' | '.' | '0'..='9' | '\u{00B7}' | '\u{0300}'..='\u{036F}' | '\u{203F}'..='\u{2040}'
        )
}

fn validate_location(
    archive: &MmcArchive,
    location: &ResourceLocation,
    issue_location: &str,
    missing_code: ValidationCode,
    report: &mut ValidationReport,
) {
    match location {
        ResourceLocation::Embedded(path) => {
            if validate_archive_path(path).is_err() {
                report.push(
                    ValidationCode::InvalidResourceLocation,
                    issue_location,
                    format!("unsafe embedded location {path}"),
                );
            } else if archive.entry(path).is_none() {
                report.push(
                    missing_code,
                    issue_location,
                    format!("embedded resource is absent: {path}"),
                );
            }
        }
        ResourceLocation::External(uri) => {
            if IriReferenceStr::new(uri).is_err() {
                report.push(
                    ValidationCode::InvalidResourceLocation,
                    issue_location,
                    format!("invalid external IRI: {uri}"),
                );
            }
        }
    }
}
