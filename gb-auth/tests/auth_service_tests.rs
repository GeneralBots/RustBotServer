#[cfg(test)]
mod tests {
    use crate::services::auth_service::AuthService;
    use crate::models::{LoginRequest, User};
    use sqlx::PgPool;
    use std::sync::Arc;
    use rstest::*;

    async fn setup_test_db() -> PgPool {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:postgres@localhost/gb_auth_test".to_string());
        
        PgPool::connect(&database_url)
            .await
            .expect("Failed to connect to database")
    }

    #[fixture]
    async fn auth_service() -> AuthService {
        let pool = setup_test_db().await;
        AuthService::new(
            Arc::new(pool),
            "test_secret".to_string(),
            3600,
        )
    }

    #[rstest]
    #[tokio::test]
    async fn test_login_success(auth_service: AuthService) {
        let request = LoginRequest {
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
        };

        let result = auth_service.login(request).await;
        assert!(result.is_ok());
    }

    #[rstest]
    #[tokio::test]
    async fn test_login_invalid_credentials(auth_service: AuthService) {
        let request = LoginRequest {
            email: "wrong@example.com".to_string(),
            password: "wrongpassword".to_string(),
        };

        let result = auth_service.login(request).await;
        assert!(result.is_err());
    }
}
