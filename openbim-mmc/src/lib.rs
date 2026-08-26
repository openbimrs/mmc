//! Safe, lossless DIN 18290-1 / Multi-Model Container 2.0 mechanics.
//!
//! `openbim-mmc` owns the MMC ZIP envelope, `MultiModel.xml`, LinkModel XML,
//! bounded parsing, conformance reporting, safe extraction, and deterministic
//! writing. Application payloads remain opaque. DIN 18290 Parts 2–4 profiles
//! belong in domain crates that consume this crate and canonical GAEB/IFC APIs.

mod archive;
mod builder;
mod error;
mod limits;
mod model;
mod path;
mod validation;
mod xml;

pub use archive::{ArchiveEntry, EntryKind, MmcArchive};
pub use builder::MmcArchiveBuilder;
pub use error::MmcError;
pub use limits::Limits;
pub use model::{
    ApplicationModel, ContainerMetadata, DataResource, Link, LinkModel, LinkModelDocument,
    LinkModelReference, MetadataEntry, ModelData, MultiModel, Rate, Relatum, ResourceLocation,
};
pub use validation::{ValidationCode, ValidationIssue, ValidationReport};

/// Exact MMC 2.0 container namespace from DIN 18290-1's digital schema lineage.
pub const MMC_NAMESPACE: &str = "http://www.buildingsmart.org/multi-model/MMContainer/2.0.0";
/// Exact MMC 2.0 LinkModel namespace.
pub const LINK_MODEL_NAMESPACE: &str = "http://www.buildingsmart.org/multi-model/LinkModel/2.0.0";
