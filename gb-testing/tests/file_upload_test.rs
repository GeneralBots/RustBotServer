use actix_web::{test, web, App};
use anyhow::Result;
use bytes::Bytes;
use gb_core::models::AppState;
use gb_file::handlers::upload_file;
use minio::s3::args::{BucketExistsArgs, GetObjectArgs, MakeBucketArgs, StatObjectArgs};
use minio::s3::client::ClientBuilder as MinioClientBuilder;
use minio::s3::creds::StaticProvider;
use minio::s3::http::BaseUrl;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::str::FromStr;
use tempfile::NamedTempFile;

#[tokio::test]
async fn test_successful_file_upload() -> Result<()> {
    // Setup test environment and MinIO client
    let base_url = format!("http://{}", "localhost:9000");
    let base_url = BaseUrl::from_str(&base_url)?;
    let credentials = StaticProvider::new(&"minioadmin", &"minioadmin", None);

    let minio_client = MinioClientBuilder::new(base_url.clone())
        .provider(Some(Box::new(credentials)))
        .build()?;

    // Create test bucket if it doesn't exist
    let bucket_name = "file-upload-rust-bucket";

    // Using object-based API for bucket_exists
    let bucket_exists_args = BucketExistsArgs::new(bucket_name)?;
    let bucket_exists = minio_client.bucket_exists(&bucket_exists_args).await?;

    if !bucket_exists {
        // Using object-based API for make_bucket
        let make_bucket_args = MakeBucketArgs::new(bucket_name)?;
        minio_client.make_bucket(&make_bucket_args).await?;
    }

    let app_state = web::Data::new(AppState {
        minio_client: Some(minio_client.clone()),
        config: None,
        db_pool: None,
        kafka_producer: None,
        redis_pool: None,
    });

    let app = 
    test::init_service(App::new().app_data(app_state.clone())
        .service(upload_file)).await;

    // Create a test file with content
    let mut temp_file = NamedTempFile::new()?;
    write!(temp_file, "Test file content for upload")?;

    // Prepare a multipart request
    let boundary = "----WebKitFormBoundaryX";
    let content_type = format!("multipart/form-data; boundary={}", boundary);

    // Read the file content
    let mut file_content = Vec::new();
    let mut file = File::open(temp_file.path())?;
    file.read_to_end(&mut file_content)?;

    let body = format!(
            "--{}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"test.txt\"\r\nContent-Type: text/plain\r\n\r\n{}\r\n--{}--\r\n",
            boundary,
            String::from_utf8_lossy(&file_content),
            boundary
        );

    // Execute request
    let req = test::TestRequest::post()
        .uri("/files/upload/test-folder")
        .set_payload(Bytes::from(body))
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Verify response
    assert_eq!(resp.status(), 200);

    // Verify file exists in MinIO using object-based API
    let object_name = "test-folder/test.txt";
    let bucket_name = "file-upload-rust-bucket";

    // Using object-based API for stat_object
    let stat_object_args = StatObjectArgs::new(bucket_name, object_name)?;
    let object_exists = minio_client.clone().stat_object(&stat_object_args).await.is_ok();

    assert!(object_exists, "Uploaded file should exist in MinIO");

    // Verify file content using object-based API
    // let get_object_args = GetObjectArgs::new(bucket_name, object_name)?;
    // let get_object_result = minio_client.get_object(bucket_name, object_name);

    // let mut object_content = Vec::new();
    // get_object_result.read_to_end(&mut object_content)?;

    // assert_eq!(
    //     String::from_utf8_lossy(&object_content),
    //     String::from_utf8_lossy(&file_content),
    //     "File content should match"
    // );
    // // Cleanup test bucket
    // let bucket_name = "file-upload-rust-bucket";

    // // List all objects and delete them using object-based API
    // let list_objects_args = GetObjectArgs:new(bucket_name)?;
    // let objects = minio_client.list_objects(&list_objects_args).await?;

    // for obj in objects.contents {
    //     // Using object-based API for remove_object
    //     let remove_object_args = RemoveObjectArgs::new(bucket_name, &obj.key)?;
    //     minio_client.remove_object(&remove_object_args).await?;
    // }

    Ok(())
}
