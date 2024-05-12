use axum::Router;
use axum::routing::{get};
use crate::routes::analytics::all::get_all_analytics;


pub fn get_router() -> Router{
    Router::new()
        .route("/", get(get_all_analytics))
}