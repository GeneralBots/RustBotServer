use actix_web::web;
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
            Filter::and([email::query::Filter::in_mailbox(inbox_id)])
                .into(),
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
) -> String {
    let (campaign_id, email) = path.into_inner();
    let _ = sqlx::query("INSERT INTO clicks (campaign_id, email, updated_at) VALUES ($1, $2, NOW()) ON CONFLICT (campaign_id, email) DO UPDATE SET updated_at = NOW()")
        .bind(campaign_id)
        .bind(email)
        .execute(state.db.as_ref().unwrap())
        .await;
    "OK".to_string()
}

#[actix_web::get("/campaigns/{campaign_id}/emails")]
pub async fn get_emails(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> String {
    let campaign_id = path.into_inner();
    let rows = sqlx::query_scalar::<_, String>("SELECT email FROM clicks WHERE campaign_id = $1")
        .bind(campaign_id)
        .fetch_all(state.db.as_ref().unwrap())
        .await
        .unwrap_or_default();
    rows.join(",")
}
