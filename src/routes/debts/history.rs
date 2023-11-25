use axum::{debug_handler, Extension, Json};
use axum::extract::Path;
use http::StatusCode;
use sea_orm::{DatabaseConnection, EntityTrait};
use sea_orm::prelude::DateTimeWithTimeZone;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use crate::core::auth::middleware::Auth;
use crate::database::prelude::RentHistory;
use crate::database::rent_history;
use axum::response::{IntoResponse, Response};
use sea_orm::QueryFilter;
use sea_orm::ColumnTrait;
use crate::routes::sell::RentHistoryProducts;

#[derive(Serialize, Deserialize, Debug)]
struct History{
    id: i32,
    products: RentHistoryProducts,
    buy_date: DateTimeWithTimeZone
}

#[derive(Serialize, Deserialize, Debug)]
struct Histories{
    histories: Vec<History>
}

#[debug_handler]
pub async fn get_history(
    Extension(Auth{user_id, business_id}): Extension<Auth>,
    Extension(database): Extension<DatabaseConnection>,
    Path(id): Path<i32>
) -> Response{
    // ToDo: Check RentUser for business_id for security purposes
    let histories = RentHistory::find()
        .filter(
            rent_history::Column::RentUserId.eq(id)
        )
        .all(&database)
        .await
        .unwrap();

    let mut response_body = Histories{histories: vec![]};

    for history in histories{
        let products_str = history.products.to_string();
        // Now, parse the string into the Products struct
        let products = match serde_json::from_str::<RentHistoryProducts>(&products_str) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Failed to parse products JSON: {:?}", e);
                continue; // Handle the error as needed
            }
        };
        response_body.histories.push(History{
            id: history.id,
            products,
            buy_date: history.buy_date
        })
    }

    (
        StatusCode::OK,
        Json(response_body)
    ).into_response()
}