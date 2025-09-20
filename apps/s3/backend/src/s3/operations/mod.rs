use axum::Router;

mod bucket;

const BUCKET_DIR: &str = "buckets";

pub fn router() -> Router {
  bucket::router()
}
