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
mod check;
mod analytics;
// mod bot;

use std::sync::Arc;
use axum::{Router, body::Body, Extension, middleware};
use axum::extract::DefaultBodyLimit;
use axum::response::IntoResponse;


use axum::routing::{get, post};
use axum::http::{Method, header};
use http::HeaderName;
use mongodb::Database;

use scylla::Session;
use sea_orm::DatabaseConnection;
use teloxide::Bot;
use teloxide::update_listeners::webhooks;
use teloxide::update_listeners::webhooks::Options;
use tokio::sync::Mutex;
use tower_cookies::CookieManagerLayer;
use tower_http::cors::{Any, CorsLayer};
use crate::core::auth::middleware::{auth_getter, business_getter, validate_business_id};
use crate::RedisPool;
use crate::routes::business::router::get_router as business_router;
use crate::routes::check::check_title_uniqueness;
use crate::routes::find::router::get_router as find_router;
use crate::routes::user::router::get_router as user_router;
use crate::routes::parent_product::router::get_router as parent_product_router;
use crate::routes::ping::ping;
use crate::routes::product::router::get_router as product_router;
use crate::routes::no_code_product::router::get_router as no_code_product_router;
use crate::routes::statistics::router::get_router as statistics_router;
use crate::routes::debts::router::get_router as debts_router;
use crate::routes::utils::media::media_path;
use crate::routes::weight_item::create::create as create_weight_item;
use crate::routes::sell::sell;
use crate::routes::analytics::router::get_router as analytics_router;

#[derive(Clone)]
pub struct AppConnections {
    pub redis: RedisPool,
    pub database: DatabaseConnection,
}


#[derive(Clone)]
pub struct SqliteDBConnection {
    pub sqlite: Arc<Mutex<rusqlite::Connection>>
}


pub fn v1_routes(connections: AppConnections) -> Router{
    let origins = [
        "https://ddin.uz".parse().unwrap(),
        "https://api.ddin.uz".parse().unwrap(),
        "https://market-place.ddin.uz".parse().unwrap(),
        "81.95.230.194".parse().unwrap(),
        "http://81.95.230.194".parse().unwrap(),
        "84.54.122.78".parse().unwrap(),
        "http://84.54.122.78".parse().unwrap()
    ];

    let cors = CorsLayer::new()
        .allow_methods([Method::POST, Method::GET, Method::OPTIONS])
        .allow_origin(origins)
        .allow_headers(vec![
            header::AUTHORIZATION,
            header::ACCEPT,
            header::CONTENT_TYPE,
            header::COOKIE,
            header::ACCESS_CONTROL_ALLOW_ORIGIN,
            header::ACCESS_CONTROL_ALLOW_CREDENTIALS,
            HeaderName::from_lowercase(b"x-business-id").unwrap(),
            HeaderName::from_lowercase(b"no_add_token").unwrap(),
        ])
        .allow_credentials(true);

    Router::new()
        .route("/request", post(product_request::upload))
        .route("/request-no-photo", post(product_request::upload_without_photo))
        .route("/expirations", get(expiration::get_expirations))
        .route("/update", post(update::update_product))


        .route("/add-parent/weight-item", post(parent_weight_item::create::upload))
        .route("/add-parent/no-code-product", post(parent_no_code_product::create::upload))

        .route("/weight-item", post(create_weight_item))
        .route("/sell", post(sell))
        .route("/check-title", get(check_title_uniqueness))
        .nest("/find", find_router())
        .nest("/debts", debts_router())
        .nest("/parent-product/", parent_product_router())
        .nest("/product/", product_router())
        .nest("/no-code-product", no_code_product_router())
        .nest("/analytics", analytics_router())
        .nest("/statistics", statistics_router())
        .route_layer(middleware::from_fn_with_state(connections.clone(), validate_business_id))
        .route_layer(middleware::from_fn(business_getter))
        .nest("/business", business_router())
        .route("/ping", get(ping))

        .route_layer(middleware::from_fn_with_state(connections, auth_getter))

        .layer(DefaultBodyLimit::max(1024 * 1024 * 10))
        .nest("/user/", user_router())
        // .route("/webhook", post(bot_webhook))
        .layer(cors)

}


pub fn create_routes(
    database: DatabaseConnection,
    redis: RedisPool,
    mongo: Database,
    sqlite: rusqlite::Connection,
    bot: Bot
) -> Router<(), Body> {

    let connections = AppConnections{redis: redis.clone(), database: database.clone()};

    let sqlite_connection = SqliteDBConnection{
        sqlite: Arc::new(Mutex::new(sqlite))
    };

    Router::new()
        .nest("/", v1_routes(connections.clone()))
        // .route("/media/*path", get(media_path))
        .layer(Extension(redis))
        .layer(Extension(database))
        .layer(Extension(mongo))
        .layer(Extension(sqlite_connection))
        .layer(Extension(bot))
        .layer(CookieManagerLayer::new())
        // .nest("/ffffffffffffffffff", router)
}
