use axum::Router;

mod bucket;
mod object;

pub fn router() -> Router {
  Router::new()
    .merge(bucket::router())
    .merge(object::router())
}
