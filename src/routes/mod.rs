mod business;
mod user;
mod parent_product;
mod product;
pub mod utils;
mod ping;
mod update;
mod expiration;

mod weight_item;
pub mod find;
pub mod rent;
mod sell;
mod parent_no_code_product;
mod no_code_product;
mod statistics;
pub mod product_request;
mod parent_weight_item;
pub mod debts;

use std::sync::Arc;
use axum::{Router, body::Body, Extension, middleware};
use axum::extract::DefaultBodyLimit;
use axum::response::IntoResponse;


use axum::routing::{get, post, put};
use axum::http::Method;
use redis::aio::Connection;

use scylla::Session;
use sea_orm::DatabaseConnection;
use tokio::sync::Mutex;
use tower_cookies::CookieManagerLayer;
use tower_http::cors::{Any, CorsLayer};
use crate::core::auth::middleware::{Auth, auth_getter};
use crate::RedisPool;
use crate::routes::business::create::create;
use crate::routes::business::router::get_router as business_router;
use crate::routes::find::router::get_router as find_router;
use crate::routes::user::router::get_router as user_router;
use crate::routes::parent_product::router::get_router as parent_product_router;
use crate::routes::ping::ping;
use crate::routes::find::sell::search;
use crate::routes::product::router::get_router as product_router;
use crate::routes::no_code_product::router::get_router as no_code_product_router;
use crate::routes::statistics::router::get_router as statistics_router;
use crate::routes::utils::media::media_path;
use crate::routes::weight_item::create::create as create_weight_item;
use crate::routes::sell::sell;

#[derive(Clone)]
pub struct AppConnections {
    pub redis: RedisPool,
    pub database: DatabaseConnection,
}

#[derive(Clone)]
pub struct ScyllaDBConnection {
    pub scylla: Arc<Session>
}


pub fn v1_routes(connections: AppConnections) -> Router{
    let cors = CorsLayer::new()
        .allow_methods([Method::POST, Method::GET])
        .allow_origin(Any)
        .allow_headers(Any);
    Router::new()
        .route("/ping", get(ping))
        .route("/request", post(product_request::upload))
        .route("/expirations", get(expiration::get_expirations))
        .route("/update", post(update::update_product))
        .route("/debts", get(debts::find::full_serializer_search).post(debts::create::create))
        .route("/debts/payment", post(debts::update::update))
        .route("/debts/history/:id", get(debts::history::get_history))

        .route("/add-parent/weight-item", post(parent_weight_item::create::upload))
        .route("/add-parent/no-code-product", post(parent_no_code_product::create::upload))

        .route("/weight-item", post(create_weight_item))
        .route("/sell", post(sell))
        .nest("/find", find_router())
        .nest("/business", business_router())
        .nest("/parent-product/", parent_product_router())
        .nest("/product/", product_router())
        .nest("/no-code-product", no_code_product_router())
        .nest("/statistics", statistics_router())
        .route_layer(middleware::from_fn_with_state(connections, auth_getter))
        .layer(DefaultBodyLimit::max(1024 * 1024 * 2000))
        .nest("/user/", user_router())
        .layer(cors)
}


pub fn create_routes(database: DatabaseConnection, redis: RedisPool, scylla: Session) -> Router<(), Body> {
    let connections = AppConnections{redis: redis.clone(), database: database.clone()};
    let scylla_connection = ScyllaDBConnection{
        scylla: Arc::new(scylla)
    };
    Router::new()
        .nest("/", v1_routes(connections.clone()))
        .route("/media/*path", get(media_path))
        .layer(Extension(redis))
        .layer(Extension(database))
        .layer(Extension(scylla_connection))
        .layer(CookieManagerLayer::new())
}
