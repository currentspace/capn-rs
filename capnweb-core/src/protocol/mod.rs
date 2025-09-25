// Cap'n Web Protocol Implementation
// This module implements the actual Cap'n Web protocol specification
// as documented at https://github.com/cloudflare/capnweb/blob/main/protocol.md

// Official Cap'n Web wire protocol (primary)
pub mod wire;

// Legacy/internal modules
pub mod message;
pub mod expression;
pub mod ids;
pub mod parser;
pub mod evaluator;
pub mod tables;
pub mod session;
pub mod pipeline;
pub mod remap_engine;
pub mod variable_state;
pub mod il_runner;
pub mod nested_capabilities;
pub mod resume_tokens;

#[cfg(test)]
mod tests;

// Primary exports: Official Cap'n Web wire protocol
pub use wire::{WireMessage, WireExpression, parse_wire_batch, serialize_wire_batch};

// Export PropertyKey from wire module specifically to avoid conflicts
pub use wire::PropertyKey as WirePropertyKey;

// Legacy exports (for backward compatibility during transition)
pub use message::*;
pub use expression::{Expression, ErrorExpression, ImportExpression, ExportExpression, PromiseExpression, PipelineExpression, RemapExpression};
pub use expression::PropertyKey as LegacyPropertyKey;
pub use ids::*;
pub use parser::*;
pub use evaluator::*;
pub use tables::*;
pub use session::*;
pub use pipeline::*;
pub use remap_engine::*;
pub use variable_state::*;
pub use il_runner::*;
pub use nested_capabilities::*;
pub use resume_tokens::*;

// Re-export PropertyKey as the primary one (wire format)
pub use wire::PropertyKey;