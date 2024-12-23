use axum::{
    extract::State,
    Json,
};
use std::sync::Arc;

use crate::{
    models::{LoginRequest, LoginResponse},
    services::auth_service::AuthService,
    Result,
};

pub async fn login(
    State(auth_service): State<Arc<AuthService>>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<LoginResponse>> {
    let response = auth_service.login(request).await?;
    Ok(Json(response))
}

pub async fn logout() -> Result<()> {
    Ok(())
}

pub async fn refresh_token() -> Result<Json<LoginResponse>> {
    todo!()
}