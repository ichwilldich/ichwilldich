use centaurus::error::Result;
use chrono::{DateTime, Utc};
use hmac::{Hmac, Mac};
use http::request::Parts;
use sha2::{Digest, Sha256};
use tracing::instrument;

use crate::s3::{
  auth::credential::{AWS4, AWS4Credential},
  header::DATE_FORMAT,
};

// SHA256 hash of an empty string
const EMPTY_STRING_SHA256_HASH: &str =
  "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";

pub const ALGORITHM: &str = "AWS4-HMAC-SHA256";
pub const ALGORITHM_CHUNKED: &str = "AWS4-HMAC-SHA256-PAYLOAD";

#[derive(Debug)]
pub enum Payload {
  Unsigned,
  SingleChunk(String),
  MultipleChunks,
}

#[derive(Debug)]
pub struct CanonicalRequest(String);

impl CanonicalRequest {
  #[instrument]
  pub fn new(parts: &Parts, auth: &mut AWS4, payload: &Payload) -> CanonicalRequest {
    let mut req = String::new();
    // HTTPMethod
    req.push_str(parts.method.as_str());
    req.push('\n');

    // CanonicalURI
    req.push_str(parts.uri.path());
    req.push('\n');

    // CanonicalQueryString
    if let Some(query) = parts.uri.query() {
      let mut query_pairs: Vec<_> = query
        .split('&')
        .map(|pair| {
          let mut split = pair.splitn(2, '=');
          let key = split.next().unwrap_or("");
          let value = split.next().unwrap_or("");
          (key, value)
        })
        .filter(|(k, _)| k != &"X-Amz-Signature")
        .collect();
      query_pairs.sort_unstable_by(|(a, _), (b, _)| a.cmp(b));
      let canonical_query = serde_urlencoded::to_string(&query_pairs).unwrap_or_default();
      req.push_str(&canonical_query);
    }
    req.push('\n');

    // CanonicalHeaders
    let mut headers: Vec<_> = parts
      .headers
      .iter()
      .map(|(k, v)| {
        (
          k.as_str().to_ascii_lowercase(),
          v.to_str().unwrap_or("").trim(),
        )
      })
      .filter(|(k, _)| {
        auth
          .signed_headers
          .iter()
          .any(|h| h.eq_ignore_ascii_case(k))
      })
      .collect();
    headers.sort_unstable_by(|(a, _), (b, _)| a.cmp(b));
    for (header, value) in &headers {
      req.push_str(&format!("{header}:{value}\n"));
    }
    req.push('\n');

    // SignedHeaders
    auth.signed_headers.sort_unstable();
    let signed_headers: String = auth.signed_headers.join(";");
    req.push_str(&format!("{}\n", signed_headers));

    // Payload
    let str = match payload {
      Payload::Unsigned => "UNSIGNED-PAYLOAD",
      Payload::SingleChunk(hash) => hash,
      Payload::MultipleChunks => "STREAMING-AWS4-HMAC-SHA256-PAYLOAD",
    };
    req.push_str(str);

    CanonicalRequest(req)
  }

  #[instrument]
  pub fn string_to_sign(
    &self,
    amz_date: &DateTime<Utc>,
    credential: &AWS4Credential,
  ) -> StringToSign {
    let mut string_to_sign = String::new();
    // Algorithm
    string_to_sign.push_str(ALGORITHM);
    string_to_sign.push('\n');

    // RequestDateTime
    string_to_sign.push_str(&amz_date.format(DATE_FORMAT).to_string());
    string_to_sign.push('\n');

    // CredentialScope
    string_to_sign.push_str(&format!(
      "{}/{}/s3/aws4_request\n",
      credential.date, credential.region
    ));

    // HashedCanonicalRequest
    string_to_sign.push_str(&hex::encode(Sha256::digest(self.0.as_bytes())));

    StringToSign(string_to_sign)
  }
}

#[derive(Debug)]
pub struct StringToSign(String);

impl StringToSign {
  #[instrument]
  pub fn new(s: String) -> Self {
    Self(s)
  }

  #[instrument]
  pub fn chunked(
    datetime: &DateTime<Utc>,
    credential: &AWS4Credential,
    previous_signature: &str,
    current_data: &[u8],
  ) -> Self {
    let mut string_to_sign = Self::chunked_base(datetime, credential, previous_signature);

    // HashedEmptyString
    string_to_sign.push_str(EMPTY_STRING_SHA256_HASH);
    string_to_sign.push('\n');

    // HashedChunkData
    string_to_sign.push_str(&hex::encode(Sha256::digest(current_data)));

    StringToSign(string_to_sign)
  }

  #[instrument]
  pub fn chunked_trailer(
    datetime: &DateTime<Utc>,
    credential: &AWS4Credential,
    previous_signature: &str,
    tailing_header_name: &str,
    trailing_header_value: &str,
  ) -> Self {
    let mut string_to_sign = Self::chunked_base(datetime, credential, previous_signature);

    // Trailing checksum header
    string_to_sign.push_str(&format!("{tailing_header_name}:{trailing_header_value}\n"));

    StringToSign(string_to_sign)
  }

