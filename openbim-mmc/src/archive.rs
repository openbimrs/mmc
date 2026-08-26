use std::{
    collections::{BTreeMap, HashMap},
    fs::{self, OpenOptions},
    io::{Cursor, Read, Write},
    path::{Path, PathBuf},
};

use zip::{write::SimpleFileOptions, CompressionMethod, ZipArchive, ZipWriter};

use crate::{
    path::{collision_key, validate_archive_path},
    validation, xml, Limits, LinkModelDocument, MmcError, MultiModel, ValidationReport,
};

/// Functional role of one physical ZIP entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryKind {
    Metadata,
    LinkModel,
    Payload,
}

/// One bounded, owned physical archive entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArchiveEntry {
    path: String,
    bytes: Vec<u8>,
    kind: EntryKind,
}

impl ArchiveEntry {
    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }

    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    #[must_use]
    pub const fn kind(&self) -> EntryKind {
        self.kind
    }
}

/// Fully owned, lossless MMC archive and typed MMC 2.0 projections.
#[derive(Debug, Clone)]
pub struct MmcArchive {
    original: Vec<u8>,
    entries: Vec<ArchiveEntry>,
    entry_index: BTreeMap<String, usize>,
    container: MultiModel,
    link_models: Vec<LinkModelDocument>,
    limits: Limits,
}

impl MmcArchive {
    /// Parse an MMC archive with conservative production budgets.
    pub fn parse(bytes: impl AsRef<[u8]>) -> Result<Self, MmcError> {
        Self::parse_with_limits(bytes, Limits::default())
    }

