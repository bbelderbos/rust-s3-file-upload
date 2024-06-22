use rusoto_core::{Region, credential::EnvironmentProvider, HttpClient};
use rusoto_s3::{S3Client, S3, PutObjectRequest};
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use std::error::Error;
use clap::Parser;
use std::path::Path;

#[derive(Parser, Debug)]
#[clap(about, version, author)]
struct Args {
    #[clap(short, long, env = "S3_BUCKET_NAME")]
    bucket: String,
    #[clap(short, long, env = "AWS_REGION")]
    region: String,
    #[clap(short, long)]
    file: String,
}

async fn upload_image_to_s3(bucket: &str, key: &str, file: &str, region: Region) -> Result<(), Box<dyn Error>> {
    let s3_client = S3Client::new_with(HttpClient::new()?, EnvironmentProvider::default(), region);

    let mut file = File::open(file).await?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).await?;

    let put_request = PutObjectRequest {
        bucket: bucket.to_string(),
        key: key.to_string(),
        body: Some(buffer.into()),
        ..Default::default()
    };

    s3_client.put_object(put_request).await?;

    println!("File uploaded successfully to {}/{}", bucket, key);
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let key = Path::new(&args.file)
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or("Invalid file path")?;

    let region = args.region.parse::<Region>()?;

    upload_image_to_s3(&args.bucket, key, &args.file, region).await?;

    Ok(())
}
