use std::{env::set_var, time::Duration};

use backend::App;
use reqwest::Client;
use tokio::{spawn, time::sleep};
use uuid::Uuid;

pub fn prepare_env() {
  let storage_path = format!("/tmp/sagittarius-test-{}", Uuid::new_v4());

  unsafe {
    // use random ports for tests so they can be run in parallel
    set_var("PORT", "0");
    set_var("S3_PORT", "0");
    set_var("STORAGE_PATH", &storage_path);
  }
}

pub struct Ports {
  pub backend_port: u16,
  pub s3_port: u16,
}

pub async fn launch_app() -> Ports {
  let app = App::new().await;
  let (backend_port, s3_port) = app.ports();

  spawn(app.run());
  sleep(Duration::from_millis(100)).await; // wait for server to start

  Ports {
    backend_port,
    s3_port,
  }
}

pub async fn run() -> Ports {
  prepare_env();
  launch_app().await
}

pub fn reqwest_client() -> Client {
  Client::builder()
    .timeout(Duration::from_secs(10))
    .connect_timeout(Duration::from_secs(10))
    .build()
    .unwrap()
}
