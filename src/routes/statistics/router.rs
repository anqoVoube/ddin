use axum::Router;
use axum::routing::get;
use crate::routes::statistics::best::full;
use crate::routes::statistics::product::product_stats;

pub fn get_router() -> Router{
    Router::new()
        .route("/full", get(full))
        .route("/product", get(product_stats))
}