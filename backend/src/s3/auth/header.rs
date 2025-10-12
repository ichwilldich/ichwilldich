use std::time::SystemTime;

use axum::{
  RequestPartsExt,
  body::{Body, BodyDataStream},
  extract::Request,
};
use axum_extra::{
  TypedHeader,
  headers::{Authorization, ContentEncoding, Date},
};
use centaurus::{bail, error::Result};
use chrono::{DateTime, Utc};
use eyre::{Context, OptionExt};
use futures::StreamExt;
use http::request::Parts;
use memchr::memchr;
use sha2::{Digest, Sha256};
use tracing::instrument;

use crate::{
  config::EnvConfig,
  s3::{
    auth::{
      Identity, S3Auth, SECRET,
      body::{Body as BodyTrait, BodyWriter},
      credential::AWS4,
      sig_v4::{CanonicalRequest, Payload, StringToSign},
    },
    header::{AwzContentSha256, AwzContentSha256Header, AwzDate, AwzDecodedContentLength},
  },
};

#[instrument]
pub async fn header_auth<T: BodyTrait>(req: Request) -> Result<S3Auth<T>> {
  let (mut parts, body) = req.into_parts();
  let mut auth = parts
    .extract::<TypedHeader<Authorization<AWS4>>>()
    .await
    .ok()
    .map(|TypedHeader(Authorization(auth))| auth);

  let TypedHeader(AwzContentSha256Header(content_hash)) = parts
    .extract::<TypedHeader<AwzContentSha256Header>>()
    .await
    .unwrap_or(TypedHeader(AwzContentSha256Header(
      AwzContentSha256::UnsignedPayload,
    )));

  if !content_hash.is_unsigned() && auth.is_none() {
    bail!(
      FORBIDDEN,
      "Signed payload type requires Authorization header"
    );
  }

  let date = {
    if let Ok(TypedHeader(AwzDate(date))) = parts.extract::<TypedHeader<AwzDate>>().await {
      DateTime::<Utc>::from_naive_utc_and_offset(date, Utc)
    } else if let Ok(TypedHeader(date)) = parts.extract::<TypedHeader<Date>>().await {
      DateTime::<Utc>::from(SystemTime::from(date))
    } else {
      Utc::now()
    }
  };
  if let Some(auth) = &auth {
    check_headers(&parts, auth)?;
  }

  let Ok(config) = parts.extract::<EnvConfig>().await;

  let mut writer = T::Writer::new(&config.storage_path).await?;
  let body = if content_hash.is_chunked() {
    BodyOrHash::Body(body)
  } else {
    let mut stream = body.into_data_stream();
    let mut sha256 = Sha256::new();
    while let Some(chunk) = stream.next().await {
      if let Ok(chunk) = chunk {
        sha256.update(&chunk);
        writer.write(&chunk).await?;
      } else {
        bail!(INTERNAL_SERVER_ERROR, "Error reading body");
      }
    }
    BodyOrHash::Hash(hex::encode(sha256.finalize()))
  };

  let payload = if let BodyOrHash::Hash(hash) = &body {
    if matches!(content_hash, AwzContentSha256::UnsignedPayload) {
      Payload::Unsigned
    } else {
      Payload::SingleChunk(hash.clone())
    }
  } else {
    Payload::MultipleChunks
  };

  let signature = if let Some(auth) = &mut auth {
    let signature = CanonicalRequest::new(&parts, auth, &payload)
      .string_to_sign(&date, &auth.credential)
      .sign(SECRET, &auth.credential)?;

    if signature != auth.signature {
      bail!(FORBIDDEN, "Signature mismatch");
    }

    signature
  } else {
    String::new()
  };

  if let BodyOrHash::Body(body) = body {
    let body = body.into_data_stream();
    process_chunks(
      &mut parts,
      body,
      &signature,
      &auth,
      &date,
      &content_hash,
      &mut writer,
    )
    .await?;
  };

  Ok(S3Auth {
    identity: if let Some(auth) = auth {
      Identity::AccessKey(auth.credential.access_key)
    } else {
      Identity::Anonymous
    },
    body: T::from_writer(writer).await?,
    additional: None,
  })
}

enum BodyOrHash {
  Body(Body),
  Hash(String),
}

