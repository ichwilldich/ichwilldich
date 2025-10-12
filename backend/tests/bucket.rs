use s3::{Bucket, BucketConfiguration, Region, creds::Credentials};

use crate::common::run;

#[allow(unused)]
mod common;

#[tokio::test]
async fn create_bucket_works() {
  let ports = run().await;

  let region = Region::Custom {
    region: "test".into(),
    endpoint: format!("http://localhost:{}", ports.s3_port),
  };
  let credentials = Credentials::new(Some("key"), Some("secret"), None, None, None).unwrap();
  let _ = Bucket::create_with_path_style(
    "test-bucket",
    region,
    credentials,
    BucketConfiguration::default(),
  )
  .await
  .unwrap();
}
