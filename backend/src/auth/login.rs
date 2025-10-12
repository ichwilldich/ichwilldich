use axum::{
  Json, Router,
  routing::{get, post},
};
use axum_extra::extract::CookieJar;
use centaurus::{bail, error::Result};
use serde::{Deserialize, Serialize};
use tracing::{debug, instrument};

use crate::{
  auth::{
    jwt_auth::InternalAuth,
    jwt_state::{AuthType, JwtState},
    pw_state::PasswordState,
    res::TokenRes,
  },
  db::Connection,
};

pub fn router() -> Router {
  Router::new()
    .route("/key", get(key))
    .route("/auth", post(authenticate))
}

#[derive(Serialize, Deserialize)]
struct KeyRes {
  key: String,
}

async fn key(state: PasswordState) -> Json<KeyRes> {
  Json(KeyRes { key: state.pub_key })
}

#[derive(Deserialize, Serialize, Debug)]
struct LoginReq {
  name: String,
  password: String,
}

#[instrument(skip(state, jwt, db, payload, cookies), fields(name = %payload.name))]
async fn authenticate(
  state: PasswordState,
  jwt: JwtState,
  db: Connection,
  mut cookies: CookieJar,
  Json(payload): Json<LoginReq>,
) -> Result<(CookieJar, TokenRes)> {
  let user = db.user().get_user_by_name(payload.name).await?;
  let hash = state.pw_hash(&user.salt, &payload.password)?;

  if hash != user.password {
    debug!("Invalid password for user: {}", user.id);
    bail!(UNAUTHORIZED, "Invalid credentials");
  }

  debug!("User authenticated: {}", user.id);
  cookies = cookies.add(jwt.create_token::<InternalAuth>(user.id, AuthType::Internal)?);

  Ok((cookies, TokenRes))
}
