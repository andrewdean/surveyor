//! Outbox relay: events leave the system through here.
pub mod relay;
pub mod store;

pub use relay::Relay;
