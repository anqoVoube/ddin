mod business;
mod user;
mod parent_product;
mod product;
mod utils;

use axum::{Router, body::Body, Extension};


use axum::routing::get;

use sea_orm::DatabaseConnection;
use crate::routes::business::router::get_router as business_router;
use crate::routes::user::router::get_router as user_router;
use crate::routes::parent_product::router::get_router as parent_product_router;
use crate::routes::product::router::get_router as product_router;
use crate::routes::utils::media::media_path;


pub fn v1_routes() -> Router{
    Router::new()
        .nest("/business", business_router())
        .nest("/user/", user_router())
        .nest("/parent-product/", parent_product_router())
        .nest("/product/", product_router())
}




pub fn create_routes(database: DatabaseConnection) -> Router<(), Body> {
    // Router with trailing slash deletion

    Router::new()
        .nest("/", v1_routes())
        .route("/media/*path", get(media_path))
        .layer(Extension(database))

}
