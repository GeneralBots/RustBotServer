use crate::services::{config::EmailConfig, state::AppState};
use actix_web::error::ErrorInternalServerError;
use actix_web::http::header::ContentType;
use actix_web::{web, HttpResponse, Result};
use lettre::{transport::smtp::authentication::Credentials, Message, SmtpTransport, Transport};
use serde::{Deserialize, Serialize};
use tokio_stream::StreamExt;
use mailparse::{parse_mail, MailHeaderMap};  // Added MailHeaderMap import
use imap::types::{Seq};


#[derive(Debug, Serialize)]
pub struct EmailResponse {
    pub id: String,
    pub name: String,
    pub email: String,
    pub subject: String,
    pub text: String,
    date: String,
    read: bool,
    labels: Vec<String>,
}

async fn internal_send_email(config: &EmailConfig, to: &str, subject: &str, body: &str) {
    let email = Message::builder()
        .from(config.from.parse().unwrap())
        .to(to.parse().unwrap())
        .subject(subject)
        .body(body.to_string())
        .unwrap();

    let creds = Credentials::new(config.username.clone(), config.password.clone());

    SmtpTransport::relay(&config.server)
        .unwrap()
        .port(config.port)
        .credentials(creds)
        .build()
        .send(&email)
        .unwrap();
}

#[actix_web::get("/emails/list")]
pub async fn list_emails(
    state: web::Data<AppState>,
) -> Result<web::Json<Vec<EmailResponse>>, actix_web::Error> {
    let _config = state
        .config
        .as_ref()
        .ok_or_else(|| ErrorInternalServerError("Configuration not available"))?;

    // Establish connection
    let tls = native_tls::TlsConnector::builder().build().map_err(|e| {
        ErrorInternalServerError(format!("Failed to create TLS connector: {:?}", e))
    })?;

    let client = imap::connect(
        (_config.email.server.as_str(), 993),
        _config.email.server.as_str(),
        &tls,
    )
    .map_err(|e| ErrorInternalServerError(format!("Failed to connect to IMAP: {:?}", e)))?;

    // Login
    let mut session = client
        .login(&_config.email.username, &_config.email.password)
        .map_err(|e| ErrorInternalServerError(format!("Login failed: {:?}", e)))?;

    // Select INBOX
    session
        .select("INBOX")
        .map_err(|e| ErrorInternalServerError(format!("Failed to select INBOX: {:?}", e)))?;

    // Search for all messages
    let messages = session
        .search("ALL")
        .map_err(|e| ErrorInternalServerError(format!("Failed to search emails: {:?}", e)))?;

    let mut email_list = Vec::new();

    // Get last 20 messages
    let recent_messages: Vec<_> = messages.iter().cloned().collect();  // Collect items into a Vec
    let recent_messages: Vec<Seq> = recent_messages.into_iter().rev().take(20).collect();  // Now you can reverse and take the last 20
    for seq in recent_messages {
        // Fetch the entire message (headers + body)
        let fetch_result = session.fetch(seq.to_string(), "RFC822");
        let messages = fetch_result
            .map_err(|e| ErrorInternalServerError(format!("Failed to fetch email: {:?}", e)))?;

        for msg in messages.iter() {
            let body = msg
                .body()
                .ok_or_else(|| ErrorInternalServerError("No body found"))?;

            // Parse the complete email message
            let parsed = parse_mail(body)
                .map_err(|e| ErrorInternalServerError(format!("Failed to parse email: {:?}", e)))?;

            // Extract headers
            let headers = parsed.get_headers();
            let subject = headers.get_first_value("Subject").unwrap_or_default();
            let from = headers.get_first_value("From").unwrap_or_default();
            let date = headers.get_first_value("Date").unwrap_or_default();

            // Extract body text (handles both simple and multipart emails)
            let body_text = if let Some(body_part) = parsed.subparts.iter().find(|p| p.ctype.mimetype == "text/plain") {
                body_part.get_body().unwrap_or_default()
            } else {
                parsed.get_body().unwrap_or_default()
            };

            // Create preview
            let preview = body_text.lines().take(3).collect::<Vec<_>>().join(" ");
            let preview_truncated = if preview.len() > 150 {
                format!("{}...", &preview[..150])
            } else {
                preview
            };

            // Parse From field
            let (from_name, from_email) = parse_from_field(&from);

            email_list.push(EmailResponse {
                id: seq.to_string(),
                name: from_name,
                email: from_email,
                subject: if subject.is_empty() {
                    "(No Subject)".to_string()
                } else {
                    subject
                },
                text: preview_truncated,
                date: if date.is_empty() {
                    chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string()
                } else {
                    date
                },
                read: false,
                labels: Vec::new(),
            });
        }
    }

    session
        .logout()
        .map_err(|e| ErrorInternalServerError(format!("Failed to logout: {:?}", e)))?;

    Ok(web::Json(email_list))
}

