// Database connection module
pub mod clients;
pub mod errors;
pub mod manager;
pub mod types;
pub mod utils;

// Re-export common types
pub use types::{
    ConnectionInfo, QueryResult
};
