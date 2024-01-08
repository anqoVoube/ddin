use axum::Router;
use axum::routing::{get, post};

use crate::routes::debts::create::create;
use crate::routes::debts::find::full_serializer_search;
use crate::routes::debts::history::get_history;
use crate::routes::debts::update::update;

pub fn get_router() -> Router{
    Router::new()
        .route("/", get(full_serializer_search).post(create))
        .route("/payment", post(update))
        .route("/history/:id", get(get_history))
}