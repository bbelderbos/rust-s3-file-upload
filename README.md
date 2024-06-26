# s3_file_manager

`s3_file_manager` is a Rust crate for uploading files to AWS S3 and listing objects with pagination support.

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
s3_file_manager = "0.2.0"
```

## Or use as a command line tool

```bash
$ cargo install s3_file_manager
$ s3fm --help
A Rust crate for uploading files to AWS S3 and listing objects with pagination support.

Usage: s3fm [OPTIONS] --bucket <BUCKET> --region <REGION> <--file-pattern <FILE_PATTERN>|--list-images>

Options:
  -b, --bucket <BUCKET>                          [env: S3_BUCKET_NAME=my-bucket]
  -r, --region <REGION>                          [env: AWS_REGION=us-east-2]
  -f, --file-pattern <FILE_PATTERN>
  -l, --list-images
  -m, --max-items <MAX_ITEMS>                    [default: 100]
  -c, --continuation-token <CONTINUATION_TOKEN>
  -h, --help                                     Print help
  -V, --version                                  Print version
```

This assumes that you have the AWS credentials (`AWS_ACCESS_KEY_ID` and `AWS_ACCESS_SECRET_KEY`) set up in your environment. If not, you can set them up using the `aws configure` command.
