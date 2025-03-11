use actix_multipart::Multipart;
use actix_web::{post, web, HttpRequest, HttpResponse};
use gb_core::models::AppError;
use gb_core::models::AppState;
use gb_core::utils::{create_response, extract_user_id};
use minio::s3::builders::ObjectContent;
use minio::s3::Client;
use std::io::Write;
use tempfile::NamedTempFile;
use minio::s3::types::ToStream;
use tokio_stream::StreamExt;


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

#[actix_web::post("/files/delete")]
pub async fn delete_file(
    req: HttpRequest,
    _state: web::Data<AppState>,
    _file_path: web::Json<String>,
) -> Result<HttpResponse, AppError> {
    let _user_id = extract_user_id(&req)?;

    Ok(create_response(
        true,
        Some("File deleted successfully".to_string()),
    ))
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