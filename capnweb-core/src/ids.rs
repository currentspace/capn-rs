use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CallId(u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PromiseId(u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CapId(u64);

impl CallId {
    pub fn new(value: u64) -> Self {
        CallId(value)
    }

    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

impl PromiseId {
    pub fn new(value: u64) -> Self {
        PromiseId(value)
    }

    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

impl CapId {
    pub fn new(value: u64) -> Self {
        CapId(value)
    }

    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

impl fmt::Display for CallId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CallId({})", self.0)
    }
}

impl fmt::Display for PromiseId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PromiseId({})", self.0)
    }
}

impl fmt::Display for CapId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CapId({})", self.0)
    }
}

impl From<u64> for CallId {
    fn from(value: u64) -> Self {
        CallId::new(value)
    }
}

impl From<u64> for PromiseId {
    fn from(value: u64) -> Self {
        PromiseId::new(value)
    }
}

impl From<u64> for CapId {
    fn from(value: u64) -> Self {
        CapId::new(value)
    }
}

pub struct CallIdAllocator {
    next: AtomicU64,
}

pub struct PromiseIdAllocator {
    next: AtomicU64,
}

pub struct CapIdAllocator {
    next: AtomicU64,
}

impl CallIdAllocator {
    pub fn new() -> Self {
        CallIdAllocator {
            next: AtomicU64::new(1),
        }
    }

    pub fn allocate(&self) -> CallId {
        let id = self.next.fetch_add(1, Ordering::Relaxed);
        CallId::new(id)
    }

    pub fn peek_next(&self) -> u64 {
        self.next.load(Ordering::Relaxed)
    }
}

impl PromiseIdAllocator {
    pub fn new() -> Self {
        PromiseIdAllocator {
            next: AtomicU64::new(1),
        }
    }

    pub fn allocate(&self) -> PromiseId {
        let id = self.next.fetch_add(1, Ordering::Relaxed);
        PromiseId::new(id)
    }

    pub fn peek_next(&self) -> u64 {
        self.next.load(Ordering::Relaxed)
    }
}

impl CapIdAllocator {
    pub fn new() -> Self {
        CapIdAllocator {
            next: AtomicU64::new(1),
        }
    }

    pub fn allocate(&self) -> CapId {
        let id = self.next.fetch_add(1, Ordering::Relaxed);
        CapId::new(id)
    }

    pub fn peek_next(&self) -> u64 {
        self.next.load(Ordering::Relaxed)
    }
}

impl Default for CallIdAllocator {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for PromiseIdAllocator {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for CapIdAllocator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn test_id_creation_and_conversion() {
        let call_id = CallId::new(42);
        assert_eq!(call_id.as_u64(), 42);
        assert_eq!(format!("{}", call_id), "CallId(42)");

        let promise_id = PromiseId::new(100);
        assert_eq!(promise_id.as_u64(), 100);
        assert_eq!(format!("{}", promise_id), "PromiseId(100)");

        let cap_id = CapId::new(999);
        assert_eq!(cap_id.as_u64(), 999);
        assert_eq!(format!("{}", cap_id), "CapId(999)");
    }

    #[test]
    fn test_id_from_u64() {
        let call_id: CallId = 42u64.into();
        assert_eq!(call_id.as_u64(), 42);

        let promise_id: PromiseId = 100u64.into();
        assert_eq!(promise_id.as_u64(), 100);

        let cap_id: CapId = 999u64.into();
        assert_eq!(cap_id.as_u64(), 999);
    }

    #[test]
    fn test_id_equality_and_hash() {
        let id1 = CallId::new(42);
        let id2 = CallId::new(42);
        let id3 = CallId::new(43);

        assert_eq!(id1, id2);
        assert_ne!(id1, id3);

        let mut set = HashSet::new();
        set.insert(id1);
        assert!(set.contains(&id2));
        assert!(!set.contains(&id3));
    }

    #[test]
    fn test_allocator_monotonic() {
        let allocator = CallIdAllocator::new();

        let id1 = allocator.allocate();
        let id2 = allocator.allocate();
        let id3 = allocator.allocate();

        assert_eq!(id1.as_u64(), 1);
        assert_eq!(id2.as_u64(), 2);
        assert_eq!(id3.as_u64(), 3);
        assert_eq!(allocator.peek_next(), 4);
    }

    #[test]
    fn test_allocator_thread_safety() {
        let allocator = Arc::new(CallIdAllocator::new());
        let mut handles = vec![];
        let num_threads = 10;
        let ids_per_thread = 100;

        for _ in 0..num_threads {
            let alloc = Arc::clone(&allocator);
            let handle = thread::spawn(move || {
                let mut ids = vec![];
                for _ in 0..ids_per_thread {
                    ids.push(alloc.allocate().as_u64());
                }
                ids
            });
            handles.push(handle);
        }

        let mut all_ids = HashSet::new();
        for handle in handles {
            let ids = handle.join().unwrap();
            for id in ids {
                assert!(all_ids.insert(id), "Duplicate ID found: {}", id);
            }
        }

        assert_eq!(all_ids.len(), num_threads * ids_per_thread);
        assert_eq!(allocator.peek_next(), (num_threads * ids_per_thread + 1) as u64);
    }

    #[test]
    fn test_serialization_deserialization() {
        let call_id = CallId::new(42);
        let json = serde_json::to_string(&call_id).unwrap();
        assert_eq!(json, "42");

        let deserialized: CallId = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, call_id);

        let promise_id = PromiseId::new(100);
        let json = serde_json::to_string(&promise_id).unwrap();
        assert_eq!(json, "100");

        let deserialized: PromiseId = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, promise_id);

        let cap_id = CapId::new(999);
        let json = serde_json::to_string(&cap_id).unwrap();
        assert_eq!(json, "999");

        let deserialized: CapId = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, cap_id);
    }

    #[test]
    fn test_multiple_allocators_independence() {
        let call_allocator = CallIdAllocator::new();
        let promise_allocator = PromiseIdAllocator::new();
        let cap_allocator = CapIdAllocator::new();

        let call1 = call_allocator.allocate();
        let promise1 = promise_allocator.allocate();
        let cap1 = cap_allocator.allocate();

        assert_eq!(call1.as_u64(), 1);
        assert_eq!(promise1.as_u64(), 1);
        assert_eq!(cap1.as_u64(), 1);

        let call2 = call_allocator.allocate();
        let promise2 = promise_allocator.allocate();
        let cap2 = cap_allocator.allocate();

        assert_eq!(call2.as_u64(), 2);
        assert_eq!(promise2.as_u64(), 2);
        assert_eq!(cap2.as_u64(), 2);
    }
}