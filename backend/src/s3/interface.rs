use axum::Extension;
use centaurus::{FromReqExtension, bail, error::Result, path};

use crate::{config::EnvConfig, macros::DualRouterExt, router_extension, s3::BUCKET_DIR};
use std::{ops::Deref, path::Path, sync::Arc};

use crate::s3::storage::Storage;

#[derive(Clone, FromReqExtension)]
pub struct S3Interface {
  storage: Arc<dyn Storage + Send + Sync>,
}

impl S3Interface {
  pub fn new<S: Storage + Send + Sync + 'static>(storage: S) -> Self {
    Self {
      storage: Arc::new(storage),
    }
  }
  pub async fn create_bucket(&self, bucket: &String) -> Result<()> {
    if self.list_dir(&path!(BUCKET_DIR)).await?.contains(bucket) {
      bail!(CONFLICT, "Bucket {bucket} already exists");
    }

    self.create_dir(&path!(BUCKET_DIR, &bucket)).await?;

    Ok(())
  }

  pub async fn delete_bucket(&self, bucket: &String) -> Result<()> {
    if !self.list_dir(&path!(BUCKET_DIR)).await?.contains(bucket) {
      bail!(NOT_FOUND, "Bucket {bucket} not found");
    }

    let objects = self.list_dir(&path!(BUCKET_DIR, &bucket)).await?;
    if !objects.is_empty() {
      bail!(PRECONDITION_FAILED, "Bucket {bucket} is not empty");
    }

    self.delete_dir(&path!(BUCKET_DIR, &bucket)).await?;

    Ok(())
  }

  pub async fn list_buckets(&self) -> Result<Vec<String>> {
    let buckets = self.list_dir(&path!(BUCKET_DIR)).await?;
    Ok(buckets)
  }

  pub async fn put_object(&self, bucket: &String, object: &String, body: &Path) -> Result<()> {
    if !self.list_dir(&path!(BUCKET_DIR)).await?.contains(bucket) {
      bail!(NOT_FOUND, "Bucket {bucket} not found");
    }
    self
      .mv_file(body, &path!(BUCKET_DIR, bucket, object))
      .await?;

    Ok(())
  }

  pub async fn delete_object(&self, bucket: &String, object: &String) -> Result<()> {
    if !self.list_dir(&path!(BUCKET_DIR)).await?.contains(bucket) {
      bail!(NOT_FOUND, "Bucket {bucket} not found");
    }
    if !self
      .list_dir(&path!(BUCKET_DIR, bucket))
      .await?
      .contains(object)
    {
      bail!(NOT_FOUND, "Object {object} not found in bucket {bucket}");
    }

    self.delete_file(&path!(BUCKET_DIR, bucket, object)).await?;

    Ok(())
  }
}

impl Deref for S3Interface {
  type Target = dyn Storage + Send + Sync;

  fn deref(&self) -> &Self::Target {
    &*self.storage
  }
}

router_extension!(
  async fn interface(self, config: &EnvConfig) -> Self {
    self.layer(Extension(
      config
        .storage_type
        .storage(config.storage_path.clone())
        .await
        .expect("Failed to initialize storage"),
    ))
  }
);
