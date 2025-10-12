use s3::{Bucket, BucketConfiguration, Region, creds::Credentials};

use crate::common::run;

#[allow(unused)]
mod common;

#[tokio::test]
async fn put_object() {
  let ports = run().await;

  let region = Region::Custom {
    region: "test".into(),
    endpoint: format!("http://localhost:{}", ports.s3_port),
  };
  let credentials = Credentials::new(Some("key"), Some("secret"), None, None, None).unwrap();

  // First, create the bucket
  let _ = Bucket::create_with_path_style(
    "test-bucket",
    region.clone(),
    credentials.clone(),
    BucketConfiguration::default(),
  )
  .await
  .unwrap();

  // Then, upload an object
  let b = Bucket::new("test-bucket", region, credentials)
    .unwrap()
    .with_path_style();

  let data = b"test data".to_vec();
  b.put_object("test", &data).await.unwrap();
}

#[tokio::test]
async fn delete_object() {
  let ports = run().await;

  let region = Region::Custom {
    region: "test".into(),
    endpoint: format!("http://localhost:{}", ports.s3_port),
  };
  let credentials = Credentials::new(Some("key"), Some("secret"), None, None, None).unwrap();

  // First, create the bucket
  let _ = Bucket::create_with_path_style(
    "test-bucket",
    region.clone(),
    credentials.clone(),
    BucketConfiguration::default(),
  )
  .await
  .unwrap();

  // Then, upload an object
  let b = Bucket::new("test-bucket", region, credentials)
    .unwrap()
    .with_path_style();

  let data = b"test data".to_vec();
  b.put_object("test", &data).await.unwrap();

  // Finally, delete the object
  b.delete_object("test").await.unwrap();
}
