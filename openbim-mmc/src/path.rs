use unicode_normalization::UnicodeNormalization;

use crate::MmcError;

pub(crate) fn validate_archive_path(path: &str) -> Result<(), MmcError> {
    let reason = if path.is_empty() {
        Some("empty path")
    } else if path.starts_with('/') || path.starts_with("//") {
        Some("absolute path")
    } else if path.contains('\\') {
        Some("backslash separator")
    } else if path.ends_with('/') {
        Some("directory or trailing separator")
    } else if path.contains('\0') {
        Some("NUL byte")
    } else if path
        .chars()
        .any(|ch| ch.is_control() || matches!(ch, '<' | '>' | ':' | '"' | '|' | '?' | '*'))
    {
        Some("non-portable or reserved filename character")
    } else {
        let mut reason = None;
        for component in path.split('/') {
            if component.is_empty() {
                reason = Some("repeated separator");
                break;
            }
            if component == "." || component == ".." {
                reason = Some("dot or traversal component");
                break;
            }
            if component.ends_with(['.', ' ']) {
                reason = Some("Windows-trimmed component");
                break;
            }
            let stem = component
                .split('.')
                .next()
                .unwrap_or_default()
                .to_ascii_uppercase();
            if matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
                || (stem.len() == 4
                    && (stem.starts_with("COM") || stem.starts_with("LPT"))
                    && matches!(stem.as_bytes()[3], b'1'..=b'9'))
            {
                reason = Some("reserved Windows device name");
                break;
            }
        }
        reason
    };

    match reason {
        Some(reason) => Err(MmcError::UnsafeArchivePath {
            path: path.to_owned(),
            reason,
        }),
        None => Ok(()),
    }
}

/// Key used only to reject paths that collide on common case-folding or
/// Unicode-normalizing filesystems. The original spelling remains authoritative.
pub(crate) fn collision_key(path: &str) -> String {
    path.nfkc().flat_map(char::to_lowercase).collect()
}

#[cfg(test)]
mod tests {
    use super::{collision_key, validate_archive_path};

    #[test]
    fn rejects_windows_and_unicode_ambiguity() {
        assert!(validate_archive_path("C:/escape").is_err());
        assert_eq!(collision_key("CAFÉ.ifc"), collision_key("cafe\u{301}.ifc"));
    }
}
