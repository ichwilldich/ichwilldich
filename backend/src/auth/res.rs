use axum::{
  body::Body,
  response::{IntoResponse, Response},
};
use http::header::{CACHE_CONTROL, PRAGMA};
use tracing::instrument;

#[derive(Clone, Copy, Debug)]
pub struct TokenRes;

impl IntoResponse for TokenRes {
  #[instrument]
  fn into_response(self) -> Response {
    Response::builder()
      .header(CACHE_CONTROL, "no-store")
      .header(PRAGMA, "no-cache")
      .body(Body::empty())
      .unwrap()
  }
}

#[cfg(test)]
mod test {
  use super::*;

  #[tokio::test]
  async fn test_token_res() {
    let token_res = TokenRes;
    let response = token_res.into_response();
    assert_eq!(
      response
        .headers()
        .get(CACHE_CONTROL)
        .unwrap()
        .to_str()
        .unwrap(),
      "no-store"
    );
    assert_eq!(
      response.headers().get(PRAGMA).unwrap().to_str().unwrap(),
      "no-cache"
    );
  }
}
