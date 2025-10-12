use axum::{Extension, Router};
use centaurus::init::{
  axum::{add_base_layers, listener_setup, run_app},
  logging::init_logging,
};
use tokio::{fs, join, net::TcpListener};
use tracing::info;

use crate::{
  config::{AppConfig, EnvConfig},
  macros::DualRouterExt,
};

mod auth;
mod config;
mod db;
mod frontend;
mod health;
mod macros;
mod s3;

pub struct App {
  app: Router,
  app_listener: TcpListener,
  s3: Router,
  s3_listener: TcpListener,
}

impl App {
  pub async fn new() -> App {
    let config = EnvConfig::parse();
    init_logging(&config.base);
    fs::create_dir_all(&config.storage_path)
      .await
      .expect("failed to create storage path");

    let app_listener = listener_setup(config.base.port).await;
    let s3_listener = listener_setup(config.s3_port).await;

    let (app, s3) = (router(&config).await, s3_router(&config).await)
      .state(config)
      .await;

    Self {
      app,
      app_listener,
      s3,
      s3_listener,
    }
  }

  pub async fn run(self) {
    info!("Starting s3 sever");
    join!(
      run_app(self.app_listener, self.app),
      run_app(self.s3_listener, self.s3)
    );
  }

  pub fn ports(&self) -> (u16, u16) {
    (
      self.app_listener.local_addr().unwrap().port(),
      self.s3_listener.local_addr().unwrap().port(),
    )
  }
}

async fn router(config: &EnvConfig) -> Router {
  frontend::router()
    .nest(
      "/api",
      Router::new()
        .nest("/auth", auth::router())
        .merge(health::router()),
    )
    .add_base_layers(&config.base)
    .await
}

async fn s3_router(config: &EnvConfig) -> Router {
  s3::router().add_base_layers(&config.base).await
}

router_extension!(
  async fn state(self, env_config: EnvConfig) -> Self {
    use auth::auth;
    use config::config;
    use frontend::frontend;
    use s3::s3;

    let db = db::init_db(&env_config).await;
    let app_config = AppConfig::new(&db).await;

    self
      .s3(&env_config)
      .await
      .auth(&env_config, &app_config, &db)
      .await
      .frontend()
      .await
      .config(&db)
      .await
      .layer(Extension(db))
      .layer(Extension(env_config))
      .layer(Extension(app_config))
  }
);

#[cfg(test)]
mod test {
  #[tokio::test]
  async fn test_router() {
    unsafe {
      std::env::set_var("STORAGE_PATH", "/tmp/s3");
    }
    // test if there are any handler setup error that are not caught at compile time
    let _ = super::router(&super::EnvConfig::parse()).await;
  }
}
