use axum::{
  Router,
  extract::{Path, Query},
  routing::{delete, get, put},
};
use centaurus::{bail, error::Result, req::xml::Xml};
use http::{HeaderMap, StatusCode};
use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::s3::{
  auth::{Identity, S3Auth},
  interface::S3Interface,
};

pub fn router() -> Router {
  Router::new()
    .route("/{bucket}", put(create_bucket))
    .route("/{bucket}/", put(create_bucket))
    .route("/{bucket}", delete(delete_bucket))
    .route("/{bucket}/", delete(delete_bucket))
    .route("/", get(list_buckets))
}

/// TODO: Handling of additional header options
#[instrument]
async fn create_bucket(
  interface: S3Interface,
  Path(bucket): Path<String>,
  S3Auth { identity, .. }: S3Auth<Option<Xml<CreateBucketConfiguration>>>,
) -> Result<HeaderMap> {
  match identity {
    Identity::AccessKey(key) => {
      tracing::info!("AccessKey {key} creating bucket {bucket}");
    }
    Identity::Anonymous => {
      tracing::warn!("Anonymous access to create bucket");
    }
  }

  interface.create_bucket(&bucket).await?;

  let mut headers = HeaderMap::new();
  headers.insert("Location", format!("/{bucket}").parse()?);

  Ok(headers)
}

/// TODO: Handling of additional configuration options
#[derive(Deserialize, Debug)]
struct CreateBucketConfiguration {}

#[instrument]
async fn delete_bucket(
  interface: S3Interface,
  Path(bucket): Path<String>,
  S3Auth { identity, .. }: S3Auth,
) -> Result<StatusCode> {
  match identity {
    Identity::AccessKey(key) => {
      tracing::info!("AccessKey {key} deleting bucket {bucket}");
    }
    Identity::Anonymous => {
      tracing::warn!("Anonymous access to delete bucket");
    }
  }

  interface.delete_bucket(&bucket).await?;

  Ok(StatusCode::NO_CONTENT)
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
struct ListQuery {
  prefix: Option<String>,
  max_buckets: Option<usize>,
}

#[instrument]
async fn list_buckets(
  interface: S3Interface,
  Query(ListQuery {
    prefix,
    max_buckets,
  }): Query<ListQuery>,
  S3Auth { identity, .. }: S3Auth,
) -> Result<Xml<ListAllMyBucketsResult>> {
  match identity {
    Identity::AccessKey(key) => {
      tracing::info!("AccessKey {key} listing buckets");
    }
    Identity::Anonymous => {
      tracing::warn!("Anonymous access to list buckets");
    }
  }

  let buckets = interface.list_buckets().await?;

  let buckets: Vec<String> = if let Some(prefix) = prefix.clone() {
    buckets
      .into_iter()
      .filter(|b| b.starts_with(&prefix))
      .collect()
  } else {
    buckets
  };

  let buckets: Vec<String> = if let Some(max) = max_buckets {
    if !(1..=10000).contains(&max) {
      bail!(BAD_REQUEST, "max-buckets must be between 1 and 10000");
    }

    buckets.into_iter().take(max).collect()
  } else {
    buckets
  };

  let buckets: Vec<Bucket> = buckets
    .into_iter()
    .map(|name| Bucket {
      name,
      creation_date: "".into(), // TODO: fill with actual creation date if available
      bucket_region: None,
    })
    .collect();

  Ok(Xml(ListAllMyBucketsResult {
    buckets: Buckets { buckets },
    prefix,
    owner: Owner {
      id: "owner-id".into(),
      display_name: "owner-display-name".into(),
    },
  }))
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct ListAllMyBucketsResult {
  buckets: Buckets,
  prefix: Option<String>,
  owner: Owner,
}

#[derive(Serialize, Deserialize, Debug)]
struct Buckets {
  #[serde(rename = "Bucket")]
  buckets: Vec<Bucket>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
struct Bucket {
  name: String,
  creation_date: String,
  bucket_region: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct Owner {
  #[serde(rename = "ID")]
  id: String,
  display_name: String,
}
