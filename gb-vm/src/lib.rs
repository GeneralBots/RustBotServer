use minio::s3::client::Client;
use minio::s3::args::{BucketExistsArgs, MakeBucketArgs, RemoveObjectArgs, GetObjectArgs, PutObjectArgs, ListObjectsArgs};
use minio::s3::creds::StaticProvider;
use minio::s3::error::Error as MinioError;
use minio::s3::types::{BaseUrl, Item};
use std::io::Cursor;
use std::path::Path;

pub struct FileManager {
    client: Client,
    bucket_name: String,
}

impl FileManager {
    pub async fn new(endpoint: &str, access_key: &str, secret_key: &str, bucket_name: &str, use_ssl: bool) -> Result<Self, MinioError> {
        // Create BaseUrl from endpoint
        let base_url = BaseUrl::from_string(endpoint)?;
     let static_provider = StaticProvider::new(
         access_key,
         secret_key,
         None,
     );
     let client = Client::new(base_url.clone(), Some(Box::new(static_provider)), None, None).unwrap();

        
        Ok(Self {
            client,
            bucket_name: bucket_name.to_string(),
        })
    }

    pub async fn ensure_bucket_exists(&self) -> Result<(), MinioError> {
        let exists = self.client
            .bucket_exists(&BucketExistsArgs::new(&self.bucket_name)?)
            .await?;
        if !exists {
            self.client
                .make_bucket(&MakeBucketArgs::new(&self.bucket_name)?)
                .await?;
        }
        Ok(())
    }

    pub async fn upload_file(&self, path: &str, file_data: Vec<u8>) -> Result<(), MinioError> {
        let reader = Cursor::new(&file_data);
        let file_size = file_data.len() as u64;
        
        let args = PutObjectArgs::new(
            &self.bucket_name,
            path,
            reader,
            Some(file_size),
            None
        )?;
        
        self.client.put_object(&args).await?;
        Ok(())
    }

    pub async fn download_file(&self, path: &str) -> Result<Vec<u8>, MinioError> {
        let args = GetObjectArgs::new(&self.bucket_name, path)?;
        let object = self.client.get_object(&args).await?;
        let data = object.bytes().await?;
        Ok(data.to_vec())
    }

    pub async fn copy_file(&self, source_path: &str, destination_path: &str) -> Result<(), MinioError> {
        // Download the source file
        let data = self.download_file(source_path).await?;
        
        // Upload it to the destination
        let reader = Cursor::new(&data);
        let file_size = data.len() as u64;
        
        let args = PutObjectArgs::new(
            &self.bucket_name,
            destination_path,
            reader,
            Some(file_size),
            None
        )?;
        
        self.client.put_object(&args).await?;
        Ok(())
    }

    pub async fn move_file(&self, source_path: &str, destination_path: &str) -> Result<(), MinioError> {
        self.copy_file(source_path, destination_path).await?;
        self.delete_file(source_path).await?;
        Ok(())
    }

    pub async fn delete_file(&self, path: &str) -> Result<(), MinioError> {
        let args = RemoveObjectArgs::new(&self.bucket_name, path)?;
        self.client.remove_object(&args).await?;
        Ok(())
    }

    pub async fn list_files(&self, prefix: &str) -> Result<Vec<String>, MinioError> {
        // Create a predicate function that always returns true
        let predicate = |_: Vec<Item>| -> bool { true };
        
        let args = ListObjectsArgs::new(&self.bucket_name, &predicate)?;
        let objects = self.client.list_objects(&args).await?;
        
        // Filter objects based on prefix manually
        let file_names: Vec<String> = objects
            .into_iter()
            .filter(|obj| obj.name().starts_with(prefix))
            .map(|obj| obj.name().to_string())
            .collect();
            
        Ok(file_names)
    }

    pub async fn get_file_contents(&self, path: &str) -> Result<String, MinioError> {
        let data = self.download_file(path).await?;
        let contents = String::from_utf8(data)
            .map_err(|_| MinioError::InvalidResponse(400, "Invalid UTF-8 sequence".to_string()))?;
        Ok(contents)
    }

    pub async fn create_folder(&self, path: &str) -> Result<(), MinioError> {
        let folder_path = if path.ends_with('/') {
            path.to_string()
        } else {
            format!("{}/", path)
        };
        
        // Create empty file with folder path
        self.upload_file(&folder_path, vec![]).await
    }

    pub async fn share_folder(&self, path: &str) -> Result<String, MinioError> {
        // This is just a placeholder implementation
        Ok(format!("Folder shared: {}", path))
    }

    pub async fn search_files(&self, prefix: &str, query: &str) -> Result<Vec<String>, MinioError> {
        let files = self.list_files(prefix).await?;
        let results = files.into_iter().filter(|f| f.contains(query)).collect();
        Ok(results)
    }
}