mod business;
mod user;
mod parent_product;
mod product;
mod utils;

use std::num::FpCategory::Normal;
use axum::{Router, body::Body, Extension};
use sea_orm::DatabaseConnection;
use crate::routes::business::router::get_router as business_router;
use crate::routes::user::router::get_router as user_router;


pub fn v1_routes() -> Router{
    Router::new()
        .nest("/business", business_router())
        .nest("/user/", user_router())
}


pub fn create_routes(database: DatabaseConnection) -> Router<(), Body> {
    // Router with trailing slash deletion

    Router::new()
        .nest("/api/v1", v1_routes())
        .layer(Extension(database))

}