// Helper function to parse From field
fn parse_from_field(from: &str) -> (String, String) {
    if let Some(start) = from.find('<') {
        if let Some(end) = from.find('>') {
            let email = from[start+1..end].trim().to_string();
            let name = from[..start].trim().trim_matches('"').to_string();
            return (name, email);
        }
    }
    ("Unknown".to_string(), from.to_string())
}

// #[actix_web::post("/emails/suggest-answer/{email_id}")]
// pub async fn suggest_answer(
//     path: web::Path<String>,
//     state: web::Data<AppState>,
// ) -> Result<HttpResponse, actix_web::Error> {
//     let email_id = path.into_inner();
//     let config = state
//         .config
//         .as_ref()
//         .ok_or_else(|| ErrorInternalServerError("Configuration not available"))?;

//     // let mut session = create_imap_session(&config.email).await?;

//     // session
//     //     .select("INBOX")
//     //     .await
//     //     .map_err(|e| ErrorInternalServerError(format!("Failed to select INBOX: {:?}", e)))?;

//     // let messages = session
//     //     .fetch(&email_id, "RFC822.HEADER BODY[TEXT]")
//     //     .await
//     //     .map_err(|e| ErrorInternalServerError(format!("Failed to fetch email: {:?}", e)))?;

//     // let msg = messages
//     //     .iter()
//     //     .next()
//     //     .ok_or_else(|| actix_web::error::ErrorNotFound("Email not found"))?;

//     // let header = msg
//     //     .header()
//     //     .ok_or_else(|| ErrorInternalServerError("No header found"))?;

//     // let body = msg
//     //     .text()
//     //     .ok_or_else(|| ErrorInternalServerError("No body found"))?;

//     // let header_str = String::from_utf8_lossy(header);
//     // let mut subject = String::new();
//     // let mut from_info = String::new();

//     // for line in header_str.lines() {
//     //     if line.starts_with("Subject: ") {
//     //         subject = line.strip_prefix("Subject: ").unwrap_or("").to_string();
//     //     } else if line.starts_with("From: ") {
//     //         from_info = line.strip_prefix("From: ").unwrap_or("").to_string();
//     //     }
//     // }

//     // let body_text = String::from_utf8_lossy(body);

//     // let response = serde_json::json!({
//     //     "suggested_response": "Thank you for your email. I will review this and get back to you shortly.",
//     //     "prompt": format!(
//     //         "Email from: {}\nSubject: {}\n\nBody:\n{}\n\n---\n\nPlease draft a professional response to this email.",
//     //         from_info, subject, body_text.lines().take(20).collect::<Vec<_>>().join("\n")
//     //     )
//     // });

//     // session.logout().await.ok();
//     //Ok(HttpResponse::Ok().json(response))
// }

// #[actix_web::post("/emails/archive/{email_id}")]
// pub async fn archive_email(
//     path: web::Path<String>,
//     state: web::Data<AppState>,
// ) -> Result<HttpResponse, actix_web::Error> {
//     let email_id = path.into_inner();
//     let config = state
//         .config
//         .as_ref()
//         .ok_or_else(|| ErrorInternalServerError("Configuration not available"))?;

//     let mut session = create_imap_session(&config.email).await?;

//     session
//         .select("INBOX")
//         .await
//         .map_err(|e| ErrorInternalServerError(format!("Failed to select INBOX: {:?}", e)))?;

//     // Create Archive folder if it doesn't exist
//     session.create("Archive").await.ok(); // Ignore error if folder exists

//     // Move email to Archive folder
//     session.mv(&email_id, "Archive").await.map_err(|e| {
//         ErrorInternalServerError(format!("Failed to move email to archive: {:?}", e))
//     })?;

//     session.logout().await.ok();

//     Ok(HttpResponse::Ok().json(serde_json::json!({
//         "message": "Email archived successfully",
//         "email_id": email_id,
//         "archive_folder": "Archive"
//     })))
// }

#[actix_web::post("/emails/send")]
pub async fn send_email(
    payload: web::Json<(String, String, String)>,
    state: web::Data<AppState>,
) -> Result<HttpResponse, actix_web::Error> {
    let (to, subject, body) = payload.into_inner();

    println!("To: {}", to);
    println!("Subject: {}", subject);
    println!("Body: {}", body);

    // Send via SMTP
    internal_send_email(&state.config.clone().unwrap().email, &to, &subject, &body).await;

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