    /// Parse an MMC archive with explicit caller-provided budgets.
    pub fn parse_with_limits(bytes: impl AsRef<[u8]>, limits: Limits) -> Result<Self, MmcError> {
        let bytes = bytes.as_ref();
        if bytes.len() > limits.max_archive_bytes {
            return Err(MmcError::LimitExceeded {
                resource: "archive bytes",
                actual: bytes.len() as u64,
                maximum: limits.max_archive_bytes as u64,
            });
        }
        let mut zip = ZipArchive::new(Cursor::new(bytes))?;
        if zip.len() > limits.max_entries {
            return Err(MmcError::LimitExceeded {
                resource: "ZIP entries",
                actual: zip.len() as u64,
                maximum: limits.max_entries as u64,
            });
        }

        let mut entries = Vec::with_capacity(zip.len());
        let mut normalized = HashMap::<String, String>::new();
        let mut total = 0u64;
        for index in 0..zip.len() {
            let mut file = zip.by_index(index)?;
            let raw_name = file.name_raw();
            let name = std::str::from_utf8(raw_name)
                .map_err(|_| MmcError::UnsafeArchivePath {
                    path: String::from_utf8_lossy(raw_name).into_owned(),
                    reason: "entry name is not UTF-8",
                })?
                .to_owned();
            validate_archive_path(&name)?;
            if file.is_dir() {
                return Err(MmcError::UnsafeArchivePath {
                    path: name,
                    reason: "directory entries are not permitted",
                });
            }
            if let Some(mode) = file.unix_mode() {
                let file_type = mode & 0o170000;
                if file_type != 0 && file_type != 0o100000 {
                    return Err(MmcError::UnsafeArchivePath {
                        path: name,
                        reason: "non-regular ZIP entry",
                    });
                }
            }
            let key = collision_key(&name);
            if let Some(first) = normalized.insert(key, name.clone()) {
                return Err(MmcError::DuplicateArchivePath {
                    first,
                    second: name,
                });
            }

            let declared = file.size();
            if declared > limits.max_entry_bytes as u64 {
                return Err(MmcError::LimitExceeded {
                    resource: "entry bytes",
                    actual: declared,
                    maximum: limits.max_entry_bytes as u64,
                });
            }
            total = total.checked_add(declared).ok_or(MmcError::LimitExceeded {
                resource: "total uncompressed bytes",
                actual: u64::MAX,
                maximum: limits.max_total_uncompressed_bytes as u64,
            })?;
            if total > limits.max_total_uncompressed_bytes as u64 {
                return Err(MmcError::LimitExceeded {
                    resource: "total uncompressed bytes",
                    actual: total,
                    maximum: limits.max_total_uncompressed_bytes as u64,
                });
            }
            let compressed = file.compressed_size();
            if declared > 0
                && (compressed == 0
                    || declared / compressed.max(1) > limits.max_compression_ratio as u64)
            {
                return Err(MmcError::LimitExceeded {
                    resource: "compression ratio",
                    actual: declared / compressed.max(1),
                    maximum: limits.max_compression_ratio as u64,
                });
            }

            let mut payload = Vec::with_capacity(declared as usize);
            file.by_ref()
                .take(limits.max_entry_bytes as u64 + 1)
                .read_to_end(&mut payload)?;
            if payload.len() > limits.max_entry_bytes {
                return Err(MmcError::LimitExceeded {
                    resource: "entry bytes",
                    actual: payload.len() as u64,
                    maximum: limits.max_entry_bytes as u64,
                });
            }
            entries.push(ArchiveEntry {
                path: name,
                bytes: payload,
                kind: EntryKind::Payload,
            });
        }

        let roots = entries
            .iter()
            .enumerate()
            .filter(|(_, entry)| entry.path == "MultiModel.xml")
            .map(|(index, _)| index)
            .collect::<Vec<_>>();
        if roots.len() != 1 {
            return Err(MmcError::MissingRoot);
        }
        let root_index = roots[0];
        entries[root_index].kind = EntryKind::Metadata;
        let container = xml::parse_multimodel(&entries[root_index].bytes, limits)?;

        let mut entry_index = BTreeMap::new();
        for (index, entry) in entries.iter().enumerate() {
            entry_index.insert(entry.path.clone(), index);
        }
        let mut parsed_paths = BTreeMap::<String, usize>::new();
        let mut link_models = Vec::new();
        for reference in &container.link_models {
            let Some(path) = reference.location.embedded_path() else {
                continue;
            };
            if validate_archive_path(path).is_err() || parsed_paths.contains_key(path) {
                continue;
            }
            let Some(&index) = entry_index.get(path) else {
                continue;
            };
            let model = xml::parse_link_model(path, &entries[index].bytes, limits)?;
            entries[index].kind = EntryKind::LinkModel;
            parsed_paths.insert(path.to_owned(), link_models.len());
            link_models.push(LinkModelDocument {
                path: path.to_owned(),
                source: entries[index].bytes.clone(),
                model,
            });
        }

        Ok(Self {
            original: bytes.to_vec(),
            entries,
            entry_index,
            container,
            link_models,
            limits,
        })
    }

    /// Read an archive from a stream while enforcing the compressed-byte budget.
    pub fn read_from(mut reader: impl Read) -> Result<Self, MmcError> {
        Self::read_from_with_limits(&mut reader, Limits::default())
    }

    pub fn read_from_with_limits(mut reader: impl Read, limits: Limits) -> Result<Self, MmcError> {
        let mut bytes = Vec::new();
        reader
            .by_ref()
            .take(limits.max_archive_bytes as u64 + 1)
            .read_to_end(&mut bytes)?;
        Self::parse_with_limits(bytes, limits)
    }

    #[must_use]
    pub fn original_bytes(&self) -> &[u8] {
        &self.original
    }

    #[must_use]
    pub const fn container(&self) -> &MultiModel {
        &self.container
    }

    #[must_use]
    pub fn entries(&self) -> &[ArchiveEntry] {
        &self.entries
    }

    #[must_use]
    pub fn entry(&self, path: &str) -> Option<&ArchiveEntry> {
        self.entry_index
            .get(path)
            .map(|index| &self.entries[*index])
    }

    #[must_use]
    pub fn parsed_link_models(&self) -> &[LinkModelDocument] {
        &self.link_models
    }

    #[must_use]
    pub fn validate(&self) -> ValidationReport {
        validation::validate(self)
    }

