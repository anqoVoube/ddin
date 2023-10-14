use axum::Router;
use axum::routing::post;
use crate::routes::user::create::create;

pub fn get_router() -> Router{
    Router::new()
        .route("/", post(create))
}