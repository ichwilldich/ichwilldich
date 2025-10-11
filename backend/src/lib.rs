use axum::{Extension, Router};
use centaurus::init::{
  axum::{add_base_layers, listener_setup, run_app},
  logging::init_logging,
  metrics::init_metrics,
};
use tokio::{fs, join, net::TcpListener};
use tracing::{info, instrument};

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

#[derive(Debug)]
pub struct App {
  app: Router,
  app_listener: TcpListener,
  s3: Router,
  s3_listener: TcpListener,
}

impl App {
  #[instrument]
  pub async fn new() -> App {
    let config = EnvConfig::parse();
    init_logging(&config.base);
    let handle = init_metrics(config.metrics_name.clone());

    fs::create_dir_all(&config.storage_path)
      .await
      .expect("failed to create storage path");

    let metrics_enabled = config.metrics_enabled;
    let metrics_name = config.metrics_name.clone();
    let metrics_labels = config.metrics_labels.clone();

    let app_listener = listener_setup(config.base.port).await;
    let s3_listener = listener_setup(config.s3_port).await;

    let (mut app, mut s3) = (router(&config).await, s3_router(&config).await)
      .state(config)
      .await;

    use centaurus::init::metrics::metrics;
    let mut app_labels = vec![("api".into(), "management".into())];
    app_labels.extend(metrics_labels.clone());
    let mut s3_labels = vec![("api".into(), "s3".into())];
    s3_labels.extend(metrics_labels);

    if metrics_enabled {
      app = app
        .metrics(metrics_name.clone(), handle.clone(), app_labels)
        .await;
      s3 = s3.metrics(metrics_name, handle, s3_labels).await;
    }

    Self {
      app,
      app_listener,
      s3,
      s3_listener,
    }
  }

  #[instrument(skip(self))]
  pub async fn run(self) {
    info!("Starting s3 sever");
    join!(
      run_app(self.app_listener, self.app),
      run_app(self.s3_listener, self.s3)
    );
  }
}

#[instrument(skip(config))]
async fn router(config: &EnvConfig) -> Router {
  use centaurus::init::metrics::metrics_route;
  frontend::router()
    .nest(
      "/api",
      Router::new()
        .nest("/auth", auth::router())
        .merge(health::router())
        .metrics_route()
        .await,
    )
    .add_base_layers_filtered(&config.base, |path| path.starts_with("/api"))
    .await
}

#[instrument(skip(config))]
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
