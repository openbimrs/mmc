use std::collections::{HashMap, HashSet};

use iri_string::types::IriReferenceStr;

use crate::{
    path::validate_archive_path, ApplicationModel, MetadataEntry, MmcArchive, Relatum,
    ResourceLocation,
};

/// Stable machine-readable conformance issue codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ValidationCode {
    UnsupportedFormatVersion,
    TooFewApplicationModels,
    ApplicationModelMissingRepresentation,
    ModelDataMissingResource,
    LinkModelHasNoLinks,
    DuplicateModelId,
    DuplicateRepresentationId,
    DuplicateResourceId,
    DuplicateMetadataKey,

    DuplicateLinkModelLocation,
    InvalidResourceLocation,
    MissingEmbeddedResource,
    MissingEmbeddedLinkModel,
    TooFewLinkedModels,
    UnknownModelReference,
    UnknownRepresentationReference,
    UnknownResourceReference,
    MissingRepresentationDisambiguator,
    MissingResourceDisambiguator,
    LinkHasTooFewRelata,
    LinkHasTooFewModels,
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
    validate_metadata(
        &container.metadata.metadata,
        "MultiModel.xml/MetaData",
        &mut report,
    );
    if container.metadata.format_version != "2.0.0" {
        report.push(
            ValidationCode::UnsupportedFormatVersion,
            "MultiModel.xml@formatVersion",
            "MMC 2.0 namespace requires formatVersion 2.0.0",
        );
    }
    if let Some(domain) = &container.metadata.mm_domain {
        if IriReferenceStr::new(domain).is_err() {
            report.push(
                ValidationCode::InvalidResourceLocation,
                "MultiModel.xml@mmDomain",
                "mmDomain is not a valid IRI reference",
            );
        }
    }
    if container.models.is_empty() {
        report.push(
            ValidationCode::TooFewApplicationModels,
            "MultiModel.xml",
            "MMC 2.0 requires at least one ApplicationModel declaration",
        );
    }

    let mut model_ids = HashSet::new();
    let mut models_by_id = HashMap::new();
    for (model_index, model) in container.models.iter().enumerate() {
        let model_location = format!("ApplicationModel[{model_index}]");
        validate_metadata(&model.metadata, &model_location, &mut report);

        if !model_ids.insert(model.id.as_str()) {
            report.push(
                ValidationCode::DuplicateModelId,
                &model_location,
                format!("duplicate application-model id {}", model.id),
            );
        }
        models_by_id.entry(model.id.as_str()).or_insert(model);
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
            validate_metadata(&data.metadata, &data_location, &mut report);

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
                validate_metadata(&resource.metadata, &resource_location, &mut report);

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
        }
    }

    let mut link_locations = HashSet::new();
    for (reference_index, reference) in container.link_models.iter().enumerate() {
        let location = format!("LinkModel[{reference_index}]");
        validate_metadata(&reference.metadata, &location, &mut report);
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
        validate_metadata(&document.model().metadata, document.path(), &mut report);
        if document.model().format_version != "2.0.0" {
            report.push(
                ValidationCode::UnsupportedFormatVersion,
                document.path(),
                "LinkModel 2.0 namespace requires formatVersion 2.0.0",
            );
        }
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
            validate_metadata(&link.metadata, &location, &mut report);
            if link.relata.len() < 2 {
                report.push(
                    ValidationCode::LinkHasTooFewRelata,
                    &location,
                    "Link requires at least two Relatum elements",
                );
            }
            let distinct_models = link
                .relata
                .iter()
                .map(|relatum| relatum.model_id.as_str())
                .collect::<HashSet<_>>();
            if distinct_models.len() < 2 {
                report.push(
                    ValidationCode::LinkHasTooFewModels,
                    &location,
                    "Link requires relata from at least two application models",
                );
            }
            for (relatum_index, relatum) in link.relata.iter().enumerate() {
                let relatum_location = format!("{location}/Relatum[{relatum_index}]");
                validate_metadata(&relatum.metadata, &relatum_location, &mut report);

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
                if let Some(model) = models_by_id.get(relatum.model_id.as_str()) {
                    validate_relatum_references(model, relatum, &relatum_location, &mut report);
                }
                for rate in &relatum.rates {
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

fn validate_relatum_references(
    model: &ApplicationModel,
    relatum: &Relatum,
    location: &str,
    report: &mut ValidationReport,
) {
    let representation = match &relatum.format_id {
        Some(id) => match model.representations.iter().find(|data| data.id == *id) {
            Some(data) => Some(data),
            None => {
                report.push(
                    ValidationCode::UnknownRepresentationReference,
                    location,
                    format!("unknown representation {id} in model {}", model.id),
                );
                None
            }
        },
        None if model.representations.len() == 1 => model.representations.first(),
        None => {
            report.push(
                ValidationCode::MissingRepresentationDisambiguator,
                location,
                "Relatum requires @f when its application model has multiple representations",
            );
            None
        }
    };
    let Some(representation) = representation else {
        return;
    };
    match &relatum.resource_id {
        Some(id)
            if !representation
                .resources
                .iter()
                .any(|resource| resource.id == *id) =>
        {
            report.push(
                ValidationCode::UnknownResourceReference,
                location,
                format!(
                    "unknown resource {id} in representation {}",
                    representation.id
                ),
            );
        }
        None if representation.resources.len() > 1 => report.push(
            ValidationCode::MissingResourceDisambiguator,
            location,
            "Relatum requires @r when its representation has multiple resources",
        ),
        _ => {}
    }
}

fn validate_metadata(entries: &[MetadataEntry], location: &str, report: &mut ValidationReport) {
    let mut keys = HashSet::new();
    for entry in entries {
        if !keys.insert((entry.category.as_deref(), entry.key.as_str())) {
            report.push(
                ValidationCode::DuplicateMetadataKey,
                location,
                format!("duplicate metadata key {:?}/{}", entry.category, entry.key),
            );
        }
    }
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
