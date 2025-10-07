use axum::Extension;
use centaurus::{FromReqExtension, error::Result};
use clap::Parser;

pub use env::EnvConfig;

use crate::{
  config::ui::{MergedConfig, SavedConfig},
  db::Connection,
  macros::DualRouterExt,
  router_extension,
};

pub mod env;
pub mod ui;

router_extension!(
  async fn config(self, db: &Connection) -> Self {
    let app_config = AppConfig::new(db).await;
    self.layer(Extension(app_config))
  }
);

#[derive(Clone, FromReqExtension)]
pub struct AppConfig {
  pub config: MergedConfig,
}

impl AppConfig {
  pub async fn new(db: &Connection) -> Self {
    let env = SavedConfig::parse();
    let ui = db
      .config()
      .get_config()
      .await
      .expect("failed to get config from db");

    Self {
      config: env.merge(ui),
    }
  }

  pub async fn save_config(&self, db: &Connection, config: &MergedConfig) -> Result<()> {
    let config = config.to_ui();
    db.config().save_config(&config).await?;
    Ok(())
  }
}
