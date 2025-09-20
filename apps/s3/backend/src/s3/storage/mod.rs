use std::ops::Deref;
use std::path::{self, Path};
use std::sync::Arc;
use std::{io::Result, path::PathBuf};

use axum::Extension;
use clap::ValueEnum;
use ichwilldich_lib::FromReqExtension;
use tokio::fs;
use tokio::io::{AsyncRead, AsyncReadExt};

use crate::config::Config;
use crate::macros::DualRouterExt;
use crate::router_extension;

pub mod no_raid;

#[async_trait::async_trait]
pub trait Storage {
  async fn create_dir(&self, path: &Path) -> Result<()>;
  async fn delete_dir(&self, path: &Path) -> Result<()>;
  async fn list_dir(&self, path: &Path) -> Result<Vec<String>>;
  async fn stream_write_file(
    &self,
    path: &Path,
    reader: &mut (dyn AsyncRead + Unpin + Send),
  ) -> Result<()>;
  async fn stream_read_file(&self, path: &Path) -> Result<Box<dyn AsyncRead + Unpin + Send>>;
  async fn delete_file(&self, path: &Path) -> Result<()>;

  async fn write_file(&self, path: &Path, data: &[u8]) -> Result<()> {
    self.stream_write_file(path, &mut &data[..]).await
  }

  async fn read_file(&self, path: &Path) -> Result<Vec<u8>> {
    let mut reader = self.stream_read_file(path).await?;
    let mut buf = Vec::new();
    reader.read_to_end(&mut buf).await?;
    Ok(buf)
  }
}

#[derive(ValueEnum, Clone, Copy)]
pub enum StorageType {
  NoRaid,
}

impl StorageType {
  pub async fn storage(&self, base_path: PathBuf) -> Result<StorageState> {
    let base_path = path::absolute(base_path)?;
    if !base_path.exists() {
      fs::create_dir_all(&base_path).await?;
    }

    Ok(StorageState::new(match self {
      StorageType::NoRaid => no_raid::NoRaid::new(base_path),
    }))
  }
}

router_extension!(
  async fn storage(self, config: &Config) -> Self {
    self.layer(Extension(
      config
        .storage_type
        .storage(config.storage_path.clone())
        .await
        .expect("Failed to initialize storage"),
    ))
  }
);

#[derive(FromReqExtension, Clone)]
pub struct StorageState(Arc<dyn Storage + Send + Sync>);

impl StorageState {
  pub fn new<S: Storage + Send + Sync + 'static>(storage: S) -> Self {
    Self(Arc::new(storage))
  }
}

impl Deref for StorageState {
  type Target = dyn Storage + Send + Sync;

  fn deref(&self) -> &Self::Target {
    &*self.0
  }
}
