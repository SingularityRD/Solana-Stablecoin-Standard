pub mod health;
pub mod metrics;
pub mod auth;
pub mod stablecoin;
pub mod operations;
pub mod admin;
pub mod roles;
pub mod minters;
pub mod audit;
pub mod users;
pub mod compliance;
pub mod webhooks;
pub mod proofs;

// Re-export health handlers for convenience
pub use health::{handler as health_handler, detailed_handler as health_detail_handler};
pub use health::{readiness_handler as health_ready_handler, liveness_handler as health_live_handler};