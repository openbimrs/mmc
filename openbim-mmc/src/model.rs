use uuid::Uuid;

/// One extensible metadata item from an MMC `MetaData` collection.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MetadataEntry {
    pub key: String,
    pub value: String,
    pub value_type: Option<String>,
    pub category: Option<String>,
}

/// Metadata carried by the MMC `MultiModel` root.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContainerMetadata {
    pub uuid: Uuid,
    pub format_version: String,
    pub mm_domain: String,
    pub metadata: Vec<MetadataEntry>,
}

impl Default for ContainerMetadata {
    fn default() -> Self {
        Self {
            uuid: Uuid::nil(),
            format_version: "2.0.0".to_owned(),
            mm_domain: String::new(),
            metadata: Vec::new(),
        }
    }
}

/// An embedded relative archive path or a container-external absolute IRI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceLocation {
    Embedded(String),
    External(String),
}

impl Default for ResourceLocation {
    fn default() -> Self {
        Self::Embedded(String::new())
    }
}

impl ResourceLocation {
    #[must_use]
    pub fn embedded_path(&self) -> Option<&str> {
        match self {
            Self::Embedded(path) => Some(path),
            Self::External(_) => None,
        }
    }

    #[must_use]
    pub fn external_uri(&self) -> Option<&str> {
        match self {
            Self::External(uri) => Some(uri),
            Self::Embedded(_) => None,
        }
    }
}

/// One physical data resource for a representation of an application model.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DataResource {
    pub id: String,
    pub location: ResourceLocation,
    pub metadata: Vec<MetadataEntry>,
}

/// One data-format representation of an application model.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ModelData {
    pub id: String,
    pub format_type: String,
    pub format_version: Option<String>,
    pub metadata: Vec<MetadataEntry>,
    pub resources: Vec<DataResource>,
}

/// Typed projection of an MMC application-model declaration.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ApplicationModel {
    pub id: String,
    pub model_type: String,
    pub metadata: Vec<MetadataEntry>,
    pub representations: Vec<ModelData>,
}

/// Reference to an external or bundled LinkModel XML document.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LinkModelReference {
    pub location: ResourceLocation,
    pub linked_models: Vec<String>,
    pub metadata: Vec<MetadataEntry>,
}

/// Lossless source plus typed MMC container projection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MultiModel {
    pub metadata: ContainerMetadata,
    pub models: Vec<ApplicationModel>,
    pub link_models: Vec<LinkModelReference>,
    pub(crate) source: Vec<u8>,
}

impl MultiModel {
    #[must_use]
    pub fn source_bytes(&self) -> &[u8] {
        &self.source
    }
}

/// Optional weight or qualification attached to one relatum.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Rate {
    pub rate_type: String,
    pub value: String,
    pub target_model: String,
}

/// One application-model element participating in an n-ary link.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Relatum {
    pub element_id: String,
    pub model_id: String,
    pub format_id: Option<String>,
    pub resource_id: Option<String>,
    pub metadata: Vec<MetadataEntry>,
    pub rates: Vec<Rate>,
}

/// One n-ary relation between elements from at least two application models.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Link {
    pub metadata: Vec<MetadataEntry>,
    pub relata: Vec<Relatum>,
}

/// Typed projection of one MMC 2.0 LinkModel XML document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkModel {
    pub format_version: String,
    pub metadata: Vec<MetadataEntry>,
    pub links: Vec<Link>,
}

impl Default for LinkModel {
    fn default() -> Self {
        Self {
            format_version: "2.0.0".to_owned(),
            metadata: Vec::new(),
            links: Vec::new(),
        }
    }
}

/// A parsed LinkModel tied to its exact archive source entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkModelDocument {
    pub(crate) path: String,
    pub(crate) source: Vec<u8>,
    pub(crate) model: LinkModel,
}

impl LinkModelDocument {
    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }

    #[must_use]
    pub fn source_bytes(&self) -> &[u8] {
        &self.source
    }

    #[must_use]
    pub const fn model(&self) -> &LinkModel {
        &self.model
    }
}
