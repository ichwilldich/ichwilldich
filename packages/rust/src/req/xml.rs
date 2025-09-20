use axum::{
  extract::{FromRequest, OptionalFromRequest, Request, rejection::StringRejection},
  response::IntoResponse,
};
use http::StatusCode;
use serde::Deserialize;
use thiserror::Error;

pub struct Xml<T: for<'de> Deserialize<'de>>(pub T);

#[derive(Error, Debug)]
pub enum XmlRejection {
  #[error(transparent)]
  StringRejection(#[from] StringRejection),
  #[error(transparent)]
  InvalidXml(#[from] serde_xml_rs::Error),
}

impl IntoResponse for XmlRejection {
  fn into_response(self) -> axum::response::Response {
    match self {
      XmlRejection::StringRejection(rej) => rej.into_response(),
      XmlRejection::InvalidXml(_) => (StatusCode::BAD_REQUEST).into_response(),
    }
  }
}

impl<T, S: Send + Sync> FromRequest<S> for Xml<T>
where
  T: for<'de> Deserialize<'de>,
{
  type Rejection = XmlRejection;

  async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
    let string = String::from_request(req, state).await?;
    Ok(Xml(serde_xml_rs::from_str(&string)?))
  }
}

impl<T, S: Send + Sync> OptionalFromRequest<S> for Xml<T>
where
  T: for<'de> Deserialize<'de>,
{
  type Rejection = XmlRejection;

  async fn from_request(req: Request, state: &S) -> Result<Option<Self>, Self::Rejection> {
    let string = String::from_request(req, state).await?;
    match serde_xml_rs::from_str(&string) {
      Ok(xml) => Ok(Some(Xml(xml))),
      Err(_) => Ok(None),
    }
  }
}
