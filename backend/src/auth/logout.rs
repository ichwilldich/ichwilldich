use axum::{Router, routing::post};
use axum_extra::extract::{CookieJar, cookie::Cookie};
use centaurus::error::Result;
use chrono::DateTime;
use eyre::ContextCompat;
use tracing::instrument;

use crate::{
  auth::{
    jwt_auth::{COOKIE_NAME, JwtAuth},
    jwt_state::JwtInvalidState,
    res::TokenRes,
  },
  db::Connection,
};

pub fn router() -> Router {
  Router::new().route("/logout", post(logout))
}

#[instrument(skip(auth, db, state, cookies))]
async fn logout(
  auth: JwtAuth,
  db: Connection,
  mut cookies: CookieJar,
  state: JwtInvalidState,
) -> Result<(CookieJar, TokenRes)> {
  let cookie = cookies.get(COOKIE_NAME).context("Missing auth cookie")?;
  let mut count = state.count.lock().await;

  db.invalid_jwt()
    .invalidate_jwt(
      cookie.value().to_string(),
      DateTime::from_timestamp(auth.exp, 0).context("invalid timestamp")?,
      &mut count,
    )
    .await?;

  cookies = cookies.remove(Cookie::from(COOKIE_NAME));

  Ok((cookies, TokenRes))
}
