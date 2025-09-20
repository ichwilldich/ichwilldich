use crate::{config::Config, router_extension};
use axum::Router;

mod auth;
mod header;
mod operations;
pub mod storage;

pub fn router() -> Router {
  operations::router()
}

router_extension!(
  async fn s3(self, config: &Config) -> Self {
    use storage::storage;

    self.storage(config).await
  }
);
