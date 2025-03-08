mod error;
pub mod handlers;
pub mod models;
pub mod services;  // Make services module public
pub mod middleware;

pub use error::AuthError;
