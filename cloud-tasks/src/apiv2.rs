pub mod client;
pub use client::Client;

pub mod conn_pool;
pub use conn_pool::ConnectionManager;

pub const CLOUD_TASKS: &str = "cloudtasks.googleapis.com";
