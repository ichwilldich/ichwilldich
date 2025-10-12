use axum::{RequestPartsExt, extract::Request};
use centaurus::{bail, error::Result};
use chrono::{DateTime, NaiveDateTime, Utc};
use futures::StreamExt;
use tracing::instrument;

use crate::{
  config::EnvConfig,
  s3::{
    auth::{
      Identity, S3Auth, SECRET,
      body::{Body, BodyWriter},
      credential::AWS4,
      header::check_headers,
      sig_v4::{ALGORITHM, CanonicalRequest, Payload},
    },
    header::DATE_FORMAT,
  },
};

#[instrument]
pub async fn query_auth<T: Body>(req: Request, query: &[(String, String)]) -> Result<S3Auth<T>> {
  let mut data = parse_query(query)?;
  let (mut parts, body) = req.into_parts();

  check_headers(&parts, &data.auth)?;
  let date = DateTime::<Utc>::from_naive_utc_and_offset(data.date, Utc);
  let signature = CanonicalRequest::new(&parts, &mut data.auth, &Payload::Unsigned)
    .string_to_sign(&date, &data.auth.credential)
    .sign(SECRET, &data.auth.credential)?;

  if signature != data.auth.signature {
    bail!(FORBIDDEN, "Signature mismatch");
  }

  let Ok(config) = parts.extract::<EnvConfig>().await;

  let mut writer = T::Writer::new(&config.storage_path).await?;
  let mut stream = body.into_data_stream();

  while let Some(chunk) = stream.next().await {
    if let Ok(chunk) = chunk {
      writer.write(&chunk).await?;
    } else {
      bail!(INTERNAL_SERVER_ERROR, "Error reading body");
    }
  }

  Ok(S3Auth {
    identity: Identity::AccessKey(data.auth.credential.access_key),
    body: T::from_writer(writer).await?,
    additional: None,
  })
}

struct QueryData {
  algorithm: String,
  date: NaiveDateTime,
  auth: AWS4,
}

#[instrument]
fn parse_query(query: &[(String, String)]) -> Result<QueryData> {
  let mut algorithm = None;
  let mut credential = None;
  let mut date = None;
  let mut expires = None;
  let mut signed_headers = None;
  let mut signature = None;

  for (k, v) in query {
    match k.as_str() {
      "X-Amz-Algorithm" => algorithm = Some(v.to_string()),
      "X-Amz-Credential" => credential = Some(v.parse()?),
      "X-Amz-Date" => date = Some(NaiveDateTime::parse_from_str(v, DATE_FORMAT)?),
      "X-Amz-Expires" => expires = Some(v.parse()?),
      "X-Amz-SignedHeaders" => {
        let headers = v.split(';').map(|s| s.to_string()).collect::<Vec<_>>();
        if !headers.iter().any(|s| s.eq_ignore_ascii_case("host")) {
          bail!("SignedHeaders must contain 'host'");
        }

        signed_headers = Some(headers)
      }
      "X-Amz-Signature" => signature = Some(v.to_string()),
      _ => {}
    }
  }

  if algorithm.is_none()
    || credential.is_none()
    || date.is_none()
    || expires.is_none()
    || signed_headers.is_none()
    || signature.is_none()
  {
    bail!(
      "Missing required query parameters algorithm: {algorithm:?}, credential: {credential:?}, date: {date:?}, expires: {expires:?}, signed_headers: {signed_headers:?}, signature: {signature:?}"
    );
  }

  let expires = expires.unwrap();
  // Expires must be between 1 and 604800 seconds (7 days)
  if expires == 0 || expires > 604800 {
    bail!(
      FORBIDDEN,
      "X-Amz-Expires must be between 1 and 604800 seconds"
    );
  }

  let date = date.unwrap();
  if (Utc::now().naive_utc() - date).num_seconds() > expires {
    bail!(FORBIDDEN, "Request has expired");
  }

  let data = QueryData {
    algorithm: algorithm.unwrap(),
    date,
    auth: AWS4 {
      credential: credential.unwrap(),
      signed_headers: signed_headers.unwrap(),
      signature: signature.unwrap(),
    },
  };

  if data.algorithm != ALGORITHM {
    bail!("Only AWS4-HMAC-SHA256 is supported");
  }

  Ok(data)
}

