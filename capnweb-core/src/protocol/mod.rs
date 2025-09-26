// Cap'n Web Protocol Implementation
// This module implements the actual Cap'n Web protocol specification
// as documented at https://github.com/cloudflare/capnweb/blob/main/protocol.md

// Official Cap'n Web wire protocol (primary)
pub mod wire;

// Legacy/internal modules
pub mod capability_registry;
pub mod evaluator;
pub mod expression;
pub mod ids;
pub mod il_runner;
pub mod message;
pub mod nested_capabilities;
pub mod parser;
pub mod pipeline;
pub mod remap_engine;
pub mod resume_tokens;
pub mod session;
pub mod tables;
pub mod variable_state;

#[cfg(test)]
mod tests;

// Primary exports: Official Cap'n Web wire protocol
pub use wire::{parse_wire_batch, serialize_wire_batch, WireExpression, WireMessage};

// Export PropertyKey from wire module specifically to avoid conflicts
pub use wire::PropertyKey as WirePropertyKey;

// Legacy exports (for backward compatibility during transition)
pub use capability_registry::*;
pub use evaluator::*;
pub use expression::PropertyKey as LegacyPropertyKey;
pub use expression::{
    ErrorExpression, ExportExpression, Expression, ImportExpression, PipelineExpression,
    PromiseExpression, RemapExpression,
};
pub use ids::*;
pub use il_runner::*;
pub use message::*;
pub use nested_capabilities::*;
pub use parser::*;
pub use pipeline::*;
pub use remap_engine::*;
pub use resume_tokens::*;
pub use session::*;
pub use tables::*;
pub use variable_state::*;

// Re-export PropertyKey as the primary one (wire format)
pub use wire::PropertyKey;
