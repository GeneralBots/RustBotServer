mod error;
pub mod handlers;
pub mod models;
pub mod services;  // Make services module public
pub mod middleware;

pub use error::AuthError;
pub use handlers::*;
pub use models::*;
pub use services::AuthService;
pub use middleware::*;
