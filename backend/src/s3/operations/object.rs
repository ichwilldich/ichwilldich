use axum::{
  Router,
  extract::Path,
  routing::{delete, put},
};
use centaurus::{error::Result, path};
use http::HeaderMap;

use crate::s3::{
  BUCKET_DIR,
  auth::{Identity, S3Auth, body::TmpFile},
  interface::S3Interface,
};

pub fn router() -> Router {
  Router::new()
    .route("/{bucket}/{*object}", put(put_object))
    .route("/{bucket}/{*object}", delete(delete_object))
}

async fn put_object(
  interface: S3Interface,
  Path((bucket, object)): Path<(String, String)>,
  S3Auth { identity, body, .. }: S3Auth<TmpFile>,
) -> Result<HeaderMap> {
  match identity {
    Identity::AccessKey(key) => {
      tracing::info!("AccessKey {key} putting object {object}");
    }
    Identity::Anonymous => {
      tracing::warn!("Anonymous access to put object");
    }
  }

  tracing::info!(
    "put_object handler triggered: bucket={} object={}",
    bucket,
    object
  );

  interface.put_object(&bucket, &object, &body.0).await?;

  let headers = HeaderMap::new();

  Ok(headers)
}

async fn delete_object(
  interface: S3Interface,
  Path((bucket, object)): Path<(String, String)>,
  S3Auth { identity, .. }: S3Auth<()>,
) -> Result<HeaderMap> {
  match identity {
    Identity::AccessKey(key) => {
      tracing::info!("AccessKey {key} deleting object {object}");
    }
    Identity::Anonymous => {
      tracing::warn!("Anonymous access to delete object");
    }
  }

  tracing::info!(
    "delete_object handler triggered: bucket={} object={}",
    bucket,
    object
  );

  interface
    .delete_file(&path!(BUCKET_DIR, &bucket, &object))
    .await?;

  let headers = HeaderMap::new();

  Ok(headers)
}
