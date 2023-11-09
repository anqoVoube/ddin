use axum::Router;
use axum::routing::{get, post};
use crate::routes::find::purchase::search as purchase_search;
use crate::routes::find::sell::search as sell_search;
use crate::routes::find::all::search as all_search;

pub fn get_router() -> Router{
    Router::new()
        .route("/sell", get(sell_search))
        .route("/purchase", get(purchase_search))
        .route("/all", get(all_search))
}