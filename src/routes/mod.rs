mod business;
mod user;
mod parent_product;
mod product;
mod utils;

use std::sync::Arc;
use axum::{Router, body::Body, Extension, middleware};


use axum::routing::get;
use redis::aio::Connection;

use sea_orm::DatabaseConnection;
use tokio::sync::Mutex;
use crate::core::auth::middleware::auth_getter;
use crate::routes::business::router::get_router as business_router;
use crate::routes::user::router::get_router as user_router;
use crate::routes::parent_product::router::get_router as parent_product_router;
use crate::routes::product::router::get_router as product_router;
use crate::routes::utils::media::media_path;


#[derive(Clone)]
pub struct AppState {
    pub redis: Arc<Mutex<Connection>>
}


pub fn v1_routes() -> Router{
    Router::new()
        .nest("/business", business_router())
        .nest("/user/", user_router())
        .nest("/parent-product/", parent_product_router())
        .nest("/product/", product_router())
}


pub fn create_routes(database: DatabaseConnection, redis: Connection) -> Router<(), Body> {
    // Router with trailing slash deletion
    // let a = redis.clone();
    let redis_connection = AppState{redis: Arc::new(Mutex::new(redis))};
    Router::new()
        .nest("/", v1_routes())
        .route("/media/*path", get(media_path))
        // .route_layer(middleware::from_fn(business_getter))
        .route_layer(middleware::from_fn_with_state(redis_connection, auth_getter))
        // .layer(Extension(redis))
        .layer(Extension(database))
}
