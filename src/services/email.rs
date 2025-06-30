use actix_web::web;
use actix_web::{http::header::ContentType, HttpResponse};
use jmap_client::{
    client::Client,
    core::query::Filter,
    email::{self, Property},
    mailbox::{self, Role},
};

use crate::services::state::AppState;

#[actix_web::post("/emails/list")]
pub async fn list_emails() -> Result<web::Json<Vec<email::Email>>, actix_web::Error> {
    // 1. Authenticate with JMAP server
    let client = Client::new()
        .credentials(("test@", ""))
        .connect("https://mail/jmap/")
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    let inbox_id = client
        .mailbox_query(
            mailbox::query::Filter::role(Role::Inbox).into(),
            None::<Vec<_>>,
        )
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?
        .take_ids()
        .pop()
        .ok_or_else(|| actix_web::error::ErrorInternalServerError("No inbox found"))?;

    let mut emails = client
        .email_query(
            Filter::and([email::query::Filter::in_mailbox(inbox_id)]).into(),
            [email::query::Comparator::from()].into(),
        )
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    let email_ids = emails.take_ids();
    let mut email_list = Vec::new();

    for email_id in email_ids {
        if let Some(email) = client
            .email_get(
                &email_id,
                [Property::Subject, Property::Preview, Property::Keywords].into(),
            )
            .await
            .map_err(|e| actix_web::error::ErrorInternalServerError(e))?
        {
            email_list.push(email);
        }
    }

    Ok(web::Json(email_list))
}

#[actix_web::get("/campaigns/{campaign_id}/click/{email}")]
pub async fn save_click(
    path: web::Path<(String, String)>,
    state: web::Data<AppState>,
) -> HttpResponse {
    let (campaign_id, email) = path.into_inner();
    let _ = sqlx::query("INSERT INTO clicks (campaign_id, email, updated_at) VALUES ($1, $2, NOW()) ON CONFLICT (campaign_id, email) DO UPDATE SET updated_at = NOW()")
        .bind(campaign_id)
        .bind(email)
        .execute(state.db.as_ref().unwrap())
        .await;

    let pixel = [
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG header
        0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52, // IHDR chunk
        0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, // 1x1 dimension
        0x08, 0x06, 0x00, 0x00, 0x00, 0x1F, 0x15, 0xC4, 0x89, // RGBA
        0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, // IDAT chunk
        0x78, 0x9C, 0x63, 0x00, 0x01, 0x00, 0x00, 0x05, // data
        0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, // CRC
        0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, // IEND chunk
        0xAE, 0x42, 0x60, 0x82,
    ]; // EOF

    // At the end of your save_click function:
    HttpResponse::Ok()
        .content_type(ContentType::png())
        .body(pixel.to_vec()) // Using slicing to pass a reference
}

#[actix_web::get("/campaigns/{campaign_id}/emails")]
pub async fn get_emails(path: web::Path<String>, state: web::Data<AppState>) -> String {
    let campaign_id = path.into_inner();
    let rows = sqlx::query_scalar::<_, String>("SELECT email FROM clicks WHERE campaign_id = $1")
        .bind(campaign_id)
        .fetch_all(state.db.as_ref().unwrap())
        .await
        .unwrap_or_default();
    rows.join(",")
}
