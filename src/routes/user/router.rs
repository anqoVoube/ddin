use axum::Router;
use axum::routing::post;
use crate::routes::user::create::create;
use crate::routes::user::verify::verify;

pub fn get_router() -> Router{
    Router::new()
        .route("/", post(create))
        .route("/verify", post(verify))
}