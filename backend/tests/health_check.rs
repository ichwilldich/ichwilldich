use crate::common::run;

#[allow(unused)]
mod common;

#[tokio::test]
async fn health_check_works() {
  let ports = run().await;

  let response = common::reqwest_client()
    .get(format!(
      "http://localhost:{}/api/health",
      ports.backend_port
    ))
    .send()
    .await
    .unwrap();
  assert!(response.status().is_success());
}
