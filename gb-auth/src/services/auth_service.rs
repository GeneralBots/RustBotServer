use crate::{
    models::{LoginRequest, LoginResponse, User},
    Result, AuthError,
};
use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
};
use jsonwebtoken::{encode, EncodingKey, Header};
use sqlx::PgPool;
use std::sync::Arc;

pub struct AuthService {
    db: Arc<PgPool>,
    jwt_secret: String,
    jwt_expiration: i64,
}

impl AuthService {
    pub fn new(db: Arc<PgPool>, jwt_secret: String, jwt_expiration: i64) -> Self {
        Self {
            db,
            jwt_secret,
            jwt_expiration,
        }
    }

    pub async fn login(&self, request: LoginRequest) -> Result<LoginResponse> {
        let user = sqlx::query_as!(
            User,
            "SELECT * FROM users WHERE email = $1",
            request.email
        )
        .fetch_optional(&*self.db)
        .await?
        .ok_or(AuthError::InvalidCredentials)?;

        self.verify_password(&request.password, &user.password_hash)?;

        let token = self.generate_token(&user)?;

        Ok(LoginResponse {
            access_token: token,
            refresh_token: uuid::Uuid::new_v4().to_string(),
            token_type: "Bearer".to_string(),
            expires_in: self.jwt_expiration,
        })
    }

    pub fn hash_password(&self, password: &str) -> Result<String> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        
        argon2
            .hash_password(password.as_bytes(), &salt)
            .map(|hash| hash.to_string())
            .map_err(|e| AuthError::Internal(e.to_string()))
    }

    fn verify_password(&self, password: &str, hash: &str) -> Result<()> {
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|e| AuthError::Internal(e.to_string()))?;

        Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .map_err(|_| AuthError::InvalidCredentials)
    }

    fn generate_token(&self, user: &User) -> Result<String> {
        // Token generation implementation
        Ok("token".to_string())
    }
}
