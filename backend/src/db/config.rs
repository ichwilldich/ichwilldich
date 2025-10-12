use centaurus::error::Result;
use entity::config;
use sea_orm::{ActiveValue::Set, prelude::*};
use tracing::instrument;
use uuid::Uuid;

use crate::config::ui::SavedConfig;

pub struct ConfigTable<'db> {
  db: &'db DatabaseConnection,
}

impl<'db> ConfigTable<'db> {
  pub fn new(db: &'db DatabaseConnection) -> Self {
    Self { db }
  }

  #[instrument(skip(self))]
  pub async fn get_config(&self) -> Result<SavedConfig> {
    let config = config::Entity::find().one(self.db).await?;
    let config = if let Some(config) = config {
      serde_json::from_value(config.config)?
    } else {
      let config = SavedConfig::default();
      self.save_config(&config).await?;
      config
    };

    Ok(config)
  }

  #[instrument(skip(self))]
  pub async fn save_config(&self, config: &SavedConfig) -> Result<()> {
    let config_json = serde_json::to_value(config)?;
    let existing = config::Entity::find().one(self.db).await?;

    if let Some(model) = existing {
      let mut active_model: config::ActiveModel = model.into();
      active_model.config = Set(config_json);
      active_model.update(self.db).await?;
    } else {
      let active_model = config::ActiveModel {
        id: Set(Uuid::new_v4()),
        config: Set(config_json),
      };
      active_model.insert(self.db).await?;
    }
    Ok(())
  }
}
