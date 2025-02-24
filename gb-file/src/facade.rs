use minio_rs::minio::client::Client;
use minio_rs::minio::s3::args::{BucketExistsArgs, MakeBucketArgs, RemoveObjectArgs, GetObjectArgs, PutObjectArgs, ListObjectsArgs};
use minio_rs::minio::s3::response::Object;
use minio_rs::minio::s3::error::Error as MinioError;
use std::path::Path;
use std::io::Cursor;

/// Represents a file manager for handling MinIO file operations.
pub struct FileManager {
    client: Client,
    bucket_name: String,
}

impl FileManager {
    /// Creates a new `FileManager` instance.
    pub async fn new(endpoint: &str, access_key: &str, secret_key: &str, bucket_name: &str, use_ssl: bool) -> Result<Self, MinioError> {
        let client = Client::new(endpoint, access_key, secret_key, use_ssl).await?;
        Ok(Self {
            client,
            bucket_name: bucket_name.to_string(),
        })
    }

    /// Checks if the bucket exists, and creates it if it doesn't.
    pub async fn ensure_bucket_exists(&self) -> Result<(), MinioError> {
        let exists = self.client
            .bucket_exists(&BucketExistsArgs::new(&self.bucket_name))
            .await?;
        if !exists {
            self.client
                .make_bucket(&MakeBucketArgs::new(&self.bucket_name))
                .await?;
        }
        Ok(())
    }

    /// Uploads a file to the specified path.
    pub async fn upload_file(&self, path: &str, file_data: Vec<u8>) -> Result<(), MinioError> {
        let args = PutObjectArgs::new(&self.bucket_name, path, Cursor::new(file_data), file_data.len() as u64);
        self.client.put_object(&args).await?;
        Ok(())
    }

    /// Downloads a file from the specified path.
    pub async fn download_file(&self, path: &str) -> Result<Vec<u8>, MinioError> {
        let args = GetObjectArgs::new(&self.bucket_name, path);
        let object = self.client.get_object(&args).await?;
        let data = object.bytes().await?;
        Ok(data.to_vec())
    }

    /// Copies a file from the source path to the destination path.
    pub async fn copy_file(&self, source_path: &str, destination_path: &str) -> Result<(), MinioError> {
        let source_args = GetObjectArgs::new(&self.bucket_name, source_path);
        let object = self.client.get_object(&source_args).await?;
        let data = object.bytes().await?;

        let destination_args = PutObjectArgs::new(&self.bucket_name, destination_path, Cursor::new(data.clone()), data.len() as u64);
        self.client.put_object(&destination_args).await?;
        Ok(())
    }

    /// Moves a file from the source path to the destination path.
    pub async fn move_file(&self, source_path: &str, destination_path: &str) -> Result<(), MinioError> {
        self.copy_file(source_path, destination_path).await?;
        self.delete_file(source_path).await?;
        Ok(())
    }

    /// Deletes a file at the specified path.
    pub async fn delete_file(&self, path: &str) -> Result<(), MinioError> {
        let args = RemoveObjectArgs::new(&self.bucket_name, path);
        self.client.remove_object(&args).await?;
        Ok(())
    }

    /// Lists all files in the specified path.
    pub async fn list_files(&self, prefix: &str) -> Result<Vec<String>, MinioError> {
        let args = ListObjectsArgs::new(&self.bucket_name).with_prefix(prefix);
        let objects = self.client.list_objects(&args).await?;
        let file_names = objects.into_iter().map(|obj| obj.name().to_string()).collect();
        Ok(file_names)
    }

    /// Retrieves the contents of a file at the specified path.
    pub async fn get_file_contents(&self, path: &str) -> Result<String, MinioError> {
        let data = self.download_file(path).await?;
        let contents = String::from_utf8(data).map_err(|_| MinioError::InvalidResponse)?;
        Ok(contents)
    }

    /// Creates a folder at the specified path.
    pub async fn create_folder(&self, path: &str) -> Result<(), MinioError> {
        let folder_path = if path.ends_with('/') {
            path.to_string()
        } else {
            format!("{}/", path)
        };
        self.upload_file(&folder_path, vec![]).await
    }

    /// Shares a folder at the specified path (placeholder implementation).
    pub async fn share_folder(&self, path: &str) -> Result<String, MinioError> {
        Ok(format!("Folder shared: {}", path))
    }

    /// Searches for files matching the query in the specified path.
    pub async fn search_files(&self, prefix: &str, query: &str) -> Result<Vec<String>, MinioError> {
        let files = self.list_files(prefix).await?;
        let results = files.into_iter().filter(|f| f.contains(query)).collect();
        Ok(results)
    }
}