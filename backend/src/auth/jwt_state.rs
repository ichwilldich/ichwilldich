use std::sync::Arc;

use axum_extra::extract::cookie::{Cookie, SameSite};
use centaurus::{
  FromReqExtension,
  error::{ErrorReportStatusExt, Result},
};
use chrono::{Duration, Utc};
use http::StatusCode;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use rsa::{
  RsaPrivateKey, RsaPublicKey,
  pkcs1::{DecodeRsaPrivateKey, EncodeRsaPrivateKey, EncodeRsaPublicKey},
  pkcs8::LineEnding,
  rand_core::OsRng,
};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use tracing::{info, instrument};
use uuid::Uuid;

use crate::{
  auth::{
    jwt_auth::{AuthSource, COOKIE_NAME},
    pw_state::KEY_SIZE,
  },
  config::EnvConfig,
  db::Connection,
};

const JWT_KEY_NAME: &str = "jwt";

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct JwtClaims {
  pub exp: i64,
  pub iss: String,
  pub sub: String,
  pub r#type: AuthType,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum AuthType {
  Oidc,
  Internal,
}

#[derive(FromReqExtension, Clone)]
pub struct JwtState {
  header: Header,
  encoding_key: EncodingKey,
  decoding_key: DecodingKey,
  validation: Validation,
  iss: String,
  exp: i64,
}

impl JwtState {
  #[instrument(skip(self))]
  pub fn create_token<'c, T: AuthSource>(
    &self,
    uuid: T::UserID,
    r#type: AuthType,
  ) -> Result<Cookie<'c>> {
    let exp = Utc::now()
      .checked_add_signed(Duration::seconds(self.exp))
      .status_context(StatusCode::INTERNAL_SERVER_ERROR, "invalid timestamp")?
      .timestamp();

    let claims = JwtClaims {
      exp,
      iss: self.iss.clone(),
      sub: uuid.to_string(),
      r#type,
    };

    let token = encode(&self.header, &claims, &self.encoding_key)
      .status(StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(self.create_cookie(COOKIE_NAME, token))
  }

  #[instrument(skip(self))]
  pub fn create_cookie<'c>(&self, key: &'c str, token: String) -> Cookie<'c> {
    Cookie::build((key, token))
      .http_only(true)
      .max_age(time::Duration::seconds(self.exp))
      .same_site(SameSite::Lax)
      .secure(true)
      .path("/")
      .build()
  }

  #[instrument(skip(self))]
  pub fn validate_token(&self, token: &str) -> Result<JwtClaims> {
    let token_data = decode::<JwtClaims>(token, &self.decoding_key, &self.validation)?;

    Ok(token_data.claims)
  }

  #[instrument(skip(config, db))]
  pub async fn init(config: &EnvConfig, db: &Connection) -> Self {
    let (key, kid) = if let Ok(key) = db.key().get_key_by_name(JWT_KEY_NAME.into()).await {
      (key.private_key, key.id.to_string())
    } else {
      info!("Generating new RSA key for JWT with key size {KEY_SIZE}, this may take a while...");
      let mut rng = OsRng {};
      let private_key = RsaPrivateKey::new(&mut rng, KEY_SIZE).expect("Failed to create Rsa key");
      let key = private_key
        .to_pkcs1_pem(LineEnding::CRLF)
        .expect("Failed to export private key")
        .to_string();

      let uuid = Uuid::new_v4();

      db.key()
        .create_key(JWT_KEY_NAME.into(), key.clone(), uuid)
        .await
        .expect("Failed to save key");

      (key, uuid.to_string())
    };

    let private_key = RsaPrivateKey::from_pkcs1_pem(&key).expect("Failed to load public key");
    let public_key = RsaPublicKey::from(private_key);
    let public_key_pem = public_key
      .to_pkcs1_pem(LineEnding::CRLF)
      .expect("Failed to export public key");

    let mut header = Header::new(Algorithm::RS256);
    header.kid = Some(kid.clone());

    let encoding_key =
      EncodingKey::from_rsa_pem(key.as_bytes()).expect("Failed to create encoding key");
    let decoding_key =
      DecodingKey::from_rsa_pem(public_key_pem.as_bytes()).expect("Failed to create decoding key");

    let mut validation = Validation::new(Algorithm::RS256);
    validation.validate_aud = false;

    Self {
      header,
      encoding_key,
      decoding_key,
      validation,
      exp: config.auth.jwt_exp,
      iss: config.auth.jwt_iss.clone(),
    }
  }
}

#[derive(FromReqExtension, Clone, Default)]
pub struct JwtInvalidState {
  pub count: Arc<Mutex<i32>>,
}

#[cfg(test)]
mod test {
  use crate::auth::jwt_auth::InternalAuth;
  use crate::db::test::test_db;

  use super::*;

  #[tokio::test]
  async fn test_jwt_state() {
    let config = EnvConfig::default();
    let db = test_db().await;
    let jwt_state = JwtState::init(&config, &db).await;
    dbg!(&config.auth.jwt_exp);
    dbg!(&jwt_state.encoding_key);

    let user_id = Uuid::new_v4();
    let cookie = jwt_state
      .create_token::<InternalAuth>(user_id, AuthType::Internal)
      .unwrap();
    let token = cookie.value().to_string();
    let claims = jwt_state.validate_token(&token).unwrap();
    assert_eq!(claims.sub, user_id.to_string());
    assert_eq!(claims.iss, config.auth.jwt_iss);
    assert_eq!(claims.r#type, AuthType::Internal);
  }

  #[tokio::test]
  async fn test_jwt_state_invalid() {
    let config = EnvConfig::default();
    let db = test_db().await;
    let jwt_state = JwtState::init(&config, &db).await;
    let invalid_token = "invalid.token.here";
    let result = jwt_state.validate_token(invalid_token);
    assert!(result.is_err());
  }

  #[tokio::test]
  async fn test_cookie_creation() {
    let config = EnvConfig::default();
    let db = test_db().await;
    let jwt_state = JwtState::init(&config, &db).await;

    let cookie = jwt_state.create_cookie("test_key", "test_value".to_string());
    assert_eq!(cookie.name(), "test_key");
    assert_eq!(cookie.value(), "test_value");
    assert_eq!(cookie.http_only(), Some(true));
    assert_eq!(cookie.path(), Some("/"));
    assert_eq!(cookie.same_site(), Some(SameSite::Lax));
  }
}
