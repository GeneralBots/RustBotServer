use actix_multipart::Multipart;
use actix_web::{web, HttpRequest, HttpResponse};
use futures::{StreamExt, TryStreamExt};
use gb_core::models::AppState;
use std::io::Write;
use gb_core::models::AppError;
use gb_core::utils::{create_response, extract_user_id};

#[actix_web::post("/files/upload")]
pub async fn upload_file(
    req: HttpRequest,
    mut payload: Multipart,
    state: web::Data<AppState>,
) -> Result<HttpResponse, AppError> {
    let user_id = extract_user_id(&req)?;
    let folder_path = req.query_string(); // Assuming folder path is passed as query parameter

    while let Ok(Some(mut field)) = payload.try_next().await {
        let content_disposition = field.content_disposition();
        let filename = content_disposition
            .get_filename()
            .ok_or_else(|| AppError::Validation("Filename not provided".to_string()))?
            .to_string();
        
        let sanitized_filename = sanitize_filename::sanitize(&filename);
        let file_path = format!("{}/{}/{}", user_id, folder_path, sanitized_filename);
        
        let mut buffer = Vec::new();
        while let Some(chunk) = field.next().await {
            let data = chunk.map_err(|e| AppError::Internal(format!("Error reading file: {}", e)))?;
            buffer.write_all(&data).map_err(|e| AppError::Internal(format!("Error writing to buffer: {}", e)))?;
        }
        
        let content_type = field.content_type().map(|t| t.to_string()).unwrap_or_else(|| "application/octet-stream".to_string());
        
        state.minio_client
            .put_object(&state.config.minio.bucket, &file_path, &buffer, Some(content_type.as_str()), None)
            .await
            .map_err(|e| AppError::Minio(format!("Failed to upload file to Minio: {}", e)))?;
        
        return Ok(create_response(
            format!("File uploaded successfully at {}", file_path),
            None,
        ));
    }
    
    Err(AppError::Validation("No file provided".to_string()))
}

#[actix_web::post("/files/download")]
pub async fn download(
    req: HttpRequest,
    state: web::Data<AppState>,
    file_path: web::Json<String>,
) -> Result<HttpResponse, AppError> {
    let user_id = extract_user_id(&req)?;
    
    let file_content = state.minio_client
        .get_object(&state.config.minio.bucket, &file_path)
        .await
        .map_err(|e| AppError::Minio(format!("Failed to retrieve file from Minio: {}", e)))?;
    
    Ok(HttpResponse::Ok()
        .content_type("application/octet-stream")
        .append_header(("Content-Disposition", format!("attachment; filename=\"{}\"", file_path)))
        .body(file_content))
}

#[actix_web::post("/files/delete")]
pub async fn delete_file(
    req: HttpRequest,
    state: web::Data<AppState>,
    file_path: web::Json<String>,
) -> Result<HttpResponse, AppError> {
    let user_id = extract_user_id(&req)?;
    
    state.minio_client
        .remove_object(&state.config.minio.bucket, &file_path)
        .await
        .map_err(|e| AppError::Minio(format!("Failed to delete file from Minio: {}", e)))?;
    
    Ok(create_response(
        true,
        Some("File deleted successfully".to_string()),
    ))
}

#[actix_web::post("/files/list")]
pub async fn list_files(
    req: HttpRequest,
    state: web::Data<AppState>,
    folder_path: web::Json<String>,
) -> Result<HttpResponse, AppError> {
    let user_id = extract_user_id(&req)?;
    
    let objects = state.minio_client
        .list_objects(&state.config.minio.bucket, &folder_path, None, None)
        .await
        .map_err(|e| AppError::Minio(format!("Failed to list objects in Minio: {}", e)))?;
    
    Ok(create_response(objects, None))
}