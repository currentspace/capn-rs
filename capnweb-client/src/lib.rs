//! # Cap'n Web Client Library
//!
//! High-performance Rust client for the Cap'n Web RPC protocol.
//!
//! This crate provides a complete client implementation with support for:
//! - Automatic request batching for optimal network usage
//! - Promise pipelining to minimize round-trips
//! - Type-safe capability references
//! - Connection pooling and retry logic
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use capnweb_client::{Client, ClientConfig};
//! use capnweb_core::CapId;
//! use serde_json::json;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Connect to a Cap'n Web server
//! let client = Client::new_with_url("http://localhost:8080/rpc/batch")?;
//!
//! // Make a simple RPC call
//! let result = client.call(
//!     CapId::new(1),      // Capability ID
//!     "getData",          // Method name
//!     vec![json!({"id": 42})]  // Arguments
//! ).await?;
//!
//! println!("Result: {}", result);
//! # Ok(())
//! # }
//! ```
//!
//! ## Batch Operations
//!
//! Batch multiple operations for efficient network usage:
//!
//! ```rust,no_run
//! use capnweb_client::Client;
//! use capnweb_core::CapId;
//! use serde_json::json;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! # let client = Client::new_with_url("http://localhost:8080/rpc/batch")?;
//! let mut batch = client.batch();
//!
//! // Queue multiple operations
//! let user = batch.call(CapId::new(1), "getUser", vec![json!(123)]);
//! let posts = batch.call(CapId::new(1), "getPosts", vec![json!(123)]);
//!
//! // Execute all at once
//! let results = batch.execute().await?;
//!
//! // Access individual results
//! let user_data = results.get(&user)?;
//! let posts_data = results.get(&posts)?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Promise Pipelining
//!
//! Chain operations on unresolved promises to minimize latency:
//!
//! ```rust,no_run
//! use capnweb_client::Client;
//! use capnweb_core::CapId;
//! use serde_json::json;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! # let client = Client::new_with_url("http://localhost:8080/rpc/batch")?;
//! let mut batch = client.batch();
//!
//! // First call returns a user object
//! let user = batch.call(CapId::new(1), "getUser", vec![json!(123)]);
//!
//! // Pipeline on the result without waiting
//! let profile = batch.pipeline(
//!     &user,                    // Base result
//!     vec!["profile"],          // Path to property
//!     "load",                   // Method to call
//!     vec![]                    // Arguments
//! );
//!
//! let results = batch.execute().await?;
//! # Ok(())
//! # }
//! ```

pub mod client;
#[cfg(feature = "macros")]
pub mod macros;
pub mod recorder;
pub mod stubs;

pub use client::{BatchBuilder, BatchResults, Client, ClientConfig, PendingResult};
pub use recorder::{RecordedPlan, Recorder};
pub use stubs::{Capability, StubError};
