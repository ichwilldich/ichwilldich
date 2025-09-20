use ichwilldich_lib::UnitEnumStr;
use serde::{Deserialize, Serialize};

macro_rules! typed_header {
  ($name:ident, $const:ident, $name_str:literal, $inner:ident) => {
    ichwilldich_lib::typed_header!($name, $const, $name_str, $inner, |s| s.parse().ok(), |v| v
      .to_string());
  };
  ($name:ident, $const:ident, $name_str:literal) => {
    ichwilldich_lib::typed_header!(
      $name,
      $const,
      $name_str,
      String,
      |s| Some(s.to_string()),
      |v| v
    );
  };
}

typed_header!(AwzAcl, AWZ_ACL, "x-amz-acl", AwzAclEnum);
typed_header!(
  AwzBucketObjectLockEnabled,
  AWZ_BUCKET_OBJECT_LOCK_ENABLED,
  "x-amz-bucket-object-lock-enabled",
  bool
);
typed_header!(
  AwzGrantFullControl,
  AWZ_GRANT_FULL_CONTROL,
  "x-amz-grant-full-control"
);
typed_header!(AwzGrantRead, AWZ_GRANT_READ, "x-amz-grant-read");
typed_header!(AwzGrantReadAcp, AWZ_GRANT_READ_ACP, "x-amz-grant-read-acp");
typed_header!(AwzGrantWrite, AWZ_GRANT_WRITE, "x-amz-grant-write");
typed_header!(
  AwzGrantWriteAcp,
  AWZ_GRANT_WRITE_ACP,
  "x-amz-grant-write-acp"
);
typed_header!(
  AwzObjectOwnership,
  AWZ_OBJECT_OWNERSHIP,
  "x-amz-object-ownership",
  AwzObjectOwnershipEnum
);

#[derive(Deserialize, Serialize, UnitEnumStr)]
#[serde(rename_all = "kebab-case")]
enum AwzAclEnum {
  Private,
  PublicRead,
  PublicReadWrite,
  AuthenticatedRead,
}

#[derive(Deserialize, Serialize, UnitEnumStr)]
#[serde(rename_all = "kebab-case")]
enum AwzObjectOwnershipEnum {
  BucketOwnerEnforced,
  BucketOwnerPreferred,
  ObjectWriter,
}
