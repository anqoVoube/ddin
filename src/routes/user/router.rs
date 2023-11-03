use axum::Router;
use axum::routing::post;
use redis::aio::Connection;
use redis::RedisConnectionInfo;
use crate::routes::user::create::create;
use crate::routes::user::verify::verify;
use crate::routes::user::login::login;

pub fn get_router() -> Router{
    Router::new()
        .route("/", post(create))
        .route("/verify", post(verify))
        .route("/login", post(login))
}