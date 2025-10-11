use std::{fmt::Debug, marker::PhantomData, str::FromStr};

use axum::{
  RequestPartsExt,
  extract::{FromRequestParts, OptionalFromRequestParts},
};
use axum_extra::{
  TypedHeader,
  extract::CookieJar,
  headers::{Authorization, authorization::Bearer},
};
use centaurus::{bail, error::ErrorReport};
use http::request::Parts;
use tracing::instrument;
use uuid::Uuid;

use crate::{
  auth::jwt_state::{AuthType, JwtState},
  db::Connection,
};

pub const COOKIE_NAME: &str = "auth_token";

pub trait AuthSource: Debug {
  type UserID: FromStr + ToString + Debug;

  fn internal() -> bool {
    false
  }
}

#[derive(Debug)]
pub struct AllAuth;
impl AuthSource for AllAuth {
  type UserID = String;
}

#[derive(Debug)]
pub struct InternalAuth;
impl AuthSource for InternalAuth {
  type UserID = Uuid;

  fn internal() -> bool {
    true
  }
}

#[derive(Debug)]
pub struct JwtAuth<T: AuthSource = AllAuth> {
  #[allow(unused)]
  pub user_id: T::UserID,
  pub exp: i64,
  _m: PhantomData<T>,
}

impl<S: Sync, T: AuthSource> FromRequestParts<S> for JwtAuth<T> {
  type Rejection = ErrorReport;

  #[instrument(skip(_state))]
  async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
    let bearer = parts.extract::<TypedHeader<Authorization<Bearer>>>().await;

    let token = match bearer {
      Ok(TypedHeader(Authorization(bearer))) => bearer.token().to_string(),
      Err(_) => {
        let Ok(cookie) = parts.extract::<CookieJar>().await;
        match cookie.get(COOKIE_NAME) {
          Some(c) => c.value().to_string(),
          None => bail!("no auth token found"),
        }
      }
    };

    let Ok(state) = parts.extract::<JwtState>().await;
    let Ok(db) = parts.extract::<Connection>().await;

    let Ok(valid) = db.invalid_jwt().is_token_valid(&token).await else {
      bail!("failed to validate jwt");
    };
    if !valid {
      bail!(UNAUTHORIZED, "token is invalidated");
    }

    let Ok(claims) = state.validate_token(&token) else {
      bail!(UNAUTHORIZED, "invalid token");
    };
    let Ok(user_id) = T::UserID::from_str(&claims.sub) else {
      bail!(UNAUTHORIZED, "invalid user id in token");
    };

    if T::internal() && claims.r#type != AuthType::Internal {
      bail!(UNAUTHORIZED, "token is not for internal use");
    }

    Ok(JwtAuth {
      user_id,
      exp: claims.exp,
      _m: PhantomData,
    })
  }
}

impl<S: Sync> OptionalFromRequestParts<S> for JwtAuth {
  type Rejection = ErrorReport;

  #[instrument(skip(state))]
  async fn from_request_parts(
    parts: &mut Parts,
    state: &S,
  ) -> Result<Option<Self>, Self::Rejection> {
    match <Self as FromRequestParts<S>>::from_request_parts(parts, state).await {
      Ok(auth) => Ok(Some(auth)),
      Err(_) => Ok(None),
    }
  }
}

#[cfg(test)]
mod test {
  use axum::{
    body::Body,
    extract::{FromRequest, Request},
  };
  use chrono::{Duration, Utc};
  use http::request::Builder;

  use super::*;
  use crate::{config::EnvConfig, db::test::test_db};

  async fn req_builder() -> Builder {
    let config = EnvConfig::default();
    let db = test_db().await;
    let jwt_state = JwtState::init(&config, &db).await;

    Request::builder().extension(jwt_state).extension(db)
  }

  #[tokio::test]
  async fn test_jwt_auth_missing() {
    let req = req_builder().await.body(Body::empty()).unwrap();
    let auth = JwtAuth::<AllAuth>::from_request(req, &()).await;
    assert!(auth.is_err());
  }

  #[tokio::test]
  async fn test_jwt_auth_invalid() {
    let req = req_builder()
      .await
      .header("Authorization", "Bearer invalid.token.here")
      .body(Body::empty())
      .unwrap();
    let auth = JwtAuth::<AllAuth>::from_request(req, &()).await;
    assert!(auth.is_err());
  }

  async fn test_jwt_auth_valid_generic<In: AuthSource, Out: AuthSource>(
    id: In::UserID,
    r#type: AuthType,
  ) where
    In::UserID: Clone,
  {
    let config = EnvConfig::default();
    let db = test_db().await;
    let jwt_state = JwtState::init(&config, &db).await;

    let cookie = jwt_state.create_token::<In>(id.clone(), r#type).unwrap();
    let token = cookie.value().to_string();

    let req = Request::builder()
      .extension(jwt_state)
      .extension(db)
      .header("Authorization", format!("Bearer {}", token))
      .body(Body::empty())
      .unwrap();
    let auth = JwtAuth::<Out>::from_request(req, &()).await;
    assert!(auth.is_ok());
    let auth = auth.unwrap();
    assert_eq!(auth.user_id.to_string(), id.to_string());
  }

  #[tokio::test]
  async fn test_jwt_auth_valid_internal() {
    let user_id = Uuid::new_v4();
    test_jwt_auth_valid_generic::<InternalAuth, InternalAuth>(user_id, AuthType::Internal).await;
  }

  #[tokio::test]
  async fn test_jwt_auth_valid_internal_on_all() {
    let user_id = Uuid::new_v4();
    test_jwt_auth_valid_generic::<InternalAuth, AllAuth>(user_id, AuthType::Internal).await;
  }

  #[tokio::test]
  async fn test_jwt_auth_valid_all() {
    let user_id = Uuid::new_v4().to_string();
    test_jwt_auth_valid_generic::<AllAuth, AllAuth>(user_id.clone(), AuthType::Oidc).await;
  }

  #[tokio::test]
  #[should_panic]
  async fn test_jwt_auth_invalid_all_on_internal() {
    let user_id = Uuid::new_v4().to_string();
    test_jwt_auth_valid_generic::<AllAuth, InternalAuth>(user_id.clone(), AuthType::Oidc).await;
  }

  #[tokio::test]
  async fn test_invalidated_jwt() {
    let config = EnvConfig::default();
    let db = test_db().await;
    let jwt_state = JwtState::init(&config, &db).await;

    let user_id = Uuid::new_v4();
    let cookie = jwt_state
      .create_token::<InternalAuth>(user_id, AuthType::Oidc)
      .unwrap();
    let token = cookie.value().to_string();
    let exp = Utc::now() + Duration::seconds(config.auth.jwt_exp);

    // invalidate the token
    db.invalid_jwt()
      .invalidate_jwt(token.clone(), exp, &mut 0)
      .await
      .expect("failed to invalidate token");

    let req = Request::builder()
      .extension(jwt_state)
      .extension(db)
      .header("Authorization", format!("Bearer {}", token))
      .body(Body::empty())
      .unwrap();
    let auth = JwtAuth::<InternalAuth>::from_request(req, &()).await;
    assert!(auth.is_err());
  }
}
