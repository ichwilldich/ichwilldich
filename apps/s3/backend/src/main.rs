use axum::{Extension, Router};
use clap::Parser;
#[cfg(debug_assertions)]
use dotenv::dotenv;
use ichwilldich_lib::init::{add_base_layers, init_logging, listener_setup, run_app};
use tokio::join;

use crate::{config::Config, macros::DualRouterExt};

mod config;
mod macros;
mod s3;

#[tokio::main]
async fn main() {
  #[cfg(debug_assertions)]
  dotenv().ok();

  let config = Config::parse();
  init_logging(&config.base);

  let app_listener = listener_setup(config.base.port).await;
  let s3_listener = listener_setup(config.s3_port).await;

  let (app, s3) = (router(&config).await, s3_router(&config).await)
    .state(&config)
    .await
    .layer(Extension(config));

  join!(run_app(app_listener, app), run_app(s3_listener, s3));
}

async fn router(config: &Config) -> Router {
  Router::new().add_base_layers(&config.base).await
}

async fn s3_router(config: &Config) -> Router {
  s3::router().add_base_layers(&config.base).await
}

router_extension!(
  async fn state(self, config: &Config) -> Self {
    use s3::s3;

    self.s3(config).await
  }
);
