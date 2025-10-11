use std::{ops::Deref, time::Duration};

use centaurus::FromReqExtension;
use migration::MigratorTrait;
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use tracing::instrument;

mod config;
mod invalid_jwt;
mod key;
mod user;

#[instrument(skip(config))]
pub async fn init_db(config: &crate::config::EnvConfig) -> Connection {
  let mut options = ConnectOptions::new(format!(
    "sqlite:{}/sqlite.db?mode=rwc",
    config.storage_path.display()
  ));
  options
    .max_connections(config.db.database_max_connections)
    .min_connections(config.db.database_min_connections)
    .connect_timeout(Duration::from_secs(config.db.database_connect_timeout))
    .sqlx_logging(config.db.database_logging);

  let conn = Database::connect(options)
    .await
    .expect("Failed to connect to database");
  migration::Migrator::up(&conn, None)
    .await
    .expect("Failed to run database migrations");

  Connection(conn)
}

#[derive(FromReqExtension, Clone)]
pub struct Connection(DatabaseConnection);

impl Deref for Connection {
  type Target = DatabaseConnection;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl Connection {
  pub fn key(&self) -> key::KeyTable<'_> {
    key::KeyTable::new(&self.0)
  }

  pub fn user(&self) -> user::UserTable<'_> {
    user::UserTable::new(&self.0)
  }

  pub fn invalid_jwt(&self) -> invalid_jwt::InvalidJwtTable<'_> {
    invalid_jwt::InvalidJwtTable::new(&self.0)
  }

  pub fn config(&self) -> config::ConfigTable<'_> {
    config::ConfigTable::new(&self.0)
  }
}

#[cfg(test)]
pub mod test {
  use super::*;

  pub async fn test_db() -> Connection {
    let conn = Database::connect("sqlite::memory:")
      .await
      .expect("Failed to connect to database");
    migration::Migrator::up(&conn, None)
      .await
      .expect("Failed to run database migrations");

    Connection(conn)
  }

  #[tokio::test]
  async fn test_connection() {
    let _ = test_db().await;
  }
}