  #[instrument]
  fn chunked_base(
    datetime: &DateTime<Utc>,
    credential: &AWS4Credential,
    previous_signature: &str,
  ) -> String {
    let mut string_to_sign = String::new();
    // Algorithm
    string_to_sign.push_str(ALGORITHM_CHUNKED);
    string_to_sign.push('\n');

    // RequestDateTime
    string_to_sign.push_str(&datetime.format(DATE_FORMAT).to_string());
    string_to_sign.push('\n');

    // CredentialScope
    string_to_sign.push_str(&format!(
      "{}/{}/s3/aws4_request\n",
      credential.date, credential.region
    ));

    // PreviousSignature
    string_to_sign.push_str(previous_signature);
    string_to_sign.push('\n');

    string_to_sign
  }

  #[instrument]
  pub fn sign(&self, secret_key: &str, credential: &AWS4Credential) -> Result<String> {
    let date_key = hmac(format!("AWS4{}", secret_key).as_bytes(), &credential.date)?;
    let date_region_key = hmac(&date_key, &credential.region)?;
    let date_region_service_key = hmac(&date_region_key, "s3")?;
    let signing_key = hmac(&date_region_service_key, "aws4_request")?;
    let signature = hmac(&signing_key, &self.0)?;
    Ok(hex::encode(signature))
  }
}

#[instrument]
fn hmac(key: &[u8], msg: &str) -> Result<Vec<u8>> {
  let mut hmac = Hmac::<Sha256>::new_from_slice(key)?;
  hmac.update(msg.as_bytes());
  Ok(hmac.finalize().into_bytes().to_vec())
}

#[cfg(test)]
mod test {
  use axum::extract::Request;

  use super::*;

  #[test]
  fn test_empty_string_sha256() {
    let hash = hex::encode(Sha256::digest("".as_bytes()));
    assert_eq!(hash, EMPTY_STRING_SHA256_HASH);
  }

  #[test]
  fn test_algorithm() {
    // just so ai doesn't mess with it
    assert_eq!(ALGORITHM, "AWS4-HMAC-SHA256");
    assert_eq!(ALGORITHM_CHUNKED, "AWS4-HMAC-SHA256-PAYLOAD");
  }

  fn aws4() -> AWS4 {
    AWS4 {
      credential: AWS4Credential {
        access_key: "AKIAIOSFODNN7EXAMPLE".to_string(),
        date: "20130524".to_string(),
        region: "us-east-1".to_string(),
      },
      signature: String::new(),
      signed_headers: vec![
        "host".to_string(),
        "x-amz-content-sha256".to_string(),
        "x-amz-date".to_string(),
      ],
    }
  }

  fn parts() -> Parts {
    let req = Request::new(());
    let mut parts = req.into_parts().0;

    parts.method = http::Method::GET;
    parts.uri = "/test.txt?a=b&c=d&b=w".parse().unwrap();
    parts
      .headers
      .insert("Host", "examplebucket.s3.amazonaws.com".parse().unwrap());
    parts
      .headers
      .insert("x-amz-date", "20130524T000000Z".parse().unwrap());
    parts
      .headers
      .insert("x-amz-content-sha256", "UNSIGNED-PAYLOAD".parse().unwrap());

    parts
  }

  #[test]
  fn test_cr_unsigned() {
    let parts = parts();
    let mut auth = aws4();
    let payload = Payload::Unsigned;

    let canonical_request = CanonicalRequest::new(&parts, &mut auth, &payload);
    let expected_canonical_request = "GET\n\
    /test.txt\n\
    a=b&b=w&c=d\n\
    host:examplebucket.s3.amazonaws.com\n\
    x-amz-content-sha256:UNSIGNED-PAYLOAD\n\
    x-amz-date:20130524T000000Z\n\
    \n\
    host;x-amz-content-sha256;x-amz-date\n\
    UNSIGNED-PAYLOAD";

    assert_eq!(canonical_request.0, expected_canonical_request);
  }

  #[test]
  fn test_cr_single_chunk() {
    let parts = parts();
    let mut auth = aws4();
    let payload = Payload::SingleChunk(EMPTY_STRING_SHA256_HASH.to_string());

    let canonical_request = CanonicalRequest::new(&parts, &mut auth, &payload);
    let expected_canonical_request = "GET\n\
    /test.txt\n\
    a=b&b=w&c=d\n\
    host:examplebucket.s3.amazonaws.com\n\
    x-amz-content-sha256:UNSIGNED-PAYLOAD\n\
    x-amz-date:20130524T000000Z\n\
    \n\
    host;x-amz-content-sha256;x-amz-date\n\
    e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";

    assert_eq!(canonical_request.0, expected_canonical_request);
  }

