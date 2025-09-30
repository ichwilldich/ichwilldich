use axum::{
  Json, Router,
  routing::{get, post},
};
use axum_extra::extract::CookieJar;
use centaurus::{bail, error::Result};
use serde::{Deserialize, Serialize};

use crate::{
  auth::{jwt_state::JwtState, pw_state::PasswordState, res::TokenRes},
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

#[derive(Deserialize, Serialize)]
struct LoginReq {
  name: String,
  password: String,
}

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
    bail!(UNAUTHORIZED, "Invalid credentials");
  }

  cookies = cookies.add(jwt.create_token(user.id)?);

  Ok((cookies, TokenRes))
}
