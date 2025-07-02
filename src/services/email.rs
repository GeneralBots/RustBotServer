use std::str::FromStr;

use actix_web::{error::ErrorInternalServerError, http::header::ContentType, web, HttpResponse};
use jmap_client::{
    client::Client, core::query::Filter, email, 
    identity::Property, mailbox::{self, Role},
    email::Property as EmailProperty
};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct EmailResponse {
    pub id: String,
    pub name: String,
    pub email: String,
    pub subject: String,
    pub text: String,
}

use crate::services::state::AppState;

async fn create_jmap_client(
    state: &web::Data<AppState>,
) -> Result<(Client, String, String, String), actix_web::Error> {
    let config = state
        .config
        .as_ref()
        .ok_or_else(|| actix_web::error::ErrorInternalServerError("Configuration not available"))?;

    let client = Client::new()
        .credentials((
            config.email.username.as_ref(),
            config.email.password.as_ref(),
        ))
        .connect(&config.email.server)
        .await
        .map_err(|e| {
            actix_web::error::ErrorInternalServerError(format!("JMAP connection error: {}", e))
        })?;

    // 2. Get account ID and email
    let session = client.session();
    let (account_id, email) = session
        .accounts()
        .find_map(|account_id| {
            let account = session.account(account_id).unwrap();
            Some((account_id.to_string(), account.name().to_string()))
        })
        .unwrap();

    let identity = client
        .identity_get("default", Some(vec![Property::Id, Property::Email]))
        .await
        .map_err(|e| {
            actix_web::error::ErrorInternalServerError(format!("JMAP connection error: {}", e))
        })?.unwrap();

    let identity_id = identity.id().unwrap();

    println!("Account ID: {}", account_id);
    println!("Email address: {}", email);
    println!("IdentityID: {}", identity_id);

    Ok((client, account_id, email, String::from_str(identity_id)?))
}

#[actix_web::post("/emails/list")]
pub async fn list_emails(
    state: web::Data<AppState>,
) -> Result<web::Json<Vec<EmailResponse>>, actix_web::Error> {
    let (client, account_id, email, identity_id) = create_jmap_client(&state).await?;

    // Get inbox mailbox
    let inbox_id = client
        .mailbox_query(
            mailbox::query::Filter::role(Role::Inbox).into(),
            None::<Vec<_>>,
        )
        .await
        .map_err(|e| {
            actix_web::error::ErrorInternalServerError(format!("Failed to query inbox: {}", e))
        })?
        .take_ids()
        .first()
        .ok_or_else(|| actix_web::error::ErrorInternalServerError("Inbox not found"))?
        .clone();

    // Query emails in inbox
    let email_ids = client
        .email_query(
            Filter::and([email::query::Filter::in_mailbox(&inbox_id)]).into(),
            None::<Vec<_>>,
        )
        .await
        .map_err(|e| {
            actix_web::error::ErrorInternalServerError(format!("Failed to query emails: {}", e))
        })?
        .take_ids();

    let mut email_list = Vec::new();
    for email_id in email_ids {
        // Fetch email details
        let email = client
            .email_get(
                &email_id,
                [
                    EmailProperty::Id,
                    EmailProperty::Subject,
                    EmailProperty::From,
                    EmailProperty::TextBody,
                    EmailProperty::Preview,
                ]
                .into(),
            )
            .await
            .map_err(|e| {
                actix_web::error::ErrorInternalServerError(format!("Failed to get emails: {}", e))
            })?
            .unwrap();

        let from = email.from().unwrap().first();
        let (name, email_addr) = if let Some(addr) = from {
            (
                addr.name().unwrap_or("Unknown").to_string(),
                addr.email().to_string(),
            )
        } else {
            ("Unknown".to_string(), "unknown@example.com".to_string())
        };

        let text = email.preview().unwrap_or_default().to_string();

        email_list.push(EmailResponse {
            id: email.id().unwrap().to_string(),
            name,
            email: email_addr,
            subject: email.subject().unwrap_or_default().to_string(),
            text,
        });
    }

    Ok(web::Json(email_list))
}

