use axum::{Extension, Json, debug_handler};
use axum::response::{IntoResponse, Response};
use http::StatusCode;
use sea_orm::{DatabaseConnection};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

fn default_as_false() -> bool {
    false
}


#[derive(Clone, Serialize, Deserialize)]
pub struct Body {
    price: i64,
    expire_date: DateTime<Utc>,
    business_id: i64,
    parent_product_id: i64,
}

#[debug_handler]
pub async fn create(
    Extension(database): Extension<DatabaseConnection>,
    Json(Body { price, expire_date, business_id, parent_product_id }): Json<Body>
) -> Response {
    ().into_response()
}
