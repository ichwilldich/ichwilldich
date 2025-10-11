use std::{fmt::Debug, path::PathBuf};

use centaurus::{error::Result, req::xml::Xml};
use serde::Deserialize;
use tokio::{
  fs::{File, OpenOptions},
  io::AsyncWriteExt,
};
use tracing::instrument;
use uuid::Uuid;

#[async_trait::async_trait]
pub trait Body: Sized {
  type Writer: BodyWriter + Send;

  async fn from_writer(writer: Self::Writer) -> Result<Self>;
}

/// A temporary file that will be deleted when dropped
/// This should be used for file uploads because they can be large
#[derive(Debug)]
#[allow(unused)]
pub struct TmpFile(pub PathBuf);
#[derive(Debug)]
#[allow(unused)]
pub struct FileWriter(File, PathBuf);

impl Drop for TmpFile {
  #[instrument]
  fn drop(&mut self) {
    if self.0.exists() {
      let _ = std::fs::remove_file(&self.0);
    }
  }
}

#[async_trait::async_trait]
impl Body for () {
  type Writer = ();

  async fn from_writer(_: Self::Writer) -> Result<Self> {
    Ok(())
  }
}

#[async_trait::async_trait]
impl Body for TmpFile {
  type Writer = FileWriter;

  #[instrument]
  async fn from_writer(writer: Self::Writer) -> Result<Self> {
    writer.0.sync_all().await?;
    drop(writer.0);
    Ok(TmpFile(writer.1))
  }
}

#[async_trait::async_trait]
impl Body for Vec<u8> {
  type Writer = Vec<u8>;

  async fn from_writer(writer: Self::Writer) -> Result<Self> {
    Ok(writer)
  }
}

#[async_trait::async_trait]
impl<T: Body> Body for Option<T> {
  type Writer = T::Writer;

  async fn from_writer(writer: Self::Writer) -> Result<Self> {
    Ok(T::from_writer(writer).await.ok())
  }
}

#[async_trait::async_trait]
impl<T> Body for Xml<T>
where
  T: for<'de> Deserialize<'de> + Send + Sync + 'static,
{
  type Writer = Vec<u8>;

  async fn from_writer(writer: Self::Writer) -> Result<Self> {
    Ok(Xml::from_slice(&writer)?)
  }
}

#[async_trait::async_trait]
pub trait BodyWriter: Sized + Debug {
  async fn new() -> Result<Self>;
  async fn write(&mut self, buf: &[u8]) -> Result<()>;
}

#[async_trait::async_trait]
impl BodyWriter for () {
  async fn new() -> Result<Self> {
    Ok(())
  }

  async fn write(&mut self, _: &[u8]) -> Result<()> {
    Ok(())
  }
}

#[async_trait::async_trait]
impl BodyWriter for FileWriter {
  async fn new() -> Result<Self> {
    let path = std::env::temp_dir().join(format!("sagittarius-{}", Uuid::new_v4()));
    let file = OpenOptions::new()
      .create_new(true)
      .read(true)
      .append(true)
      .open(&path)
      .await?;
    Ok(FileWriter(file, path))
  }

  async fn write(&mut self, buf: &[u8]) -> Result<()> {
    self.0.write_all(buf).await?;

    Ok(())
  }
}

#[async_trait::async_trait]
impl BodyWriter for Vec<u8> {
  async fn new() -> Result<Self> {
    Ok(Vec::new())
  }

  async fn write(&mut self, buf: &[u8]) -> Result<()> {
    self.extend_from_slice(buf);
    Ok(())
  }
}

#[cfg(test)]
mod test {
  use super::*;

  #[tokio::test]
  async fn test_tmp_file() {
    let mut writer = <TmpFile as Body>::Writer::new().await.unwrap();
    writer.write(b"Hello, ").await.unwrap();
    writer.write(b"world!").await.unwrap();
    let body = <TmpFile as Body>::from_writer(writer).await.unwrap();
    let content = tokio::fs::read_to_string(&body.0).await.unwrap();
    assert_eq!(content, "Hello, world!");

    let path = body.0.clone();

    drop(body);

    assert!(!path.exists());
  }

  #[tokio::test]
  async fn test_vec_u8() {
    let mut writer = <<Vec<u8> as Body>::Writer as BodyWriter>::new()
      .await
      .unwrap();
    <Vec<u8> as BodyWriter>::write(&mut writer, b"Hello, ")
      .await
      .unwrap();
    <Vec<u8> as BodyWriter>::write(&mut writer, b"world!")
      .await
      .unwrap();
    let body = <Vec<u8> as Body>::from_writer(writer).await.unwrap();
    assert_eq!(body, b"Hello, world!");
  }
}
