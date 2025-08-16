            // .service(create_bot)
            // .service(get_bot)
            // .service(list_bots)
            // .service(update_bot)
            // .service(delete_bot)
            // .service(update_bot_status)
            // .service(execute_bot_command)



use crate::services::{config::BotConfig, state::AppState};
use actix_web::{
    delete, get, post, put,
    web::{self, Data, Json, Path},
    HttpResponse, Responder, Result,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgQueryResult, FromRow, PgPool};
use uuid::Uuid;

// 1. Core Data Structures

// 2. Request/Response DTOs
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateBotRequest {
    pub name: String,
    pub initial_config: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateBotRequest {
    pub name: Option<String>,
    pub status: Option<BotStatus>,
    pub config: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BotResponse {
    pub bot_id: Uuid,
    pub name: String,
    pub status: BotStatus,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
}

// 3. Helper Functions
impl From<Bot> for BotResponse {
    fn from(bot: Bot) -> Self {
        BotResponse {
            bot_id: bot.bot_id,
            name: bot.name,
            status: bot.status,
            created_at: bot.created_at,
            updated_at: bot.updated_at,
        }
    }
}

async fn find_bot(bot_id: Uuid, pool: &PgPool) -> Result<Bot, sqlx::Error> {
    sqlx::query_as::<_, Bot>("SELECT * FROM bots WHERE bot_id = $1")
        .bind(bot_id)
        .fetch_one(pool)
        .await
}

// 4. API Endpoints
#[post("/bots/create")]
pub async fn create_bot(
    payload: Json<CreateBotRequest>,
    state: Data<AppState>,
) -> Result<impl Responder, actix_web::Error> {
    let new_bot = sqlx::query_as::<_, Bot>(
        r#"
        INSERT INTO bots (name, status, config)
        VALUES ($1, 'active', $2)
        RETURNING *
        "#,
    )
    .bind(&payload.name)
    .bind(&payload.initial_config)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        log::error!("Failed to create bot: {}", e);
        actix_web::error::ErrorInternalServerError("Failed to create bot")
    })?;

    Ok(HttpResponse::Created().json(BotResponse::from(new_bot)))
}

#[get("/bots/{bot_id}")]
pub async fn get_bot(
    path: Path<Uuid>,
    state: Data<AppState>,
) -> Result<impl Responder, actix_web::Error> {
    let bot_id = path.into_inner();
    let bot = find_bot(bot_id, &state.db).await.map_err(|e| match e {
        sqlx::Error::RowNotFound => actix_web::error::ErrorNotFound("Bot not found"),
        _ => {
            log::error!("Failed to fetch bot: {}", e);
            actix_web::error::ErrorInternalServerError("Failed to fetch bot")
        }
    })?;

    Ok(HttpResponse::Ok().json(BotResponse::from(bot)))
}

#[get("/bots")]
pub async fn list_bots(state: Data<AppState>) -> Result<impl Responder, actix_web::Error> {
    let bots = sqlx::query_as::<_, Bot>("SELECT * FROM bots ORDER BY created_at DESC")
        .fetch_all(&state.db)
        .await
        .map_err(|e| {
            log::error!("Failed to list bots: {}", e);
            actix_web::error::ErrorInternalServerError("Failed to list bots")
        })?;

    let responses: Vec<BotResponse> = bots.into_iter().map(BotResponse::from).collect();
    Ok(HttpResponse::Ok().json(responses))
}

#[put("/bots/{bot_id}")]
pub async fn update_bot(
    path: Path<Uuid>,
    payload: Json<UpdateBotRequest>,
    state: Data<AppState>,
) -> Result<impl Responder, actix_web::Error> {
    let bot_id = path.into_inner();

    let updated_bot = sqlx::query_as::<_, Bot>(
        r#"
        UPDATE bots
        SET
            name = COALESCE($1, name),
            status = COALESCE($2, status),
            config = COALESCE($3, config),
            updated_at = NOW()
        WHERE bot_id = $4
        RETURNING *
        "#,
    )
    .bind(&payload.name)
    .bind(&payload.status)
    .bind(&payload.config)
    .bind(bot_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => actix_web::error::ErrorNotFound("Bot not found"),
        _ => {
            log::error!("Failed to update bot: {}", e);
            actix_web::error::ErrorInternalServerError("Failed to update bot")
        }
    })?;

    Ok(HttpResponse::Ok().json(BotResponse::from(updated_bot)))
}

#[delete("/bots/{bot_id}")]
pub async fn delete_bot(
    path: Path<Uuid>,
    state: Data<AppState>,
) -> Result<impl Responder, actix_web::Error> {
    let bot_id = path.into_inner();

    let result = sqlx::query("DELETE FROM bots WHERE bot_id = $1")
        .bind(bot_id)
        .execute(&state.db)
        .await
        .map_err(|e| {
            log::error!("Failed to delete bot: {}", e);
            actix_web::error::ErrorInternalServerError("Failed to delete bot")
        })?;

    if result.rows_affected() == 0 {
        return Err(actix_web::error::ErrorNotFound("Bot not found"));
    }

    Ok(HttpResponse::NoContent().finish())
}

#[put("/bots/{bot_id}/status")]
pub async fn update_bot_status(
    path: Path<Uuid>,
    new_status: Json<BotStatus>,
    state: Data<AppState>,
) -> Result<impl Responder, actix_web::Error> {
    let bot_id = path.into_inner();

    let updated_bot = sqlx::query_as::<_, Bot>(
        "UPDATE bots SET status = $1, updated_at = NOW() WHERE bot_id = $2 RETURNING *",
    )
    .bind(new_status.into_inner())
    .bind(bot_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => actix_web::error::ErrorNotFound("Bot not found"),
        _ => {
            log::error!("Failed to update bot status: {}", e);
            actix_web::error::ErrorInternalServerError("Failed to update bot status")
        }
    })?;

    Ok(HttpResponse::Ok().json(BotResponse::from(updated_bot)))
}

#[post("/bots/{bot_id}/execute")]
pub async fn execute_bot_command(
    path: Path<Uuid>,
    command: Json<serde_json::Value>,
    state: Data<AppState>,
) -> Result<impl Responder, actix_web::Error> {
    let bot_id = path.into_inner();

    // Verify bot exists
    let _ = find_bot(bot_id, &state.db).await.map_err(|e| match e {
        sqlx::Error::RowNotFound => actix_web::error::ErrorNotFound("Bot not found"),
        _ => {
            log::error!("Failed to fetch bot: {}", e);
            actix_web::error::ErrorInternalServerError("Failed to fetch bot")
        }
    })?;

    // Here you would implement your bot execution logic
    // For now, we'll just echo back the command
    Ok(HttpResponse::Ok().json(json!({
        "bot_id": bot_id,
        "command": command,
        "result": "Command executed successfully (simulated)"
    })))
}
