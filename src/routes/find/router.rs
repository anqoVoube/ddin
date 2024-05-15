use axum::Router;
use axum::routing::{get, post};
use crate::routes::find::purchase::search as purchase_search;
use crate::routes::find::sell::{search as sell_search, search_by_voice};
use crate::routes::find::all::search as all_search;
use crate::routes::find::code::google_search_title_by_code;
use crate::routes::find::most::most_searched;

pub fn get_router() -> Router{
    Router::new()
        .route("/sell", get(sell_search))
        .route("/sell-voice", get(search_by_voice))
        .route("/purchase", get(purchase_search))
        .route("/all", get(all_search))
        .route("/:code", get(google_search_title_by_code))
        .route("/most-searched", get(most_searched))
}
