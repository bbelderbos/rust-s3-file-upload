use clap::{ArgGroup, Parser};
use glob::glob;
use rusoto_core::Region;
use s3_file_manager::s3_client::{create_s3_client, list_images_in_s3, upload_image_to_s3};
use std::error::Error;

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