#[instrument]
async fn process_chunks(
  parts: &mut Parts,
  mut body: BodyDataStream,
  initial_signature: &str,
  auth: &Option<AWS4>,
  datetime: &DateTime<Utc>,
  content_hash: &AwzContentSha256,
  writer: &mut impl BodyWriter,
) -> Result<()> {
  let TypedHeader(encoding) = parts.extract::<TypedHeader<ContentEncoding>>().await?;
  if !encoding.contains("aws-chunked") {
    bail!("Content-Encoding must be 'aws-chunked'");
  }

  let TypedHeader(AwzDecodedContentLength(length)) = parts
    .extract::<TypedHeader<AwzDecodedContentLength>>()
    .await?;

  let mut buffer = Vec::new();
  let mut current_meta: Option<ChunkMeta> = None;
  let mut read_bytes = 0;
  let mut last_signature = initial_signature.to_string();
  let mut still_processing = false;
  let mut last_chunk = false;
  let trailer = content_hash.is_trailer();

  loop {
    if buffer.is_empty() || !still_processing {
      match body.next().await {
        Some(Ok(chunk)) => buffer.extend_from_slice(&chunk),
        Some(Err(e)) => bail!(INTERNAL_SERVER_ERROR, "Error reading body: {}", e),
        None => {
          last_chunk = true;
          // also set still_processing to false to avoid infinite loop
          // when the buffer is empty and it is not set by the else
          still_processing = false;
        }
      }
    } else {
      still_processing = false;
    }

    if let Some(meta) = &current_meta {
      // +2 for trailing \r\n
      if buffer.len() >= meta.length + 2 {
        let d = buffer.drain(..meta.length).collect::<Vec<u8>>();
        if buffer.len() < 2 || &buffer[..2] != b"\r\n" {
          bail!("Invalid chunk ending");
        }

        // verify chunk signature
        if let Some(auth) = &auth {
          let signature = StringToSign::chunked(datetime, &auth.credential, &last_signature, &d)
            .sign(SECRET, &auth.credential)?;
          if signature != meta.signature {
            bail!(FORBIDDEN, "Chunk signature mismatch");
          }
          last_signature = signature;
        }

        // remove trailing \r\n
        buffer.drain(..2);
        writer.write(&d).await?;
        read_bytes += d.len();

        // don't break if there is a trailer chunk and exit automatically when the stream ends
        if meta.length == 0 && trailer {
          break;
        }

        current_meta = None;
        still_processing = true;
      }
    }

    if let Some(i) = memchr(b'\n', &buffer) {
      let line = String::from_utf8_lossy(&buffer[..i - 1]); // -1 to remove \r
      let mut parts = line.split(';');
      let length = parts
        .next()
        .and_then(|s| usize::from_str_radix(s, 16).ok())
        .ok_or_eyre("Invalid chunk length")?;

      let signature = parts
        .next()
        .and_then(|s| s.strip_prefix("chunk-signature="))
        .map(|s| s.to_string())
        .ok_or_eyre("Missing chunk signature")?;

      current_meta = Some(ChunkMeta { length, signature });
      buffer.drain(..=i);
      still_processing = true;
    }

    if last_chunk && current_meta.is_none() && !still_processing {
      break;
    }
  }

  if read_bytes != length as usize {
    bail!("Decoded content length mismatch");
  }

  if trailer {
    let str = str::from_utf8(&buffer).context("Invalid trailer")?;
    let parts = str.split('\n').collect::<Vec<_>>();
    if parts.len() != 2 {
      bail!("Invalid trailer");
    }

    let header_parts = parts[0].split(':').collect::<Vec<_>>();
    if header_parts.len() != 2 {
      bail!("Invalid trailer header");
    }
    let header_name = header_parts[0].trim();
    let header_value = header_parts[1].trim();

    if let Some(auth) = &auth {
      let signature_parts = parts[1].split(':').collect::<Vec<_>>();
      if signature_parts.len() != 2 || signature_parts[0].trim() != "x-amz-trailer-signature" {
        bail!("Invalid trailer signature");
      }
      let expected_signature = signature_parts[1].trim();

      let signature = StringToSign::chunked_trailer(
        datetime,
        &auth.credential,
        &last_signature,
        header_name,
        header_value,
      )
      .sign(SECRET, &auth.credential)?;

      if signature != expected_signature {
        bail!(FORBIDDEN, "Trailer signature mismatch");
      }
    }
  }

  Ok(())
}

#[derive(Debug)]
struct ChunkMeta {
  length: usize,
  signature: String,
}

#[instrument]
pub fn check_headers(parts: &Parts, auth: &AWS4) -> Result<()> {
  for header in parts.headers.keys() {
    if header.as_str().starts_with("x-amz-")
      && !auth
        .signed_headers
        .iter()
        .any(|h| h.eq_ignore_ascii_case(header.as_str()))
    {
      bail!("SignedHeaders missing header {}", header.as_str());
    }
  }
  Ok(())
}

#[cfg(test)]
mod test {
  use crate::{config::EnvConfig, s3::auth::credential::AWS4Credential};

