use std::{
  collections::{HashMap, HashSet},
  sync::Arc,
};

use axum::{
  Json, Router,
  extract::Query,
  response::{IntoResponse, Response},
  routing::get,
};
use axum_extra::extract::{CookieJar, cookie::Cookie};
use centaurus::{
  FromReqExtension, bail,
  error::{ErrorReportStatusExt, Result},
};
use http::{StatusCode, header::LOCATION};
use jsonwebtoken::{
  DecodingKey, Validation,
  jwk::{AlgorithmParameters, JwkSet},
};
use reqwest::{Client, redirect::Policy};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use tracing::{debug, info, instrument};
use url::Url;
use uuid::Uuid;

use crate::{
  auth::{
    jwt_auth::AllAuth,
    jwt_state::{AuthType, JwtState},
  },
  config::{AppConfig, EnvConfig},
};

pub const OIDC_STATE: &str = "oidc_state";

pub fn router() -> Router {
  Router::new()
    .route("/oidc_url", get(oidc_url))
    .route("/oidc_callback", get(oidc_callback))
}

#[derive(Clone, FromReqExtension, Debug)]
pub struct OidcState {
  pub(super) config: Option<OidcConfig>,
}

#[derive(Clone, Debug)]
pub(super) struct OidcConfig {
  state: Arc<Mutex<HashSet<Uuid>>>,
  nonce: Arc<Mutex<HashSet<Uuid>>>,
  issuer: String,
  authorization_endpoint: Url,
  token_endpoint: Url,
  userinfo_endpoint: Url,
  jwk_set: JwkSet,
  client_id: String,
  client_secret: String,
  client: Client,
  scope: Option<String>,
}

#[derive(Deserialize, Debug)]
struct OidcConfiguration {
  issuer: String,
  authorization_endpoint: Url,
  token_endpoint: Url,
  userinfo_endpoint: Url,
  jwks_uri: Url,
}

impl OidcState {
  #[instrument]
  pub async fn new(app_config: &AppConfig) -> Result<Self> {
    let config = if let Some(url) = app_config.config.oidc.oidc_url.value()
      && let Some(client_id) = app_config.config.oidc.oidc_client_id.value()
      && let Some(client_secret) = app_config.config.oidc.oidc_client_secret.value()
    {
      info!("Configuring OIDC with URL: {}", url);
      let res = reqwest::get(url.clone()).await?;
      if !res.status().is_success() {
        let body = res.text().await.unwrap_or_default();
        bail!(
          "Failed to retrieve OIDC configuration from {}: {}",
          url,
          body
        );
      }
      let config: OidcConfiguration = res.json().await?;

      info!("Retrieving JWKs from: {}", config.jwks_uri);
      let res = reqwest::get(config.jwks_uri.clone()).await?;
      if !res.status().is_success() {
        let body = res.text().await.unwrap_or_default();
        bail!("Failed to retrieve JWKs from {}: {}", config.jwks_uri, body);
      }
      let jwk_set: JwkSet = res.json().await?;

      let client = Client::builder().redirect(Policy::none()).build()?;

      Some(OidcConfig {
        state: Default::default(),
        nonce: Default::default(),
        issuer: config.issuer,
        authorization_endpoint: config.authorization_endpoint,
        token_endpoint: config.token_endpoint,
        userinfo_endpoint: config.userinfo_endpoint,
        jwk_set,
        client_id: client_id.clone(),
        client_secret: client_secret.clone(),
        client,
        scope: app_config.config.oidc.oidc_scope.value().cloned(),
      })
    } else {
      None
    };

    Ok(Self { config })
  }
}

impl OidcConfig {
  #[instrument(skip(self))]
  async fn validate_jwk(&self, token: &str) -> Result<()> {
    let header = jsonwebtoken::decode_header(token)?;

    let Some(kid) = header.kid else {
      bail!(INTERNAL_SERVER_ERROR, "Missing kid in JWK header");
    };

    let Some(jwk) = self.jwk_set.find(&kid) else {
      bail!(INTERNAL_SERVER_ERROR, "JWK not found");
    };

    let decoding_key = match &jwk.algorithm {
      AlgorithmParameters::RSA(rsa) => DecodingKey::from_rsa_components(&rsa.n, &rsa.e)
        .status(StatusCode::INTERNAL_SERVER_ERROR)?,
      _ => {
        bail!(INTERNAL_SERVER_ERROR, "Unsupported JWK algorithm");
      }
    };

    let validation = {
      let mut validation = Validation::new(header.alg);
      validation.set_audience(&[self.client_id.to_string()]);
      validation.set_issuer(&[&self.issuer]);
      validation.validate_exp = false;
      validation
    };

    let data = jsonwebtoken::decode::<HashMap<String, serde_json::Value>>(
      token,
      &decoding_key,
      &validation,
    )?;

    let Some(Some(Ok(nonce))) = data
      .claims
      .get("nonce")
      .map(|nonce| nonce.as_str().map(|nonce| nonce.parse()))
    else {
      bail!(INTERNAL_SERVER_ERROR, "Missing nonce in JWK claims");
    };
    if !self.nonce.lock().await.remove(&nonce) {
      bail!(INTERNAL_SERVER_ERROR, "Invalid nonce");
    }

    Ok(())
  }
}

