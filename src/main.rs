use clap::{ArgGroup, Parser};
use glob::glob;
use rusoto_core::{credential::EnvironmentProvider, HttpClient, Region};
use rusoto_s3::{ListObjectsV2Request, PutObjectRequest, S3Client, S3};
use serde::Serialize;
use std::error::Error;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

#[derive(Parser, Debug)]
#[clap(about, version, author)]
#[clap(group(
    ArgGroup::new("operation")
        .required(true)
        .args(&["file_pattern", "list_images"]),
))]
struct Args {
    #[clap(short, long, env = "S3_BUCKET_NAME")]
    bucket: String,
    #[clap(short, long, env = "AWS_REGION")]
    region: String,
    #[clap(short, long)]
    file_pattern: Option<String>,
    #[clap(short, long)]
    list_images: bool,
    #[clap(short, long, default_value = "100")]
    max_items: i64,
    #[clap(short, long)]
    continuation_token: Option<String>,
}

#[derive(Serialize)]
struct ListResponse {
    objects: Vec<String>,
    continuation_token: Option<String>,
}

fn create_s3_client(region: Region) -> S3Client {
    S3Client::new_with(
        HttpClient::new().expect("Failed to create HTTP client"),
        EnvironmentProvider::default(),
        region,
    )
}

async fn upload_image_to_s3(
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

async fn list_images_in_s3(
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let region = args.region.parse::<Region>()?;
    let s3_client = create_s3_client(region);

    if args.list_images {
        let response = list_images_in_s3(
            &args.bucket,
            &s3_client,
            args.max_items,
            args.continuation_token.clone(),
        )
        .await?;

        for object in response.objects {
            let url = format!(
                "https://{}.s3.{}.amazonaws.com/{}",
                args.bucket, args.region, object
            );
            println!("Found object with URL: {}", url);
        }

        if let Some(token) = response.continuation_token {
            println!("Next Continuation Token: {}", token);
        }
    } else if let Some(pattern) = args.file_pattern {
        for entry in glob(&pattern)? {
            match entry {
                Ok(path) => {
                    let file = path.to_str().ok_or("Invalid file path")?;
                    let key = path
                        .file_name()
                        .and_then(|name| name.to_str())
                        .ok_or("Invalid file name")?;

                    upload_image_to_s3(&args.bucket, key, file, &s3_client).await?;
                }
                Err(e) => println!("Error reading file pattern: {}", e),
            }
        }
    } else {
        eprintln!("Either --file-pattern or --list_images must be provided");
    }

    Ok(())
}
