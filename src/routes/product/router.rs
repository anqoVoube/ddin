use axum::Router;
use axum::routing::{get, post};
use crate::routes::product::create::create;
use crate::routes::product::fetch::fetch_products;
use crate::routes::product::sell::sell;

pub fn get_router() -> Router{
    Router::new()
        .route("/", post(create))
        .route("/:code", get(fetch_products))
        .route("/sell", post(sell))

    // .route("/sell", post(sell))
}