#[derive(Serialize)]
struct OidcResponse {
  url: String,
}

#[instrument(skip(jwt, state, cookies))]
async fn oidc_url(
  state: OidcState,
  jwt: JwtState,
  mut cookies: CookieJar,
) -> Result<(CookieJar, Json<OidcResponse>)> {
  if let Some(config) = &state.config {
    let state = Uuid::new_v4();
    let nonce = Uuid::new_v4();

    let mut form = HashMap::new();
    form.insert("response_type", "code".to_string());
    form.insert("client_id", config.client_id.clone());
    form.insert("state", state.to_string());
    form.insert("nonce", nonce.to_string());

    if let Some(scope) = &config.scope {
      form.insert("scope", scope.clone());
    }

    let req = config
      .client
      .post(config.authorization_endpoint.clone())
      .form(&form)
      .build()?;

    let res = config.client.execute(req).await?;

    if !res.status().is_redirection() {
      let body = res.text().await.unwrap_or_default();
      bail!(
        INTERNAL_SERVER_ERROR,
        "OIDC authorization request failed: {}",
        body
      );
    }
    let Some(location) = res.headers().get(LOCATION).and_then(|h| h.to_str().ok()) else {
      bail!(
        INTERNAL_SERVER_ERROR,
        "OIDC authorization response missing location header"
      );
    };

    config.state.lock().await.insert(state);
    cookies = cookies.add(jwt.create_cookie(OIDC_STATE, state.to_string()));

    config.nonce.lock().await.insert(nonce);

    Ok((
      cookies,
      Json(OidcResponse {
        url: location.to_string(),
      }),
    ))
  } else {
    bail!(BAD_REQUEST, "OIDC not configured");
  }
}

#[derive(Deserialize)]
struct OidcCallbackQuery {
  code: Option<String>,
  state: Uuid,
  error: Option<String>,
}

#[derive(Debug)]
struct Redirect {
  location: Url,
}

impl IntoResponse for Redirect {
  #[instrument]
  fn into_response(self) -> Response {
    (StatusCode::FOUND, [(LOCATION, self.location.to_string())]).into_response()
  }
}

#[derive(Deserialize)]
struct TokenRes {
  id_token: String,
}

#[derive(Deserialize)]
pub struct AuthInfo {
  pub sub: String,
}

#[instrument(skip(jwt, config, oidc_state, cookies))]
async fn oidc_callback(
  Query(OidcCallbackQuery { code, state, error }): Query<OidcCallbackQuery>,
  oidc_state: OidcState,
  mut cookies: CookieJar,
  config: EnvConfig,
  jwt: JwtState,
) -> Result<(CookieJar, Redirect)> {
  let (path, error) = if let Some(config) = &oidc_state.config {
    if !config.state.lock().await.remove(&state) {
      bail!(BAD_REQUEST, "Invalid OIDC state");
    }
    let Some(cookie) = cookies.get(OIDC_STATE) else {
      bail!(BAD_REQUEST, "Missing OIDC state cookie");
    };
    if cookie.value() != state.to_string() {
      bail!(BAD_REQUEST, "OIDC state mismatch");
    }
    cookies = cookies.remove(Cookie::from(OIDC_STATE));

    if let Some(error) = error {
      ("/login", Some(error))
    } else if let Some(code) = code {
      let mut form = HashMap::new();
      form.insert("grant_type", "authorization_code".to_string());
      form.insert("code", code);

      let req = config
        .client
        .post(config.token_endpoint.clone())
        .basic_auth(config.client_id.clone(), Some(config.client_secret.clone()))
        .form(&form)
        .build()?;

      let res = config.client.execute(req).await?;
      if !res.status().is_success() {
        let body = res.text().await.unwrap_or_default();
        bail!(INTERNAL_SERVER_ERROR, "OIDC token request failed: {}", body);
      }

      let res: TokenRes = res.json().await?;
      config.validate_jwk(&res.id_token).await?;

      let req = config
        .client
        .get(config.userinfo_endpoint.clone())
        .bearer_auth(res.id_token)
        .build()?;

      let res = config.client.execute(req).await?;
      if !res.status().is_success() {
        let body = res.text().await.unwrap_or_default();
        bail!(
          INTERNAL_SERVER_ERROR,
          "OIDC userinfo request failed: {}",
          body
        );
      }
      let res: AuthInfo = res.json().await?;

      debug!("OIDC user authenticated: {}", res.sub);
      cookies = cookies.add(jwt.create_token::<AllAuth>(res.sub, AuthType::Oidc)?);

      ("/", None)
    } else {
      ("/login", Some("missing_code".to_string()))
    }
  } else {
    ("/login", Some("oidc_not_configured".to_string()))
  };

  let mut url = config.base_url;
  url.set_path(path);
  url.set_query(error.map(|e| format!("error={e}")).as_deref());

  Ok((cookies, Redirect { location: url }))
}
