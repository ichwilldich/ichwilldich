use axum::{
  extract::rejection::BytesRejection,
  response::{IntoResponse, Response},
};
use axum_extra::typed_header::TypedHeaderRejection;
use hmac::digest::InvalidLength;
use http::StatusCode;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
  #[error("BadRequest")]
  BadRequest,
  #[error("Unauthorized")]
  Unauthorized,
  #[allow(clippy::enum_variant_names)]
  #[error("InternalServerError")]
  InternalServerError,
  #[error("Conflict")]
  Conflict,
  #[error("Gone")]
  Gone,
  #[error(transparent)]
  IO(#[from] std::io::Error),
  #[error(transparent)]
  InvalidHeaderValue(#[from] http::header::InvalidHeaderValue),
  #[error(transparent)]
  TypedHeader(#[from] TypedHeaderRejection),
  #[error(transparent)]
  Bytes(#[from] BytesRejection),
  #[error(transparent)]
  InvalidLength(#[from] InvalidLength),
}

impl IntoResponse for Error {
  fn into_response(self) -> Response {
    tracing::error!("{:?}", &self);
    match self {
      Self::Unauthorized => StatusCode::UNAUTHORIZED.into_response(),
      Self::Conflict => StatusCode::CONFLICT.into_response(),
      Self::Gone => StatusCode::GONE.into_response(),
      Self::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
      _ => StatusCode::BAD_REQUEST.into_response(),
    }
  }
}