    /// Return a deterministic ZIP representation without altering any entry bytes.
    pub fn to_deterministic_bytes(&self) -> Result<Vec<u8>, MmcError> {
        let mut output = Cursor::new(Vec::new());
        {
            let mut writer = ZipWriter::new(&mut output);
            let options = SimpleFileOptions::default()
                .compression_method(CompressionMethod::Deflated)
                .unix_permissions(0o100644);
            let mut entries = self.entries.iter().collect::<Vec<_>>();
            entries.sort_by(|left, right| left.path.cmp(&right.path));
            for entry in entries {
                writer.start_file(&entry.path, options)?;
                writer.write_all(&entry.bytes)?;
            }
            writer.finish()?;
        }
        Ok(output.into_inner())
    }

    /// Safely extract entries with new-file semantics and no pre-existing symlink traversal.
    ///
    /// The caller must prevent concurrent mutation of the destination tree while extraction runs.
    pub fn extract_to(&self, root: impl AsRef<Path>) -> Result<(), MmcError> {
        let root = root.as_ref();
        let total = self
            .entries
            .iter()
            .try_fold(0u64, |sum, entry| sum.checked_add(entry.bytes.len() as u64))
            .ok_or(MmcError::LimitExceeded {
                resource: "extracted bytes",
                actual: u64::MAX,
                maximum: self.limits.max_extracted_bytes as u64,
            })?;
        if total > self.limits.max_extracted_bytes as u64 {
            return Err(MmcError::LimitExceeded {
                resource: "extracted bytes",
                actual: total,
                maximum: self.limits.max_extracted_bytes as u64,
            });
        }
        reject_symlink_ancestors(root)?;
        if root.exists() {
            let metadata = fs::symlink_metadata(root)?;
            if metadata.file_type().is_symlink() || !metadata.is_dir() {
                return Err(unsafe_extract(root, "root is not a real directory"));
            }
        } else {
            fs::create_dir(root)?;
        }

        for entry in &self.entries {
            let target = root.join(&entry.path);
            if target.exists() {
                return Err(unsafe_extract(&target, "target already exists"));
            }
            reject_existing_symlinks(root, &entry.path)?;
        }

        for entry in &self.entries {
            let target = root.join(&entry.path);
            let parent = target
                .parent()
                .expect("validated archive path has a parent");
            create_directories_without_symlinks(root, parent)?;
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&target)?;
            file.write_all(&entry.bytes)?;
            file.sync_all()?;
        }
        Ok(())
    }
}

fn reject_symlink_ancestors(path: &Path) -> Result<(), MmcError> {
    for ancestor in path.ancestors() {
        match fs::symlink_metadata(ancestor) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(unsafe_extract(ancestor, "symlink ancestor is prohibited"));
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
    }
    Ok(())
}

fn reject_existing_symlinks(root: &Path, relative: &str) -> Result<(), MmcError> {
    let mut current = PathBuf::from(root);
    for component in relative.split('/').take(relative.split('/').count() - 1) {
        current.push(component);
        match fs::symlink_metadata(&current) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(unsafe_extract(&current, "symlink ancestor is prohibited"));
            }
            Ok(metadata) if !metadata.is_dir() => {
                return Err(unsafe_extract(&current, "ancestor is not a directory"));
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
    }
    Ok(())
}

fn create_directories_without_symlinks(root: &Path, parent: &Path) -> Result<(), MmcError> {
    let relative = parent
        .strip_prefix(root)
        .map_err(|_| unsafe_extract(parent, "path escaped extraction root"))?;
    let mut current = root.to_path_buf();
    for component in relative.components() {
        current.push(component);
        match fs::create_dir(&current) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                let metadata = fs::symlink_metadata(&current)?;
                if metadata.file_type().is_symlink() || !metadata.is_dir() {
                    return Err(unsafe_extract(&current, "unsafe directory ancestor"));
                }
            }
            Err(error) => return Err(error.into()),
        }
    }
    Ok(())
}

fn unsafe_extract(path: &Path, reason: &'static str) -> MmcError {
    MmcError::UnsafeExtractionPath {
        path: path.display().to_string(),
        reason,
    }
}
