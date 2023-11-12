use axum::Router;
use axum::routing::get;
use crate::routes::statistics::best::full;

pub fn get_router() -> Router{
    Router::new()
        .route("/full", get(full))
}