#[actix_web::post("/emails/suggest-answer/{email_id}")]
pub async fn suggest_answer(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> Result<HttpResponse, actix_web::Error> {
    let email_id = path.into_inner();
    let (client, account_id, email, identity_id) = create_jmap_client(&state).await?;

    // Fetch the specific email
    let email = client
        .email_get(
            &email_id,
            [
                EmailProperty::Id,
                EmailProperty::Subject,
                EmailProperty::From,
                EmailProperty::TextBody,
                EmailProperty::Preview,
            ]
            .into(),
        )
        .await
        .map_err(|e| {
            actix_web::error::ErrorInternalServerError(format!("Failed to get email: {}", e))
        })?
        .into_iter()
        .next()
        .ok_or_else(|| actix_web::error::ErrorNotFound("Email not found"))?;

    let from = email.from().unwrap().first();
    let sender_info = if let Some(addr) = from {
        format!("{} <{}>", addr.name().unwrap_or("Unknown"), addr.email())
    } else {
        "Unknown sender".to_string()
    };

    let subject = email.subject().unwrap_or_default();
    let body_text = email.preview().unwrap_or_default();

    let response = serde_json::json!({
        "suggested_response": "Thank you for your email. I will review this and get back to you shortly.",
        "prompt": format!(
            "Email from: {}\nSubject: {}\n\nBody:\n{}\n\n---\n\nPlease draft a professional response to this email.",
            sender_info, subject, body_text
        )
    });

    Ok(HttpResponse::Ok().json(response))
}

#[actix_web::post("/emails/archive/{email_id}")]
pub async fn archive_email(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> Result<HttpResponse, actix_web::Error> {
    let email_id = path.into_inner();
    let (client, account_id, email, identity_id) = create_jmap_client(&state).await?;

    // Get Archive mailbox (or create if it doesn't exist)
    let archive_id = match client
        .mailbox_query(
            mailbox::query::Filter::name("Archive").into(),
            None::<Vec<_>>,
        )
        .await
    {
        Ok(mut result) => {
            let ids = result.take_ids();
            if let Some(id) = ids.first() {
                id.clone()
            } else {
                // Create Archive mailbox if it doesn't exist
                client
                    .mailbox_create("Archive", None::<String>, Role::Archive)
                    .await
                    .map_err(|e| {
                        actix_web::error::ErrorInternalServerError(format!(
                            "Failed to create archive mailbox: {}",
                            e
                        ))
                    })?
                    .take_id()
            }
        }
        Err(e) => {
            return Err(actix_web::error::ErrorInternalServerError(format!(
                "Failed to query mailboxes: {}",
                e
            )));
        }
    };

    // Move email to Archive mailbox
    client
        .email_set_mailboxes(&email_id, [&archive_id])
        .await
        .map_err(|e| {
            actix_web::error::ErrorInternalServerError(format!("Failed to archive email: {}", e))
        })?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Email archived successfully",
        "email_id": email_id,
        "archive_mailbox_id": archive_id
    })))
}

#[actix_web::post("/emails/send")]
pub async fn send_email(
    payload: web::Json<(String, String, String)>,
    state: web::Data<AppState>,
) -> Result<HttpResponse, actix_web::Error> {
    // Destructure the tuple into individual components
    let (to, subject, body) = payload.into_inner();

    println!("To: {}", to);
    println!("Subject: {}", subject);
    println!("Body: {}", body);

    let (client, account_id, email, identity_id) = create_jmap_client(&state).await?;

    let email_submission = client
        .email_submission_create("111", account_id)
        .await
        .map_err(|e| {
            actix_web::error::ErrorInternalServerError(format!("Failed to create email: {}", e))
        })?;

    let email_id = email_submission.email_id().unwrap();
    println!("Email-ID: {}", email_id);

    client
        .email_submission_create(email_id, identity_id)
        .await
        .map_err(|e| {
            actix_web::error::ErrorInternalServerError(format!("Failed to send email: {}", e))
        })?;

    Ok(HttpResponse::Ok().finish())
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
