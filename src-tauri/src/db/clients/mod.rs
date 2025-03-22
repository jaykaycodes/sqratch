pub mod common;
pub mod postgres;
pub mod mysql;
pub mod sqlite;

// Re-export common interfaces
pub use common::{DatabaseClient, Transaction};
