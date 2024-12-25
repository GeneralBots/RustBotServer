#[cfg(test)]
mod tests {
    use gb_auth::services::auth_service::AuthService;
    use gb_auth::models::LoginRequest; 
    use sqlx::PgPool;
use std::sync::Arc;
    use rstest::*;
    
    #[fixture]
    async fn auth_service() -> AuthService {
        let db_pool = PgPool::connect("postgresql://postgres:postgres@localhost:5432/test_db")
            .await
            .expect("Failed to create database connection");
        
        AuthService::new(
            Arc::new(db_pool),
            "test_secret".to_string(),
            3600 
        )
    }

    #[rstest]
    #[tokio::test] 
    async fn test_login_success() -> Result<(), Box<dyn std::error::Error>> {
        let auth_service = auth_service().await;
        let request = LoginRequest {
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
        };

        let result = auth_service.login(request).await;
        assert!(result.is_ok());
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn test_login_invalid_credentials() -> Result<(), Box<dyn std::error::Error>> {
        let auth_service = auth_service().await;
        let request = LoginRequest {
            email: "wrong@example.com".to_string(), 
            password: "wrongpassword".to_string(), 
        };

        let result = auth_service.login(request).await;
        assert!(result.is_err());
        Ok(())
    }
}