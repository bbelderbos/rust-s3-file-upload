use clap::{ArgGroup, Parser};
use glob::glob;
use rusoto_core::{credential::EnvironmentProvider, HttpClient, Region};
use rusoto_s3::{ListObjectsV2Request, PutObjectRequest, S3Client, S3};
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
}

fn create_s3_client(region: &str) -> S3Client {
    S3Client::new_with(
        HttpClient::new().expect("Failed to create HTTP client"),
        EnvironmentProvider::default(),
        Region::Custom {
            name: region.to_owned(),
            endpoint: format!("https://{}.s3.amazonaws.com", region),
        },
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
        acl: Some("public-read".to_string()), // Set ACL to public-read
        ..Default::default()
    };

    s3_client.put_object(put_request).await?;

    println!("File uploaded successfully to {}/{}", bucket, key);
    Ok(())
}

async fn list_images_in_s3(
    bucket: &str,
    region: &str,
    s3_client: &S3Client,
    max_items: i64,
) -> Result<(), Box<dyn Error>> {
    let mut continuation_token = None;

    loop {
        let list_request = ListObjectsV2Request {
            bucket: bucket.to_string(),
            max_keys: Some(max_items),
            continuation_token: continuation_token.clone(),
            ..Default::default()
        };

        let result = s3_client.list_objects_v2(list_request).await?;

        if let Some(contents) = result.contents {
            for object in contents {
                if let Some(key) = object.key {
                    // Construct the full URL for public objects
                    let url = format!("https://{}.s3.{}.amazonaws.com/{}", bucket, region, key);
                    println!("Found object with URL: {}", url);
                }
            }
        }

        if !result.is_truncated.unwrap_or(false) {
            break;
        }

        continuation_token = result.next_continuation_token;
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let s3_client = create_s3_client(&args.region);

    if args.list_images {
        list_images_in_s3(&args.bucket, &args.region, &s3_client, args.max_items).await?;
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
