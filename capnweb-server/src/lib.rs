// Official Cap'n Web wire protocol server
pub mod wire_server;
pub mod server_wire_handler;

// Legacy servers (TO BE REMOVED - only wire protocol should be used)
pub mod server;
pub mod cap_table;
pub mod runner;
pub mod limits;
pub mod promise_table;
pub mod lifecycle;
pub mod logging;
#[cfg(feature = "all-transports")]
pub mod ws_h1;
#[cfg(feature = "all-transports")]
pub mod ws_wire;
#[cfg(feature = "h3-server")]
pub mod h3_server;
pub mod capnweb_server;
pub mod advanced_capability;

// Primary exports: Official Cap'n Web wire protocol
pub use wire_server::{WireServer, WireServerConfig, WireCapability, RpcTargetAdapter};

// Legacy exports
pub use server::{RpcTarget, ServerConfig, Server};
pub use advanced_capability::{
    AdvancedCapability, AdvancedCapabilityBuilder, AdvancedCapabilityConfig
};
pub use cap_table::CapTable;
pub use runner::PlanRunner;
pub use limits::RateLimits;
pub use promise_table::{PromiseTable, PromiseTableStats};
pub use lifecycle::{CapabilityLifecycle, Disposable, LifecycleStats};
pub use capnweb_server::{CapnWebServer as NewCapnWebServer, CapnWebServerConfig};
pub use logging::{init_logging, init_test_logging};