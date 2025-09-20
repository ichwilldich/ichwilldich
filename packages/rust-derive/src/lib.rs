use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemEnum, ItemStruct, parse_macro_input};

#[proc_macro_derive(FromReqExtension)]
pub fn from_req(input: TokenStream) -> TokenStream {
  let input = parse_macro_input!(input as ItemStruct);

  let mut input_generics = input.clone();
  input_generics
    .generics
    .params
    .push(syn::parse_quote! { FromRequestGeneric: Sync });
  let (impl_generics, _, _) = input_generics.generics.split_for_impl();

  let name = input.ident;
  let (_, type_generics, where_clause) = input.generics.split_for_impl();

  quote! {
    impl #impl_generics ichwilldich_lib::axum::extract::FromRequestParts<FromRequestGeneric> for #name #type_generics #where_clause {
      type Rejection = std::convert::Infallible;

      async fn from_request_parts(
        parts: &mut ichwilldich_lib::http::request::Parts,
        _state: &FromRequestGeneric,
      ) -> std::result::Result<Self, Self::Rejection> {
        use ichwilldich_lib::axum::RequestPartsExt;

        Ok(
          parts
            .extract::<ichwilldich_lib::axum::Extension<Self>>()
            .await
            .expect(
              format!(
                "Should not fail. Did you add Extension({}) to your app?",
                std::any::type_name::<Self>()
              )
              .as_str(),
            )
            .0,
        )
      }
    }
  }
  .into()
}

#[proc_macro_derive(UnitEnumStr)]
pub fn unit_enum_str(input: TokenStream) -> TokenStream {
  let input = parse_macro_input!(input as ItemEnum);
  let name = input.ident;

  quote! {
    impl std::fmt::Display for #name {
      fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = ichwilldich_lib::json::to_value(self).expect("Only enums with unit variants are supported");
        let ichwilldich_lib::json::Value::String(s) = value else {
          unreachable!("Only enums with unit variants are supported")
        };

        write!(f, "{s}")
      }
    }

    impl std::str::FromStr for #name {
      type Err = ichwilldich_lib::json::Error;

      fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value = ichwilldich_lib::json::Value::String(s.to_string());
        ichwilldich_lib::json::from_value(value)
      }
    }
  }.into()
}
