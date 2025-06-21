
use actix_web::{middleware, web, App, HttpServer};
use tracing_subscriber::fmt::format::FmtSpan;
use dotenv::dotenv;

use actix_multipart::Multipart;
use actix_web::{post, HttpResponse};
use minio::s3::builders::ObjectContent;
use minio::s3::Client;
use std::io::Write;
use tempfile::NamedTempFile;
use minio::s3::types::ToStream;
use tokio_stream::StreamExt;

use minio::s3::client::{Client as MinioClient, ClientBuilder as MinioClientBuilder};
use minio::s3::creds::StaticProvider;
use minio::s3::http::BaseUrl;
use std::str::FromStr;

use services::config::*;
mod services;

// App state shared across all handlers
pub struct AppState {
    pub minio_client: Option<MinioClient>,
    pub config: Option<AppConfig>,

}

pub async fn init_minio(config: &AppConfig) -> Result<MinioClient, minio::s3::error::Error> {
    let scheme = if config.minio.use_ssl { "https" } else { "http" };
    let base_url = format!("{}://{}", scheme, config.minio.endpoint);
    let base_url = BaseUrl::from_str(&base_url)?;
    let credentials = StaticProvider::new(
        &config.minio.access_key,
        &config.minio.secret_key,
        None,
    );

    let minio_client = MinioClientBuilder::new(base_url)
        .provider(Some(credentials))
        .build()?;

    Ok(minio_client)
}

#[post("/files/upload/{folder_path}")]
pub async fn upload_file(
    folder_path: web::Path<String>,
    mut payload: Multipart,
    state: web::Data<AppState>,
) -> Result<HttpResponse, actix_web::Error> {
    let folder_path = folder_path.into_inner();

    // Create a temporary file to store the uploaded file.

    let mut temp_file = NamedTempFile::new().map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Failed to create temp file: {}", e))
    })?;

    let mut file_name = None;

    // Iterate over the multipart stream.

    while let Some(mut field) = payload.try_next().await? {
        let content_disposition = field.content_disposition();
        file_name = content_disposition
            .get_filename()
            .map(|name| name.to_string());

        // Write the file content to the temporary file.
        while let Some(chunk) = field.try_next().await? {
            temp_file.write_all(&chunk).map_err(|e| {
                actix_web::error::ErrorInternalServerError(format!(
                    "Failed to write to temp file: {}",
                    e
                ))
            })?;
        }
    }

    // Get the file name or use a default name
    let file_name = file_name.unwrap_or_else(|| "unnamed_file".to_string());

    // Construct the object name using the folder path and file name
    let object_name = format!("{}/{}", folder_path, file_name);

    // Upload the file to the MinIO bucket
    let client: Client = state.minio_client.clone().unwrap();
    let bucket_name = "file-upload-rust-bucket";

    let content = ObjectContent::from(temp_file.path());
    client
        .put_object_content(bucket_name, &object_name, content)
        .send()
        .await
        .map_err(|e| {
            actix_web::error::ErrorInternalServerError(format!(
                "Failed to upload file to MinIO: {}",
                e
            ))
        })?;

    // Clean up the temporary file
    temp_file.close().map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Failed to close temp file: {}", e))
    })?;

    Ok(HttpResponse::Ok().body(format!(
        "Uploaded file '{}' to folder '{}'",
        file_name, folder_path
    )))
}



#[post("/files/list/{folder_path}")]
pub async fn list_file(
    folder_path: web::Path<String>,
    state: web::Data<AppState>,
) -> Result<HttpResponse, actix_web::Error> {
    let folder_path = folder_path.into_inner();

    let client: Client = state.minio_client.clone().unwrap();
    let bucket_name = "file-upload-rust-bucket";

    // Create the stream using the to_stream() method
    let mut objects_stream = client
        .list_objects(bucket_name)
        .prefix(Some(folder_path))
        .to_stream()
        .await;

    let mut file_list = Vec::new();
    
    // Use StreamExt::next() to iterate through the stream
    while let Some(items) = objects_stream.next().await {
        match items {
            Ok(result) => {
                for item in result.contents {
                    file_list.push(item.name);
                }
            },
            Err(e) => {
                return Err(actix_web::error::ErrorInternalServerError(
                    format!("Failed to list files in MinIO: {}", e)
                ));
            }
        }
    }

    Ok(HttpResponse::Ok().json(file_list))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    dotenv().ok();

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_span_events(FmtSpan::CLOSE)
        .init();

    // log::set_max_level(LevelFilter::Info);

    let config = AppConfig::from_env();
    let minio_client = init_minio(&config).await.expect("Failed to initialize Minio");

    let app_state = web::Data::new(AppState {
        config: Some(config.clone()),
        minio_client: Some(minio_client),
    });

    // Start HTTP server
    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .app_data(app_state.clone())
            .service(upload_file) // Uncomment and import or define upload_file below
    })
    .bind((config.server.host.clone(), config.server.port))?
    .run()
    .await
}
