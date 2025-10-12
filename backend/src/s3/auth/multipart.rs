use std::collections::HashMap;

use axum::{
  RequestExt,
  extract::{FromRequest, Multipart, Request},
};
use base64::prelude::*;
use centaurus::{bail, error::Result};
use chrono::NaiveDateTime;
use eyre::Context;
use tracing::instrument;

use crate::{
  config::EnvConfig,
  s3::{
    auth::{
      Identity, S3Auth, SECRET,
      body::{Body, BodyWriter},
      credential::AWS4Credential,
      sig_v4::{ALGORITHM, StringToSign},
    },
    header::DATE_FORMAT,
  },
};

#[instrument]
pub async fn multipart_auth<T: Body>(mut req: Request) -> Result<S3Auth<T>> {
  let Ok(config) = req.extract_parts::<EnvConfig>().await;
  let multipart = Multipart::from_request(req, &()).await?;
  let mut writer = T::Writer::new(&config.storage_path).await?;

  let data = parse_multipart(multipart, &mut writer).await?;

  let identity = if let Some(auth_info) = data.auth_info {
    let signature = StringToSign::new(auth_info.policy).sign(SECRET, &auth_info.credential)?;

    if signature != auth_info.signature {
      bail!(FORBIDDEN, "Signature mismatch");
    }

    Identity::AccessKey(auth_info.credential.access_key.clone())
  } else {
    Identity::Anonymous
  };

  Ok(S3Auth {
    identity,
    body: T::from_writer(writer).await?,
    additional: Some(data.additional),
  })
}

// TODO: handling of policy
struct MultipartData {
  auth_info: Option<MultipartAuthInfo>,
  additional: HashMap<String, String>,
}

struct MultipartAuthInfo {
  policy: String,
  _algorithm: String,
  credential: AWS4Credential,
  _date: NaiveDateTime,
  signature: String,
}

#[instrument]
async fn parse_multipart(
  mut multipart: Multipart,
  writer: &mut impl BodyWriter,
) -> Result<MultipartData> {
  let mut policy = None;
  let mut algorithm = None;
  let mut credential = None;
  let mut date = None;
  let mut signature = None;
  let mut additional = HashMap::new();

  while let Some(field) = multipart.next_field().await? {
    let name = field.name().unwrap_or("").to_string();

    if name == "file" {
      let data = field.bytes().await?;
      writer.write(&data).await?;
      continue;
    }

    let value = field.text().await?;
    match name.as_str() {
      "policy" => policy = Some(value),
      "x-amz-algorithm" => algorithm = Some(value),
      "x-amz-credential" => credential = Some(value),
      "x-amz-date" => date = Some(value),
      "x-amz-signature" => signature = Some(value),
      _ => {
        additional.insert(name, value.clone());
      }
    }
  }

  let auth_info = if let Some(algorithm) = algorithm
    && let Some(credential) = credential
    && let Some(date) = date
    && let Some(signature) = signature
    && let Some(policy) = policy
  {
    // check algorithm
    if algorithm != ALGORITHM {
      bail!("Only AWS4-HMAC-SHA256 is supported");
    }

    // check policy is valid base64
    BASE64_STANDARD
      .decode(&policy)
      .context("Invalid policy field")?;

    Some(MultipartAuthInfo {
      _algorithm: algorithm,
      credential: credential.parse()?,
      _date: NaiveDateTime::parse_from_str(&date, DATE_FORMAT)?,
      signature,
      policy,
    })
  } else {
    None
  };

  let multipart_data = MultipartData {
    auth_info,
    additional,
  };

  Ok(multipart_data)
}

#[cfg(test)]
mod test {
  use http::header::CONTENT_TYPE;
  use mime::BOUNDARY;
  use std::io::Write;

  use super::*;