  #[test]
  fn test_cr_multiple_chunks() {
    let parts = parts();
    let mut auth = aws4();
    let payload = Payload::MultipleChunks;

    let canonical_request = CanonicalRequest::new(&parts, &mut auth, &payload);
    let expected_canonical_request = "GET\n\
    /test.txt\n\
    a=b&b=w&c=d\n\
    host:examplebucket.s3.amazonaws.com\n\
    x-amz-content-sha256:UNSIGNED-PAYLOAD\n\
    x-amz-date:20130524T000000Z\n\
    \n\
    host;x-amz-content-sha256;x-amz-date\n\
    STREAMING-AWS4-HMAC-SHA256-PAYLOAD";

    assert_eq!(canonical_request.0, expected_canonical_request);
  }

  fn cr() -> CanonicalRequest {
    let parts = parts();
    let mut auth = aws4();
    let payload = Payload::Unsigned;

    CanonicalRequest::new(&parts, &mut auth, &payload)
  }

  #[test]
  fn test_cr_sts() {
    let canonical_request = cr();
    let amz_date = DateTime::parse_from_rfc3339("2013-05-24T00:00:00Z")
      .unwrap()
      .with_timezone(&Utc);
    let credential = &aws4().credential;

    let string_to_sign = canonical_request.string_to_sign(&amz_date, credential);
    let expected_string_to_sign = "AWS4-HMAC-SHA256\n\
    20130524T000000Z\n\
    20130524/us-east-1/s3/aws4_request\n\
    a8ed10a1bc6059e6b958a64277969dcdc70444f16bed458169f04592b2fd4d98";

    assert_eq!(string_to_sign.0, expected_string_to_sign);
  }

  #[test]
  fn test_sts_chunked() {
    let amz_date = DateTime::parse_from_rfc3339("2013-05-24T00:00:00Z")
      .unwrap()
      .with_timezone(&Utc);
    let credential = &aws4().credential;
    let previous_signature = "a8ed10a1bc6059e6b958a64277969dcdc70444f16bed458169f04592b2fd4d98";
    let current_data = b"Hello World";

    let string_to_sign =
      StringToSign::chunked(&amz_date, credential, previous_signature, current_data);
    let expected_string_to_sign = "AWS4-HMAC-SHA256-PAYLOAD\n\
    20130524T000000Z\n\
    20130524/us-east-1/s3/aws4_request\n\
    a8ed10a1bc6059e6b958a64277969dcdc70444f16bed458169f04592b2fd4d98\n\
    e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855\n\
    a591a6d40bf420404a011733cfb7b190d62c65bf0bcda32b57b277d9ad9f146e";

    assert_eq!(string_to_sign.0, expected_string_to_sign);
  }

  #[test]
  fn test_sts_chunked_trailer() {
    let amz_date = DateTime::parse_from_rfc3339("2013-05-24T00:00:00Z")
      .unwrap()
      .with_timezone(&Utc);
    let credential = &aws4().credential;
    let previous_signature = "a8ed10a1bc6059e6b958a64277969dcdc70444f16bed458169f04592b2fd4d98";
    let trailing_header_name = "x-amz-checksum-sha256";
    let trailing_header_value = "a591a6d40bf420404a011733cfb7b190d62c65bf0bcda32b57b277d9ad9f146e";
    let string_to_sign = StringToSign::chunked_trailer(
      &amz_date,
      credential,
      previous_signature,
      trailing_header_name,
      trailing_header_value,
    );
    let expected_string_to_sign = "AWS4-HMAC-SHA256-PAYLOAD\n\
    20130524T000000Z\n\
    20130524/us-east-1/s3/aws4_request\n\
    a8ed10a1bc6059e6b958a64277969dcdc70444f16bed458169f04592b2fd4d98\n\
    x-amz-checksum-sha256:a591a6d40bf420404a011733cfb7b190d62c65bf0bcda32b57b277d9ad9f146e\n";
    assert_eq!(string_to_sign.0, expected_string_to_sign);
  }

  fn sts() -> StringToSign {
    let canonical_request = cr();
    let amz_date = DateTime::parse_from_rfc3339("2013-05-24T00:00:00Z")
      .unwrap()
      .with_timezone(&Utc);
    let credential = &aws4().credential;

    canonical_request.string_to_sign(&amz_date, credential)
  }

  #[test]
  fn test_sign() {
    let secret_key = "wJalrXUtnFEMI/K7MDENG+bPxRfiCYEXAMPLEKEY";
    let credential = &aws4().credential;
    let string_to_sign = sts().sign(secret_key, credential);
    let expected_signature = "e8c68eaa3147a30f4cde5eca6a0888571e3716ff1856c5cebc916ddcb6a7eb0a";
    assert_eq!(string_to_sign.unwrap(), expected_signature);
  }

  #[test]
  fn test_hmac() {
    let input = b"1234";
    let msg = "20130524/us-east-1/s3/aws4_request";
    let hmac = hmac(input, msg).unwrap();
    let expected_hmac = vec![
      211, 52, 83, 205, 49, 226, 232, 59, 239, 166, 97, 166, 177, 201, 200, 21, 20, 197, 254, 98,
      46, 43, 116, 163, 5, 37, 202, 211, 123, 67, 93, 184,
    ];
    assert_eq!(hmac, expected_hmac);
  }
}
