use std::fmt::Debug;
use std::path::{self, Path};
use std::{io::Result, path::PathBuf};

use serde::{Deserialize, Serialize};
use tokio::fs;
use tokio::io::{AsyncRead, AsyncReadExt};

use crate::s3::BUCKET_DIR;
use crate::s3::interface::S3Interface;

pub mod no_raid;

#[async_trait::async_trait]
pub trait Storage: Debug {
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

#[derive(Deserialize, Serialize, Clone, Copy, PartialEq, Debug)]
pub enum StorageType {
  NoRaid,
}

impl StorageType {
  pub async fn storage(&self, base_path: PathBuf) -> Result<S3Interface> {
    let base_path = path::absolute(base_path)?;
    if !base_path.exists() {
      fs::create_dir_all(&base_path).await?;
    }
    let bucket_path = base_path.join(BUCKET_DIR);
    if !bucket_path.exists() {
      fs::create_dir_all(&bucket_path).await?;
    }

    Ok(S3Interface::new(match self {
      StorageType::NoRaid => no_raid::NoRaid::new(base_path),
    }))
  }
}

#[cfg(test)]
mod test {
  use super::*;

  #[tokio::test]
  async fn test_no_raid_storage() -> Result<()> {
    let temp_dir = std::env::temp_dir().join(format!("sagittarius-{}", uuid::Uuid::new_v4()));
    let storage = no_raid::NoRaid::new(temp_dir.clone());

    // Test create_dir and list_dir
    storage.create_dir(Path::new("test_bucket")).await?;
    let buckets = storage.list_dir(Path::new("")).await?;
    assert!(buckets.contains(&"test_bucket".to_string()));

    // Test write_file and read_file
    let data = b"Hello, world!";
    storage
      .write_file(Path::new("test_bucket/hello.txt"), data)
      .await?;
    let read_data = storage
      .read_file(Path::new("test_bucket/hello.txt"))
      .await?;
    assert_eq!(data.to_vec(), read_data);

    // Test delete_file
    storage
      .delete_file(Path::new("test_bucket/hello.txt"))
      .await?;
    let files = storage.list_dir(Path::new("test_bucket")).await?;
    assert!(!files.contains(&"hello.txt".to_string()));

    // Test delete_dir
    storage.delete_dir(Path::new("test_bucket")).await?;
    let buckets = storage.list_dir(Path::new("")).await?;
    assert!(!buckets.contains(&"test_bucket".to_string()));

    // Clean up
    fs::remove_dir_all(temp_dir).await?;

    Ok(())
  }
}
