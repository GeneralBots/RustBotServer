use axum::{Json, Extension};
use crate::services::AuthService;
use crate::AuthError;
use crate::models::{LoginRequest, LoginResponse};
use std::sync::Arc;

pub async fn login_handler(
    Extension(auth_service): Extension<Arc<AuthService>>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, AuthError> {
    let response = auth_service.login(request).await?;
    Ok(Json(response))
}