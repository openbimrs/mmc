/// Resource budgets applied before and during ZIP/XML parsing and extraction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Limits {
    pub max_archive_bytes: usize,
    pub max_entries: usize,
    pub max_entry_bytes: usize,
    pub max_total_uncompressed_bytes: usize,
    pub max_xml_bytes: usize,
    pub max_xml_events: usize,
    pub max_models: usize,
    pub max_link_models: usize,
    pub max_links: usize,
    pub max_linked_elements: usize,
    pub max_compression_ratio: usize,
    pub max_extracted_bytes: usize,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            max_archive_bytes: 512 * 1024 * 1024,
            max_entries: 10_000,
            max_entry_bytes: 256 * 1024 * 1024,
            max_total_uncompressed_bytes: 2 * 1024 * 1024 * 1024,
            max_xml_bytes: 32 * 1024 * 1024,
            max_xml_events: 2_000_000,
            max_models: 100_000,
            max_link_models: 100_000,
            max_links: 1_000_000,
            max_linked_elements: 5_000_000,
            max_compression_ratio: 1_000,
            max_extracted_bytes: 2 * 1024 * 1024 * 1024,
        }
    }
}
