use gb_core::{Result, Error};
use crate::models::{LoginRequest, LoginResponse};
use crate::models::user::{DbUser, UserRole, UserStatus};
use std::sync::Arc;
use sqlx::PgPool;
use argon2::{
    password_hash::{PasswordHash, PasswordHasher, SaltString, PasswordVerifier},
    Argon2,
};
use rand::rngs::OsRng;
use chrono::{DateTime, Utc, Duration};  // Add chrono imports
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Serialize, Deserialize};

pub struct AuthService {
    db: Arc<PgPool>,
    jwt_secret: String,
    jwt_expiration: i64
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
            DbUser,
            r#"
            SELECT 
                id, 
                email, 
                password_hash,
                role as "role!: String",
                status as "status!: String",
                created_at as "created_at!: DateTime<Utc>",
                updated_at as "updated_at!: DateTime<Utc>"
            FROM users 
            WHERE email = $1
            "#,
            request.email
        )
        .fetch_optional(&*self.db)
        .await
        .map_err(|e| Error::internal(e.to_string()))?
        .ok_or_else(|| Error::internal("Invalid credentials"))?;

        // Convert the string fields to their respective enum types
        let user = DbUser {
            id: user.id,
            email: user.email,
            password_hash: user.password_hash,
            role: UserRole::from(user.role),
            status: UserStatus::from(user.status),
            created_at: user.created_at,
            updated_at: user.updated_at,
        };

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
            .map_err(|e| Error::internal(e.to_string()))
    }

    fn verify_password(&self, password: &str, hash: &str) -> Result<()> {
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|e| Error::internal(e.to_string()))?;

        Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .map_err(|_| Error::internal("Invalid credentials"))
    }

    fn generate_token(&self, user: &DbUser) -> Result<String> {
        #[derive(Debug, Serialize, Deserialize)]
        struct Claims {
            sub: String,
            exp: i64,
            iat: i64,
        }

        let now = Utc::now();
        let exp = now + Duration::seconds(self.jwt_expiration);

        let claims = Claims {
            sub: user.id.to_string(),
            exp: exp.timestamp(),
            iat: now.timestamp(),
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_bytes()),
        )
        .map_err(|e| Error::internal(e.to_string()))
    }
}