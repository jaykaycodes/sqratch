// Define modules in the database module - only visible within this module
pub(self) mod client;
pub(self) mod errors;
pub(self) mod manager;
pub(self) mod postgres;
pub(self) mod types;

// Re-export specific items for use with crate::
pub use errors::*;
pub use manager::{ConnectionManager, ConnectionManagerSafe};
pub use types::*;
