use axum::{Extension, Json, Router, routing::get};
use serde::Serialize;
use tracing::instrument;

use crate::{
  auth::{
    jwt_state::{JwtInvalidState, JwtState},
    oidc::OidcState,
    pw_state::PasswordState,
  },
  config::AppConfig,
  db::Connection,
  macros::DualRouterExt,
  router_extension,
};

pub mod jwt_auth;
mod jwt_state;
mod login;
mod logout;
mod oidc;
mod pw_state;
mod res;
mod test_token;

pub fn router() -> Router {
  test_token::router()
    .merge(login::router())
    .merge(logout::router())
    .merge(oidc::router())
    .route("/sso_config", get(get_sso_type))
}

router_extension!(
  async fn auth(
    self,
    config: &crate::config::EnvConfig,
    app_config: &AppConfig,
    db: &Connection,
  ) -> Self {
    self
      .layer(Extension(JwtState::init(config, db).await))
      .layer(Extension(PasswordState::init(config, db).await))
      .layer(Extension(JwtInvalidState::default()))
      .layer(Extension(
        OidcState::new(app_config)
          .await
          .expect("Failed to initialize OIDC state"),
      ))
  }
);

#[derive(Serialize)]
enum SSOType {
  Oidc,
  None,
}

#[derive(Serialize)]
struct SSOConfig {
  sso_type: SSOType,
  instant_redirect: bool,
}

#[instrument]
async fn get_sso_type(oidc: OidcState, config: AppConfig) -> Json<SSOConfig> {
  let sso_type = if oidc.config.is_some() {
    SSOType::Oidc
  } else {
    SSOType::None
  };

  Json(SSOConfig {
    sso_type,
    instant_redirect: config
      .config
      .oidc
      .sso_instant_redirect
      .value()
      .cloned()
      .unwrap_or(true),
  })
}