  use super::*;

  fn request(auth: bool) -> Request {
    unsafe {
      std::env::set_var("STORAGE_PATH", "/tmp/s3");
    }
    let mut builder = Request::builder()
      .uri("http://localhost/")
      .extension(EnvConfig::default());

    if auth {
      builder = builder.header("Authorization", "AWS4-HMAC-SHA256 Credential=test/21240426/us-east-1/s3/aws4_request, SignedHeaders=host;x-amz-content-sha256;x-amz-date, Signature=e737cff2fc158b249645312df82c5a72abc11a42e7b8a20a41cbff1f9430b4c1");
    }

    builder
      .header("x-amz-content-sha256", "UNSIGNED-PAYLOAD")
      .header("x-amz-date", "21240426T000000Z")
      .body(Body::new("Hello, world!".to_string()))
      .unwrap()
  }

  #[tokio::test]
  async fn test_header_auth() {
    let req = request(true);
    let auth = header_auth::<Vec<u8>>(req).await.unwrap();
    assert_eq!(auth.identity, Identity::AccessKey("test".to_string()));
    assert_eq!(auth.body, b"Hello, world!".to_vec());
  }

  #[tokio::test]
  async fn test_header_auth_no_auth() {
    let req = request(false);
    let auth = header_auth::<Vec<u8>>(req).await.unwrap();
    assert_eq!(auth.identity, Identity::Anonymous);
    assert_eq!(auth.body, b"Hello, world!".to_vec());
  }

  fn process_chunks_data() -> (Parts, BodyDataStream) {
    let req = Request::builder()
      .header("Content-Encoding", "aws-chunked")
      .header("x-awz-decoded-content-length", "13")
      .body(Body::new(
        "b;chunk-signature=eb4da889094a48f5c7d765c9bc36a22561aa0eb233b6ff2daa48b175be876b2d\
      \r\nHello, worl\r\n\
      2;chunk-signature=fdd149ed8f89c43e576c91eb79c9af3ca2cfaae017cf44898a826692f18fea49\
      \r\nd!\r\n\
      0;chunk-signature=74710388795df5c9cd091e2180cbcb977d424f571117c8242ccfaf86c03dcab8\r\n\r\n"
          .to_string(),
      ))
      .unwrap();
    let (parts, body) = req.into_parts();
    (parts, body.into_data_stream())
  }

  #[tokio::test]
  async fn test_process_chunks() {
    let (mut parts, body) = process_chunks_data();
    let data_dir = std::env::temp_dir();
    let mut writer = <Vec<u8> as BodyWriter>::new(&data_dir).await.unwrap();
    let date = DateTime::parse_from_rfc3339("2124-04-26T00:00:00Z")
      .unwrap()
      .with_timezone(&Utc);

    let result = process_chunks(
      &mut parts,
      body,
      "e737cff2fc158b249645312df82c5a72abc11a42e7b8a20a41cbff1f9430b4c1",
      &Some(AWS4 {
        credential: AWS4Credential {
          access_key: "test".to_string(),
          date: "21240426".to_string(),
          region: "us-east-1".to_string(),
        },
        signature: "".to_string(),
        signed_headers: vec![
          "host".to_string(),
          "x-amz-content-sha256".to_string(),
          "x-amz-date".to_string(),
        ],
      }),
      &date,
      &AwzContentSha256::StreamingAws4HmacSha256Payload,
      &mut writer,
    )
    .await;
    assert!(result.is_ok());
    assert_eq!(writer, b"Hello, world!".to_vec());
  }

  #[test]
  fn test_check_headers() {
    let req = Request::builder()
      .header("Authorization", "AWS4-HMAC-SHA256 Credential=test/21240426/us-east-1/s3/aws4_request, SignedHeaders=host;x-amz-content-sha256;x-amz-date, Signature=e737cff2fc158b249645312df82c5a72abc11a42e7b8a20a41cbff1f9430b4c1")
      .header("x-amz-content-sha256", "UNSIGNED-PAYLOAD")
      .header("x-amz-date", "21240426T000000Z")
      .header("x-amz-meta-custom", "value")
      .body(Body::new("Hello, world!".to_string()))
      .unwrap();
    let (parts, _) = req.into_parts();
    let result = check_headers(
      &parts,
      &AWS4 {
        credential: AWS4Credential {
          access_key: "test".to_string(),
          date: "21240426".to_string(),
          region: "us-east-1".to_string(),
        },
        signature: "".to_string(),
        signed_headers: vec![
          "host".to_string(),
          "x-amz-content-sha256".to_string(),
          "x-amz-date".to_string(),
        ],
      },
    );
    assert!(result.is_err());
  }
}
