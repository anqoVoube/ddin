use axum::Router;
use axum::routing::{get, post};
use crate::routes::business::create::create;
use crate::routes::parent_product::fetch::get_object_by_code;

pub fn get_router() -> Router{
    Router::new()
        .route("/", post(create))
        .route("/:code", get(get_object_by_code))
}