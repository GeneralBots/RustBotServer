use axum::{
    http::{Request, Response},
    middleware::Next,
    body::Body,
};
use headers::{Authorization, authorization::Bearer};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Serialize, Deserialize};
use crate::AuthError;

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: i64,
}

pub async fn auth_middleware(
    auth: Authorization<Bearer>,
    request: Request<Body>,
    next: Next,
) -> Result<Response<Body>, AuthError> {
    let token = auth.token();
    let key = DecodingKey::from_secret(b"secret");
    let validation = Validation::default();

    match decode::<Claims>(token, &key, &validation) {
        Ok(_claims) => {
            let response = next.run(request).await;
            Ok(response)
        }
        Err(_) => Err(AuthError::InvalidToken),
    }
}