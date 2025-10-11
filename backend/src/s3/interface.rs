use axum::Extension;
use centaurus::{FromReqExtension, bail, error::Result, path};
use tracing::instrument;

use crate::{config::EnvConfig, macros::DualRouterExt, router_extension, s3::BUCKET_DIR};
use std::{ops::Deref, sync::Arc};

use crate::s3::storage::Storage;

#[derive(Clone, FromReqExtension, Debug)]
pub struct S3Interface {
  storage: Arc<dyn Storage + Send + Sync>,
}

impl S3Interface {
  pub fn new<S: Storage + Send + Sync + 'static>(storage: S) -> Self {
    Self {
      storage: Arc::new(storage),
    }
  }

  #[instrument]
  pub async fn create_bucket(&self, bucket: &String) -> Result<()> {
    if self.list_dir(&path!(BUCKET_DIR)).await?.contains(bucket) {
      bail!(CONFLICT, "Bucket {bucket} already exists");
    }

    self.create_dir(&path!(BUCKET_DIR, &bucket)).await?;

    Ok(())
  }

  #[instrument]
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

  #[instrument]
  pub async fn list_buckets(&self) -> Result<Vec<String>> {
    let buckets = self.list_dir(&path!(BUCKET_DIR)).await?;
    Ok(buckets)
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
