use axum::Router;
use axum::routing::{get, post};
use crate::routes::business::create::create;
use crate::routes::business::fetch::{get_object, list};

pub fn get_router() -> Router{
    Router::new()
        .route("/", post(create).get(list))
        // .route("/:count", get(list))
        // .route("/", post(create))
        .route("/:business_id", get(get_object))
}