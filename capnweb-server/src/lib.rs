// Official Cap'n Web wire protocol server
pub mod server_wire_handler;
pub mod wire_server;

// Legacy servers (TO BE REMOVED - only wire protocol should be used)
pub mod advanced_capability;
pub mod cap_table;
pub mod capnweb_server;
#[cfg(feature = "h3-server")]
pub mod h3_server;
pub mod lifecycle;
pub mod limits;
pub mod logging;
pub mod promise_table;
pub mod runner;
pub mod server;
#[cfg(feature = "all-transports")]
pub mod ws_h1;
#[cfg(feature = "all-transports")]
pub mod ws_wire;

// Primary exports: Official Cap'n Web wire protocol
pub use wire_server::{RpcTargetAdapter, WireCapability, WireServer, WireServerConfig};

// Legacy exports
pub use advanced_capability::{
    AdvancedCapability, AdvancedCapabilityBuilder, AdvancedCapabilityConfig,
};
pub use cap_table::CapTable;
pub use capnweb_server::{CapnWebServer as NewCapnWebServer, CapnWebServerConfig};
pub use lifecycle::{CapabilityLifecycle, Disposable, LifecycleStats};
pub use limits::RateLimits;
pub use logging::{init_logging, init_test_logging};
pub use promise_table::{PromiseTable, PromiseTableStats};
pub use runner::PlanRunner;
pub use server::{RpcTarget, Server, ServerConfig};
