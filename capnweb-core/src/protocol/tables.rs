use dashmap::DashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
// use tokio::sync::oneshot; // TODO: Remove when promise handling is implemented

use super::ids::{ImportId, ExportId, IdAllocator};
// use super::expression::Expression; // TODO: Remove when expression integration is complete
use crate::RpcTarget;

/// Value that can be stored in tables
#[derive(Debug, Clone)]
pub enum Value {
    Null,
    Bool(bool),
    Number(serde_json::Number),
    String(String),
    Array(Vec<Value>),
    Object(std::collections::HashMap<String, Box<Value>>),
    Date(f64),
    Error(String, String, Option<String>), // type, message, stack
    Stub(StubReference),
    Promise(PromiseReference),
}

/// Reference to a stub (since Arc<dyn RpcTarget> can't implement Clone/Debug directly)
#[derive(Debug, Clone)]
pub struct StubReference {
    pub id: String,
    #[allow(dead_code)]
    stub: Arc<dyn RpcTarget>,
}

impl StubReference {
    pub fn new(stub: Arc<dyn RpcTarget>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            stub,
        }
    }

    pub fn get(&self) -> Arc<dyn RpcTarget> {
        self.stub.clone()
    }
}

/// Reference to a promise
#[derive(Debug, Clone)]
pub struct PromiseReference {
    pub id: String,
}

/// State of a promise
#[derive(Debug)]
pub enum PromiseState {
    Pending(tokio::sync::watch::Receiver<Option<Result<Value, Value>>>),
    Resolved(Value),
    Rejected(Value),
}

/// Entry in the import table
#[derive(Debug)]
pub struct ImportEntry {
    pub value: ImportValue,
    pub refcount: AtomicU32,
}

/// Value stored in an import
#[derive(Debug, Clone)]
pub enum ImportValue {
    Stub(StubReference),
    Promise(PromiseReference),
    Value(Value),
}

/// Entry in the export table
#[derive(Debug)]
pub struct ExportEntry {
    pub value: ExportValue,
    pub export_count: AtomicU32,
}

/// Value stored in an export
#[derive(Debug)]
pub enum ExportValue {
    Stub(StubReference),
    Promise(Arc<tokio::sync::Mutex<Option<tokio::sync::watch::Sender<Option<Result<Value, Value>>>>>>),
    Resolved(Value),
    Rejected(Value),
}

/// Reference to export value (for returning from get())
#[derive(Debug)]
pub enum ExportValueRef {
    Stub(StubReference),
    Promise(Arc<tokio::sync::Mutex<Option<tokio::sync::watch::Sender<Option<Result<Value, Value>>>>>>),
    Resolved(Value),
    Rejected(Value),
}

/// Import table manages imported capabilities and promises
pub struct ImportTable {
    allocator: Arc<IdAllocator>,
    entries: DashMap<ImportId, ImportEntry>,
}

impl ImportTable {
    /// Create a new import table
    pub fn new(allocator: Arc<IdAllocator>) -> Self {
        Self {
            allocator,
            entries: DashMap::new(),
        }
    }

    /// Allocate a new local import ID
    pub fn allocate_local(&self) -> ImportId {
        self.allocator.allocate_import()
    }

    /// Insert a new import entry
    pub fn insert(&self, id: ImportId, value: ImportValue) -> Result<(), TableError> {
        let entry = ImportEntry {
            value,
            refcount: AtomicU32::new(1),
        };

        if self.entries.insert(id, entry).is_some() {
            return Err(TableError::DuplicateImport(id));
        }

        Ok(())
    }

    /// Get an import entry
    pub fn get(&self, id: ImportId) -> Option<ImportValue> {
        self.entries.get(&id).map(|entry| match &entry.value {
            ImportValue::Stub(stub) => ImportValue::Stub(stub.clone()),
            ImportValue::Promise(promise) => ImportValue::Promise(promise.clone()),
            ImportValue::Value(val) => ImportValue::Value(val.clone()),
        })
    }

