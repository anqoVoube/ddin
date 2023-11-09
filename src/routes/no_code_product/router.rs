use axum::Router;
use axum::routing::{get, post};
use crate::routes::no_code_product::create::create;

pub fn get_router() -> Router{
    Router::new()
        .route("/", post(create))
}