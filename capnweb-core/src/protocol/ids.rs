use serde::{Deserialize, Serialize};
use std::fmt;

/// Import ID - represents an entry in the import table
/// Positive IDs (1, 2, 3...) are chosen by the importing side
/// Negative IDs (-1, -2, -3...) are chosen by the exporting side
/// ID 0 is reserved for the "main" interface
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ImportId(pub i64);

impl ImportId {
    /// Create a new import ID for the main interface
    pub fn main() -> Self {
        ImportId(0)
    }

    /// Check if this is the main interface ID
    pub fn is_main(&self) -> bool {
        self.0 == 0
    }

    /// Check if this ID was allocated locally (positive)
    pub fn is_local(&self) -> bool {
        self.0 > 0
    }

    /// Check if this ID was allocated remotely (negative)
    pub fn is_remote(&self) -> bool {
        self.0 < 0
    }

    /// Convert to the corresponding export ID on the other side
    pub fn to_export_id(&self) -> ExportId {
        ExportId(-self.0)
    }
}

impl fmt::Display for ImportId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Import#{}", self.0)
    }
}

/// Export ID - represents an entry in the export table
/// Negative IDs (-1, -2, -3...) are chosen by the exporting side
/// Positive IDs (1, 2, 3...) are chosen by the importing side
/// ID 0 is reserved for the "main" interface
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ExportId(pub i64);

impl ExportId {
    /// Create a new export ID for the main interface
    pub fn main() -> Self {
        ExportId(0)
    }

    /// Check if this is the main interface ID
    pub fn is_main(&self) -> bool {
        self.0 == 0
    }

    /// Check if this ID was allocated locally (negative)
    pub fn is_local(&self) -> bool {
        self.0 < 0
    }

    /// Check if this ID was allocated remotely (positive)
    pub fn is_remote(&self) -> bool {
        self.0 > 0
    }

    /// Convert to the corresponding import ID on the other side
    pub fn to_import_id(&self) -> ImportId {
        ImportId(-self.0)
    }
}

impl fmt::Display for ExportId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Export#{}", self.0)
    }
}

/// ID allocator for managing import and export IDs
#[derive(Debug)]
pub struct IdAllocator {
    next_positive: std::sync::atomic::AtomicI64,
    next_negative: std::sync::atomic::AtomicI64,
}

impl IdAllocator {
    /// Create a new ID allocator
    pub fn new() -> Self {
        Self {
            next_positive: std::sync::atomic::AtomicI64::new(1),
            next_negative: std::sync::atomic::AtomicI64::new(-1),
        }
    }

    /// Allocate a new local import ID (positive)
    pub fn allocate_import(&self) -> ImportId {
        let id = self
            .next_positive
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        ImportId(id)
    }

    /// Allocate a new local export ID (negative)
    pub fn allocate_export(&self) -> ExportId {
        let id = self
            .next_negative
            .fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
        ExportId(id)
    }

    /// Register a remote import ID (negative)
    pub fn register_remote_import(&self, id: i64) -> ImportId {
        // Remote imports are negative from our perspective
        ImportId(id)
    }

    /// Register a remote export ID (positive)
    pub fn register_remote_export(&self, id: i64) -> ExportId {
        // Remote exports are positive from our perspective
        ExportId(id)
    }
}

impl Default for IdAllocator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_main_ids() {
        let import = ImportId::main();
        let export = ExportId::main();

        assert!(import.is_main());
        assert!(export.is_main());
        assert_eq!(import.0, 0);
        assert_eq!(export.0, 0);
    }

    #[test]
    fn test_local_remote_detection() {
        let local_import = ImportId(5);
        let remote_import = ImportId(-3);
        let local_export = ExportId(-2);
        let remote_export = ExportId(4);

        assert!(local_import.is_local());
        assert!(!local_import.is_remote());

        assert!(!remote_import.is_local());
        assert!(remote_import.is_remote());

        assert!(local_export.is_local());
        assert!(!local_export.is_remote());

        assert!(!remote_export.is_local());
        assert!(remote_export.is_remote());
    }

    #[test]
    fn test_id_conversion() {
        let import = ImportId(5);
        let export = import.to_export_id();
        assert_eq!(export, ExportId(-5));

        let import2 = export.to_import_id();
        assert_eq!(import2, ImportId(5));
    }

    #[test]
    fn test_id_allocator() {
        let allocator = IdAllocator::new();

        // Test import allocation (positive)
        let import1 = allocator.allocate_import();
        let import2 = allocator.allocate_import();
        assert_eq!(import1, ImportId(1));
        assert_eq!(import2, ImportId(2));

        // Test export allocation (negative)
        let export1 = allocator.allocate_export();
        let export2 = allocator.allocate_export();
        assert_eq!(export1, ExportId(-1));
        assert_eq!(export2, ExportId(-2));

        // Test remote registration
        let remote_import = allocator.register_remote_import(-5);
        let remote_export = allocator.register_remote_export(7);
        assert_eq!(remote_import, ImportId(-5));
        assert_eq!(remote_export, ExportId(7));
    }

    #[test]
    fn test_display() {
        let import = ImportId(42);
        let export = ExportId(-17);

        assert_eq!(format!("{}", import), "Import#42");
        assert_eq!(format!("{}", export), "Export#-17");
    }
}
