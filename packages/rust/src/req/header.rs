#[macro_export]
macro_rules! typed_header {
  ($name:ident, $const:ident, $name_str:literal, $inner:ident, |$s:ident| $decode:expr, |$v:ident| $encode:expr) => {
    pub struct $name($inner);
    pub static $const: ichwilldich_lib::http::HeaderName =
      ichwilldich_lib::http::HeaderName::from_static($name_str);

    impl $name {
      pub fn inner(&self) -> &$inner {
        &self.0
      }

      pub fn into_inner(self) -> $inner {
        self.0
      }
    }

    impl ichwilldich_lib::axum_extra::headers::Header for $name {
      fn name() -> &'static ichwilldich_lib::http::HeaderName {
        &$const
      }

      fn decode<'i, I>(values: &mut I) -> Result<Self, ichwilldich_lib::axum_extra::headers::Error>
      where
        Self: Sized,
        I: Iterator<Item = &'i ichwilldich_lib::http::HeaderValue>,
      {
        values
          .next()
          .and_then(|v| v.to_str().ok())
          .and_then(|$s| $decode)
          .map(|v| $name(v))
          .ok_or(ichwilldich_lib::axum_extra::headers::Error::invalid())
      }

      fn encode<E: Extend<ichwilldich_lib::http::HeaderValue>>(&self, values: &mut E) {
        let $v = &self.0;
        let value = $encode;
        let bytes = value.as_bytes();
        let val =
          ichwilldich_lib::http::HeaderValue::from_bytes(bytes).expect("valid header value");
        values.extend(std::iter::once(val));
      }
    }
  };
}