    /// Increment the refcount for an import
    pub fn add_ref(&self, id: ImportId) -> Result<(), TableError> {
        self.entries
            .get(&id)
            .map(|entry| {
                entry.refcount.fetch_add(1, Ordering::SeqCst);
            })
            .ok_or(TableError::UnknownImport(id))
    }

    /// Release an import with the given refcount
    pub fn release(&self, id: ImportId, refcount: u32) -> Result<bool, TableError> {
        let mut should_remove = false;

        self.entries.alter(&id, |_key, entry| {
            let current = entry.refcount.load(Ordering::SeqCst);
            if current >= refcount {
                let new_count = current - refcount;
                entry.refcount.store(new_count, Ordering::SeqCst);
                if new_count == 0 {
                    should_remove = true;
                }
            }
            entry
        });

        if should_remove {
            self.entries.remove(&id);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Update a promise import to resolved state
    pub fn resolve_promise(&self, id: ImportId, value: Value) -> Result<(), TableError> {
        self.entries.alter(&id, |_key, mut entry| {
            match &mut entry.value {
                ImportValue::Promise(_promise) => {
                    // Update to resolved value
                    entry.value = ImportValue::Value(value);
                }
                _ => {}
            }
            entry
        });
        Ok(())
    }
}

/// Export table manages exported capabilities and promises
pub struct ExportTable {
    allocator: Arc<IdAllocator>,
    entries: DashMap<ExportId, ExportEntry>,
}

impl ExportTable {
    /// Create a new export table
    pub fn new(allocator: Arc<IdAllocator>) -> Self {
        Self {
            allocator,
            entries: DashMap::new(),
        }
    }

    /// Allocate a new local export ID
    pub fn allocate_local(&self) -> ExportId {
        self.allocator.allocate_export()
    }

    /// Insert a new export entry
    pub fn insert(&self, id: ExportId, value: ExportValue) -> Result<(), TableError> {
        let entry = ExportEntry {
            value,
            export_count: AtomicU32::new(1),
        };

        if self.entries.insert(id, entry).is_some() {
            return Err(TableError::DuplicateExport(id));
        }

        Ok(())
    }

    /// Export a stub
    pub fn export_stub(&self, stub: Arc<dyn RpcTarget>) -> ExportId {
        let id = self.allocate_local();
        let stub_ref = StubReference::new(stub);
        let _ = self.insert(id, ExportValue::Stub(stub_ref));
        id
    }

    /// Export a new promise
    pub fn export_promise(&self) -> (ExportId, tokio::sync::watch::Receiver<Option<Result<Value, Value>>>) {
        let id = self.allocate_local();
        let (tx, rx) = tokio::sync::watch::channel(None);
        let _ = self.insert(id, ExportValue::Promise(Arc::new(tokio::sync::Mutex::new(Some(tx)))));
        (id, rx)
    }

    /// Get an export entry (returns clone for stub/value types)
    pub fn get(&self, id: ExportId) -> Option<ExportValueRef> {
        self.entries.get(&id).map(|entry| match &entry.value {
            ExportValue::Stub(stub) => ExportValueRef::Stub(stub.clone()),
            ExportValue::Promise(promise) => ExportValueRef::Promise(promise.clone()),
            ExportValue::Resolved(val) => ExportValueRef::Resolved(val.clone()),
            ExportValue::Rejected(val) => ExportValueRef::Rejected(val.clone()),
        })
    }

    /// Resolve an exported promise
    pub async fn resolve(&self, id: ExportId, value: Value) -> Result<(), TableError> {
        if let Some(mut entry) = self.entries.get_mut(&id) {
            match &entry.value {
                ExportValue::Promise(promise_sender) => {
                    // Get the sender and send resolution
                    if let Some(sender) = promise_sender.lock().await.take() {
                        let _ = sender.send(Some(Ok(value.clone())));
                    }
                    // Update entry to resolved state
                    entry.value = ExportValue::Resolved(value);
                }
                _ => {
                    // Already resolved or not a promise
                }
            }
        }
        Ok(())
    }

    /// Reject an exported promise
    pub async fn reject(&self, id: ExportId, error: Value) -> Result<(), TableError> {
        if let Some(mut entry) = self.entries.get_mut(&id) {
            match &entry.value {
                ExportValue::Promise(promise_sender) => {
                    // Get the sender and send rejection
                    if let Some(sender) = promise_sender.lock().await.take() {
                        let _ = sender.send(Some(Err(error.clone())));
                    }
                    // Update entry to rejected state
                    entry.value = ExportValue::Rejected(error);
                }
                _ => {
                    // Already resolved or not a promise
                }
            }
        }
        Ok(())
    }

    /// Increment the export count
    pub fn add_export(&self, id: ExportId) -> Result<(), TableError> {
        self.entries
            .get(&id)
            .map(|entry| {
                entry.export_count.fetch_add(1, Ordering::SeqCst);
            })
            .ok_or(TableError::UnknownExport(id))
    }

    /// Release an export
    pub fn release(&self, id: ExportId) -> Result<bool, TableError> {
        let mut should_remove = false;

        self.entries.alter(&id, |_key, entry| {
            let current = entry.export_count.load(Ordering::SeqCst);
            if current > 0 {
                let new_count = current - 1;
                entry.export_count.store(new_count, Ordering::SeqCst);
                if new_count == 0 {
                    should_remove = true;
                }
            }
            entry
        });

        if should_remove {
            self.entries.remove(&id);
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

/// Error type for table operations
#[derive(Debug, thiserror::Error)]
pub enum TableError {
    #[error("Duplicate import ID: {0}")]
    DuplicateImport(ImportId),

    #[error("Duplicate export ID: {0}")]
    DuplicateExport(ExportId),

    #[error("Unknown import ID: {0}")]
    UnknownImport(ImportId),

    #[error("Unknown export ID: {0}")]
    UnknownExport(ExportId),

    #[error("Cannot resolve non-promise export")]
    NotAPromise,

    #[error("Export already resolved")]
    AlreadyResolved,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_import_table() {
        let allocator = Arc::new(IdAllocator::new());
        let table = ImportTable::new(allocator.clone());

        // Test insertion and retrieval
        let id = table.allocate_local();
        assert_eq!(id, ImportId(1));

        let stub = Arc::new(crate::MockRpcTarget::new());
        let stub_ref = StubReference::new(stub);
        table.insert(id, ImportValue::Stub(stub_ref)).unwrap();

        match table.get(id).unwrap() {
            ImportValue::Stub(_) => {},
            _ => panic!("Expected stub"),
        }

        // Test refcounting
        table.add_ref(id).unwrap();
        assert!(!table.release(id, 1).unwrap()); // Should not remove yet
        assert!(table.release(id, 1).unwrap()); // Should remove now
        assert!(table.get(id).is_none());
    }

    #[tokio::test]
    async fn test_export_table() {
        let allocator = Arc::new(IdAllocator::new());
        let table = ExportTable::new(allocator.clone());

        // Test promise export and resolution
        let (id, mut rx) = table.export_promise();
        assert_eq!(id, ExportId(-1));

        // Resolve the promise
        table.resolve(id, Value::String("result".to_string())).await.unwrap();

        // Check that watchers receive the resolution
        rx.changed().await.unwrap();
        match rx.borrow().as_ref().unwrap() {
            Ok(Value::String(s)) => assert_eq!(s, "result"),
            _ => panic!("Expected resolved string"),
        }

        // Check that the export is now resolved
        match table.get(id).unwrap() {
            ExportValueRef::Resolved(Value::String(s)) => assert_eq!(s, "result"),
            _ => panic!("Expected resolved export"),
        }
    }

    #[test]
    fn test_stub_export() {
        let allocator = Arc::new(IdAllocator::new());
        let table = ExportTable::new(allocator.clone());

        let stub = Arc::new(crate::MockRpcTarget::new());
        let id = table.export_stub(stub.clone());

        match table.get(id).unwrap() {
            ExportValueRef::Stub(_) => {},
            _ => panic!("Expected stub export"),
        }
    }
}