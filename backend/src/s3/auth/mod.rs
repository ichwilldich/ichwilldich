use std::collections::HashMap;

use axum::{
  RequestPartsExt,
  extract::{FromRequest, Request},
};
use axum_extra::{
  TypedHeader,
  headers::{ContentType, Mime},
};
use centaurus::error::ErrorReport;
use http::Method;
use tracing::instrument;

use crate::s3::auth::{
  body::Body, header::header_auth, multipart::multipart_auth, query::query_auth,
};

pub mod body;
mod credential;
mod header;
mod multipart;
mod query;
mod sig_v4;

/// TODO: DB: credential lookup, region check
/// https://docs.aws.amazon.com/AmazonS3/latest/API/sig-v4-header-based-auth.html
pub struct S3Auth<T: Body = ()> {
  pub identity: Identity,
  #[allow(unused)]
  pub body: T,
  #[allow(unused)]
  /// TODO: currently only used for multipart uploads which are not implemented yet
  pub additional: Option<HashMap<String, String>>,
}

#[derive(Debug, PartialEq)]
pub enum Identity {
  AccessKey(String),
  Anonymous,
}

const SECRET: &str = "secret";

impl<S: Sync + Send, T: Body> FromRequest<S> for S3Auth<T> {
  type Rejection = ErrorReport;

  #[instrument(skip(_state))]
  async fn from_request(req: Request, _state: &S) -> std::result::Result<Self, Self::Rejection> {
    let (mut req, body) = req.into_parts();
    if req.method == Method::POST
      && let Ok(TypedHeader(mime)) = req.extract::<TypedHeader<ContentType>>().await
    {
      let mime: Mime = mime.into();
      if mime.type_() == mime::MULTIPART && mime.subtype() == mime::FORM_DATA {
        return multipart_auth(Request::from_parts(req, body)).await;
      }
    }

    let query: Vec<(String, String)> =
      serde_urlencoded::from_str(req.uri.query().unwrap_or("")).unwrap_or_default();
    if query.iter().any(|(k, _)| k == "X-Amz-Signature") {
      return query_auth(Request::from_parts(req, body), &query).await;
    }

    header_auth(Request::from_parts(req, body)).await
  }
}