  fn write_field(multipart: &mut Vec<u8>, key: &str, content: &str) {
    write!(multipart, "--{}\r\n", BOUNDARY).unwrap();
    write!(
      multipart,
      "Content-Disposition: form-data; name=\"{}\"\r\n\r\n",
      key
    )
    .unwrap();
    write!(multipart, "{}\r\n", content).unwrap();
  }

  fn multipart() -> Result<Request> {
    let mut multipart: Vec<u8> = Vec::new();

    let policy = r#"{}"#;
    let policy_b64 = BASE64_STANDARD.encode(policy);

    // meta
    write_field(&mut multipart, "key", "test.txt");
    write_field(&mut multipart, "policy", &policy_b64);
    write_field(&mut multipart, "x-amz-algorithm", ALGORITHM);
    write_field(
      &mut multipart,
      "x-amz-credential",
      "test/21240426/us-east-1/s3/aws4_request",
    );
    write_field(&mut multipart, "x-amz-date", "20240426T000000Z");
    write_field(
      &mut multipart,
      "x-amz-signature",
      "7a03f608465682aeb5bd284b7c2bfa84dd90bc5e4dc118cad4b22ec1754dfcb2",
    );

    // file
    write!(multipart, "--{}\r\n", BOUNDARY).unwrap();
    write!(
      multipart,
      "Content-Disposition: form-data; name=\"file\"; filename=\"test.txt\"\r\n"
    )
    .unwrap();
    write!(multipart, "Content-Type: text/plain\r\n\r\n").unwrap();
    write!(multipart, "Hello, world!\r\n").unwrap();
    write!(multipart, "--{}--\r\n", BOUNDARY).unwrap();

    unsafe {
      std::env::set_var("STORAGE_PATH", "/tmp/s3");
    }
    Ok(
      Request::builder()
        .header(
          CONTENT_TYPE,
          format!("multipart/form-data; boundary={}", BOUNDARY),
        )
        .extension(EnvConfig::default())
        .body(multipart.into())
        .unwrap(),
    )
  }

  #[tokio::test]
  async fn test_multipart_auth() -> Result<()> {
    let req = multipart()?;
    let auth: S3Auth<Vec<u8>> = multipart_auth(req).await?;

    assert_eq!(auth.identity, Identity::AccessKey("test".to_string()));
    assert_eq!(auth.body, b"Hello, world!".to_vec());

    Ok(())
  }

  #[tokio::test]
  async fn test_multipart_auth_anonymous() -> Result<()> {
    let mut multipart: Vec<u8> = Vec::new();

    // file
    write!(multipart, "--{}\r\n", BOUNDARY).unwrap();
    write!(
      multipart,
      "Content-Disposition: form-data; name=\"file\"; filename=\"test.txt\"\r\n"
    )
    .unwrap();
    write!(multipart, "Content-Type: text/plain\r\n\r\n").unwrap();
    write!(multipart, "Hello, world!\r\n").unwrap();
    write!(multipart, "--{}--\r\n", BOUNDARY).unwrap();

    unsafe {
      std::env::set_var("STORAGE_PATH", "/tmp/s3");
    }
    let req = Request::builder()
      .header(
        CONTENT_TYPE,
        format!("multipart/form-data; boundary={}", BOUNDARY),
      )
      .extension(EnvConfig::default())
      .body(multipart.into())
      .unwrap();

    let auth: S3Auth<Vec<u8>> = multipart_auth(req).await?;
    assert_eq!(auth.identity, Identity::Anonymous);
    assert_eq!(auth.body, b"Hello, world!".to_vec());
    Ok(())
  }

  #[tokio::test]
  async fn test_parse_multipart() -> Result<()> {
    let req = multipart()?;
    let multipart = Multipart::from_request(req, &()).await?;

    let data_dir = std::env::temp_dir();
    let result = parse_multipart(multipart, &mut <() as BodyWriter>::new(&data_dir).await?).await;
    assert!(result.is_ok());

    Ok(())
  }
}