#[cfg(test)]
mod test {
  use axum::body::Body;

  use super::*;

  fn query() -> Vec<(String, String)> {
    vec![
      (
        "X-Amz-Algorithm".to_string(),
        "AWS4-HMAC-SHA256".to_string(),
      ),
      (
        "X-Amz-Credential".to_string(),
        "AKIAIOSFODNN7EXAMPLE/21231129/us-east-1/s3/aws4_request".to_string(),
      ),
      ("X-Amz-Date".to_string(), "21231129T000000Z".to_string()),
      ("X-Amz-Expires".to_string(), "86400".to_string()),
      ("X-Amz-SignedHeaders".to_string(), "host".to_string()),
      (
        "X-Amz-Signature".to_string(),
        "d82434f5d71d0f64a8b69f0fcc01c94b553546ee4aef01ad14da02c82b6127a8".to_string(),
      ),
    ]
  }

  fn edit_query(mut query: Vec<(String, String)>, key: &str, value: &str) -> Vec<(String, String)> {
    for (k, v) in &mut query {
      if k == key {
        *v = value.to_string();
      }
    }
    query
  }

  fn req() -> Request {
    let query = query();
    let query: String = query
      .iter()
      .map(|(k, v)| format!("{}={}", k, v))
      .collect::<Vec<_>>()
      .join("&");

    unsafe {
      std::env::set_var("STORAGE_PATH", "/tmp/s3");
    }
    Request::builder()
      .uri(format!("http://localhost/test.txt?{}", query))
      .extension(EnvConfig::default())
      .body(Body::new("123".to_string()))
      .unwrap()
  }

  #[tokio::test]
  async fn test_query_auth() {
    let req = req();
    let query = query();

    let auth = query_auth::<Vec<u8>>(req, &query).await.unwrap();
    assert_eq!(
      auth.identity,
      Identity::AccessKey("AKIAIOSFODNN7EXAMPLE".to_string())
    );
    assert_eq!(auth.body, b"123".to_vec());
  }

  #[test]
  fn test_parse_query() {
    let query = query();

    let data = parse_query(&query).unwrap();
    assert_eq!(data.algorithm, "AWS4-HMAC-SHA256");
    assert_eq!(data.auth.credential.access_key, "AKIAIOSFODNN7EXAMPLE");
    assert_eq!(data.auth.credential.date, "21231129");
    assert_eq!(data.auth.credential.region, "us-east-1");
    assert_eq!(data.auth.signed_headers, vec!["host"]);
    assert_eq!(
      data.auth.signature,
      "d82434f5d71d0f64a8b69f0fcc01c94b553546ee4aef01ad14da02c82b6127a8"
    );
  }

  #[test]
  fn test_parse_query_any_missing() {
    for key in [
      "X-Amz-Algorithm",
      "X-Amz-Credential",
      "X-Amz-Date",
      "X-Amz-Expires",
      "X-Amz-SignedHeaders",
      "X-Amz-Signature",
    ] {
      let mut query = query();
      query.retain(|(k, _)| k != key);
      assert!(parse_query(&query).is_err());
    }
  }

  #[test]
  fn test_parse_query_invalid_date() {
    let query = edit_query(query(), "X-Amz-Date", "invalid-date");
    assert!(parse_query(&query).is_err());
  }

  #[test]
  fn test_parse_query_expired_date() {
    let query = edit_query(query(), "X-Amz-Date", "20000101T000000Z");
    assert!(parse_query(&query).is_err());
  }

  #[test]
  fn test_parse_query_invalid_algorithm() {
    let query = edit_query(query(), "X-Amz-Algorithm", "");
    assert!(parse_query(&query).is_err());
  }

  #[test]
  fn test_parse_query_missing_credential() {
    let query = edit_query(query(), "X-Amz-Credential", "");
    assert!(parse_query(&query).is_err());
  }
}
