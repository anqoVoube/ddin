use axum::Router;
use axum::routing::{get, post};
use crate::routes::find::sell::search as sell_search;
use crate::routes::find::purchase::search as purchase_search;

pub fn get_router() -> Router{
    Router::new()
        .route("/sell", get(purchase_search))
        .route("/purchase", get(sell_search))
}