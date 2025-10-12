use std::path::PathBuf;

use centaurus::{FromReqExtension, config::BaseConfig};
use figment::{
  Figment,
  providers::{Env, Serialized},
};
use serde::{Deserialize, Serialize};
use tracing::instrument;
use url::Url;

use crate::s3::storage::StorageType;

#[derive(Deserialize, Serialize, Clone, FromReqExtension, Debug)]
pub struct EnvConfig {
  #[serde(flatten)]
  pub base: BaseConfig,
  #[serde(flatten)]
  pub db: DBConfig,
  #[serde(flatten)]
  pub auth: AuthConfig,

  pub base_url: Url,

  // storage
  pub storage_type: StorageType,
  pub storage_path: PathBuf,

  // s3
  pub s3_port: u16,

  pub metrics_enabled: bool,
  pub metrics_name: String,
  pub metrics_labels: Vec<(String, String)>,
}

impl EnvConfig {
  #[instrument]
  pub fn parse() -> Self {
    let config = Figment::new()
      .merge(Serialized::defaults(Self::default()))
      .merge(Env::raw().global());

    config.extract().expect("failed to load configuration")
  }
}

impl Default for EnvConfig {
  fn default() -> Self {
    Self {
      base: BaseConfig::default(),
      db: DBConfig::default(),
      auth: AuthConfig::default(),
      base_url: Url::parse("http://localhost:8080").unwrap(),
      storage_type: StorageType::NoRaid,
      storage_path: PathBuf::from("/data"),
      s3_port: 9000,
      metrics_enabled: true,
      metrics_name: "sagittarius".to_string(),
      metrics_labels: vec![],
    }
  }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct DBConfig {
  pub database_max_connections: u32,
  pub database_min_connections: u32,
  pub database_connect_timeout: u64,
  pub database_logging: bool,
}

impl Default for DBConfig {
  fn default() -> Self {
    Self {
      database_max_connections: 1024,
      database_min_connections: 1,
      database_connect_timeout: 5,
      database_logging: false,
    }
  }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct AuthConfig {
  // jwt
  pub jwt_iss: String,
  pub jwt_exp: i64,

  // auth
  pub auth_pepper: String,

  // initial user
  pub initial_user_username: String,
  pub initial_user_password: String,
  pub overwrite_initial_user: bool,
}

impl Default for AuthConfig {
  fn default() -> Self {
    Self {
      jwt_iss: "sagittarius".to_string(),
      jwt_exp: 604800,
      auth_pepper: "sagittarius_pepper_123456".to_string(),
      initial_user_username: "admin".to_string(),
      initial_user_password: "admin".to_string(),
      overwrite_initial_user: false,
    }
  }
}

#[cfg(test)]
mod test {
  use super::*;

  fn config() -> EnvConfig {
    unsafe {
      std::env::set_var("STORAGE_PATH", "/tmp/s3_");
    }

    EnvConfig::parse()
  }

  #[test]
  fn test_storage_type() {
    unsafe {
      std::env::set_var("STORAGE_TYPE", "NoRaid");
    }
    let cfg = config();
    assert_eq!(cfg.storage_type, StorageType::NoRaid);
  }

  #[test]
  fn test_storage_path() {
    let cfg = config();
    assert_eq!(cfg.storage_path, PathBuf::from("/tmp/s3_"));
  }

  #[test]
  fn test_s3_port() {
    unsafe {
      std::env::set_var("S3_PORT", "9010");
    }
    let cfg = config();
    assert_eq!(cfg.s3_port, 9010);
  }

  #[test]
  fn test_database_max_connections() {
    unsafe {
      std::env::set_var("DATABASE_MAX_CONNECTIONS", "1023");
    }
    let cfg = config();
    assert_eq!(cfg.db.database_max_connections, 1023);
  }

  #[test]
  fn test_database_min_connections() {
    unsafe {
      std::env::set_var("DATABASE_MIN_CONNECTIONS", "2");
    }
    let cfg = config();
    assert_eq!(cfg.db.database_min_connections, 2);
  }

  #[test]
  fn test_database_connect_timeout() {
    unsafe {
      std::env::set_var("DATABASE_CONNECT_TIMEOUT", "6");
    }
    let cfg = config();
    assert_eq!(cfg.db.database_connect_timeout, 6);
  }

  #[test]
  fn test_database_logging() {
    unsafe {
      std::env::set_var("DATABASE_LOGGING", "true");
    }
    let cfg = config();
    assert!(cfg.db.database_logging);
  }

  #[test]
  fn test_jwt_iss() {
    unsafe {
      std::env::set_var("JWT_ISS", "sagittarius_iss");
    }
    let cfg = config();
    assert_eq!(cfg.auth.jwt_iss, "sagittarius_iss");
  }

  #[test]
  fn test_jwt_exp() {
    unsafe {
      std::env::set_var("JWT_EXP", "604810");
    }
    let cfg = config();
    assert_eq!(cfg.auth.jwt_exp, 604810);
  }

  #[test]
  fn test_auth_pepper() {
    unsafe {
      std::env::set_var("AUTH_PEPPER", "_my_pepper__22_123");
    }
    let cfg = config();
    assert_eq!(cfg.auth.auth_pepper, "_my_pepper__22_123");
  }

  #[test]
  fn test_initial_user_username() {
    unsafe {
      std::env::set_var("INITIAL_USER_USERNAME", "admin123");
    }
    let cfg = config();
    assert_eq!(cfg.auth.initial_user_username, "admin123");
  }

  #[test]
  fn test_initial_user_password() {
    unsafe {
      std::env::set_var("INITIAL_USER_PASSWORD", "admin123");
    }
    let cfg = config();
    assert_eq!(cfg.auth.initial_user_password, "admin123");
  }

  #[test]
  fn test_overwrite_initial_user() {
    unsafe {
      std::env::set_var("OVERWRITE_INITIAL_USER", "true");
    }
    let cfg = config();
    assert!(cfg.auth.overwrite_initial_user);
  }
}
