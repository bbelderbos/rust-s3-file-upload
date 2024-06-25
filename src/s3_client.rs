use rusoto_core::{credential::EnvironmentProvider, HttpClient, Region};
use rusoto_s3::{ListObjectsV2Request, PutObjectRequest, S3Client, S3};
use serde::Serialize;
use std::error::Error;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

#[derive(Serialize)]
pub struct ListResponse {
    pub objects: Vec<String>,
    pub continuation_token: Option<String>,
}

pub fn create_s3_client(region: Region) -> S3Client {
    S3Client::new_with(
        HttpClient::new().expect("Failed to create HTTP client"),
        EnvironmentProvider::default(),
        region,
    )
}

pub async fn upload_image_to_s3(
    bucket: &str,
    key: &str,
    file: &str,
    s3_client: &S3Client,
) -> Result<(), Box<dyn Error>> {
    let mut file = File::open(file).await?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).await?;

    let put_request = PutObjectRequest {
        bucket: bucket.to_string(),
        key: key.to_string(),
        body: Some(buffer.into()),
        acl: Some("public-read".to_string()), // Set ACL to public-read (requires public bucket)
        ..Default::default()
    };

    s3_client.put_object(put_request).await?;

    println!("File uploaded successfully to {}/{}", bucket, key);
    Ok(())
}

pub async fn list_images_in_s3(
    bucket: &str,
    s3_client: &S3Client,
    max_items: i64,
    continuation_token: Option<String>,
) -> Result<ListResponse, Box<dyn Error>> {
    let list_request = ListObjectsV2Request {
        bucket: bucket.to_string(),
        max_keys: Some(max_items),
        continuation_token,
        ..Default::default()
    };

    let result = s3_client.list_objects_v2(list_request).await?;
    let objects = result
        .contents
        .unwrap_or_default()
        .into_iter()
        .filter_map(|obj| obj.key)
        .collect::<Vec<String>>();

    let response = ListResponse {
        objects,
        continuation_token: result.next_continuation_token,
    };

    Ok(response)
}
